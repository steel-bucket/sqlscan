use crate::searchlibs::{SearchError, SearchResults};
use regex::Regex;
use reqwest::Client;
use std::time::Duration;
use tokio::time::sleep;

pub struct BingSearch {
    client: Client,
    base_url: String,
    regex: Regex,
    user_agent: String,
}

impl Default for BingSearch {
    fn default() -> Self {
        let client = Client::new();
        let regex = Regex::new(r#"<h2><a href="(.*?)""#).expect("Failed to compile regex");

        Self {
            client,
            base_url: "http://www.bing.com/search".to_string(),
            regex,
            user_agent: "python-bing/0.0.1".to_string(),
        }
    }
}

impl BingSearch {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn with_user_agent(mut self, name: &str, version: &str) -> Self {
        self.user_agent = format!("{}/{}", name, version);
        self
    }

    async fn get_page(&self, url: &str) -> Result<String, SearchError> {
        let response = self
            .client
            .get(url)
            .header("Accept", "text/html")
            .header("Connection", "close")
            .header("User-Agent", &self.user_agent)
            .header("Accept-Encoding", "identity")
            .send()
            .await?;

        let html = response.text().await?;
        Ok(html)
    }

    fn parse_links(&self, html: &str) -> Vec<String> {
        self.regex
            .captures_iter(html)
            .filter_map(|cap| cap.get(1))
            .map(|m| m.as_str().to_string())
            .collect()
    }

    pub async fn search(&self, query: &str, stop: usize) -> Result<SearchResults, SearchError> {
        let mut links = Vec::new();
        let mut start = 1;
        let pages = ((stop as f64 / 10.0).ceil() as usize).max(1);

        for _ in 0..pages {
            let search_url = format!(
                "{}?q={}&first={}",
                self.base_url,
                urlencoding::encode(query),
                start
            );

            let html = self.get_page(&search_url).await?;
            let page_results = self.parse_links(&html);

            for result in page_results {
                if !links.contains(&result) {
                    links.push(result);
                }
            }

            start += 10;

            // Add a small delay between requests
            sleep(Duration::from_millis(500)).await;
        }

        // Limit results to the requested stop count
        if links.len() > stop {
            links.truncate(stop);
        }

        Ok(links)
    }
}
