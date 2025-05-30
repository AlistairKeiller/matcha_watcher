mod commands;

use ::serenity::all::UserId;
use poise::{FrameworkOptions, serenity_prelude as serenity};
use std::{collections::HashSet, env::var, sync::Mutex};

type Error = Box<dyn std::error::Error + Send + Sync>;
type Context<'a> = poise::Context<'a, Data, Error>;

pub struct Data {
    subscribers: Mutex<HashSet<UserId>>,
}

#[tokio::main]
async fn main() {
    tracing_subscriber::fmt::init();

    let options: FrameworkOptions<Data, Error> = poise::FrameworkOptions {
        commands: vec![commands::subscribe(), commands::unsubscribe()],
        pre_command: |ctx| {
            Box::pin(async move {
                println!("Executing command {}...", ctx.command().qualified_name);
            })
        },
        post_command: |ctx| {
            Box::pin(async move {
                println!("Executed command {}!", ctx.command().qualified_name);
            })
        },
        event_handler: |_ctx, event, _framework, _data| {
            Box::pin(async move {
                println!(
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
                println!("Logged in as {}", _ready.user.name);
                poise::builtins::register_globally(ctx, &framework.options().commands).await?;
                Ok(Data {
                    subscribers: Mutex::new(HashSet::new()),
                })
            })
        })
        .options(options)
        .build();

    dotenv::dotenv().expect("Failed to load .env file");
    let token = var("DISCORD_TOKEN")
        .expect("Missing `DISCORD_TOKEN` env var, see README for more information.");
    let intents =
        serenity::GatewayIntents::non_privileged() | serenity::GatewayIntents::MESSAGE_CONTENT;

    let client = serenity::ClientBuilder::new(token, intents)
        .framework(framework)
        .await;

    client.unwrap().start().await.unwrap()
}
