use std::env;
use std::path::Path;
use std::str::FromStr;
use std::sync::Arc;

use anyhow::Result;
use sqlx::sqlite::{SqliteConnectOptions, SqlitePoolOptions};
use sqlx::ConnectOptions;

mod conduit;
mod discord;
mod models;
mod util;
mod worker;

#[tokio::main]
async fn main() -> Result<()> {
    dotenv::dotenv()?;

    env::set_var("RUST_LOG", "INFO");

    tracing_subscriber::fmt::init();

    let migrations = sqlx::migrate::Migrator::new(Path::new("./migrations")).await?;

    let database_url = env::var("DATABASE_URL")?;

    let mut connect_options = SqliteConnectOptions::from_str(&database_url)?;
    connect_options = connect_options.disable_statement_logging();

    let pool = Arc::new(
        SqlitePoolOptions::new()
            .max_connections(15)
            .connect_with(connect_options)
            .await?,
    );

    migrations.run(&*pool).await?;

    let (responder_tx, responder_rx) = tokio::sync::mpsc::channel(100);

    tokio::task::spawn(worker::alert::start(pool.clone(), responder_tx.clone()));

    tokio::task::block_in_place(|| discord::start(pool.clone(), responder_rx)).await?;

    Ok(())
}
