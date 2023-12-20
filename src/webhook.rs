use rss::{Channel, Item};
use webhook::client::{WebhookClient, WebhookResult};

use crate::config::Subscription;

pub async fn send(
    webhook_url: &str,
    channel: &Channel,
    item: &Item,
    sub: &Subscription,
) -> WebhookResult<()> {
    let content = item.description().unwrap_or("no description").as_bytes();
    let body = html2text::from_read(content, 800);
    let color = sub.clone().color.unwrap_or("15900157".to_owned());

    WebhookClient::new(webhook_url)
        .send(|m| {
            let msg = m
                .username("tinyrss")
                .avatar_url("https://i.imgur.com/Hkk3njM.png")
                .embed(|e| {
                    e.author(&sub.name, Some(channel.link().to_owned()), None)
                        .description(&body)
                        .color(&color)
                });

            if let Some(icon) = &sub.icon {
                // msg.embeds[0].author.unwrap().icon_url = Some(icon.to_owned());
                msg.embeds[0].author(
                    &sub.name,
                    Some(channel.link().to_owned()),
                    Some(icon.to_owned()),
                );
                msg.embeds[0].thumbnail(icon);
            }

            if let Some(url) = item.link() {
                msg.embeds[0].url(url);
            }

            if let Some(title) = item.title() {
                if title.len() <= 35 {
                    msg.embeds[0].title(title);
                }
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
