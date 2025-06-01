use scraper::Selector;
use std::{collections::HashSet, sync::LazyLock};
use tokio::sync::RwLock;

pub struct Site {
    pub url: &'static str,
    pub product_card_selector: Selector,
    pub out_of_stock_filter: Option<Selector>,
    pub name_selector: Selector,
    pub href_selector: Selector,
    pub base_url: &'static str,
    pub matchas_in_stock: RwLock<HashSet<Matcha>>,
}

#[derive(PartialEq, Eq, Hash, Debug, Clone)]
pub struct Matcha {
    pub name: String,
    pub url: String,
}

pub static SITES: LazyLock<[Site; 3]> = LazyLock::new(|| {
    [
        Site {
            url: "https://global.ippodo-tea.co.jp/collections/matcha",
            product_card_selector: Selector::parse("li.m-product-card").unwrap(),
            out_of_stock_filter: Some(Selector::parse("button.out-of-stock").unwrap()),
            name_selector: Selector::parse(".m-product-card__name a").unwrap(),
            href_selector: Selector::parse(".m-product-card__name a").unwrap(),
            base_url: "https://global.ippodo-tea.co.jp",
            matchas_in_stock: RwLock::new(HashSet::new()),
        },
        Site {
            url: "https://www.marukyu-koyamaen.co.jp/english/shop/products/catalog/matcha",
            product_card_selector: Selector::parse("li.instock").unwrap(),
            out_of_stock_filter: None,
            name_selector: Selector::parse(".product-name h4").unwrap(),
            href_selector: Selector::parse("a.woocommerce-loop-product__link").unwrap(),
            base_url: "",
            matchas_in_stock: RwLock::new(HashSet::new()),
        },
        Site {
            url: "https://www.marukyu-koyamaen.co.jp/english/shop/products/catalog/sweets",
            product_card_selector: Selector::parse("li.instock").unwrap(),
            out_of_stock_filter: None,
            name_selector: Selector::parse(".product-name h4").unwrap(),
            href_selector: Selector::parse("a.woocommerce-loop-product__link").unwrap(),
            base_url: "",
            matchas_in_stock: RwLock::new(HashSet::new()),
        },
    ]
});
