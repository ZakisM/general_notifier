use anyhow::{bail, Result};
use sqlx::SqlitePool;

use crate::models::alert::Alert;

pub async fn all(pool: &SqlitePool) -> Result<Vec<Alert>> {
    let alerts = sqlx::query_as!(Alert, "SELECT * FROM alert",)
        .fetch_all(pool)
        .await?;

    Ok(alerts)
}

pub async fn list(pool: &SqlitePool, discord_id: i64) -> Result<Vec<Alert>> {
    let alerts = sqlx::query_as!(
        Alert,
        "SELECT * FROM alert WHERE discord_id = ?",
        discord_id
    )
    .fetch_all(pool)
    .await?;

    Ok(alerts)
}

pub async fn insert(pool: &SqlitePool, alert: Alert) -> Result<()> {
    let mut conn = pool.acquire().await?;

    sqlx::query!(
        "INSERT INTO alert (alert_id, url, matching_text, non_matching, discord_id) VALUES ( ?1, ?2, ?3, ?4, ?5 )",
        alert.alert_id,
        alert.url,
        alert.matching_text,
        alert.non_matching,
        alert.discord_id,
    )
    .execute(&mut conn)
    .await?;

    Ok(())
}

pub async fn delete(pool: &SqlitePool, discord_id: i64, alert_id: &str) -> Result<()> {
    let mut conn = pool.acquire().await?;

    let rows_affected = sqlx::query!(
        "DELETE FROM alert WHERE discord_id = ?1 AND alert_id = ?2",
        discord_id,
        alert_id,
    )
    .execute(&mut conn)
    .await?
    .rows_affected();

    if rows_affected == 0 {
        bail!("Could not find this alert to delete")
    } else {
        Ok(())
    }
}
