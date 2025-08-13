// src/sql_vuln/reverse_ip.rs
use crate::web::useragents::UserAgents;
use reqwest::Client;
use serde_json::Value;
use std::collections::HashMap;
use url::Url;

pub struct ReverseIpLookup;

impl ReverseIpLookup {
    pub async fn reverse_ip(url: &str) -> Result<Vec<String>, Box<dyn std::error::Error>> {
        let domain = Self::extract_domain(url);

        let source = "http://domains.yougetsignal.com/domains.php";
        let user_agents = UserAgents::new();
        let user_agent = user_agents.get_random();

        let client = Client::new();

        let mut params = HashMap::new();
        params.insert("remoteAddress", domain);
        params.insert("key", "".to_string());

        let response = client
            .post(source)
            .header(
                "Content-Type",
                "application/x-www-form-urlencoded; charset=UTF-8",
            )
            .header("User-Agent", user_agent)
            .form(&params)
            .send()
            .await?;

        let result: Value = response.json().await?;

        if result["status"] == "Success" {
            let mut domains = Vec::new();

            if let Some(domain_array) = result["domainArray"].as_array() {
                for domain_entry in domain_array {
                    if let Some(domain_info) = domain_entry.as_array() {
                        if let Some(domain_name) = domain_info.first() {
                            if let Some(domain_str) = domain_name.as_str() {
                                domains.push(domain_str.to_string());
                            }
                        }
                    }
                }
            }

            Ok(domains)
        } else {
            let message = result["message"].as_str().unwrap_or("Unknown error");
            Err(format!("Reverse IP lookup failed: {}", message).into())
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
}
