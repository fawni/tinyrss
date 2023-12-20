use std::{error::Error, sync::Arc};

use rss::Channel;
use sqlx::{Pool, Sqlite, SqlitePool};

use crate::config::Config;

pub type Database = Arc<DatabaseState>;

pub struct Record {
    pub title: String,
    pub link: String,
    pub last: String,
}

pub struct DatabaseState {
    pool: Pool<Sqlite>,
}

impl DatabaseState {
    pub async fn new() -> Result<Self, Box<dyn Error>> {
        let pool = SqlitePool::connect(&std::env::var("DATABASE_URL")?).await?;
        sqlx::migrate!().run(&pool).await?;
        Ok(Self { pool })
    }

    pub async fn init(&self, config: &Config) -> Result<(), Box<dyn Error>> {
        for webhook in &config.webhooks {
            for sub in &webhook.subscriptions {
                let content = reqwest::get(&sub.url).await?.bytes().await?;
                let channel = Channel::read_from(&content[..])?;
                let last = channel
                    .items()
                    .first()
                    .map_or("null", |item| &item.guid().unwrap().value);

                sqlx::query!(
                    "INSERT INTO subscriptions (title, link, last) VALUES (?, ?, ?) ON CONFLICT DO NOTHING",
                    sub.name,
                    channel.link,
                    last
                )
                .execute(&self.pool).await?;
            }
        }
        Ok(())
    }

    pub async fn get(&self, name: &str) -> Result<Record, sqlx::Error> {
        sqlx::query_as!(Record, "SELECT * FROM subscriptions WHERE title = ?", name)
            .fetch_one(&self.pool)
            .await
    }

    pub async fn set(&self, name: &str, last: &str) -> Result<(), sqlx::Error> {
        sqlx::query!(
            "UPDATE subscriptions SET last = ? WHERE title = ?",
            last,
            name
        )
        .execute(&self.pool)
        .await?;

        Ok(())
    }
}

pub async fn open() -> Result<Database, Box<dyn Error>> {
    let db = Arc::new(DatabaseState::new().await?);

    Ok(db)
}
