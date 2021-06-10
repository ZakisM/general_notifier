use std::convert::TryInto;
use std::sync::Arc;

use anyhow::{Context, Result};
use serenity::model::id::UserId;
use serenity::CacheAndHttp;

#[derive(Debug)]
pub struct ResponseMessage {
    pub discord_id: i64,
    pub message: String,
}

impl ResponseMessage {
    pub async fn send(&self, cache_http: Arc<CacheAndHttp>) -> Result<()> {
        let discord_id = self
            .discord_id
            .try_into()
            .context("Failed to convert discord_id into i64 to send message.")?;

        let dm_channel = UserId(discord_id).create_dm_channel(&cache_http).await?;

        dm_channel
            .send_message(&cache_http.http, |m| m.content(&self.message))
            .await?;

        Ok(())
    }
}
