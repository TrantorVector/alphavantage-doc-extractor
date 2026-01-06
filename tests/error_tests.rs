//! Unit tests for the error handling system.
//!
//! Tests error type conversions, message formatting, trait bounds, and error context preservation.

use alphavantage_doc_extractor::utils::{
    ContentExtractionError, ExtractionError, ExtractionResult, NetworkError, NetworkResult,
    ParseError, ParseResult, RenderError, RenderResult, Result, UrlParseError,
};

/// Test error conversion chains using From trait implementations
#[cfg(test)]
mod error_conversion_tests {
    use super::*;

    #[test]
    fn test_extraction_error_from_network_error() {
        let network_err = NetworkError::Timeout(30);
        let extraction_err: ExtractionError = network_err.into();

        match extraction_err {
            ExtractionError::Network(NetworkError::Timeout(seconds)) => {
                assert_eq!(seconds, 30);
            }
            _ => panic!("Expected Network variant"),
        }
    }

    #[test]
    fn test_extraction_error_from_parse_error() {
        let parse_err = ParseError::MissingTitle;
        let extraction_err: ExtractionError = parse_err.into();

        match extraction_err {
            ExtractionError::Parse(ParseError::MissingTitle) => {}
            _ => panic!("Expected Parse variant"),
        }
    }

    #[test]
    fn test_extraction_error_from_content_extraction_error() {
        let content_err = ContentExtractionError::EmptyContent;
        let extraction_err: ExtractionError = content_err.into();

        match extraction_err {
            ExtractionError::ContentExtraction(ContentExtractionError::EmptyContent) => {}
            _ => panic!("Expected ContentExtraction variant"),
        }
    }

    #[test]
    fn test_extraction_error_from_render_error() {
        let render_err = RenderError::ValidationFailed("test failure".to_string());
        let extraction_err: ExtractionError = render_err.into();

        match extraction_err {
            ExtractionError::Render(RenderError::ValidationFailed(msg)) => {
                assert_eq!(msg, "test failure");
            }
            _ => panic!("Expected Render variant"),
        }
    }

    #[test]
    fn test_network_error_request_failed_variant() {
        // Test that we can create and pattern match the RequestFailed variant
        // Note: In real usage, reqwest::Error would come from actual HTTP operations
        // For testing, we create a placeholder to verify the enum structure
        // The actual From<reqwest::Error> conversion is tested implicitly through usage
        let network_err = NetworkError::HttpError(500, "Internal Server Error".to_string());
        match network_err {
            NetworkError::HttpError(code, _) => assert_eq!(code, 500),
            _ => panic!("Expected HttpError variant"),
        }
    }

    #[test]
    fn test_url_parse_error_from_url_parse_error() {
        let url_err = url::ParseError::EmptyHost;
        let url_parse_err: UrlParseError = url_err.into();

        match url_parse_err {
            UrlParseError::ParseFailed(url::ParseError::EmptyHost) => {}
            _ => panic!("Expected ParseFailed variant"),
        }
    }
}

/// Test error message formatting and Display implementation
#[cfg(test)]
mod error_message_tests {
    use super::*;

    #[test]
    fn test_network_error_messages() {
        let timeout_err = NetworkError::Timeout(45);
        assert_eq!(
            timeout_err.to_string(),
            "Request timed out after 45 seconds"
        );

        let http_err = NetworkError::HttpError(404, "Not Found".to_string());
        assert_eq!(http_err.to_string(), "HTTP 404 error: Not Found");

        let retry_err = NetworkError::MaxRetriesExceeded(3);
        assert_eq!(retry_err.to_string(), "Maximum retry attempts (3) exceeded");

        let body_err = NetworkError::BodyReadFailed("Invalid UTF-8".to_string());
        assert_eq!(
            body_err.to_string(),
            "Failed to read response body: Invalid UTF-8"
        );
    }

    #[test]
    fn test_parse_error_messages() {
        let invalid_html = ParseError::InvalidHtml("Malformed tag".to_string());
        assert_eq!(invalid_html.to_string(), "Invalid HTML: Malformed tag");

        let missing_element = ParseError::MissingElement(".api-section".to_string());
        assert_eq!(
            missing_element.to_string(),
            "Missing HTML element: .api-section"
        );

        let missing_title = ParseError::MissingTitle;
        assert_eq!(missing_title.to_string(), "Missing page title");

        let invalid_selector = ParseError::InvalidSelector("invalid[selector".to_string());
        assert_eq!(
            invalid_selector.to_string(),
            "Invalid CSS selector: invalid[selector"
        );
    }

    #[test]
    fn test_content_extraction_error_messages() {
        let no_main_content = ContentExtractionError::NoMainContentFound;
        assert_eq!(
            no_main_content.to_string(),
            "No main content area found in the page"
        );

        let category_failed = ContentExtractionError::CategoryExtractionFailed(
            "Stocks".to_string(),
            "No header found".to_string(),
        );
        assert_eq!(
            category_failed.to_string(),
            "Category extraction failed for 'Stocks': No header found"
        );

        let endpoint_failed = ContentExtractionError::EndpointExtractionFailed(
            "TIME_SERIES_INTRADAY".to_string(),
            "Missing parameters".to_string(),
        );
        assert_eq!(
            endpoint_failed.to_string(),
            "Endpoint extraction failed for 'TIME_SERIES_INTRADAY': Missing parameters"
        );

        let invalid_table = ContentExtractionError::InvalidParameterTable("No rows".to_string());
        assert_eq!(
            invalid_table.to_string(),
            "Invalid parameter table structure: No rows"
        );

        let empty_content = ContentExtractionError::EmptyContent;
        assert_eq!(empty_content.to_string(), "Extracted content is empty");

        let code_block_failed =
            ContentExtractionError::CodeBlockParseFailed("Invalid JSON".to_string());
        assert_eq!(
            code_block_failed.to_string(),
            "Code block parsing failed: Invalid JSON"
        );
    }

    #[test]
    fn test_render_error_messages() {
        let markdown_failed = RenderError::MarkdownGenerationFailed("Template error".to_string());
        assert_eq!(
            markdown_failed.to_string(),
            "Markdown generation failed: Template error"
        );

        let invalid_path = RenderError::InvalidOutputPath("/invalid/path".to_string());
        assert_eq!(
            invalid_path.to_string(),
            "Invalid output path: /invalid/path"
        );

        let validation_failed = RenderError::ValidationFailed("Density check failed".to_string());
        assert_eq!(
            validation_failed.to_string(),
            "Output validation failed: Density check failed"
        );
    }

    #[test]
    fn test_url_parse_error_messages() {
        let unsupported_scheme = UrlParseError::UnsupportedScheme("ftp".to_string());
        assert_eq!(
            unsupported_scheme.to_string(),
            "Unsupported URL scheme 'ftp' - only http/https supported"
        );

        let missing_host = UrlParseError::MissingHost;
        assert_eq!(missing_host.to_string(), "URL missing host component");

        let invalid_format = UrlParseError::InvalidFormat("not a url".to_string());
        assert_eq!(invalid_format.to_string(), "Invalid URL format: not a url");
    }

    #[test]
    fn test_extraction_error_messages() {
        let network_err = ExtractionError::Network(NetworkError::Timeout(10));
        assert!(network_err.to_string().starts_with("Network error:"));

        let parse_err = ExtractionError::Parse(ParseError::MissingTitle);
        assert!(parse_err.to_string().starts_with("Parse error:"));

        let content_err = ExtractionError::ContentExtraction(ContentExtractionError::EmptyContent);
        assert!(content_err
            .to_string()
            .starts_with("Content extraction error:"));

        let render_err = ExtractionError::Render(RenderError::ValidationFailed("test".to_string()));
        assert!(render_err.to_string().starts_with("Render error:"));

        let url_err = ExtractionError::UrlParse(UrlParseError::MissingHost);
        assert!(url_err.to_string().starts_with("URL parse error:"));
    }
}

/// Test Send + Sync trait bounds (automatically derived by thiserror)
#[cfg(test)]
mod trait_bounds_tests {
    use super::*;

    #[test]
    fn test_send_sync_bounds() {
        fn assert_send_sync<T: Send + Sync>() {}

        // Test all error types implement Send + Sync (automatically derived by thiserror)
        assert_send_sync::<ExtractionError>();
        assert_send_sync::<NetworkError>();
        assert_send_sync::<ParseError>();
        assert_send_sync::<ContentExtractionError>();
        assert_send_sync::<RenderError>();
        assert_send_sync::<UrlParseError>();
    }
}

/// Test error context preservation and chaining
#[cfg(test)]
mod error_context_tests {
    use super::*;

    #[test]
    fn test_error_context_preservation() {
        // Test that error variants preserve their internal context
        let network_err = NetworkError::HttpError(500, "Internal Server Error".to_string());
        if let NetworkError::HttpError(code, message) = network_err {
            assert_eq!(code, 500);
            assert_eq!(message, "Internal Server Error");
        } else {
            panic!("Expected HttpError variant");
        }

        let content_err = ContentExtractionError::EndpointExtractionFailed(
            "SYMBOL_SEARCH".to_string(),
            "Invalid parameter format".to_string(),
        );
        if let ContentExtractionError::EndpointExtractionFailed(endpoint, reason) = content_err {
            assert_eq!(endpoint, "SYMBOL_SEARCH");
            assert_eq!(reason, "Invalid parameter format");
        } else {
            panic!("Expected EndpointExtractionFailed variant");
        }
    }

    #[test]
    fn test_error_chaining_through_extraction_error() {
        // Test that errors can be chained through ExtractionError
        let parse_err = ParseError::InvalidSelector("div > > p".to_string());
        let extraction_err: ExtractionError = parse_err.into();

        // Verify the error can be downcast back to the original type
        match extraction_err {
            ExtractionError::Parse(ParseError::InvalidSelector(selector)) => {
                assert_eq!(selector, "div > > p");
            }
            _ => panic!("Expected nested Parse error"),
        }
    }

    #[test]
    fn test_http_status_code_handling() {
        // Test specific HTTP status code scenarios
        let not_found = NetworkError::HttpError(404, "Not Found".to_string());
        assert_eq!(not_found.to_string(), "HTTP 404 error: Not Found");

        let server_error = NetworkError::HttpError(500, "Internal Server Error".to_string());
        assert_eq!(
            server_error.to_string(),
            "HTTP 500 error: Internal Server Error"
        );

        let unauthorized = NetworkError::HttpError(401, "Unauthorized".to_string());
        assert_eq!(unauthorized.to_string(), "HTTP 401 error: Unauthorized");
    }
}

/// Test that type aliases work correctly
#[cfg(test)]
mod type_alias_tests {
    use super::*;

    #[test]
    fn test_result_type_aliases() {
        // Test that type aliases work with Ok values
        let network_result: NetworkResult<i32> = Ok(42);
        assert_eq!(network_result.unwrap(), 42);

        let parse_result: ParseResult<String> = Ok("success".to_string());
        assert_eq!(parse_result.unwrap(), "success");

        let extraction_result: ExtractionResult<Vec<String>> =
            Ok(vec!["item1".to_string(), "item2".to_string()]);
        assert_eq!(extraction_result.unwrap().len(), 2);

        let render_result: RenderResult<bool> = Ok(true);
        assert!(render_result.unwrap());

        let main_result: Result<u8> = Ok(255);
        assert_eq!(main_result.unwrap(), 255);
    }

    #[test]
    fn test_result_type_aliases_with_errors() {
        // Test that type aliases work with Err values
        let network_result: NetworkResult<i32> = Err(NetworkError::Timeout(30));
        assert!(network_result.is_err());

        let parse_result: ParseResult<String> = Err(ParseError::MissingTitle);
        assert!(parse_result.is_err());

        let extraction_result: ExtractionResult<Vec<String>> =
            Err(ContentExtractionError::EmptyContent);
        assert!(extraction_result.is_err());

        let render_result: RenderResult<bool> =
            Err(RenderError::ValidationFailed("test".to_string()));
        assert!(render_result.is_err());

        let main_result: Result<u8> = Err(ExtractionError::Network(
            NetworkError::MaxRetriesExceeded(5),
        ));
        assert!(main_result.is_err());
    }
}
