mod commands;
mod config;

use poise::{FrameworkOptions, serenity_prelude as serenity};
use serenity::all::UserId;
use serenity::prelude::TypeMapKey;
use std::{collections::HashSet, env::var, sync::Arc};
use tokio::sync::RwLock;

type Error = Box<dyn std::error::Error + Send + Sync>;
type Context<'a> = poise::Context<'a, Data, Error>;

pub struct Data {
    subscribers: RwLock<HashSet<UserId>>,
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
                tracing::info!("Executing command {}...", ctx.command().qualified_name);
            })
        },
        post_command: |ctx| {
            Box::pin(async move {
                tracing::info!("Executed command {}!", ctx.command().qualified_name);
            })
        },
        event_handler: |_ctx, event, _framework, _data| {
            Box::pin(async move {
                tracing::info!(
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
                tracing::info!("Logged in as {}", _ready.user.name);
                poise::builtins::register_globally(ctx, &framework.options().commands).await?;
                {
                    let mut data = ctx.data.write().await;
                    let subscribers = match tokio::fs::read_to_string("subscribers.json").await {
                        Ok(content) => {
                            serde_json::from_str::<HashSet<UserId>>(&content).unwrap_or_default()
                        }
                        Err(_) => HashSet::new(),
                    };
                    data.insert::<Data>(Arc::new(RwLock::new(Data {
                        subscribers: RwLock::new(subscribers),
                    })));
                }
                tokio::spawn(commands::watch_matcha(
                    ctx.clone(),
                    ctx.data.read().await.get::<Data>().unwrap().clone(),
                ));
                Ok(Data {
                    subscribers: RwLock::new(HashSet::new()),
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
