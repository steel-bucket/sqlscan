use crate::web::useragents::UserAgents;
use reqwest::Client;
use std::time::Duration;
use thiserror::Error;
use url::Url;

#[derive(Error, Debug)]
pub enum WebError {
    #[error("HTTP request failed: {0}")]
    HttpError(#[from] reqwest::Error),

    #[error("URL parsing error: {0}")]
    UrlError(#[from] url::ParseError),

    #[error("Timeout error")]
    TimeoutError,

    #[error("HTTP 500 error: {0}")]
    Http500Error(String),
}

pub async fn get_html(
    url: &str,
    return_last_url: bool,
) -> Result<(String, Option<String>), WebError> {
    // Ensure URL has proper scheme
    let full_url = if url.starts_with("http://") || url.starts_with("https://") {
        url.to_string()
    } else {
        format!("http://{}", url)
    };

    // Validate URL
    let parsed_url = Url::parse(&full_url)?;

    // Get random user agent
    let user_agents = UserAgents::new();
    let user_agent = user_agents.get_random();

    // Create client with timeout
    let client = Client::builder().timeout(Duration::from_secs(10)).build()?;

    // Make request
    let response = client
        .get(&full_url)
        .header("User-Agent", user_agent)
        .send()
        .await;

    match response {
        Ok(resp) => {
            let final_url = if return_last_url {
                Some(resp.url().to_string())
            } else {
                None
            };

            let status = resp.status();
            let html = resp.text().await?;

            if status.is_server_error() && status.as_u16() == 500 {
                // Handle HTTP 500 but still return content if available
                if !html.is_empty() {
                    return Ok((html, final_url));
                } else {
                    return Err(WebError::Http500Error(
                        "HTTP 500 with empty response".to_string(),
                    ));
                }
            }

            Ok((html, final_url))
        }
        Err(e) => {
            if e.is_timeout() {
                Err(WebError::TimeoutError)
            } else {
                Err(WebError::HttpError(e))
            }
        }
    }
}

// Convenience function that matches the original Python API
pub async fn gethtml(url: &str, last_url: bool) -> Option<(String, Option<String>)> {
    match get_html(url, last_url).await {
        Ok(result) => Some(result),
        Err(_) => None,
    }
}
