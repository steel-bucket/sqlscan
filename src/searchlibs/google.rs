use crate::searchlibs::common::{SearchError, SearchResults, deduplicate_urls, filter_google_urls};
use reqwest::Client;
use scraper::{Html, Selector};
use std::collections::HashMap;
use std::time::Duration;
use tokio::time::sleep;
use url::Url;

pub struct GoogleSearch {
    client: Client,
    tld: String,
    lang: String,
    safe: String,
    user_agent: String,
}

impl Default for GoogleSearch {
    fn default() -> Self {
        let client = Client::builder()
            .cookie_store(true)
            .build()
            .expect("Failed to create HTTP client");

        Self {
            client,
            tld: "com".to_string(),
            lang: "en".to_string(),
            safe: "off".to_string(),
            user_agent: "Mozilla/4.0 (compatible; MSIE 8.0; Windows NT 6.0)".to_string(),
        }
    }
}

impl GoogleSearch {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn with_tld(mut self, tld: &str) -> Self {
        self.tld = tld.to_string();
        self
    }

    pub fn with_lang(mut self, lang: &str) -> Self {
        self.lang = lang.to_string();
        self
    }

    pub fn with_safe_search(mut self, safe: &str) -> Self {
        self.safe = safe.to_string();
        self
    }

    async fn get_page(&self, url: &str) -> Result<String, SearchError> {
        let response = self
            .client
            .get(url)
            .header("User-Agent", &self.user_agent)
            .send()
            .await?;

        let html = response.text().await?;
        Ok(html)
    }

    fn build_search_url(
        &self,
        query: &str,
        num: usize,
        start: usize,
        tbs: &str,
        search_type: &str,
    ) -> String {
        let base_url = if start > 0 {
            if num == 10 {
                format!(
                    "https://www.google.{}/search?hl={}&q={}&start={}&tbs={}&safe={}&tbm={}",
                    self.tld,
                    self.lang,
                    urlencoding::encode(query),
                    start,
                    tbs,
                    self.safe,
                    search_type
                )
            } else {
                format!(
                    "https://www.google.{}/search?hl={}&q={}&num={}&start={}&tbs={}&safe={}&tbm={}",
                    self.tld,
                    self.lang,
                    urlencoding::encode(query),
                    num,
                    start,
                    tbs,
                    self.safe,
                    search_type
                )
            }
        } else {
            if num == 10 {
                format!(
                    "https://www.google.{}/search?hl={}&q={}&btnG=Google+Search&tbs={}&safe={}&tbm={}",
                    self.tld,
                    self.lang,
                    urlencoding::encode(query),
                    tbs,
                    self.safe,
                    search_type
                )
            } else {
                format!(
                    "https://www.google.{}/search?hl={}&q={}&num={}&btnG=Google+Search&tbs={}&safe={}&tbm={}",
                    self.tld,
                    self.lang,
                    urlencoding::encode(query),
                    num,
                    tbs,
                    self.safe,
                    search_type
                )
            }
        };

        base_url
    }

    fn parse_links(&self, html: &str, only_standard: bool) -> Result<Vec<String>, SearchError> {
        let document = Html::parse_document(html);
        let mut links = Vec::new();

        // Multiple selector strategies for Google search results
        let selectors = [
            "div.g a[href]",    // Modern Google results
            "#search a[href]",  // General search container
            ".rc a[href]",      // Result container
            "h3 a[href]",       // Heading links
            "a[href^='/url?']", // Google redirect URLs
            "a[href^='http']",  // Direct HTTP links
        ];

        for selector_str in &selectors {
            if let Ok(selector) = Selector::parse(selector_str) {
                for element in document.select(&selector) {
                    if let Some(href) = element.value().attr("href") {
                        // Skip obvious non-result links
                        if href.contains("google.com")
                            || href.starts_with("#")
                            || href.starts_with("javascript:")
                            || href.contains("webcache")
                        {
                            continue;
                        }

                        if only_standard {
                            // Check if this looks like a standard result
                            if let Some(parent) = element.parent() {
                                let parent_element = parent.value().as_element();
                                if parent_element.map_or(true, |el| el.name() != "h3") {
                                    // Also check for common result patterns
                                    let has_result_class = element
                                        .value()
                                        .classes()
                                        .any(|c| c.contains("result") || c.contains("title"));
                                    if !has_result_class {
                                        continue;
                                    }
                                }
                            }
                        }

                        // Process the URL
                        let processed_url = if href.starts_with("/url?") {
                            // Handle Google redirect URLs
                            self.extract_url_from_google_redirect(href)
                        } else if href.starts_with("http") {
                            Some(href.to_string())
                        } else {
                            None
                        };

                        if let Some(url) = processed_url {
                            if let Some(filtered_link) = filter_google_urls(&url) {
                                if !links.contains(&filtered_link) {
                                    links.push(filtered_link);
                                }
                            }
                        }
                    }
                }

                // If we found results with this selector, we can break
                if !links.is_empty() {
                    break;
                }
            }
        }

        Ok(links)
    }

    fn extract_url_from_google_redirect(&self, redirect_url: &str) -> Option<String> {
        // Parse Google redirect URL like /url?q=https://example.com&sa=U&ved=...
        if let Some(start) = redirect_url.find("q=") {
            let after_q = &redirect_url[start + 2..];
            if let Some(end) = after_q.find('&') {
                let url = &after_q[..end];
                return Some(urlencoding::decode(url).ok()?.to_string());
            } else {
                let url = after_q;
                return Some(urlencoding::decode(url).ok()?.to_string());
            }
        }
        None
    }

    pub async fn search(
        &self,
        query: &str,
        num: usize,
        start: usize,
        stop: Option<usize>,
        pause: f64,
        only_standard: bool,
        tbs: &str,
    ) -> Result<SearchResults, SearchError> {
        // Grab the home page cookie first
        let home_url = format!("https://www.google.{}/", self.tld);
        self.get_page(&home_url).await?;

        let mut all_links = Vec::new();
        let mut current_start = start;

        loop {
            if let Some(stop_val) = stop {
                if current_start >= stop_val {
                    break;
                }
            }

            let url = self.build_search_url(query, num, current_start, tbs, "");

            // Sleep between requests
            if pause > 0.0 {
                sleep(Duration::from_secs_f64(pause)).await;
            }

            let html = self.get_page(&url).await?;
            let links = self.parse_links(&html, only_standard)?;

            if links.is_empty() {
                break;
            }

            all_links.extend(links);
            current_start += num;

            // Check if there are more pages by looking for navigation
            let document = Html::parse_document(&html);
            if let Ok(nav_selector) = Selector::parse("#nav") {
                if document.select(&nav_selector).next().is_none() {
                    break;
                }
            }
        }

        Ok(deduplicate_urls(all_links))
    }

    pub async fn search_images(
        &self,
        query: &str,
        num: usize,
        start: usize,
        stop: Option<usize>,
        pause: f64,
    ) -> Result<SearchResults, SearchError> {
        self.search_with_type(query, num, start, stop, pause, "isch")
            .await
    }

    pub async fn search_news(
        &self,
        query: &str,
        num: usize,
        start: usize,
        stop: Option<usize>,
        pause: f64,
    ) -> Result<SearchResults, SearchError> {
        self.search_with_type(query, num, start, stop, pause, "nws")
            .await
    }

    pub async fn search_videos(
        &self,
        query: &str,
        num: usize,
        start: usize,
        stop: Option<usize>,
        pause: f64,
    ) -> Result<SearchResults, SearchError> {
        self.search_with_type(query, num, start, stop, pause, "vid")
            .await
    }

    pub async fn search_shop(
        &self,
        query: &str,
        num: usize,
        start: usize,
        stop: Option<usize>,
        pause: f64,
    ) -> Result<SearchResults, SearchError> {
        self.search_with_type(query, num, start, stop, pause, "shop")
            .await
    }

    pub async fn search_books(
        &self,
        query: &str,
        num: usize,
        start: usize,
        stop: Option<usize>,
        pause: f64,
    ) -> Result<SearchResults, SearchError> {
        self.search_with_type(query, num, start, stop, pause, "bks")
            .await
    }

    pub async fn search_apps(
        &self,
        query: &str,
        num: usize,
        start: usize,
        stop: Option<usize>,
        pause: f64,
    ) -> Result<SearchResults, SearchError> {
        self.search_with_type(query, num, start, stop, pause, "app")
            .await
    }

    async fn search_with_type(
        &self,
        query: &str,
        num: usize,
        start: usize,
        stop: Option<usize>,
        pause: f64,
        search_type: &str,
    ) -> Result<SearchResults, SearchError> {
        // Grab the home page cookie first
        let home_url = format!("https://www.google.{}/", self.tld);
        self.get_page(&home_url).await?;

        let mut all_links = Vec::new();
        let mut current_start = start;

        loop {
            if let Some(stop_val) = stop {
                if current_start >= stop_val {
                    break;
                }
            }

            let url = self.build_search_url(query, num, current_start, "0", search_type);

            // Sleep between requests
            if pause > 0.0 {
                sleep(Duration::from_secs_f64(pause)).await;
            }

            let html = self.get_page(&url).await?;
            let links = self.parse_links(&html, false)?;

            if links.is_empty() {
                break;
            }

            all_links.extend(links);
            current_start += num;
        }

        Ok(deduplicate_urls(all_links))
    }

    pub async fn lucky(&self, query: &str) -> Result<Option<String>, SearchError> {
        let results = self.search(query, 1, 0, Some(1), 0.0, false, "0").await?;
        Ok(results.into_iter().next())
    }
}
