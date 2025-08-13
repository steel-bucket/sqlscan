// src/sql_vuln/server_info.rs
use crate::sql_vuln::std_utils::{ServerData, StdUtils};
use crate::web::web::get_html;
use scraper::{Html, Selector};
use std::sync::Arc;
use tokio::task::JoinSet;
use url::Url;

pub struct ServerInfoChecker;

impl ServerInfoChecker {
    pub async fn check(urls: Vec<String>) -> Vec<ServerData> {
        let mut domains_info = Vec::new();
        let mut join_set = JoinSet::new();
        let max_concurrent = num_cpus::get() * 2;

        // Process URLs in batches to avoid overwhelming the server
        for chunk in urls.chunks(max_concurrent) {
            for url in chunk {
                let url_clone = url.clone();
                join_set.spawn(Self::get_server_info(url_clone));
            }

            // Collect results from this batch
            let mut batch_results = Vec::new();
            while let Some(result) = join_set.join_next().await {
                match result {
                    Ok((url, server, lang)) => {
                        batch_results.push(ServerData {
                            website: url,
                            server,
                            language: lang,
                        });
                    }
                    Err(e) => {
                        StdUtils::stderr(&format!("Task join error: {}", e));
                    }
                }
            }
            domains_info.extend(batch_results);
        }

        domains_info
    }

    async fn get_server_info(url: String) -> (String, String, String) {
        let domain = Self::extract_domain(&url);
        let lookup_url = format!("https://aruljohn.com/webserver/{}", domain);

        match get_html(&lookup_url, false).await {
            Ok((html, _)) => {
                let (server, lang) = Self::parse_server_info(&html);
                (url, server, lang)
            }
            Err(_) => (url, String::new(), String::new()),
        }
    }

    fn extract_domain(url: &str) -> String {
        if let Ok(parsed) = Url::parse(url) {
            if let Some(host) = parsed.host_str() {
                return host.to_string();
            }
        }

        // Fallback: try to extract domain from path
        if url.contains("/") {
            if let Some(first_part) = url.split("/").next() {
                return first_part.to_string();
            }
        }

        url.to_string()
    }

    fn parse_server_info(html: &str) -> (String, String) {
        let document = Html::parse_document(html);

        // Check for error message
        if let Ok(error_selector) = Selector::parse("p.err") {
            if document.select(&error_selector).next().is_some() {
                return (String::new(), String::new());
            }
        }

        let mut info = Vec::new();

        // Look for table rows with server information
        if let Ok(row_selector) = Selector::parse("tr") {
            for row in document.select(&row_selector) {
                if let Ok(title_selector) = Selector::parse("td.title") {
                    if row.select(&title_selector).next().is_some() {
                        if let Ok(cell_selector) = Selector::parse("td") {
                            let cells: Vec<_> = row.select(&cell_selector).collect();
                            if cells.len() > 1 {
                                let text = cells[1].text().collect::<String>().trim().to_string();
                                info.push(text);
                            }
                        }
                    }
                }
            }
        }

        match info.len() {
            0 => (String::new(), String::new()),
            1 => (info[0].clone(), String::new()),
            _ => (info[0].clone(), info[1].clone()),
        }
    }
}
