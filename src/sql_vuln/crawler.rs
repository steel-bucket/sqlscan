// src/sql_vuln/crawler.rs
use crate::web::useragents::UserAgents;
use regex::Regex;
use reqwest::Client;
use scraper::{Html, Selector};
use std::collections::HashSet;
use url::Url;

pub struct WebCrawler {
    client: Client,
    visited: HashSet<String>,
    max_depth: usize,
    parameter_regex: Regex,
}

impl WebCrawler {
    pub fn new() -> Self {
        let user_agents = UserAgents::new();
        let client = Client::builder()
            .user_agent(user_agents.get_random())
            .build()
            .expect("Failed to create HTTP client");

        let parameter_regex = Regex::new(r"(.*?)(.php\?|.asp\?|.aspx\?|.jsp\?)(.*?)=(.*)")
            .expect("Failed to compile regex");

        Self {
            client,
            visited: HashSet::new(),
            max_depth: 1,
            parameter_regex,
        }
    }

    pub fn set_max_depth(&mut self, depth: usize) {
        self.max_depth = depth;
    }

    pub async fn crawl(&mut self, url: &str) -> Result<Vec<String>, Box<dyn std::error::Error>> {
        self.visited.clear();
        let mut links_with_parameters = Vec::new();

        let base_url = self.extract_base_url(url)?;
        self.crawl_recursive(&base_url, &base_url, 0, &mut links_with_parameters)
            .await?;

        Ok(links_with_parameters)
    }

    async fn crawl_recursive(
        &mut self,
        url: &str,
        base_url: &str,
        depth: usize,
        results: &mut Vec<String>,
    ) -> Result<(), Box<dyn std::error::Error>> {
        if depth > self.max_depth || self.visited.contains(url) {
            return Ok(());
        }

        self.visited.insert(url.to_string());

        // Check if current URL has parameters
        if self.parameter_regex.is_match(url) && !results.contains(&url.to_string()) {
            results.push(url.to_string());
        }

        // Fetch the page
        match self.client.get(url).send().await {
            Ok(response) => {
                if response.status().is_success() {
                    let html = response.text().await?;
                    let links = self.extract_links(&html, base_url)?;

                    // Recursively crawl found links
                    for link in links {
                        if link.starts_with(base_url) && !self.visited.contains(&link) {
                            Box::pin(self.crawl_recursive(&link, base_url, depth + 1, results))
                                .await?;
                        }
                    }
                }
            }
            Err(_) => {
                // Continue with other links if one fails
            }
        }

        Ok(())
    }

    fn extract_links(
        &self,
        html: &str,
        base_url: &str,
    ) -> Result<Vec<String>, Box<dyn std::error::Error>> {
        let document = Html::parse_document(html);
        let mut links = Vec::new();

        if let Ok(selector) = Selector::parse("a[href]") {
            for element in document.select(&selector) {
                if let Some(href) = element.value().attr("href") {
                    if let Ok(absolute_url) = self.resolve_url(href, base_url) {
                        links.push(absolute_url);
                    }
                }
            }
        }

        Ok(links)
    }

    fn resolve_url(&self, href: &str, base_url: &str) -> Result<String, url::ParseError> {
        let base = Url::parse(base_url)?;
        let resolved = base.join(href)?;
        Ok(resolved.to_string())
    }

    fn extract_base_url(&self, url: &str) -> Result<String, url::ParseError> {
        let parsed = Url::parse(url)?;
        Ok(format!(
            "{}://{}",
            parsed.scheme(),
            parsed.host_str().unwrap_or("")
        ))
    }
}

impl Default for WebCrawler {
    fn default() -> Self {
        Self::new()
    }
}
