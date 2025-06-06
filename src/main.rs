mod commands;

use dashmap::DashSet;
use poise::{FrameworkOptions, serenity_prelude as serenity};
use serenity::all::UserId;
use std::{env::var, sync::Arc};
use tracing::{error, info};

type Error = Box<dyn std::error::Error + Send + Sync>;
type Context<'a> = poise::Context<'a, Data, Error>;

pub struct Data {
    subscribers: Arc<DashSet<UserId>>,
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
                let subscribers: Arc<DashSet<UserId>> =
                    Arc::new(match tokio::fs::read_to_string("subscribers.json").await {
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
                    });
                tokio::spawn(commands::watch_matcha(ctx.clone(), subscribers.clone()));
                Ok(Data {
                    subscribers: subscribers.clone(),
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
