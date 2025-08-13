// src/main.rs
use clap::{Arg, Command};
use std::process;
use tokio;

mod crawler_bench;
mod scanner_bench;
mod searchlibs;
mod setup;
mod sql_vuln;
mod tests;
mod web;

use sql_vuln::crawler::WebCrawler;
use sql_vuln::reverse_ip::ReverseIpLookup;
use sql_vuln::scanner::SqlInjectionScanner;
use sql_vuln::server_info::ServerInfoChecker;
use sql_vuln::{SqlErrorChecker, StdUtils};
use web::search::{BingSearchEngine, GoogleSearchEngine, SearchEngine, YahooSearchEngine};

const VERSION: &str = "2.0";
const AUTHOR: &str = "Ghost (Rust Port)";

#[tokio::main]
async fn main() {
    let matches = build_cli().get_matches();

    // Handle dork + engine search
    if let (Some(dork), Some(engine)) = (
        matches.get_one::<String>("dork"),
        matches.get_one::<String>("engine"),
    ) {
        handle_dork_search(dork, engine, &matches).await;
    }
    // Handle target + reverse IP
    else if let Some(target) = matches.get_one::<String>("target") {
        if matches.get_flag("reverse") {
            handle_reverse_ip_scan(target, &matches).await;
        } else {
            handle_single_scan(target, &matches).await;
        }
    }
    // Show help if no valid combination provided
    else {
        build_cli().print_help().unwrap();
        println!("\n\nExamples:");
        println!(
            "  {} -d \"inurl:php?id=\" -e google -p 50",
            env!("CARGO_PKG_NAME")
        );
        println!("  {} -t \"example.com\" -r", env!("CARGO_PKG_NAME"));
        println!(
            "  {} -t \"http://example.com/page.php?id=1\"",
            env!("CARGO_PKG_NAME")
        );
    }
}

fn build_cli() -> Command {
    Command::new("sqlscan")
        .version(VERSION)
        .author(AUTHOR)
        .about("Massive SQL injection vulnerability scanner")
        .arg(
            Arg::new("dork")
                .short('d')
                .long("dork")
                .value_name("DORK")
                .help("SQL injection dork (e.g., inurl:example)")
                .action(clap::ArgAction::Set),
        )
        .arg(
            Arg::new("engine")
                .short('e')
                .long("engine")
                .value_name("ENGINE")
                .help("Search engine [google, bing, yahoo]")
                .action(clap::ArgAction::Set),
        )
        .arg(
            Arg::new("pages")
                .short('p')
                .long("pages")
                .value_name("NUMBER")
                .help("Number of pages to search")
                .default_value("10")
                .value_parser(clap::value_parser!(usize)),
        )
        .arg(
            Arg::new("target")
                .short('t')
                .long("target")
                .value_name("URL")
                .help("Scan target website")
                .action(clap::ArgAction::Set),
        )
        .arg(
            Arg::new("reverse")
                .short('r')
                .long("reverse")
                .help("Reverse domain lookup")
                .action(clap::ArgAction::SetTrue),
        )
        .arg(
            Arg::new("output")
                .short('o')
                .long("output")
                .value_name("FILE")
                .help("Output result to JSON file")
                .action(clap::ArgAction::Set),
        )
        .arg(
            Arg::new("save-searches")
                .short('s')
                .long("save-searches")
                .help("Save search results even if no vulnerabilities found")
                .action(clap::ArgAction::SetTrue),
        )
}

async fn handle_dork_search(dork: &str, engine: &str, matches: &clap::ArgMatches) {
    let pages = *matches.get_one::<usize>("pages").unwrap();

    StdUtils::stdout("Searching for websites with given dork");

    // Get search results based on engine
    let websites = match engine.to_lowercase().as_str() {
        "google" => {
            let google = GoogleSearchEngine::new();
            match google.search(dork, pages).await {
                Ok(urls) => urls,
                Err(e) => {
                    StdUtils::stderr(&format!("Google search failed: {}", e));
                    process::exit(1);
                }
            }
        }
        "bing" => {
            let bing = BingSearchEngine::new();
            match bing.search(dork, pages).await {
                Ok(urls) => urls,
                Err(e) => {
                    StdUtils::stderr(&format!("Bing search failed: {}", e));
                    process::exit(1);
                }
            }
        }
        "yahoo" => {
            let yahoo = YahooSearchEngine::new();
            match yahoo.search(dork, pages).await {
                Ok(urls) => urls,
                Err(e) => {
                    StdUtils::stderr(&format!("Yahoo search failed: {}", e));
                    process::exit(1);
                }
            }
        }
        _ => {
            StdUtils::stderr("Invalid search engine. Use: google, bing, yahoo");
            process::exit(1);
        }
    };

    StdUtils::stdout(&format!("{} websites found", websites.len()));

    // Save searches if requested and no vulnerabilities found
    if matches.get_flag("save-searches") && websites.is_empty() {
        if let Err(e) = StdUtils::dump(&websites, "searches.txt") {
            StdUtils::stderr(&format!("Failed to save searches: {}", e));
        } else {
            StdUtils::stdout("Saved as searches.txt");
        }
        return;
    }

    // Scan for vulnerabilities
    let scanner = SqlInjectionScanner::new();
    let vulnerables = scanner.scan(websites.clone()).await;

    if vulnerables.is_empty() {
        if matches.get_flag("save-searches") {
            if let Err(e) = StdUtils::dump(&websites, "searches.txt") {
                StdUtils::stderr(&format!("Failed to save searches: {}", e));
            } else {
                StdUtils::stdout("Saved as searches.txt");
            }
        }
        StdUtils::stdout("No SQL injection vulnerabilities found");
        return;
    }

    StdUtils::stdout("Scanning server information");

    // Get server info for vulnerable URLs
    let vulnerable_urls: Vec<String> = vulnerables.iter().map(|(url, _)| url.clone()).collect();
    let server_data = ServerInfoChecker::check(vulnerable_urls).await;

    // Combine vulnerability and server data
    let mut table_data = Vec::new();
    for ((url, db), server_info) in vulnerables.iter().zip(server_data.iter()) {
        table_data.push((
            url.clone(),
            db.clone(),
            server_info.server.clone(),
            server_info.language.clone(),
        ));
    }

    StdUtils::full_print(&table_data);

    // Save to JSON if requested
    if let Some(output_file) = matches.get_one::<String>("output") {
        if let Err(e) = StdUtils::dump_json(&table_data, output_file) {
            StdUtils::stderr(&format!("Failed to save JSON: {}", e));
        } else {
            StdUtils::stdout(&format!("Dumped result into {}", output_file));
        }
    }
}

async fn handle_reverse_ip_scan(target: &str, matches: &clap::ArgMatches) {
    StdUtils::stdout(&format!("Finding domains with same server as {}", target));

    let domains = match ReverseIpLookup::reverse_ip(target).await {
        Ok(domains) => domains,
        Err(e) => {
            StdUtils::stderr(&format!("Reverse IP lookup failed: {}", e));
            process::exit(1);
        }
    };

    if domains.is_empty() {
        StdUtils::stdout("No domain found with reverse IP lookup");
        return;
    }

    StdUtils::stdout(&format!("Found {} websites", domains.len()));

    // Ask whether user wants to save domains
    StdUtils::stdout("Scanning multiple websites with crawling will take long");
    let save_option = StdUtils::stdin(
        "Do you want to save domains? [Y/N]",
        &["Y", "N"],
        true,
        false,
    );

    if save_option == "Y" {
        if let Err(e) = StdUtils::dump(&domains, "domains.txt") {
            StdUtils::stderr(&format!("Failed to save domains: {}", e));
        } else {
            StdUtils::stdout("Saved as domains.txt");
        }
    }

    // Ask whether user wants to start crawling
    let crawl_option = StdUtils::stdin(
        "Do you want to start crawling? [Y/N]",
        &["Y", "N"],
        true,
        false,
    );

    if crawl_option == "N" {
        return;
    }

    // Scan each domain
    let scanner = SqlInjectionScanner::new();
    let mut all_vulnerables = Vec::new();

    for domain in domains {
        if let Some(vulnerables) = single_scan(&domain, &scanner).await {
            all_vulnerables.extend(vulnerables);
        }
    }

    StdUtils::stdout("Finished scanning all reverse domains");

    if all_vulnerables.is_empty() {
        StdUtils::stdout("No vulnerable websites from reverse domains");
        return;
    }

    StdUtils::stdout("Scanning server information");

    // Get server info
    let vulnerable_urls: Vec<String> = all_vulnerables.iter().map(|(url, _)| url.clone()).collect();
    let server_data = ServerInfoChecker::check(vulnerable_urls).await;

    // Combine data
    let mut table_data = Vec::new();
    for ((url, db), server_info) in all_vulnerables.iter().zip(server_data.iter()) {
        table_data.push((
            url.clone(),
            db.clone(),
            server_info.server.clone(),
            server_info.language.clone(),
        ));
    }

    StdUtils::full_print(&table_data);

    // Save to JSON if requested
    if let Some(output_file) = matches.get_one::<String>("output") {
        if let Err(e) = StdUtils::dump_json(&table_data, output_file) {
            StdUtils::stderr(&format!("Failed to save JSON: {}", e));
        } else {
            StdUtils::stdout(&format!("Dumped result into {}", output_file));
        }
    }
}

async fn handle_single_scan(target: &str, matches: &clap::ArgMatches) {
    let scanner = SqlInjectionScanner::new();

    if let Some(vulnerables) = single_scan(target, &scanner).await {
        // Get server info for the target
        StdUtils::stdout("Getting server info of domain can take a few minutes");
        let server_data = ServerInfoChecker::check(vec![target.to_string()]).await;

        // Print server info
        StdUtils::print_server_info(&server_data);
        println!(); // Space between tables

        // Print vulnerabilities
        StdUtils::normal_print(&vulnerables);

        // Save to JSON if requested
        if let Some(output_file) = matches.get_one::<String>("output") {
            let table_data: Vec<_> = vulnerables
                .iter()
                .zip(server_data.iter())
                .map(|((url, db), server_info)| {
                    (
                        url.clone(),
                        db.clone(),
                        server_info.server.clone(),
                        server_info.language.clone(),
                    )
                })
                .collect();

            if let Err(e) = StdUtils::dump_json(&table_data, output_file) {
                StdUtils::stderr(&format!("Failed to save JSON: {}", e));
            } else {
                StdUtils::stdout(&format!("Dumped result into {}", output_file));
            }
        }
    } else {
        StdUtils::stdout("No SQL injection vulnerabilities found");
    }
}

async fn single_scan(url: &str, scanner: &SqlInjectionScanner) -> Option<Vec<(String, String)>> {
    let parsed_url = match url::Url::parse(url) {
        Ok(parsed) => parsed,
        Err(_) => {
            // Try with http:// prefix
            match url::Url::parse(&format!("http://{}", url)) {
                Ok(parsed) => parsed,
                Err(e) => {
                    StdUtils::stderr(&format!("Invalid URL: {}", e));
                    return None;
                }
            }
        }
    };

    let full_url = parsed_url.to_string();

    // If URL has query parameters, scan directly first
    if !parsed_url.query().unwrap_or("").is_empty() {
        let result = scanner.scan(vec![full_url.clone()]).await;
        if !result.is_empty() {
            return Some(result);
        }

        println!(); // Move to new line
        StdUtils::stdout("No SQL injection vulnerability found");
        let option = StdUtils::stdin(
            "Do you want to crawl and continue scanning? [Y/N]",
            &["Y", "N"],
            true,
            false,
        );

        if option == "N" {
            return None;
        }
    }

    // Crawl and scan the links
    StdUtils::stdout(&format!("Going to crawl {}", full_url));

    let mut crawler = WebCrawler::new();
    let urls = match crawler.crawl(&full_url).await {
        Ok(urls) => urls,
        Err(e) => {
            StdUtils::stderr(&format!("Crawling failed: {}", e));
            return None;
        }
    };

    if urls.is_empty() {
        StdUtils::stdout("Found no suitable URLs to test SQLi");
        return None;
    }

    StdUtils::stdout(&format!("Found {} URLs from crawling", urls.len()));
    let vulnerables = scanner.scan(urls).await;

    if vulnerables.is_empty() {
        StdUtils::stdout("No SQL injection vulnerability found");
        None
    } else {
        Some(vulnerables)
    }
}
