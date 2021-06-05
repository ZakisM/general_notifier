#[macro_use]
extern crate log;

use std::env;

use anyhow::Result;
use reqwest::Client;
use sqlx::sqlite::SqlitePoolOptions;

#[tokio::main]
async fn main() -> Result<()> {
    dotenv::dotenv()?;
    env::set_var("RUST_LOG", "INFO");

    pretty_env_logger::init_timed();

    let migrations = sqlx::migrate::Migrator::new(std::path::Path::new("./migrations")).await?;

    let database_url = env::var("DATABASE_URL")?;

    let pool = SqlitePoolOptions::new()
        .max_connections(15)
        .connect(&database_url)
        .await?;

    migrations.run(&pool).await?;

    let mut conn = pool.acquire().await?;

    let row_id = sqlx::query!(
        "INSERT INTO user (hashed_token, discord_id, discord_name) VALUES ( ?1, ?2, ?3 )",
        "abc123",
        "Zak123",
        "Zak"
    )
    .execute(&mut conn)
    .await?
    .last_insert_rowid();

    info!("Inserted into: {}", row_id);

    let client = Client::new();

    let res = client
        .get("https://changelogs.ubuntu.com/meta-release")
        .send()
        .await?
        .text()
        .await?;

    if res
        .lines()
        .any(|l| l.to_lowercase().contains("version: 21.04"))
    {
        info!("Found");
    }

    Ok(())
}
