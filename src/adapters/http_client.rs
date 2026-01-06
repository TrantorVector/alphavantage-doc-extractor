//! HTTP client adapter with retry logic and exponential backoff.
//!
//! This module provides a robust HTTP client for fetching web content,
//! specifically designed for the Alpha Vantage documentation extraction use case.

use crate::domain::{RawHtml, ValidatedUrl};
use crate::utils::{NetworkError, NetworkResult};
use reqwest::Client;
use std::time::Duration;
use tokio::time::sleep;
use tracing::{info, instrument, warn};

/// HTTP client with retry logic and exponential backoff.
pub struct HttpClient {
    client: Client,
    max_retries: u32,
}

impl HttpClient {
    /// Create a new HttpClient with sensible defaults.
    ///
    /// Configures the client with:
    /// - User agent: "alphavantage-doc-extractor/2.0.0"
    /// - Timeout: 30 seconds
    /// - Max redirects: 5
    /// - Max retries: 3
    pub fn new() -> NetworkResult<Self> {
        let client = Client::builder()
            .user_agent("alphavantage-doc-extractor/2.0.0")
            .timeout(Duration::from_secs(30))
            .redirect(reqwest::redirect::Policy::limited(5))
            .build()
            .map_err(NetworkError::RequestFailed)?;

        Ok(Self {
            client,
            max_retries: 3,
        })
    }

    /// Fetch HTML content from a validated URL.
    ///
    /// This is the main entry point for fetching web content. It handles
    /// retries automatically and returns structured RawHtml on success.
    #[instrument(skip(self))]
    pub async fn fetch(&self, url: ValidatedUrl) -> NetworkResult<RawHtml> {
        let url_str = url.as_str();
        info!("Fetching HTML from: {}", url_str);

        let html_content = self.fetch_with_retry(url_str).await?;

        info!(
            "Successfully fetched {} bytes from {}",
            html_content.len(),
            url_str
        );

        Ok(RawHtml::new(html_content, url))
    }

    /// Fetch content with retry logic and exponential backoff.
    ///
    /// Implements exponential backoff with delays of 1s, 2s, 4s, etc.
    /// Logs warnings on each retry attempt.
    async fn fetch_with_retry(&self, url: &str) -> NetworkResult<String> {
        for attempt in 0..self.max_retries {
            match self.fetch_once(url).await {
                Ok(content) => return Ok(content),
                Err(error) => {
                    if attempt < self.max_retries - 1 {
                        // Exponential backoff: 2^attempt seconds
                        let backoff_seconds = 2_u64.pow(attempt);
                        let delay = Duration::from_secs(backoff_seconds);

                        warn!(
                            attempt = attempt + 1,
                            max_attempts = self.max_retries,
                            backoff_seconds = backoff_seconds,
                            url = %url,
                            "Request failed, retrying after {} seconds: {:?}", backoff_seconds, error
                        );

                        sleep(delay).await;
                    } else {
                        return Err(error);
                    }
                }
            }
        }

        Err(NetworkError::MaxRetriesExceeded(self.max_retries))
    }

    /// Perform a single HTTP request attempt.
    ///
    /// Makes the actual HTTP request and validates the response.
    async fn fetch_once(&self, url: &str) -> NetworkResult<String> {
        let response = self
            .client
            .get(url)
            .send()
            .await
            .map_err(NetworkError::RequestFailed)?;

        let status = response.status();

        if !status.is_success() {
            let body = response
                .text()
                .await
                .unwrap_or_else(|_| "Failed to read error response body".to_string());

            return Err(NetworkError::HttpError(status.as_u16(), body));
        }

        let body = response
            .text()
            .await
            .map_err(|e| NetworkError::BodyReadFailed(e.to_string()))?;

        Ok(body)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::domain::RawUrl;

    #[test]
    fn test_http_client_creation() {
        let result = HttpClient::new();
        assert!(result.is_ok());
    }

    #[tokio::test]
    async fn test_fetch_successful_response() {
        let client = HttpClient::new().unwrap();

        // Test with httpbin.org which provides reliable test endpoints
        let url =
            ValidatedUrl::try_from(RawUrl::new("https://httpbin.org/html".to_string())).unwrap();

        let result = client.fetch(url).await;
        assert!(result.is_ok());

        let html = result.unwrap();
        assert!(html.content.contains("<html>"));
        assert!(html.content.len() > 100); // Should have substantial content
    }

    #[tokio::test]
    async fn test_fetch_404_error() {
        let client = HttpClient::new().unwrap();

        let url = ValidatedUrl::try_from(RawUrl::new("https://httpbin.org/status/404".to_string()))
            .unwrap();

        let result = client.fetch(url).await;
        assert!(result.is_err());

        match result.unwrap_err() {
            NetworkError::HttpError(status, _) => assert_eq!(status, 404),
            _ => panic!("Expected HttpError"),
        }
    }

    #[tokio::test]
    async fn test_invalid_url() {
        let client = HttpClient::new().unwrap();

        // This should fail because the domain doesn't exist
        let url = ValidatedUrl::try_from(RawUrl::new(
            "https://nonexistent-domain-12345.com".to_string(),
        ))
        .unwrap();

        let result = client.fetch(url).await;
        assert!(result.is_err());
    }
}
