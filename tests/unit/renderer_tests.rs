//! Unit tests for the MarkdownRenderer.

use alphavantage_doc_extractor::domain::{
    ApiCategory, ApiEndpoint, CodeExample, DocumentMetadata, DocumentStructure, MarkdownRenderer,
    Parameter, RawUrl, ValidatedUrl,
};

/// Test that renderer can be created
#[test]
fn test_renderer_creation() {
    let _renderer = MarkdownRenderer::new();
}

/// Test rendering a document with single category
#[test]
fn test_render_document_single_category() {
    let renderer = MarkdownRenderer::new();

    let url = ValidatedUrl::validate(RawUrl::new("https://example.com".to_string())).unwrap();
    let metadata = DocumentMetadata::new("Test API Documentation".to_string(), url).with_counts(1, 1);

    let endpoint = ApiEndpoint::new(
        "TEST_FUNCTION".to_string(),
        "This is a test endpoint".to_string(),
    )
    .with_required_params(vec![Parameter::new(
        "api_key".to_string(),
        "string".to_string(),
        "Your API key".to_string(),
    )]);

    let category = ApiCategory::new("Test Category".to_string()).add_endpoint(endpoint);

    let document = DocumentStructure::new(metadata, vec![category]);

    let result = renderer.render(&document, true);
    assert!(result.is_ok());

    let markdown = result.unwrap();
    assert!(markdown.contains("# Test API Documentation"));
    assert!(markdown.contains("## Test Category"));
    assert!(markdown.contains("### TEST_FUNCTION"));
    assert!(markdown.contains("This is a test endpoint"));
}

/// Test rendering a category with multiple endpoints
#[test]
fn test_render_category_multiple_endpoints() {
    let renderer = MarkdownRenderer::new();

    let endpoint1 = ApiEndpoint::new(
        "ENDPOINT_ONE".to_string(),
        "First endpoint".to_string(),
    )
    .with_required_params(vec![Parameter::new(
        "param1".to_string(),
        "string".to_string(),
        "First parameter".to_string(),
    )]);

    let endpoint2 = ApiEndpoint::new(
        "ENDPOINT_TWO".to_string(),
        "Second endpoint".to_string(),
    )
    .with_required_params(vec![Parameter::new(
        "param2".to_string(),
        "integer".to_string(),
        "Second parameter".to_string(),
    )]);

    let category = ApiCategory::new("Multi Endpoint Category".to_string())
        .add_endpoint(endpoint1)
        .add_endpoint(endpoint2);

    let result = renderer.render_category(&category);
    assert!(result.contains("## Multi Endpoint Category"));
    assert!(result.contains("### ENDPOINT_ONE"));
    assert!(result.contains("### ENDPOINT_TWO"));
    assert!(result.contains("First endpoint"));
    assert!(result.contains("Second endpoint"));
}

/// Test rendering an endpoint with all fields
#[test]
fn test_render_endpoint_complete() {
    let renderer = MarkdownRenderer::new();

    let required_params = vec![
        Parameter::new("api_key".to_string(), "string".to_string(), "API key".to_string()),
        Parameter::new("symbol".to_string(), "string".to_string(), "Stock symbol".to_string()),
    ];

    let optional_params = vec![Parameter::new(
        "outputsize".to_string(),
        "string".to_string(),
        "Output size".to_string(),
    )
    .with_default("compact".to_string())
    .with_options(vec!["compact".to_string(), "full".to_string()])];

    let code_example = CodeExample::python(
        r#"import requests

response = requests.get("https://api.example.com/data")
print(response.json())"#.to_string(),
    );

    let endpoint = ApiEndpoint::new(
        "GET_STOCK_DATA".to_string(),
        "Retrieves stock data for the given symbol".to_string(),
    )
    .with_required_params(required_params)
    .with_optional_params(optional_params)
    .with_python_example(code_example)
    .with_request_pattern("https://api.example.com/query?function=GET_STOCK_DATA&symbol={symbol}&apikey={api_key}&outputsize={outputsize}".to_string())
    .set_premium(true);

    let result = renderer.render_endpoint(&endpoint);
    assert!(result.contains("### GET_STOCK_DATA `[PREMIUM]`"));
    assert!(result.contains("Retrieves stock data for the given symbol"));
    assert!(result.contains("**Required Parameters:**"));
    assert!(result.contains("**Optional Parameters:**"));
    assert!(result.contains("```python"));
    assert!(result.contains("import requests"));
    assert!(result.contains("**API Request Pattern:**"));
    assert!(result.contains("```http"));
}

/// Test rendering parameter table with defaults and options
#[test]
fn test_render_parameter_table_complete() {
    let renderer = MarkdownRenderer::new();

    let params = vec![
        Parameter::new("simple".to_string(), "string".to_string(), "Simple parameter".to_string()),
        Parameter::new("with_default".to_string(), "integer".to_string(), "Has default".to_string())
            .with_default("100".to_string()),
        Parameter::new("with_options".to_string(), "string".to_string(), "Has options".to_string())
            .with_options(vec!["option1".to_string(), "option2".to_string()]),
        Parameter::new("with_all".to_string(), "boolean".to_string(), "Has default and options".to_string())
            .with_default("true".to_string())
            .with_options(vec!["true".to_string(), "false".to_string()]),
    ];

    let result = renderer.render_parameter_table(&params);

    // Check table structure
    assert!(result.contains("| Parameter | Type | Description | Default | Options |"));
    assert!(result.contains("|-----------|------|-------------|---------|---------|"));

    // Check parameter rows
    assert!(result.contains("| simple | string | Simple parameter | - | - |"));
    assert!(result.contains("| with_default | integer | Has default | 100 | - |"));
    assert!(result.contains("| with_options | string | Has options | - | option1, option2 |"));
    assert!(result.contains("| with_all | boolean | Has default and options | true | true, false |"));
}

/// Test rendering parameter row with pipe character escaping
#[test]
fn test_render_parameter_row_escaping() {
    let renderer = MarkdownRenderer::new();

    let param = Parameter::new(
        "test".to_string(),
        "string".to_string(),
        "Description with | pipe character".to_string(),
    )
    .with_options(vec!["option|1".to_string(), "option|2".to_string()]);

    let result = renderer.render_parameter_row(&param);
    assert!(result.contains("Description with \\| pipe character"));
    assert!(result.contains("option\\|1, option\\|2"));
}

/// Test rendering code example
#[test]
fn test_render_code_example() {
    let renderer = MarkdownRenderer::new();

    let example = CodeExample::python("print('Hello, World!')".to_string());

    let result = renderer.render_code_example(&example);
    assert_eq!(
        result,
        "```python\nprint('Hello, World!')\n```"
    );
}

/// Test rendering request pattern
#[test]
fn test_render_request_pattern() {
    let renderer = MarkdownRenderer::new();

    let pattern = "https://api.example.com/query?function=TEST&param=value";
    let result = renderer.render_request_pattern(pattern);

    assert!(result.contains("**API Request Pattern:**"));
    assert!(result.contains("```http"));
    assert!(result.contains(pattern));
    assert!(result.contains("```"));
}

/// Test handling empty optional parameters
#[test]
fn test_render_parameters_empty_optional() {
    let renderer = MarkdownRenderer::new();

    let required = vec![Parameter::new(
        "required".to_string(),
        "string".to_string(),
        "Required param".to_string(),
    )];

    let optional = vec![];

    let result = renderer.render_parameters(&required, &optional);
    assert!(result.contains("**Required Parameters:**"));
    assert!(!result.contains("**Optional Parameters:**"));
}

/// Test handling missing code examples
#[test]
fn test_render_endpoint_no_code_example() {
    let renderer = MarkdownRenderer::new();

    let endpoint = ApiEndpoint::new(
        "NO_CODE_ENDPOINT".to_string(),
        "Endpoint without code example".to_string(),
    )
    .with_required_params(vec![Parameter::new(
        "param".to_string(),
        "string".to_string(),
        "A parameter".to_string(),
    )]);

    let result = renderer.render_endpoint(&endpoint);
    assert!(!result.contains("```"));
    assert!(!result.contains("**API Request Pattern:**"));
}

/// Test document header rendering
#[test]
fn test_render_document_header() {
    let renderer = MarkdownRenderer::new();

    let url = ValidatedUrl::validate(RawUrl::new("https://docs.example.com".to_string())).unwrap();
    let metadata = DocumentMetadata::new("Sample API Docs".to_string(), url).with_counts(42, 7);

    let result = renderer.render_document_header(&metadata);

    assert!(result.contains("# Sample API Docs"));
    assert!(result.contains("https://docs.example.com"));
    assert!(result.contains("Endpoints: 42"));
    assert!(result.contains("Categories: 7"));
    assert!(result.contains("---"));
}
