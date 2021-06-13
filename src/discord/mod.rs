use std::convert::TryInto;
use std::env;
use std::iter::FromIterator;
use std::sync::Arc;

use anyhow::Context as ErrorContext;
use anyhow::Result;
use regex::RegexBuilder;
use serenity::async_trait;
use serenity::client::{Client, Context, EventHandler};
use serenity::framework::standard::{
    macros::{command, group, hook},
    Args, CommandError, CommandResult, StandardFramework,
};
use serenity::model::channel::Message;
use serenity::prelude::TypeMapKey;
use serenity::utils::MessageBuilder;
use sqlx::SqlitePool;
use tokio::sync::mpsc::Receiver;

use crate::conduit;
use crate::models::alert::Alert;
use crate::models::response_message::ResponseMessage;

const COMMAND_PREFIX: &str = "~";

struct Database;

impl TypeMapKey for Database {
    type Value = Arc<SqlitePool>;
}

#[group]
#[commands(add, list, delete)]
struct General;

struct Handler;

#[async_trait]
impl EventHandler for Handler {}

async fn send_error_to_user(
    ctx: &Context,
    msg: &Message,
    command_name: &str,
    e: &CommandError,
) -> Result<()> {
    let dm_channel = msg.author.id.create_dm_channel(&ctx).await?;

    let mut response = MessageBuilder::new();

    response.push_codeblock_safe(
        format!(
            "Failed to run command \"{}{}\" due to error: {}",
            COMMAND_PREFIX, command_name, e
        ),
        None,
    );

    dm_channel
        .send_message(ctx, |m| m.content(response))
        .await?;

    Ok(())
}

#[hook]
async fn after(ctx: &Context, msg: &Message, command_name: &str, command_result: CommandResult) {
    match command_result {
        Ok(_) => info!("Processed command: '{}'", command_name),
        Err(ref e) => {
            error!("Command '{}' returned error: {:?}", command_name, e);

            if let Err(e) = send_error_to_user(ctx, msg, command_name, e).await {
                error!("Failed to send error to user: {:?}", e);
            }
        }
    }
}

pub async fn start(
    sqlite_pool: Arc<SqlitePool>,
    mut responder_rx: Receiver<ResponseMessage>,
) -> Result<()> {
    let framework = StandardFramework::new()
        .configure(|c| c.with_whitespace(true).prefix(COMMAND_PREFIX))
        .after(after)
        .group(&GENERAL_GROUP);

    let token = env::var("DISCORD_TOKEN").expect("Missing Discord Bot token");

    let mut client = Client::builder(token)
        .event_handler(Handler)
        .framework(framework)
        .await
        .context("Error creating client")?;

    {
        let mut data = client.data.write().await;

        data.insert::<Database>(sqlite_pool);
    }

    let cache_http = client.cache_and_http.clone();

    tokio::task::spawn(async move {
        while let Some(response_message) = responder_rx.recv().await {
            if let Err(e) = response_message.send(cache_http.clone()).await {
                error!("Failed to send response_message: {}", e);
            }
        }
    });

    Ok(client.start().await?)
}

#[command]
async fn add(ctx: &Context, msg: &Message, args: Args) -> CommandResult {
    let dm_channel = msg.author.id.create_dm_channel(&ctx).await?;

    let mut args = args.raw_quoted();

    let data = ctx.data.read().await;

    let pool = data
        .get::<Database>()
        .context("Failed to read Database pool.")?;

    let alert = Alert::from_args(&mut args, msg.author.id.0)?;

    let _ = RegexBuilder::new(&alert.matching_text)
        .case_insensitive(true)
        .build()?;

    conduit::alert::insert(pool, alert).await?;

    dm_channel
        .send_message(ctx, |m| {
            m.content("Successfully added alert! Use ~list to see your current alerts")
        })
        .await?;

    Ok(())
}

#[command]
async fn list(ctx: &Context, msg: &Message) -> CommandResult {
    let dm_channel = msg.author.id.create_dm_channel(&ctx).await?;

    let data = ctx.data.read().await;

    let pool = data
        .get::<Database>()
        .context("Failed to read Database pool.")?;

    let alerts = conduit::alert::list(
        pool,
        msg.author
            .id
            .0
            .try_into()
            .context("Failed to convert discord_id to i64")?,
    )
    .await?;

    let mut response = MessageBuilder::new();

    if !alerts.is_empty() {
        let results: String = alerts
            .into_iter()
            .enumerate()
            .map(|(i, a)| {
                format!(
                    "{}.\n    Id: {}\n    Url: {}\n    Matching Text: {}\n    Non Matching: {}\n",
                    i + 1,
                    a.alert_id,
                    a.url,
                    a.matching_text,
                    if a.non_matching == 1 { "True" } else { "False" },
                )
            })
            .collect();

        // If message is too large then send it in chunks;
        if results.len() > 1900 {
            let results = results.chars().collect::<Vec<_>>();

            for chunk in results.chunks(1900) {
                let mut response = MessageBuilder::new();

                response.push_codeblock_safe(String::from_iter(chunk), None);

                dm_channel
                    .send_message(ctx, |m| m.content(response))
                    .await?;
            }

            return Ok(());
        }

        response.push_codeblock_safe(results, None);
    } else {
        response.push_codeblock_safe("You currently have 0 alerts.", None);
    }

    dm_channel
        .send_message(ctx, |m| m.content(response))
        .await?;

    Ok(())
}

#[command]
async fn delete(ctx: &Context, msg: &Message, args: Args) -> CommandResult {
    let dm_channel = msg.author.id.create_dm_channel(&ctx).await?;

    let mut args = args.raw_quoted();

    let data = ctx.data.read().await;

    let pool = data
        .get::<Database>()
        .context("Failed to read Database pool.")?;

    let alert_id = args.next().context("Missing alert id")?.parse()?;

    conduit::alert::delete(
        pool,
        msg.author
            .id
            .0
            .try_into()
            .context("Failed to convert discord_id to i64")?,
        alert_id,
    )
    .await?;

    dm_channel
        .send_message(ctx, |m| {
            m.content("Successfully deleted alert! Use ~list to see your current alerts")
        })
        .await?;

    Ok(())
}
