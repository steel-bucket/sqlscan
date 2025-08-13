pub mod bing;
pub mod common;
pub mod google;
pub mod yahoo;

pub use bing::BingSearch;
pub use google::GoogleSearch;
pub use yahoo::YahooSearch;

use std::collections::HashSet;
use thiserror::Error;

#[derive(Debug, Clone)]
pub struct SearchResult {
    pub url: String,
    pub title: Option<String>,
    pub snippet: Option<String>,
}

#[derive(Error, Debug)]
pub enum SearchError {
    #[error("HTTP request failed: {0}")]
    HttpError(#[from] reqwest::Error),

    #[error("HTML parsing error: {0}")]
    ParseError(String),

    #[error("URL parsing error: {0}")]
    UrlError(#[from] url::ParseError),

    #[error("Invalid parameter: {0}")]
    InvalidParameter(String),
}

pub type SearchResults = Vec<String>;

pub fn deduplicate_urls(urls: Vec<String>) -> Vec<String> {
    let mut seen = HashSet::new();
    let mut result = Vec::new();

    for url in urls {
        if seen.insert(url.clone()) {
            result.push(url);
        }
    }

    result
}

pub fn filter_google_urls(url: &str) -> Option<String> {
    use url::Url;

    if let Ok(parsed) = Url::parse(url) {
        // Valid results are absolute URLs not pointing to a Google domain
        if let Some(host) = parsed.host_str() {
            if !host.contains("google") {
                return Some(url.to_string());
            }
        }

        // Handle Google redirect URLs
        if url.starts_with("/url?") {
            if let Ok(query_pairs) = Url::parse(&format!("http://example.com{}", url)) {
                for (key, value) in query_pairs.query_pairs() {
                    if key == "q" {
                        if let Ok(decoded_url) = Url::parse(&value) {
                            if let Some(host) = decoded_url.host_str() {
                                if !host.contains("google") {
                                    return Some(value.to_string());
                                }
                            }
                        }
                    }
                }
            }
        }
    }

    None
}
