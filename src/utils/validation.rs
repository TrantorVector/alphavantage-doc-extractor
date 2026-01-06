//! Input validation utilities for the Alpha Vantage Documentation Extractor.
//!
//! This module implements the parse-don't-validate principle with strong typing.
//! All validation happens at type conversion time, ensuring invalid data cannot exist.

use crate::domain::{RawUrl, ValidatedUrl};
use crate::utils::UrlParseError;

/// Implement URL validation through type conversion.
///
/// This implements the parse-don't-validate principle - URLs are validated
/// at the time they're converted from RawUrl to ValidatedUrl, ensuring
/// only valid URLs can exist as ValidatedUrl instances.
impl TryFrom<RawUrl> for ValidatedUrl {
    type Error = UrlParseError;

    fn try_from(raw: RawUrl) -> Result<Self, Self::Error> {
        ValidatedUrl::validate(raw)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::domain::RawUrl;

    #[test]
    fn test_valid_https_url() {
        let raw = RawUrl::new("https://www.alphavantage.co/documentation/".to_string());
        let result = ValidatedUrl::try_from(raw);
        assert!(result.is_ok());
        assert_eq!(
            result.unwrap().as_str(),
            "https://www.alphavantage.co/documentation/"
        );
    }

    #[test]
    fn test_valid_http_url() {
        let raw = RawUrl::new("http://example.com".to_string());
        let result = ValidatedUrl::try_from(raw);
        assert!(result.is_ok());
        assert_eq!(result.unwrap().as_str(), "http://example.com/");
    }

    #[test]
    fn test_invalid_scheme_ftp() {
        let raw = RawUrl::new("ftp://example.com/file".to_string());
        let result = ValidatedUrl::try_from(raw);
        assert!(result.is_err());
        match result.unwrap_err() {
            UrlParseError::UnsupportedScheme(scheme) => assert_eq!(scheme, "ftp"),
            _ => panic!("Expected UnsupportedScheme error"),
        }
    }

    #[test]
    fn test_missing_host() {
        let raw = RawUrl::new("https://".to_string());
        let result = ValidatedUrl::try_from(raw);
        assert!(result.is_err());
        match result.unwrap_err() {
            UrlParseError::ParseFailed(_) => {} // url::Url::parse fails for empty host
            _ => panic!("Expected ParseFailed error"),
        }
    }

    #[test]
    fn test_malformed_url() {
        let raw = RawUrl::new("not-a-valid-url".to_string());
        let result = ValidatedUrl::try_from(raw);
        assert!(result.is_err());
        match result.unwrap_err() {
            UrlParseError::ParseFailed(_) => {}
            _ => panic!("Expected ParseFailed error"),
        }
    }
}
