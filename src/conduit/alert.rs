use std::convert::TryInto;

use anyhow::{Context, Result};
use sqlx::SqlitePool;

use crate::models::alert::Alert;

pub async fn list(pool: &SqlitePool, discord_id: u64) -> Result<Vec<Alert>> {
    let discord_id: i64 = discord_id
        .try_into()
        .context("Failed to convert discord_id into i64")?;

    let alerts = sqlx::query_as!(
        Alert,
        "SELECT * FROM alert WHERE discord_id = ? ",
        discord_id
    )
    .fetch_all(pool)
    .await?;

    Ok(alerts)
}

pub async fn insert(pool: &SqlitePool, alert: Alert) -> Result<()> {
    let mut conn = pool.acquire().await?;

    let row_id = sqlx::query!(
        "INSERT INTO alert (alert_id, url, matching_text, discord_id) VALUES ( ?1, ?2, ?3, ?4 )",
        alert.alert_id,
        alert.url,
        alert.matching_text,
        alert.discord_id
    )
    .execute(&mut conn)
    .await?
    .last_insert_rowid();

    info!("Inserted into: {}", row_id);

    Ok(())
}
