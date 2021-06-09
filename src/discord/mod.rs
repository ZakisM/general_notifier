use std::sync::Arc;

use anyhow::Context as ErrorContext;
use anyhow::Result;
use comfy_table::modifiers::UTF8_ROUND_CORNERS;
use comfy_table::presets::UTF8_FULL;
use comfy_table::{ColumnConstraint, Table};
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

use crate::conduit;
use crate::models::alert::Alert;

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

    dm_channel
        .send_message(ctx, |m| {
            m.content(format!(
                "Failed to run command \"{}{}\" due to error: {}",
                COMMAND_PREFIX, command_name, e
            ))
        })
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

pub async fn start(sqlite_pool: Arc<SqlitePool>) -> Result<()> {
    let framework = StandardFramework::new()
        .configure(|c| c.with_whitespace(true).prefix(COMMAND_PREFIX))
        .after(after)
        .group(&GENERAL_GROUP);

    // let token = env::var("DISCORD_TOKEN").expect("Missing Discord Bot token");

    let mut client = Client::builder("ODUxODgyOTY2NjUyMjg5MDU1.YL-v1g.AI6p-BwToRVNi_OqzT9nztsiINE")
        .event_handler(Handler)
        .framework(framework)
        .await
        .context("Error creating client")?;

    {
        let mut data = client.data.write().await;

        data.insert::<Database>(sqlite_pool);
    }

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

    let alert_count = conduit::alert::count(pool, msg.author.id.0).await?;

    let alert = Alert::from_args(&mut args, msg.author.id.0, alert_count + 1)?;

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

    let alerts = conduit::alert::list(pool, msg.author.id.0).await?;

    let mut table = Table::new();
    table
        .load_preset(UTF8_FULL)
        .apply_modifier(UTF8_ROUND_CORNERS)
        .set_header(vec!["", "URL", "Matching Text"]);

    alerts.into_iter().for_each(|a| {
        table.add_row(vec![format!("{}.", a.alert_number), a.url, a.matching_text]);
    });

    let column = table
        .get_column_mut(1)
        .context("Failed to format table to display results")?;

    column.set_constraint(ColumnConstraint::MaxWidth(100));

    let mut response = MessageBuilder::new();
    response.push_codeblock_safe(table, None);

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

    let alert_number = args.next().context("Missing alert number")?.parse()?;

    conduit::alert::delete(pool, msg.author.id.0, alert_number).await?;

    dm_channel
        .send_message(ctx, |m| {
            m.content("Successfully deleted alert! Use ~list to see your current alerts")
        })
        .await?;

    Ok(())
}
