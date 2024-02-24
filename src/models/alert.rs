use std::convert::TryInto;

use anyhow::{Context, Result};
use reqwest::Url;
use serenity::framework::standard::RawArguments;

use crate::hash_id;

#[derive(Debug)]
pub struct Alert {
    pub alert_id: String,
    pub url: String,
    pub matching_text: String,
    pub non_matching: i64,
    pub discord_id: i64,
}

impl Alert {
    pub fn new<T, U>(url: T, matching_text: U, non_matching: i64, discord_id: i64) -> Self
    where
        T: AsRef<str>,
        U: AsRef<str>,
    {
        let url = url.as_ref().to_owned();
        let matching_text = matching_text.as_ref().to_owned();

        Alert {
            alert_id: hash_id!(url, matching_text, non_matching, discord_id),
            url,
            matching_text,
            non_matching,
            discord_id,
        }
    }

    pub fn from_args<T>(args: &mut RawArguments, discord_id: T) -> Result<Self>
    where
        T: TryInto<i64>,
        <T as std::convert::TryInto<i64>>::Error: std::error::Error + Send + Sync + 'static,
    {
        let url: Url = args
            .next()
            .context("Missing URL.")?
            .parse()
            .context("Please enter a valid URL.")?;

        let matching_text = args.next().context("Missing matching text.")?;

        let non_matching = args.find(|a| *a == "-n").map(|_| 1).unwrap_or(0);

        let discord_id: i64 = discord_id
            .try_into()
            .context("Failed to convert discord_id into i64")?;

        Ok(Self::new(
            url,
            matching_text.replace("'''", "\"").replace('~', ""),
            non_matching,
            discord_id,
        ))
    }
}
