use std::collections::HashMap;
use std::sync::Arc;
use std::time::Duration;

use anyhow::Result;
use regex::RegexBuilder;
use reqwest::{Client, ClientBuilder};
use sqlx::SqlitePool;
use tokio::sync::mpsc::Sender;

use crate::conduit;
use crate::models::alert::Alert;
use crate::models::response_message::ResponseMessage;

pub async fn start(pool: Arc<SqlitePool>, responder_tx: Sender<ResponseMessage>) -> Result<()> {
    let client = ClientBuilder::new()
        .brotli(true)
        .cookie_store(true)
        .timeout(Duration::from_secs(10))
        .build()?;

    loop {
        match conduit::alert::all(&pool).await {
            Ok(alerts) => {
                /* Group the alerts with the same URL to avoid having to send the same HTTP request
                multiple times */
                let alerts_grouped: HashMap<&str, Vec<&Alert>> =
                    alerts.iter().fold(HashMap::new(), |mut curr, next| {
                        let url_alerts = curr.entry(&next.url).or_default();
                        url_alerts.push(next);
                        curr
                    });

                for (url, alerts) in alerts_grouped {
                    if let Err(e) = check_alert(
                        pool.clone(),
                        client.clone(),
                        url,
                        alerts,
                        responder_tx.clone(),
                    )
                    .await
                    {
                        error!("Failed to check_alert: {}", e);
                    }
                }
            }
            Err(e) => error!("Failed to read all alerts: {}", e),
        }

        tokio::time::sleep(Duration::from_secs(60 * 5)).await;
    }
}

pub async fn check_alert(
    pool: Arc<SqlitePool>,
    client: Client,
    url: &str,
    alerts: Vec<&Alert>,
    responder_tx: Sender<ResponseMessage>,
) -> Result<()> {
    let splash_url = format!("http://splash:8050/render.html?url={}&timeout=10", url);
    let res = client.get(&splash_url).send().await?.text().await?;

    info!("Sent request to {}", &url);

    for alert in alerts {
        let regex = RegexBuilder::new(&alert.matching_text)
            .case_insensitive(true)
            .build()?;

        let captures = regex.captures(&res);

        if alert.non_matching == 0 && captures.is_some()
            || alert.non_matching == 1 && captures.is_none()
        {
            responder_tx
                .send(ResponseMessage {
                    discord_id: alert.discord_id,
                    message: format!(
                        "Found matching text: [{}] at URL: {}",
                        alert.matching_text, alert.url
                    ),
                })
                .await?;

            conduit::alert::delete(&pool, alert.discord_id, &alert.alert_id).await?;
        }
    }

    Ok(())
}
