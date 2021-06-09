#[macro_use]
extern crate tracing;

use std::env;
use std::path::Path;
use std::str::FromStr;
use std::sync::Arc;

use anyhow::Result;
use reqwest::ClientBuilder;
use sqlx::sqlite::{SqliteConnectOptions, SqlitePoolOptions};
use sqlx::ConnectOptions;

mod conduit;
mod discord;
mod models;
mod util;

#[tokio::main]
async fn main() -> Result<()> {
    dotenv::dotenv()?;

    env::set_var("RUST_LOG", "INFO");

    tracing_subscriber::fmt::init();

    let migrations = sqlx::migrate::Migrator::new(Path::new("./migrations")).await?;

    let database_url = env::var("DATABASE_URL")?;

    let mut connect_options = SqliteConnectOptions::from_str(&database_url)?;
    connect_options.disable_statement_logging();

    let pool = Arc::new(
        SqlitePoolOptions::new()
            .max_connections(15)
            .connect_with(connect_options)
            .await?,
    );

    migrations.run(&*pool).await?;

    // let user = User::new("abc123", 12345, "Zak");
    //
    // conduit::user::insert(&pool, user).await?;

    let client = ClientBuilder::new()
        .brotli(true)
        .cookie_store(true)
        .build()?;

    // let res = client
    //     .get("https://www.amazon.co.uk/s?k=lg+27gp950-b&ref=nb_sb_noss_1")
    //     .send()
    //     .await?
    //     .text()
    //     .await?;
    //
    // dbg!(&res);

    // if res
    //     .lines()
    //     .any(|l| l.to_lowercase().contains("version: 21.04"))
    // {
    //     info!("Found");
    // }

    tokio::task::block_in_place(|| discord::start(pool.clone())).await?;

    Ok(())
}
