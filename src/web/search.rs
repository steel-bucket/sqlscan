use crate::searchlibs::{
    BingSearch as BingSearchLib, GoogleSearch as GoogleSearchLib, YahooSearch as YahooSearchLib,
};
use async_trait::async_trait;
use thiserror::Error;

#[derive(Error, Debug)]
pub enum SearchError {
    #[error("Service unreachable (503)")]
    ServiceUnavailable,

    #[error("Gateway timeout (504)")]
    GatewayTimeout,

    #[error("Search engine error: {0}")]
    EngineError(String),

    #[error("Unknown error occurred")]
    Unknown,
}

#[async_trait]
pub trait SearchEngine {
    async fn search(&self, query: &str, pages: usize) -> Result<Vec<String>, SearchError>;
}

pub struct Search;

impl Search {
    pub fn new() -> Self {
        Self
    }
}

pub struct GoogleSearchEngine {
    engine: GoogleSearchLib,
}

impl GoogleSearchEngine {
    pub fn new() -> Self {
        Self {
            engine: GoogleSearchLib::new().with_lang("en").with_tld("com"),
        }
    }
}

#[async_trait]
impl SearchEngine for GoogleSearchEngine {
    async fn search(&self, query: &str, pages: usize) -> Result<Vec<String>, SearchError> {
        match self
            .engine
            .search(query, 10, 0, Some(pages * 10), 2.0, false, "0")
            .await
        {
            Ok(urls) => Ok(urls),
            Err(e) => {
                let error_msg = e.to_string();
                if error_msg.contains("503") {
                    Err(SearchError::ServiceUnavailable)
                } else if error_msg.contains("504") {
                    Err(SearchError::GatewayTimeout)
                } else {
                    Err(SearchError::EngineError(error_msg))
                }
            }
        }
    }
}

pub struct BingSearchEngine {
    engine: BingSearchLib,
}

impl BingSearchEngine {
    pub fn new() -> Self {
        Self {
            engine: BingSearchLib::new(),
        }
    }
}

#[async_trait]
impl SearchEngine for BingSearchEngine {
    async fn search(&self, query: &str, pages: usize) -> Result<Vec<String>, SearchError> {
        match self.engine.search(query, pages * 10).await {
            Ok(urls) => Ok(urls),
            Err(e) => {
                let error_msg = e.to_string();
                if error_msg.contains("503") {
                    Err(SearchError::ServiceUnavailable)
                } else if error_msg.contains("504") {
                    Err(SearchError::GatewayTimeout)
                } else {
                    Err(SearchError::EngineError(error_msg))
                }
            }
        }
    }
}

pub struct YahooSearchEngine {
    engine: YahooSearchLib,
}

impl YahooSearchEngine {
    pub fn new() -> Self {
        Self {
            engine: YahooSearchLib::new(),
        }
    }
}

#[async_trait]
impl SearchEngine for YahooSearchEngine {
    async fn search(&self, query: &str, pages: usize) -> Result<Vec<String>, SearchError> {
        match self.engine.search(query, 10, pages).await {
            Ok(urls) => Ok(urls),
            Err(e) => {
                let error_msg = e.to_string();
                if error_msg.contains("503") {
                    Err(SearchError::ServiceUnavailable)
                } else if error_msg.contains("504") {
                    Err(SearchError::GatewayTimeout)
                } else {
                    Err(SearchError::EngineError(error_msg))
                }
            }
        }
    }
}
