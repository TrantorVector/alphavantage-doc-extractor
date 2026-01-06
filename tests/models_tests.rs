//! Unit tests for domain models.
//!
//! Tests builder patterns, validation rules, edge cases, and serialization.

use alphavantage_doc_extractor::domain::{
    ApiCategory, ApiEndpoint, CodeExample, DocumentMetadata, DocumentStructure, OutputMetadata,
    Parameter, RawHtml, RawUrl, ValidatedUrl,
};
use alphavantage_doc_extractor::utils::UrlParseError;
use chrono::Utc;
use serde_json;

/// Test newtypes and URL validation
#[cfg(test)]
mod newtype_tests {
    use super::*;

    #[test]
    fn test_raw_url_creation() {
        let raw = RawUrl::new("https://example.com".to_string());
        assert_eq!(raw.to_string(), "https://example.com");
        assert_eq!(raw.into_inner(), "https://example.com");
    }

    #[test]
    fn test_validated_url_success() {
        let raw = RawUrl::new("https://www.alphavantage.co/documentation/".to_string());
        let validated = ValidatedUrl::validate(raw).unwrap();
        assert_eq!(
            validated.as_str(),
            "https://www.alphavantage.co/documentation/"
        );
        assert!(validated.as_str().starts_with("https://"));
    }

    #[test]
    fn test_validated_url_invalid_syntax() {
        let raw = RawUrl::new("not-a-url".to_string());
        let result = ValidatedUrl::try_from(raw);
        assert!(result.is_err());
        match result.unwrap_err() {
            UrlParseError::ParseFailed(_) => {}
            _ => panic!("Expected ParseFailed error"),
        }
    }

    #[test]
    fn test_validated_url_unsupported_scheme() {
        let raw = RawUrl::new("ftp://example.com".to_string());
        let result = ValidatedUrl::try_from(raw);
        assert!(result.is_err());
        match result.unwrap_err() {
            UrlParseError::UnsupportedScheme(scheme) => assert_eq!(scheme, "ftp"),
            _ => panic!("Expected UnsupportedScheme error"),
        }
    }

    #[test]
    fn test_validated_url_missing_host() {
        let raw = RawUrl::new("https://".to_string());
        let result = ValidatedUrl::try_from(raw);
        // url::Url::parse fails for "https://" with empty host
        assert!(result.is_err());
        match result.unwrap_err() {
            UrlParseError::ParseFailed(_) => {} // url::Url::parse fails for this
            _ => panic!("Expected ParseFailed error"),
        }
    }

    #[test]
    fn test_raw_html_creation() {
        let url = ValidatedUrl::validate(RawUrl::new("https://example.com".to_string())).unwrap();
        let html = RawHtml::new("<html></html>".to_string(), url.clone());

        assert_eq!(html.content, "<html></html>");
        assert_eq!(html.source_url.as_str(), "https://example.com/");
        assert!(html.fetched_at <= Utc::now());
        assert_eq!(html.content_length(), 13);
    }
}

/// Test core domain types and builder patterns
#[cfg(test)]
mod core_type_tests {
    use super::*;

    #[test]
    fn test_document_metadata_creation() {
        let url = ValidatedUrl::validate(RawUrl::new("https://example.com".to_string())).unwrap();
        let metadata = DocumentMetadata::new("Test Title".to_string(), url);

        assert_eq!(metadata.title, "Test Title");
        assert_eq!(metadata.source_url, "https://example.com/");
        assert!(metadata.extracted_at <= Utc::now());
        assert!(metadata.endpoint_count.is_none());
        assert!(metadata.category_count.is_none());
    }

    #[test]
    fn test_document_metadata_with_counts() {
        let url = ValidatedUrl::validate(RawUrl::new("https://example.com".to_string())).unwrap();
        let metadata = DocumentMetadata::new("Test Title".to_string(), url).with_counts(10, 5);

        assert_eq!(metadata.endpoint_count, Some(10));
        assert_eq!(metadata.category_count, Some(5));
    }

    #[test]
    fn test_api_category_creation() {
        let category = ApiCategory::new("Stocks".to_string());

        assert_eq!(category.name, "Stocks");
        assert!(category.description.is_none());
        assert!(category.endpoints.is_empty());
    }

    #[test]
    fn test_api_category_builder() {
        let endpoint = ApiEndpoint::new(
            "TIME_SERIES_DAILY".to_string(),
            "Get daily time series".to_string(),
        );
        let category = ApiCategory::new("Stocks".to_string())
            .with_description("Stock market data".to_string())
            .add_endpoint(endpoint);

        assert_eq!(category.name, "Stocks");
        assert_eq!(category.description, Some("Stock market data".to_string()));
        assert_eq!(category.endpoints.len(), 1);
    }

    #[test]
    fn test_api_endpoint_creation() {
        let endpoint = ApiEndpoint::new(
            "TIME_SERIES_DAILY".to_string(),
            "Get daily time series".to_string(),
        );

        assert_eq!(endpoint.function_name, "TIME_SERIES_DAILY");
        assert_eq!(endpoint.description, "Get daily time series");
        assert!(endpoint.required_params.is_empty());
        assert!(endpoint.optional_params.is_empty());
        assert!(endpoint.request_pattern.is_none());
        assert!(endpoint.python_example.is_none());
        assert!(!endpoint.premium_only);
    }

    #[test]
    fn test_api_endpoint_builder() {
        let required_param = Parameter::new(
            "symbol".to_string(),
            "string".to_string(),
            "Stock symbol".to_string(),
        );
        let python_example = CodeExample::python("print('hello')".to_string());

        let endpoint = ApiEndpoint::new(
            "TIME_SERIES_DAILY".to_string(),
            "Get daily time series".to_string(),
        )
        .with_required_params(vec![required_param])
        .with_request_pattern("https://www.alphavantage.co/query?function={function}".to_string())
        .with_python_example(python_example)
        .set_premium(true);

        assert_eq!(endpoint.required_params.len(), 1);
        assert_eq!(
            endpoint.request_pattern,
            Some("https://www.alphavantage.co/query?function={function}".to_string())
        );
        assert!(endpoint.python_example.is_some());
        assert!(endpoint.premium_only);
    }

    #[test]
    fn test_parameter_creation() {
        let param = Parameter::new(
            "symbol".to_string(),
            "string".to_string(),
            "Stock symbol".to_string(),
        );

        assert_eq!(param.name, "symbol");
        assert_eq!(param.value_type, "string");
        assert_eq!(param.description, "Stock symbol");
        assert!(param.default.is_none());
        assert!(param.options.is_empty());
    }

    #[test]
    fn test_parameter_builder() {
        let param = Parameter::new(
            "datatype".to_string(),
            "string".to_string(),
            "Return data format".to_string(),
        )
        .with_default("json".to_string())
        .with_options(vec!["json".to_string(), "csv".to_string()]);

        assert_eq!(param.default, Some("json".to_string()));
        assert_eq!(param.options, vec!["json".to_string(), "csv".to_string()]);
    }

    #[test]
    fn test_code_example_creation() {
        let python = CodeExample::python("print('hello')".to_string());
        assert_eq!(python.language, "python");
        assert_eq!(python.code, "print('hello')");

        let bash = CodeExample::bash("echo hello".to_string());
        assert_eq!(bash.language, "bash");
        assert_eq!(bash.code, "echo hello");
    }

    #[test]
    fn test_document_structure_creation() {
        let url = ValidatedUrl::validate(RawUrl::new("https://example.com".to_string())).unwrap();
        let metadata = DocumentMetadata::new("Test".to_string(), url);
        let category = ApiCategory::new("Test Category".to_string());
        let structure = DocumentStructure::new(metadata, vec![category]);

        assert_eq!(structure.metadata.title, "Test");
        assert_eq!(structure.categories.len(), 1);
        assert_eq!(structure.total_endpoints(), 0);
    }
}

/// Test validation rules for all types
#[cfg(test)]
mod validation_tests {
    use super::*;

    #[test]
    fn test_valid_api_endpoint() {
        let required_param = Parameter::new(
            "symbol".to_string(),
            "string".to_string(),
            "Stock symbol".to_string(),
        );
        let endpoint = ApiEndpoint::new(
            "TIME_SERIES_DAILY".to_string(),
            "Get daily time series".to_string(),
        )
        .with_required_params(vec![required_param]);

        assert!(endpoint.validate().is_ok());
    }

    #[test]
    fn test_invalid_function_name() {
        let endpoint = ApiEndpoint::new(
            "time_series_daily".to_string(),
            "Get daily time series".to_string(),
        );
        let result = endpoint.validate();
        assert!(result.is_err());
        assert!(result.unwrap_err().contains("UPPERCASE_WITH_UNDERSCORES"));
    }

    #[test]
    fn test_empty_description() {
        let endpoint = ApiEndpoint::new("TIME_SERIES_DAILY".to_string(), "".to_string());
        let result = endpoint.validate();
        assert!(result.is_err());
        assert!(result.unwrap_err().contains("must have a description"));
    }

    #[test]
    fn test_no_required_params() {
        let endpoint = ApiEndpoint::new(
            "TIME_SERIES_DAILY".to_string(),
            "Get daily time series".to_string(),
        );
        let result = endpoint.validate();
        assert!(result.is_err());
        assert!(result
            .unwrap_err()
            .contains("at least one required parameter"));
    }

    #[test]
    fn test_invalid_parameter() {
        let bad_param = Parameter::new(
            "".to_string(),
            "string".to_string(),
            "Empty name".to_string(),
        );
        let endpoint = ApiEndpoint::new("TIME_SERIES_DAILY".to_string(), "Test".to_string())
            .with_required_params(vec![bad_param]);
        let result = endpoint.validate();
        assert!(result.is_err());
        assert!(result
            .unwrap_err()
            .contains("Parameter name cannot be empty"));
    }

    #[test]
    fn test_invalid_category() {
        let category = ApiCategory::new("".to_string());
        let result = category.validate();
        assert!(result.is_err());
        assert!(result
            .unwrap_err()
            .contains("Category name cannot be empty"));
    }

    #[test]
    fn test_category_without_endpoints() {
        let category = ApiCategory::new("Empty Category".to_string());
        let result = category.validate();
        assert!(result.is_err());
        assert!(result
            .unwrap_err()
            .contains("must have at least one endpoint"));
    }

    #[test]
    fn test_empty_code_example() {
        let example = CodeExample::python("".to_string());
        let result = example.validate();
        assert!(result.is_err());
        assert!(result.unwrap_err().contains("cannot be empty"));
    }

    #[test]
    fn test_invalid_document_structure() {
        let url = ValidatedUrl::validate(RawUrl::new("https://example.com".to_string())).unwrap();
        let metadata = DocumentMetadata::new("Test".to_string(), url);
        let structure = DocumentStructure::new(metadata, vec![]);

        let result = structure.validate();
        assert!(result.is_err());
        assert!(result.unwrap_err().contains("at least one category"));
    }
}

/// Test edge cases and error conditions
#[cfg(test)]
mod edge_case_tests {
    use super::*;

    #[test]
    fn test_empty_strings() {
        // Test that empty strings are properly rejected
        let param = Parameter::new("".to_string(), "".to_string(), "".to_string());
        assert!(param.validate().is_err());

        let category = ApiCategory::new("".to_string());
        assert!(category.validate().is_err());

        let endpoint = ApiEndpoint::new("VALID_NAME".to_string(), "".to_string());
        assert!(endpoint.validate().is_err());
    }

    #[test]
    fn test_whitespace_only() {
        let param = Parameter::new("   ".to_string(), "string".to_string(), "   ".to_string());
        assert!(param.validate().is_err());

        let category = ApiCategory::new("   ".to_string());
        assert!(category.validate().is_err());

        let endpoint = ApiEndpoint::new("VALID_NAME".to_string(), "   ".to_string());
        assert!(endpoint.validate().is_err());
    }

    #[test]
    fn test_special_characters_in_function_names() {
        let invalid_names = vec![
            "time_series-daily",
            "TIME_SERIES.DAILY",
            "TIME SERIES DAILY",
            "TIME_SERIES_123", // Numbers not allowed in this context
        ];

        for name in invalid_names {
            let endpoint = ApiEndpoint::new(name.to_string(), "Test".to_string());
            assert!(
                endpoint.validate().is_err(),
                "Function name '{}' should be invalid",
                name
            );
        }
    }

    #[test]
    fn test_valid_function_names() {
        let valid_names = vec![
            "TIME_SERIES_DAILY",
            "SYMBOL_SEARCH",
            "CURRENCY_EXCHANGE_RATE",
            "SMA",
            "API_KEY",
        ];

        for name in valid_names {
            let required_param =
                Parameter::new("test".to_string(), "string".to_string(), "test".to_string());
            let endpoint = ApiEndpoint::new(name.to_string(), "Test".to_string())
                .with_required_params(vec![required_param]);
            assert!(
                endpoint.validate().is_ok(),
                "Function name '{}' should be valid",
                name
            );
        }
    }
}

/// Test serialization and deserialization
#[cfg(test)]
mod serialization_tests {
    use super::*;

    #[test]
    fn test_document_metadata_serialization() {
        let url = ValidatedUrl::validate(RawUrl::new("https://example.com".to_string())).unwrap();
        let metadata = DocumentMetadata::new("Test Title".to_string(), url).with_counts(10, 5);

        let serialized = serde_json::to_string(&metadata).unwrap();
        let deserialized: DocumentMetadata = serde_json::from_str(&serialized).unwrap();

        assert_eq!(deserialized.title, metadata.title);
        assert_eq!(deserialized.source_url, metadata.source_url);
        assert_eq!(deserialized.endpoint_count, metadata.endpoint_count);
        assert_eq!(deserialized.category_count, metadata.category_count);
    }

    #[test]
    fn test_api_endpoint_serialization() {
        let required_param = Parameter::new(
            "symbol".to_string(),
            "string".to_string(),
            "Stock symbol".to_string(),
        );
        let python_example = CodeExample::python("print('hello')".to_string());

        let endpoint = ApiEndpoint::new(
            "TIME_SERIES_DAILY".to_string(),
            "Get daily time series".to_string(),
        )
        .with_required_params(vec![required_param])
        .with_python_example(python_example)
        .set_premium(true);

        let serialized = serde_json::to_string(&endpoint).unwrap();
        let deserialized: ApiEndpoint = serde_json::from_str(&serialized).unwrap();

        assert_eq!(deserialized.function_name, endpoint.function_name);
        assert_eq!(deserialized.description, endpoint.description);
        assert_eq!(
            deserialized.required_params.len(),
            endpoint.required_params.len()
        );
        assert!(deserialized.premium_only);
    }

    #[test]
    fn test_parameter_serialization() {
        let param = Parameter::new(
            "datatype".to_string(),
            "string".to_string(),
            "Return data format".to_string(),
        )
        .with_default("json".to_string())
        .with_options(vec!["json".to_string(), "csv".to_string()]);

        let serialized = serde_json::to_string(&param).unwrap();
        let deserialized: Parameter = serde_json::from_str(&serialized).unwrap();

        assert_eq!(deserialized.name, param.name);
        assert_eq!(deserialized.value_type, param.value_type);
        assert_eq!(deserialized.default, param.default);
        assert_eq!(deserialized.options, param.options);
        assert_eq!(deserialized.description, param.description);
    }

    #[test]
    fn test_code_example_serialization() {
        let example = CodeExample::python("import requests\nprint('hello')".to_string());

        let serialized = serde_json::to_string(&example).unwrap();
        let deserialized: CodeExample = serde_json::from_str(&serialized).unwrap();

        assert_eq!(deserialized.language, example.language);
        assert_eq!(deserialized.code, example.code);
    }

    #[test]
    fn test_output_metadata_creation() {
        let metadata = OutputMetadata::new(
            "Alpha Vantage API".to_string(),
            "https://www.alphavantage.co/documentation/".to_string(),
            50,
            8,
            102400,
        );

        assert_eq!(metadata.source_title, "Alpha Vantage API");
        assert_eq!(metadata.endpoint_count, 50);
        assert_eq!(metadata.category_count, 8);
        assert_eq!(metadata.output_size_bytes, 102400);
        assert!(metadata.extracted_at <= Utc::now());
    }
}
