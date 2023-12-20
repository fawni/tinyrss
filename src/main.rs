use std::{error::Error, time::Duration};

use reqwest::Client;
use rss::Channel;

mod config;
mod db;
mod webhook;

#[tokio::main]
async fn main() -> Result<(), Box<dyn Error>> {
    twink::log::setup();
    dotenvy::dotenv().ok();

    let config = config::read()?;
    log::info!("read config successfully");

    let client = Client::builder()
        .connect_timeout(Duration::from_secs(10))
        .build()?;

    let database = db::open().await?;
    log::info!("connected to database");

    database.init(&config).await?;
    log::info!("initialized database");

    log::info!("starting watch loop");
    loop {
        for (i, hook) in config.webhooks.iter().enumerate() {
            for sub in &hook.subscriptions {
                let last = database.get(&sub.name).await?.last;
                let content = client.get(&sub.url).send().await?.bytes().await?;
                let channel = Channel::read_from(&content[..])?;

                let mut items = channel
                    .items
                    .iter()
                    .take_while(|x| x.guid().unwrap().value != last)
                    .collect::<Vec<_>>();
                items.reverse();

                if items.is_empty() {
                    continue;
                }

                log::info!("new item(s) found for webhook \x1b[1m{i}\x1b[0m");

                for item in items {
                    let guid = item.guid().unwrap().value();
                    log::info!(
                        "sending webhook for subscription: \x1b[1;36m{}\x1b[0m, item: \x1b[32m{}\x1b[0m",
                        sub.name,
                        guid
                    );

                    if let Err(e) = webhook::send(&hook.url, &channel, item, sub).await {
                        log::error!("failed to send webhook: \x1b[31m{e}\x1b[0m");
                    } else {
                        database.set(&sub.name, guid).await?;
                        log::info!("sent successfully");
                    };
                }
            }
        }

        log::info!("finished fetching, waiting for {}m", config.refetch_delay);
        std::thread::sleep(Duration::from_secs(config.refetch_delay * 60));
        log::info!("wait duration is over, fetching");
    }
}
