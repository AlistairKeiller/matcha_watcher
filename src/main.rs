mod commands;
mod config;

use dashmap::DashSet;
use poise::{FrameworkOptions, serenity_prelude as serenity};
use serenity::all::UserId;
use serenity::prelude::TypeMapKey;
use std::{env::var, sync::Arc};
use tokio::sync::RwLock;
use tracing::{error, info};

type Error = Box<dyn std::error::Error + Send + Sync>;
type Context<'a> = poise::Context<'a, Data, Error>;

pub struct Data {
    subscribers: DashSet<UserId>,
}

impl TypeMapKey for Data {
    type Value = Arc<RwLock<Data>>;
}

#[tokio::main]
async fn main() -> Result<(), Error> {
    tracing_subscriber::fmt::init();

    let options: FrameworkOptions<Data, Error> = poise::FrameworkOptions {
        commands: vec![commands::subscribe(), commands::unsubscribe()],
        pre_command: |ctx| {
            Box::pin(async move {
                info!("Executing command {}...", ctx.command().qualified_name);
            })
        },
        post_command: |ctx| {
            Box::pin(async move {
                info!("Executed command {}!", ctx.command().qualified_name);
            })
        },
        event_handler: |_ctx, event, _framework, _data| {
            Box::pin(async move {
                info!(
                    "Got an event in event handler: {:?}",
                    event.snake_case_name()
                );
                Ok(())
            })
        },
        ..Default::default()
    };

    let framework = poise::Framework::builder()
        .setup(move |ctx, _ready, framework| {
            Box::pin(async move {
                info!("Logged in as {}", _ready.user.name);
                poise::builtins::register_globally(ctx, &framework.options().commands).await?;
                let subscribers = match tokio::fs::read_to_string("subscribers.json").await {
                    Ok(content) => match serde_json::from_str::<DashSet<UserId>>(&content) {
                        Ok(subscribers) => subscribers,
                        Err(e) => {
                            error!("Failed to parse subscribers.json: {}", e);
                            DashSet::new()
                        }
                    },
                    Err(e) => {
                        error!("Failed to read subscribers.json: {}", e);
                        DashSet::new()
                    }
                };
                ctx.data
                    .write()
                    .await
                    .insert::<Data>(Arc::new(RwLock::new(Data {
                        subscribers: subscribers,
                    })));
                tokio::spawn(commands::watch_matcha(
                    ctx.clone(),
                    match ctx.data.read().await.get::<Data>() {
                        Some(data) => data.clone(),
                        None => {
                            error!("Failed to retrieve Data from TypeMap");
                            return Err("Failed to retrieve Data".into());
                        }
                    },
                ));
                Ok(Data {
                    subscribers: DashSet::new(),
                })
            })
        })
        .options(options)
        .build();

    dotenv::dotenv().ok();
    let token =
        var("DISCORD_TOKEN").map_err(|e| format!("Missing `DISCORD_TOKEN` env var: {}", e))?;
    let intents = serenity::GatewayIntents::non_privileged();

    let mut client = serenity::ClientBuilder::new(token, intents)
        .framework(framework)
        .await?;

    client.start().await?;

    Ok(())
}
