use std::{borrow::Borrow, convert::TryInto, sync::Arc};

use anyhow::{Context, Result};
use serenity::{
    all::{Cache, CreateMessage, Http},
    model::id::UserId,
};

#[derive(Debug)]
pub struct ResponseMessage {
    pub discord_id: i64,
    pub message: String,
}

impl ResponseMessage {
    pub async fn send(&self, cache_http: (Arc<Cache>, Arc<Http>)) -> Result<()> {
        let cache_http = (cache_http.0.borrow(), cache_http.1.as_ref());

        let discord_id = self
            .discord_id
            .try_into()
            .context("Failed to convert discord_id into i64 to send message.")?;

        let dm_channel = UserId::new(discord_id)
            .create_dm_channel(cache_http)
            .await?;

        dm_channel
            .send_message(cache_http, CreateMessage::new().content(&self.message))
            .await?;

        Ok(())
    }
}
