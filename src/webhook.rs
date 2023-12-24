use rss::{Channel, Item};
use webhook::client::{WebhookClient, WebhookResult};

use crate::config::{Config, Subscription};

const MAX_DESCRIPTION_LEN: u16 = 600;

pub async fn send(
    webhook_url: &str,
    channel: &Channel,
    item: &Item,
    sub: &Subscription,
    config: &Config,
) -> WebhookResult<()> {
    let description = item.description().unwrap_or_default();
    let body = html2text::from_read(description.as_bytes(), 800);
    let color = sub.clone().color.unwrap_or(config.color).to_string();

    let image = if description.contains("<img src=\"") {
        let img = description
            .chars()
            .skip(description.find("<img src=\"").unwrap() + "<img src=\"".len())
            .take_while(|c| !c.eq(&'\"'))
            .collect::<String>();
        Some(img)
    } else {
        None
    };

    WebhookClient::new(webhook_url)
        .send(|m| {
            let msg = m
                .username("tinyrss")
                .avatar_url("https://i.imgur.com/Hkk3njM.png")
                .embed(|e| {
                    e.author(&sub.name, Some(channel.link().to_owned()), None)
                        .color(&color)
                });

            if body.len() <= MAX_DESCRIPTION_LEN.into() {
                msg.embeds[0].description(&body);
            } else {
                msg.embeds[0].description(&format!(
                    "{}...",
                    &body
                        .chars()
                        .take((MAX_DESCRIPTION_LEN - 3).into())
                        .collect::<String>()
                ));
            }

            if let Some(icon) = &sub.icon {
                msg.embeds[0].author(
                    &sub.name,
                    Some(channel.link().to_owned()),
                    Some(icon.to_owned()),
                );
                msg.embeds[0].thumbnail(icon);
            }

            if let Some(title) = item.title() {
                msg.embeds[0].title(title);
            }

            if let Some(url) = item.link() {
                msg.embeds[0].url(url);
                if msg.embeds[0].title.is_none() {
                    msg.embeds[0].title = Some("link".to_owned());
                }
            }

            if let Some(img) = &image {
                msg.embeds[0].image(img);
            }

            if let Some(pub_date) = item.pub_date() {
                msg.embeds[0].timestamp(
                    &chrono::DateTime::parse_from_rfc2822(pub_date)
                        .unwrap()
                        .to_rfc3339(),
                );
            }

            msg
        })
        .await?;

    Ok(())
}
