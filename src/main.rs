use std::collections::HashMap;

use anyhow::{Context, Result};
use clap::{Parser, Subcommand, ValueEnum};
use reqwest::Client;
use scraper::{Html, Selector};
use serde::Deserialize;
use strum::{AsRefStr, EnumIter, IntoEnumIterator};
use std::cmp::Reverse;


const USER_AGENT: &str = "crabdocs";

#[derive(Clone, Copy, ValueEnum, AsRefStr)]
#[strum(serialize_all = "kebab-case")]
enum SortOrder {
    Relevance,
    Downloads,
    RecentDownloads,
    RecentUpdates,
    New,
}

#[derive(Clone, Copy, PartialEq, Eq, Hash, ValueEnum, AsRefStr, EnumIter)]
#[strum(serialize_all = "lowercase")]
enum ItemType {
    Module,
    Struct,
    Enum,
    Trait,
    Fn,
    Type,
    Const,
    Static,
    Macro,
    Union,
}

fn parse_item_type(link: &str) -> Option<ItemType> {
    ItemType::iter().find(|t| link.contains(&format!("{}.", t.as_ref())))
}

#[derive(Parser)]
struct Cli {
    #[command(subcommand)]
    command: Commands,
}

#[derive(Subcommand)]
enum Commands {
    /// Search for Rust crates by keywords on crates.io
    Search {
        /// Search query
        query: String,
        /// Number of results per page (max: 100)
        #[arg(short, long, default_value = "10")]
        per_page: u32,
        /// Sort order
        #[arg(short, long, value_enum, default_value = "relevance")]
        sort: SortOrder,
        /// Page number (1-indexed)
        #[arg(long, default_value = "1")]
        page: u32,
    },
    /// Search for items within a crate's documentation
    SearchItemsIn {
        /// Name of the crate
        crate_name: String,
        /// Search query
        query: String,
        /// Crate version
        #[arg(short, long, default_value = "latest")]
        version: String,
        /// Filter by item type
        #[arg(short = 't', long, value_enum)]
        item_type: Option<ItemType>,
    },
    /// Show README/overview content of the specified crate
    ShowReadme {
        /// Name of the crate
        crate_name: String,
        /// Crate version
        #[arg(short, long, default_value = "latest")]
        version: String,
    },
    /// Show summary of item types in a crate
    ShowItemsSummary {
        /// Name of the crate
        crate_name: String,
        /// Crate version
        #[arg(short, long, default_value = "latest")]
        version: String,
    },
    /// Show documentation of a specific item
    ShowItemDoc {
        /// Name of the crate
        crate_name: String,
        /// Type of the item
        #[arg(value_enum)]
        item_type: ItemType,
        /// Path to the item (e.g., `tokio::sync::Mutex`)
        item_path: String,
        /// Crate version
        #[arg(short, long, default_value = "latest")]
        version: String,
    },
}

#[derive(Deserialize)]
struct CratesResponse {
    crates: Vec<CrateInfo>,
    meta: CratesMeta,
}

#[derive(Deserialize)]
struct CratesMeta {
    total: u64,
    next_page: Option<String>,
}

#[derive(Deserialize)]
struct CrateInfo {
    name: String,
    description: Option<String>,
    downloads: u64,
    newest_version: String,
}

async fn fetch_html(client: &Client, url: &str) -> Result<Html> {
    let text = client
        .get(url)
        .header("User-Agent", USER_AGENT)
        .send()
        .await
        .context("Failed to send request")?
        .text()
        .await
        .context("Failed to read response body")?;
    Ok(Html::parse_document(&text))
}

fn docs_rs_url(crate_name: &str, version: &str, path: &str) -> String {
    format!("https://docs.rs/{crate_name}/{version}/{path}")
}

#[tokio::main]
async fn main() -> Result<()> {
    let cli = Cli::parse();

    match cli.command {
        Commands::Search { query, per_page, sort, page } => {
            search(&query, per_page.min(100), sort, page.max(1)).await
        }
        Commands::ShowReadme { crate_name, version } => {
            show_readme(&crate_name, &version).await
        }
        Commands::ShowItemDoc { crate_name, item_type, item_path, version } => {
            show_item_doc(&crate_name, item_type, &item_path, &version).await
        }
        Commands::ShowItemsSummary { crate_name, version } => {
            show_items_summary(&crate_name, &version).await
        }
        Commands::SearchItemsIn { crate_name, query, version, item_type } => {
            search_items_in(&crate_name, &query, &version, item_type).await
        }
    }
}

async fn search(query: &str, per_page: u32, sort: SortOrder, page: u32) -> Result<()> {
    let data: CratesResponse = Client::new()
        .get("https://crates.io/api/v1/crates")
        .query(&[
            ("q", query),
            ("per_page", &per_page.to_string()),
            ("sort", sort.as_ref()),
            ("page", &page.to_string()),
        ])
        .header("User-Agent", USER_AGENT)
        .send()
        .await
        .context("Failed to send request to crates.io")?
        .json()
        .await
        .context("Failed to parse crates.io response")?;

    println!("# Crate Search Results for \"{query}\"\n");

    if data.crates.is_empty() {
        println!("Total: 0 crates\n\nNo results found.");
        return Ok(());
    }

    let total_pages = data.meta.total.div_ceil(u64::from(per_page));
    let start = (page - 1) * per_page + 1;
    let end = start + u32::try_from(data.crates.len())? - 1;

    println!(
        "Total: {} crates | Page: {page}/{total_pages} | Showing: {start}-{end}\n",
        data.meta.total
    );

    for c in &data.crates {
        println!("## {} ({}, {} downloads)\n", c.name, c.newest_version, c.downloads);
        println!("{}", c.description.as_deref().unwrap_or("N/A"));
        println!("See `crabdocs show-readme {}`\n", c.name);
    }

    if data.meta.next_page.is_some() {
        println!("---\n\nUse `--page {}` to see more results.*", page + 1);
    }

    Ok(())
}

async fn show_readme(crate_name: &str, version: &str) -> Result<()> {
    let url = docs_rs_url(crate_name, version, &format!("{crate_name}/index.html"));
    let document = fetch_html(&Client::new(), &url).await?;

    let selector = Selector::parse(".rustdoc .docblock").unwrap();
    let content = document.select(&selector).next().map(|el| el.html());

    println!("# {crate_name} Documentation\n");
    match content {
        Some(html) => println!("{}", html2md::parse_html(&html)),
        None => println!("No documentation content found at {url}"),
    }
    Ok(())
}

async fn show_item_doc(crate_name: &str, item_type: ItemType, item_path: &str, version: &str) -> Result<()> {
    let type_str = item_type.as_ref();
    let path = if item_type == ItemType::Module {
        format!("{}/index.html", item_path.replace("::", "/"))
    } else {
        let (module, name) = item_path.rsplit_once("::").unwrap_or(("", item_path));
        format!("{}/{type_str}.{name}.html", module.replace("::", "/"))
    };

    let url = docs_rs_url(crate_name, version, &path);
    let document = fetch_html(&Client::new(), &url).await?;

    let selectors = ["#main-content", ".rustdoc .item-decl, .rustdoc .docblock"];
    let content = selectors.iter().find_map(|sel| {
        let html: String = document
            .select(&Selector::parse(sel).unwrap())
            .map(|el| el.html())
            .collect();
        (!html.is_empty()).then_some(html)
    });

    println!("# {item_path} ({type_str})\n");
    match content {
        Some(html) => {
            println!("Documentation URL: {url}\n");
            println!("{}", html2md::parse_html(&html));
        }
        None => println!("No documentation content found at {url}"),
    }
    Ok(())
}

async fn show_items_summary(crate_name: &str, version: &str) -> Result<()> {
    let url = docs_rs_url(crate_name, version, &format!("{crate_name}/all.html"));
    let document = fetch_html(&Client::new(), &url).await?;

    let selector = Selector::parse("#main-content a").unwrap();
    let mut counts: HashMap<ItemType, u32> = HashMap::new();

    for el in document.select(&selector) {
        if let Some(item_type) = el.value().attr("href").and_then(parse_item_type) {
            *counts.entry(item_type).or_default() += 1;
        }
    }

    println!("# Items in {crate_name}\n");

    if counts.is_empty() {
        println!("No items found.");
        return Ok(());
    }

    let total: u32 = counts.values().sum();
    println!("Total: {total} items\n");

    let mut sorted: Vec<_> = counts.into_iter().collect();
    sorted.sort_by_key(|(_, count)| Reverse(*count));

    for (item_type, count) in sorted {
        println!("- {}: {count}", item_type.as_ref());
    }
    Ok(())
}

async fn search_items_in(
    crate_name: &str,
    query: &str,
    version: &str,
    filter: Option<ItemType>,
) -> Result<()> {
    let url = docs_rs_url(crate_name, version, &format!("{crate_name}/all.html"));
    let document = fetch_html(&Client::new(), &url).await?;

    let selector = Selector::parse("#main-content a").unwrap();
    let query_lower = query.to_lowercase();

    let mut items: Vec<_> = document
        .select(&selector)
        .filter_map(|el| {
            let name = el.text().collect::<String>().trim().to_string();
            let href = el.value().attr("href")?;
            let item_type = parse_item_type(href)?;

            if name.is_empty() {
                return None;
            }
            if !query.is_empty() && !name.to_lowercase().contains(&query_lower) {
                return None;
            }
            if filter.is_some_and(|f| f != item_type) {
                return None;
            }

            Some((name, item_type))
        })
        .collect();

    items.dedup();

    let search_term = if query.is_empty() { "all items" } else { query };
    println!("# Search Results for \"{search_term}\" in {crate_name}\n");
    println!("Found {} items\n", items.len());

    if items.is_empty() {
        println!("No matching items found.");
    } else {
        for (name, item_type) in &items {
            let type_str = item_type.as_ref();
            println!("- {name} ({type_str})");
            println!("  `crabdocs show-item-doc {crate_name} {type_str} {crate_name}::{name}`\n");
        }
    }
    Ok(())
}
