use async_trait::async_trait;
use sqlx::postgres::PgPoolOptions;
use sqlx::{Pool, Postgres};
use std::env;
use std::fs::{self, File};
use std::io::Write;

#[async_trait]
pub trait Sink {
    async fn consume(&mut self, data: &serde_json::Value);
}

pub struct FileSink {
    file: File,
}

impl FileSink {
    pub fn open(conf: &toml::Value) -> Result<Self, std::io::Error> {
        let path = conf["sink_to"]
            .as_str()
            .expect("Missing 'sink_to' in conf.toml");
        let file = fs::OpenOptions::new()
            .create(true)
            .append(true)
            .open(path)?;

        Ok(FileSink { file })
    }
}

#[async_trait]
impl Sink for FileSink {
    async fn consume(&mut self, data: &serde_json::Value) {
        if let Err(e) = serde_json::to_writer(&self.file, &data) {
            eprintln!("Failed to write to file, err={}", e);
        } else {
            writeln!(self.file).unwrap();
        }
    }
}

pub struct PostgresSink {
    pool: Pool<Postgres>,
}

impl PostgresSink {
    pub async fn open(conf: &toml::Value) -> Result<Self, sqlx::Error> {
        let url = conf["db_url"]
            .as_str()
            .expect("Missing 'db_url' in conf.toml");
        let username = env::var("PG_USERNAME").unwrap_or("postgres".to_string());
        let password = env::var("PG_PASSWORD").unwrap_or("postgres".to_string());
        let url = url
            .replace("${PG_USERNAME}", &username)
            .replace("${PG_PASSWORD}", &password);

        let pool = PgPoolOptions::new().connect(&url).await?;
        sqlx::query(
            r"
            CREATE TABLE IF NOT EXISTS douban_game (
                id bigserial NOT NULL,
                created_at timestamp DEFAULT CURRENT_TIMESTAMP NOT NULL,
                games jsonb NULL,
                CONSTRAINT douban_game_pk PRIMARY KEY (id)
            );
            ",
        )
        .execute(&pool)
        .await?;

        Ok(PostgresSink { pool })
    }
}

#[async_trait]
impl Sink for PostgresSink {
    async fn consume(&mut self, data: &serde_json::Value) {
        sqlx::query(
            r"
            INSERT INTO douban_game (games)
            VALUES ($1)
            ",
        )
        .bind(&data)
        .execute(&self.pool)
        .await
        .unwrap();
    }
}
