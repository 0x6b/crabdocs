use anyhow::{Context, Result};
use clap::{Parser, Subcommand, ValueEnum};
use scraper::{Html, Selector};
use serde::Deserialize;

#[derive(Debug, Clone, Copy, ValueEnum)]
enum SortOrder {
    Relevance,
    Downloads,
    RecentDownloads,
    RecentUpdates,
    New,
}

impl SortOrder {
    fn as_api_param(&self) -> &'static str {
        match self {
            SortOrder::Relevance => "relevance",
            SortOrder::Downloads => "downloads",
            SortOrder::RecentDownloads => "recent-downloads",
            SortOrder::RecentUpdates => "recent-updates",
            SortOrder::New => "new",
        }
    }
}

#[derive(Debug, Clone, Copy, ValueEnum)]
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

impl ItemType {
    fn as_url_segment(&self) -> &'static str {
        match self {
            ItemType::Module => "module",
            ItemType::Struct => "struct",
            ItemType::Enum => "enum",
            ItemType::Trait => "trait",
            ItemType::Fn => "fn",
            ItemType::Type => "type",
            ItemType::Const => "const",
            ItemType::Static => "static",
            ItemType::Macro => "macro",
            ItemType::Union => "union",
        }
    }
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
        /// Search keywords for finding relevant crates
        query: String,
        /// Number of results per page (default: 10, max: 100)
        #[arg(short, long, default_value = "10")]
        per_page: u32,
        /// Sort order
        #[arg(short, long, value_enum, default_value = "relevance")]
        sort: SortOrder,
        /// Page number (1-indexed, default: 1)
        #[arg(long, default_value = "1")]
        page: u32,
    },
    /// Get README/overview content of the specified crate
    Readme {
        /// Name of the crate
        crate_name: String,
        /// Specific version (defaults to latest)
        #[arg(short, long, default_value = "latest")]
        version: String,
    },
    /// Get documentation content of a specific item (module, struct, trait, enum, fn, etc.)
    Item {
        /// Name of the crate
        crate_name: String,
        /// Type of item
        #[arg(value_enum)]
        item_type: ItemType,
        /// Full path of the item including module (e.g. wasmtime::component::Component)
        item_path: String,
        /// Specific version (defaults to latest)
        #[arg(short, long, default_value = "latest")]
        version: String,
    },
    /// List item types available in a crate (structs, traits, functions, etc.)
    Items {
        /// Name of the crate
        crate_name: String,
        /// Specific version (defaults to latest)
        #[arg(short, long, default_value = "latest")]
        version: String,
    },
    /// Search for items within a crate's documentation
    SearchIn {
        /// Name of the crate to search
        crate_name: String,
        /// Search keyword (trait name, struct name, function name, etc.)
        query: String,
        /// Specific version (defaults to latest)
        #[arg(short, long, default_value = "latest")]
        version: String,
        /// Filter by item type
        #[arg(short = 't', long, value_enum)]
        item_type: Option<ItemType>,
    },
}

#[derive(Debug, Deserialize)]
struct CratesResponse {
    crates: Vec<CrateInfo>,
    meta: CratesMeta,
}

#[derive(Debug, Deserialize)]
struct CratesMeta {
    total: u64,
    next_page: Option<String>,
}

#[derive(Debug, Deserialize)]
struct CrateInfo {
    name: String,
    description: Option<String>,
    downloads: u64,
    newest_version: String,
    documentation: Option<String>,
}

struct SearchResult {
    name: String,
    item_type: String,
    link: String,
}

#[tokio::main]
async fn main() -> Result<()> {
    let cli = Cli::parse();

    match cli.command {
        Commands::Search {
            query,
            per_page,
            sort,
            page,
        } => search_crates(&query, per_page.min(100), sort, page.max(1)).await?,
        Commands::Readme {
            crate_name,
            version,
        } => get_readme(&crate_name, &version).await?,
        Commands::Item {
            crate_name,
            item_type,
            item_path,
            version,
        } => get_item(&crate_name, item_type, &item_path, &version).await?,
        Commands::Items {
            crate_name,
            version,
        } => list_items(&crate_name, &version).await?,
        Commands::SearchIn {
            crate_name,
            query,
            version,
            item_type,
        } => search_in_crate(&crate_name, &query, &version, item_type).await?,
    }

    Ok(())
}

async fn search_crates(query: &str, per_page: u32, sort: SortOrder, page: u32) -> Result<()> {
    let client = reqwest::Client::new();
    let response = client
        .get("https://crates.io/api/v1/crates")
        .query(&[
            ("q", query),
            ("per_page", &per_page.to_string()),
            ("sort", sort.as_api_param()),
            ("page", &page.to_string()),
        ])
        .header("User-Agent", "docs-rs-cli")
        .send()
        .await
        .context("Failed to send request to crates.io")?;

    let data: CratesResponse = response
        .json()
        .await
        .context("Failed to parse crates.io response")?;

    println!("# Crate Search Results for \"{}\"\n", query);

    if data.crates.is_empty() {
        println!("**Total:** 0 crates\n");
        println!("No results found.");
        return Ok(());
    }

    let total_pages = (data.meta.total as f64 / per_page as f64).ceil() as u64;
    let start_index = (page - 1) * per_page + 1;
    let end_index = start_index + data.crates.len() as u32 - 1;

    println!(
        "**Total:** {} crates | **Page:** {}/{} | **Showing:** {}-{}\n",
        data.meta.total, page, total_pages, start_index, end_index
    );

    for crate_info in data.crates {
        println!("## {} ({})\n", crate_info.name, crate_info.newest_version);
        println!(
            "**Description:** {}\n",
            crate_info.description.as_deref().unwrap_or("No description available")
        );
        println!("**Downloads:** {}\n", crate_info.downloads);
        println!(
            "**Documentation:** {}\n",
            crate_info.documentation.as_deref().unwrap_or("N/A")
        );
        println!("---\n");
    }

    if data.meta.next_page.is_some() {
        println!("*Use `--page {}` to see more results.*", page + 1);
    }

    Ok(())
}

async fn get_readme(crate_name: &str, version: &str) -> Result<()> {
    let url = format!(
        "https://docs.rs/{}/{}/{}/index.html",
        crate_name, version, crate_name
    );

    let client = reqwest::Client::new();
    let response = client
        .get(&url)
        .header("User-Agent", "docs-rs-cli")
        .send()
        .await
        .context("Failed to fetch documentation")?;

    let html = response
        .text()
        .await
        .context("Failed to read response body")?;

    let document = Html::parse_document(&html);
    let docblock_selector = Selector::parse(".rustdoc .docblock").unwrap();

    let content = document
        .select(&docblock_selector)
        .next()
        .map(|el| el.html())
        .unwrap_or_default();

    if content.is_empty() {
        println!("# {} Documentation\n", crate_name);
        println!("No documentation content found at {}", url);
        return Ok(());
    }

    let markdown = html2md::parse_html(&content);

    println!("# {} Documentation\n", crate_name);
    println!("{}", markdown);

    Ok(())
}

async fn get_item(crate_name: &str, item_type: ItemType, item_path: &str, version: &str) -> Result<()> {
    let type_segment = item_type.as_url_segment();
    let url = if matches!(item_type, ItemType::Module) {
        format!(
            "https://docs.rs/{}/{}/{}/index.html",
            crate_name,
            version,
            item_path.replace("::", "/")
        )
    } else {
        let path_parts: Vec<&str> = item_path.split("::").collect();
        let item_name = path_parts.last().unwrap_or(&"");
        let module_path = path_parts[..path_parts.len() - 1].join("/");
        format!(
            "https://docs.rs/{}/{}/{}/{}.{}.html",
            crate_name, version, module_path, type_segment, item_name
        )
    };

    let client = reqwest::Client::new();
    let response = client
        .get(&url)
        .header("User-Agent", "docs-rs-cli")
        .send()
        .await
        .context("Failed to fetch documentation")?;

    let html = response
        .text()
        .await
        .context("Failed to read response body")?;

    let document = Html::parse_document(&html);

    let main_content_selector = Selector::parse("#main-content").unwrap();
    let item_decl_selector = Selector::parse(".rustdoc .item-decl").unwrap();
    let docblock_selector = Selector::parse(".rustdoc .docblock").unwrap();

    let content = if let Some(main_content) = document.select(&main_content_selector).next() {
        main_content.html()
    } else {
        let mut content = String::new();
        if let Some(item_decl) = document.select(&item_decl_selector).next() {
            content.push_str(&item_decl.html());
        }
        if let Some(docblock) = document.select(&docblock_selector).next() {
            content.push_str(&docblock.html());
        }
        content
    };

    if content.is_empty() {
        println!("# {} ({})\n", item_path, type_segment);
        println!("No documentation content found at {}", url);
        return Ok(());
    }

    let markdown = html2md::parse_html(&content);

    println!("# {} ({})\n", item_path, type_segment);
    println!("**Documentation URL:** {}\n", url);
    println!("{}", markdown);

    Ok(())
}

async fn list_items(crate_name: &str, version: &str) -> Result<()> {
    let url = format!(
        "https://docs.rs/{}/{}/{}/all.html",
        crate_name, version, crate_name
    );

    let client = reqwest::Client::new();
    let response = client
        .get(&url)
        .header("User-Agent", "docs-rs-cli")
        .send()
        .await
        .context("Failed to fetch all.html")?;

    let html = response
        .text()
        .await
        .context("Failed to read response body")?;

    let document = Html::parse_document(&html);
    let link_selector = Selector::parse("#main-content a").unwrap();

    let mut counts = std::collections::HashMap::new();

    for element in document.select(&link_selector) {
        let item_link = element.value().attr("href").unwrap_or_default();

        let item_type = if item_link.contains("struct.") {
            "struct"
        } else if item_link.contains("trait.") {
            "trait"
        } else if item_link.contains("fn.") {
            "fn"
        } else if item_link.contains("enum.") {
            "enum"
        } else if item_link.contains("type.") {
            "type"
        } else if item_link.contains("const.") {
            "const"
        } else if item_link.contains("static.") {
            "static"
        } else if item_link.contains("macro.") {
            "macro"
        } else if item_link.contains("union.") {
            "union"
        } else {
            continue;
        };

        *counts.entry(item_type).or_insert(0) += 1;
    }

    println!("# Items in {}\n", crate_name);

    if counts.is_empty() {
        println!("No items found.");
        return Ok(());
    }

    let total: u32 = counts.values().sum();
    println!("**Total:** {} items\n", total);

    // Sort by count descending
    let mut sorted: Vec<_> = counts.into_iter().collect();
    sorted.sort_by(|a, b| b.1.cmp(&a.1));

    for (item_type, count) in sorted {
        println!("- **{}:** {}", item_type, count);
    }

    Ok(())
}

async fn search_in_crate(
    crate_name: &str,
    query: &str,
    version: &str,
    item_type_filter: Option<ItemType>,
) -> Result<()> {
    let url = format!(
        "https://docs.rs/{}/{}/{}/all.html",
        crate_name, version, crate_name
    );

    let client = reqwest::Client::new();
    let response = client
        .get(&url)
        .header("User-Agent", "docs-rs-cli")
        .send()
        .await
        .context("Failed to fetch all.html")?;

    let html = response
        .text()
        .await
        .context("Failed to read response body")?;

    let document = Html::parse_document(&html);
    let link_selector = Selector::parse("#main-content a").unwrap();

    let mut items: Vec<SearchResult> = Vec::new();

    for element in document.select(&link_selector) {
        let item_name = element.text().collect::<String>().trim().to_string();
        let item_link = element.value().attr("href").unwrap_or_default();

        if item_name.is_empty() || item_link.is_empty() {
            continue;
        }

        let item_type = if item_link.contains("struct.") {
            "struct"
        } else if item_link.contains("trait.") {
            "trait"
        } else if item_link.contains("fn.") {
            "fn"
        } else if item_link.contains("enum.") {
            "enum"
        } else if item_link.contains("type.") {
            "type"
        } else if item_link.contains("const.") {
            "const"
        } else if item_link.contains("static.") {
            "static"
        } else if item_link.contains("macro.") {
            "macro"
        } else if item_link.contains("union.") {
            "union"
        } else {
            continue;
        };

        let matches_query = query.is_empty()
            || item_name.to_lowercase().contains(&query.to_lowercase());

        let matches_type = item_type_filter
            .map(|f| item_type == f.as_url_segment())
            .unwrap_or(true);

        if matches_query && matches_type {
            let full_link = if item_link.starts_with("http") {
                item_link.to_string()
            } else {
                format!(
                    "https://docs.rs/{}/{}/{}/{}",
                    crate_name, version, crate_name, item_link
                )
            };

            items.push(SearchResult {
                name: item_name,
                item_type: item_type.to_string(),
                link: full_link,
            });
        }
    }

    // Deduplicate by name and type
    items.dedup_by(|a, b| a.name == b.name && a.item_type == b.item_type);

    let search_term = if query.is_empty() { "all items" } else { query };
    println!("# Search Results for \"{}\" in {}\n", search_term, crate_name);
    println!("Found {} items\n", items.len());

    if items.is_empty() {
        println!("No matching items found.");
    } else {
        for item in items {
            println!("## {} ({})\n", item.name, item.item_type);
            println!("**Link:** {}\n", item.link);
            println!("---\n");
        }
    }

    Ok(())
}
