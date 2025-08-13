// src/sql_vuln/scanner.rs
use crate::sql_vuln::{SqlErrorChecker, StdUtils};
use crate::web::web::get_html;
use std::sync::Arc;
use tokio::task::JoinSet;
use url::Url;

pub struct SqlInjectionScanner {
    error_checker: SqlErrorChecker,
    payloads: Vec<&'static str>,
}

impl SqlInjectionScanner {
    pub fn new() -> Self {
        Self {
            error_checker: SqlErrorChecker::new(),
            payloads: vec![
                "'", "')", "';", "\"", "\")", "\";", "`", "`)", "`;", "\\", "%27", "%%2727",
                "%25%27", "%60", "%5C",
            ],
        }
    }

    pub async fn scan(&self, urls: Vec<String>) -> Vec<(String, String)> {
        let mut vulnerables = Vec::new();
        let mut join_set = JoinSet::new();
        let error_checker = Arc::new(SqlErrorChecker::new());
        let max_concurrent = num_cpus::get() * 2;

        // Process URLs in batches
        for chunk in urls.chunks(max_concurrent) {
            for url in chunk {
                let url_clone = url.clone();
                let checker_clone = Arc::clone(&error_checker);
                let payloads_clone = self.payloads.clone();

                join_set.spawn(Self::check_sql_injection(
                    url_clone,
                    checker_clone,
                    payloads_clone,
                ));
            }

            // Collect results from this batch
            while let Some(result) = join_set.join_next().await {
                match result {
                    Ok(Some((url, db))) => {
                        vulnerables.push((url, db));
                    }
                    Ok(None) => {
                        // Not vulnerable
                    }
                    Err(e) => {
                        StdUtils::stderr(&format!("Scan task failed: {}", e));
                    }
                }
            }
        }

        vulnerables
    }

    async fn check_sql_injection(
        url: String,
        error_checker: Arc<SqlErrorChecker>,
        payloads: Vec<&'static str>,
    ) -> Option<(String, String)> {
        StdUtils::stdout_no_newline(&format!("scanning {}", url));

        let parsed_url = match Url::parse(&url) {
            Ok(parsed) => parsed,
            Err(_) => {
                println!(); // Move cursor to new line
                return None;
            }
        };

        let query = parsed_url.query().unwrap_or("");
        if query.is_empty() {
            println!(); // Move cursor to new line
            return None;
        }

        let domain = format!(
            "{}://{}{}",
            parsed_url.scheme(),
            parsed_url.host_str().unwrap_or(""),
            parsed_url.path()
        );

        let queries: Vec<&str> = query.split('&').collect();

        for payload in &payloads {
            let modified_params: Vec<String> = queries
                .iter()
                .map(|param| format!("{}{}", param, payload))
                .collect();

            let test_url = format!("{}?{}", domain, modified_params.join("&"));

            match get_html(&test_url, false).await {
                Ok((html, _)) => {
                    let (is_vulnerable, db) = error_checker.check(&html);
                    if is_vulnerable {
                        if let Some(database) = db {
                            StdUtils::show_sign(" vulnerable");
                            return Some((url, database));
                        }
                    }
                }
                Err(_) => {
                    // Continue with next payload
                    continue;
                }
            }
        }

        println!(); // Move cursor to new line
        None
    }
}

impl Default for SqlInjectionScanner {
    fn default() -> Self {
        Self::new()
    }
}
