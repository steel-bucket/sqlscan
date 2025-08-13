use crate::searchlibs::common::{SearchError, SearchResults};
use reqwest::Client;
use scraper::{Html, Selector};
use std::time::Duration;
use tokio::time::sleep;

pub struct YahooSearch {
    client: Client,
    base_url: String,
    content_type: String,
    user_agent: String,
}

impl Default for YahooSearch {
    fn default() -> Self {
        let client = Client::new();

        Self {
            client,
            base_url: "https://search.yahoo.com/search".to_string(),
            content_type: "application/x-www-form-urlencoded; charset=UTF-8".to_string(),
            user_agent: "yahoo search".to_string(),
        }
    }
}

impl YahooSearch {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn with_user_agent(mut self, user_agent: &str) -> Self {
        self.user_agent = user_agent.to_string();
        self
    }

    pub fn with_content_type(mut self, content_type: &str) -> Self {
        self.content_type = content_type.to_string();
        self
    }

    async fn get_page(&self, url: &str) -> Result<String, SearchError> {
        let response = self
            .client
            .get(url)
            .header("Content-Type", &self.content_type)
            .header("User-Agent", &self.user_agent)
            .send()
            .await?;

        let html = response.text().await?;
        Ok(html)
    }

    fn parse_links(&self, html: &str) -> Result<Vec<String>, SearchError> {
        let document = Html::parse_document(html);
        let mut links = Vec::new();

        // Look for divs containing search results
        if let Ok(div_selector) = Selector::parse("div") {
            for div_element in document.select(&div_selector) {
                // Look for links with the specific class mentioned in the Python code
                if let Ok(link_selector) = Selector::parse("a.ac-algo.fz-l.ac-21th.lh-24") {
                    for link_element in div_element.select(&link_selector) {
                        if let Some(href) = link_element.value().attr("href") {
                            if !links.contains(&href.to_string()) {
                                links.push(href.to_string());
                            }
                        }
                    }
                }

                // Also try a more general approach for Yahoo search results
                if let Ok(general_link_selector) = Selector::parse("a[href]") {
                    for link_element in div_element.select(&general_link_selector) {
                        if let Some(href) = link_element.value().attr("href") {
                            // Filter out obvious non-result links
                            if href.starts_with("http")
                                && !href.contains("yahoo.com")
                                && !links.contains(&href.to_string())
                            {
                                links.push(href.to_string());
                            }
                        }
                    }
                }
            }
        }

        Ok(links)
    }

    pub async fn search(
        &self,
        query: &str,
        per_page: usize,
        pages: usize,
    ) -> Result<SearchResults, SearchError> {
        let mut all_urls = Vec::new();

        for page in 0..pages {
            let start_index = (page + 1) * 10;
            let search_url = format!(
                "{}?p={}&n={}&b={}",
                self.base_url,
                urlencoding::encode(query),
                per_page,
                start_index
            );

            let html = self.get_page(&search_url).await?;
            let mut page_urls = self.parse_links(&html)?;
            all_urls.append(&mut page_urls);

            // Add a small delay between requests to be respectful
            sleep(Duration::from_millis(500)).await;
        }

        // Remove duplicates while preserving order
        let mut unique_urls = Vec::new();
        for url in all_urls {
            if !unique_urls.contains(&url) {
                unique_urls.push(url);
            }
        }

        Ok(unique_urls)
    }
}
