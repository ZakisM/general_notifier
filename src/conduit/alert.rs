use anyhow::{bail, Result};
use sqlx::SqlitePool;

use crate::models::alert::Alert;

pub async fn all(pool: &SqlitePool) -> Result<Vec<Alert>> {
    let alerts = sqlx::query_as!(Alert, "SELECT * FROM alert",)
        .fetch_all(pool)
        .await?;

    Ok(alerts)
}

pub async fn count(pool: &SqlitePool, discord_id: i64) -> Result<i32> {
    let count = sqlx::query!(
        "SELECT COUNT(*) as count FROM alert WHERE discord_id = ?",
        discord_id
    )
    .fetch_one(pool)
    .await?;

    Ok(count.count)
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
        "INSERT INTO alert (alert_id, url, matching_text, discord_id, alert_number) VALUES ( ?1, ?2, ?3, ?4, ?5 )",
        alert.alert_id,
        alert.url,
        alert.matching_text,
        alert.discord_id,
        alert.alert_number
    )
        .execute(&mut conn)
        .await?;

    Ok(())
}

pub async fn delete(pool: &SqlitePool, discord_id: i64, alert_number: i64) -> Result<()> {
    let mut conn = pool.acquire().await?;

    let rows_affected = sqlx::query!(
        "DELETE FROM alert WHERE discord_id = ?1 and alert_number = ?2",
        discord_id,
        alert_number,
    )
    .execute(&mut conn)
    .await?
    .rows_affected();

    if rows_affected == 0 {
        bail!("Could not find this alert number")
    } else {
        sqlx::query!(
            "UPDATE alert SET alert_number = alert_number - 1 WHERE discord_id = ?1 AND alert_number > ?2",
            discord_id,
            alert_number,
        )
            .execute(&mut conn)
            .await?;

        Ok(())
    }
}
