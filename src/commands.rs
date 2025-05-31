use std::collections::HashSet;
use std::sync::Arc;

use serenity::all::UserId;
use tokio::sync::RwLock;
use tracing::{error, info};

use crate::config::{Matcha, SITES, Site};
use crate::{Context, Data, Error};

async fn write_subscribers(subscribers: HashSet<UserId>) {
    let serialized = serde_json::to_string(&subscribers).expect("Failed to serialize subscribers");
    if let Err(e) = tokio::fs::write("subscribers.json", serialized).await {
        error!("Failed to write subscribers to file: {}", e);
    }
}

#[poise::command(slash_command)]
pub async fn subscribe(ctx: Context<'_>) -> Result<(), Error> {
    if ctx
        .data()
        .subscribers
        .read()
        .await
        .contains(&ctx.author().id)
    {
        ctx.say("You are already subscribed.").await?;
    } else {
        ctx.data().subscribers.write().await.insert(ctx.author().id);
        write_subscribers(ctx.data().subscribers.read().await.clone()).await;
        ctx.say("You are now subscribed.").await?;
    }
    Ok(())
}

#[poise::command(slash_command)]
pub async fn unsubscribe(ctx: Context<'_>) -> Result<(), Error> {
    if ctx
        .data()
        .subscribers
        .read()
        .await
        .contains(&ctx.author().id)
    {
        ctx.data()
            .subscribers
            .write()
            .await
            .remove(&ctx.author().id);
        write_subscribers(ctx.data().subscribers.read().await.clone()).await;
        ctx.say("You have been unsubscribed.").await?;
    } else {
        ctx.say("You are not currently subscribed.").await?;
    }
    Ok(())
}

pub async fn fetch_products(site: &Site) -> Result<HashSet<Matcha>, Error> {
    let client = reqwest::Client::new();
    let res = client
        .get(site.url)
        .header("User-Agent", "Mozilla/5.0 (Windows NT 10.0; Win64; x64) AppleWebKit/537.36 (KHTML, like Gecko) Chrome/91.0.4472.124 Safari/537.36")
        .header("Accept-Language", "en-US,en;q=0.9")
        .header("Accept", "text/html,application/xhtml+xml,application/xml;q=0.9,image/webp,*/*;q=0.8")
        .send()
        .await?
        .error_for_status()?;
    let document = scraper::Html::parse_document(&res.text().await?);
    let product_cards = document
        .select(&site.product_card_selector)
        .filter(|element| {
            site.out_of_stock_filter
                .as_ref()
                .is_none_or(|filter| element.select(filter).next().is_none())
        });
    let mut products = HashSet::new();
    for product_card in product_cards {
        let url = site.base_url.to_string()
            + product_card
                .select(&site.href_selector)
                .next()
                .and_then(|href| href.value().attr("href"))
                .ok_or_else(|| Error::from("Failed to find href"))?;

        let name = product_card
            .select(&site.name_selector)
            .next()
            .map(|name| name.inner_html().trim().to_string())
            .ok_or_else(|| Error::from("Failed to find name"))?;
        products.insert(Matcha { name, url });
    }
    Ok(products)
}

pub async fn watch_matcha(ctx: serenity::all::Context, data: Arc<RwLock<Data>>) {
    loop {
        for site in SITES.iter() {
            info!("checking site {}", site.url);
            let products = match fetch_products(site).await {
                Ok(products) => products,
                Err(e) => {
                    error!("Error checking site {}: {}", site.url, e);
                    continue;
                }
            };
            if products == *site.matchas_in_stock.read().await {
                info!("No changes found on site {}", site.url);
                continue;
            }
            let matchas_in_stock = site.matchas_in_stock.read().await.clone();
            let added = products.difference(&matchas_in_stock);
            let removed = matchas_in_stock.difference(&products);
            *site.matchas_in_stock.write().await = products.clone();
            info!(
                "Changes detected for site {}. Added: {:?}, Removed: {:?}",
                site.url, added, removed
            );
            let mut product_message = String::new();
            if added.clone().next().is_some() {
                product_message.push_str(&format!(
                    "🟢 Now in stock: {}\n",
                    added
                        .map(|p| format!("[{}]({})", p.name, p.url))
                        .collect::<Vec<_>>()
                        .join(", ")
                ));
            }
            if removed.clone().next().is_some() {
                product_message.push_str(&format!(
                    "🔴 Out of stock: {}\n",
                    removed
                        .map(|p| format!("[{}]({})", p.name, p.url))
                        .collect::<Vec<_>>()
                        .join(", ")
                ));
            }
            for user in data.read().await.subscribers.read().await.iter() {
                if let Err(e) = user
                    .create_dm_channel(&ctx)
                    .await
                    .unwrap()
                    .say(&ctx, product_message.clone())
                    .await
                {
                    error!("Failed to send message to user {}: {}", user, e);
                }
            }
        }
    }
}
