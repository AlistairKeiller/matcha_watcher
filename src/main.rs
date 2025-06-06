mod commands;

use dashmap::DashSet;
use poise::{FrameworkOptions, serenity_prelude as serenity};
use scraper::Selector;
use std::{collections::HashSet, env::var, sync::Arc};
use tracing::{error, info};

use crate::commands::Site;

type Error = Box<dyn std::error::Error + Send + Sync>;
type Context<'a> = poise::Context<'a, Data, Error>;

pub struct Data {
    subscribers: Arc<DashSet<serenity::UserId>>,
}

#[tokio::main]
async fn main() -> Result<(), Error> {
    tracing_subscriber::fmt()
        .with_max_level(if cfg!(debug_assertions) {
            tracing::Level::DEBUG
        } else {
            tracing::Level::WARN
        })
        .init();

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
                let subscribers: Arc<DashSet<serenity::UserId>> =
                    Arc::new(match tokio::fs::read_to_string("subscribers.json").await {
                        Ok(content) => {
                            match serde_json::from_str::<DashSet<serenity::UserId>>(&content) {
                                Ok(subscribers) => subscribers,
                                Err(e) => {
                                    error!("Failed to parse subscribers.json: {}", e);
                                    DashSet::new()
                                }
                            }
                        }
                        Err(e) => {
                            error!("Failed to read subscribers.json: {}", e);
                            DashSet::new()
                        }
                    });
                let sites = [
                    Site {
                        url: "https://global.ippodo-tea.co.jp/collections/matcha",
                        product_card_selector: Selector::parse("li.m-product-card").unwrap(),
                        out_of_stock_filter: Some(Selector::parse("button.out-of-stock").unwrap()),
                        name_selector: Selector::parse(".m-product-card__name a").unwrap(),
                        href_selector: Selector::parse(".m-product-card__name a").unwrap(),
                        base_url: "https://global.ippodo-tea.co.jp",
                        matchas_in_stock: HashSet::new(),
                    },
                    Site {
                        url: "https://www.marukyu-koyamaen.co.jp/english/shop/products/catalog/matcha",
                        product_card_selector: Selector::parse("li.instock").unwrap(),
                        out_of_stock_filter: None,
                        name_selector: Selector::parse(".product-name h4").unwrap(),
                        href_selector: Selector::parse("a.woocommerce-loop-product__link").unwrap(),
                        base_url: "",
                        matchas_in_stock: HashSet::new(),
                    },
                    Site {
                        url: "https://www.marukyu-koyamaen.co.jp/english/shop/products/catalog/sweets",
                        product_card_selector: Selector::parse("li.instock").unwrap(),
                        out_of_stock_filter: None,
                        name_selector: Selector::parse(".product-name h4").unwrap(),
                        href_selector: Selector::parse("a.woocommerce-loop-product__link").unwrap(),
                        base_url: "",
                        matchas_in_stock: HashSet::new(),
                    },
                    Site {
                        url: "https://www.sazentea.com/en/products/c22-ceremonial-grade-matcha",
                        product_card_selector: Selector::parse("div.product").unwrap(),
                        out_of_stock_filter: None,
                        name_selector: Selector::parse(".product-name a").unwrap(),
                        href_selector: Selector::parse(".product-name a").unwrap(),
                        base_url: "https://www.sazentea.com",
                        matchas_in_stock: HashSet::new(),
                    }
                ];
                for site in sites {
                    tokio::spawn(commands::watch_matcha(ctx.clone(), subscribers.clone(), site));
                }
                Ok(Data {
                    subscribers: subscribers.clone(),
                })
            })
        })
        .options(options)
        .build();

    let token =
        var("DISCORD_TOKEN").map_err(|e| format!("Missing `DISCORD_TOKEN` env var: {}", e))?;
    let intents = serenity::GatewayIntents::non_privileged();

    let mut client = serenity::ClientBuilder::new(token, intents)
        .framework(framework)
        .await?;

    client.start().await?;

    Ok(())
}
