//! Markdown rendering for API documentation.
//!
//! This module provides functionality to render extracted API documentation
//! into well-formatted Markdown output with tables, code blocks, and proper
//! structure.
//!
//! The renderer follows the single responsibility principle - it only handles
//! the conversion from domain models to markdown strings.

use crate::domain::{ApiCategory, ApiEndpoint, CodeExample, DocumentStructure, Parameter};
use crate::utils::error::RenderResult;
use tracing::instrument;

/// Markdown renderer for API documentation.
///
/// Converts DocumentStructure into well-formatted markdown strings
/// with proper headings, tables, code blocks, and structure.
pub struct MarkdownRenderer;

impl Default for MarkdownRenderer {
    fn default() -> Self {
        Self::new()
    }
}

impl MarkdownRenderer {
    /// Create a new MarkdownRenderer instance.
    pub fn new() -> Self {
        Self
    }

    /// Render a complete document structure to markdown.
    ///
    /// Builds a complete markdown document with header, all categories,
    /// and LLM optimizations for better parsing.
    #[instrument(skip(self, document), fields(request_id = %uuid::Uuid::new_v4()))]
    pub fn render(&self, document: &DocumentStructure) -> RenderResult<String> {
        let mut markdown = String::new();

        // Add document header
        markdown.push_str(&self.render_document_header(&document.metadata));
        markdown.push('\n');

        // Add each category with proper spacing
        for category in &document.categories {
            markdown.push_str(&self.render_category(category));
            markdown.push_str("\n\n\n"); // Three newlines between categories
        }

        // Apply LLM optimizations
        let optimized = self.optimize_for_llm(markdown, &document.metadata, &document.categories);

        Ok(optimized)
    }

    /// Render the document header with metadata.
    ///
    /// Creates a title, metadata block, and horizontal rule.
    pub fn render_document_header(&self, metadata: &crate::domain::DocumentMetadata) -> String {
        let title = format!("# {}", metadata.title);

        let endpoint_count = metadata.endpoint_count.unwrap_or(0);
        let category_count = metadata.category_count.unwrap_or(0);

        let metadata_line = format!(
            "**Source:** {}, **Extracted:** {}, **Endpoints:** {}, **Categories:** {}",
            metadata.source_url,
            metadata.extracted_at.format("%Y-%m-%d %H:%M UTC"),
            endpoint_count,
            category_count
        );

        format!("{}\n\n{}\n\n---", title, metadata_line)
    }

    /// Render a complete category with all its endpoints.
    ///
    /// Creates a category heading, description (if present), and all endpoints.
    pub fn render_category(&self, category: &ApiCategory) -> String {
        let mut markdown = String::new();

        // Category heading
        markdown.push_str(&format!("## {}", category.name));
        markdown.push('\n');

        // Category description if present
        if let Some(ref desc) = category.description {
            markdown.push('\n');
            markdown.push_str(desc);
            markdown.push('\n');
        }

        // Render each endpoint with spacing
        for endpoint in &category.endpoints {
            markdown.push('\n');
            markdown.push_str(&self.render_endpoint(endpoint));
            markdown.push('\n');
        }

        markdown
    }

    /// Render a complete endpoint with all its components.
    ///
    /// Includes function name, premium badge, description, parameters,
    /// code example, and request pattern.
    pub fn render_endpoint(&self, endpoint: &ApiEndpoint) -> String {
        let mut markdown = String::new();

        // Function heading with premium badge if applicable
        markdown.push_str(&format!("### {}", endpoint.function_name));
        if endpoint.premium_only {
            markdown.push_str(" `[PREMIUM]`");
        }
        markdown.push('\n');

        // Description
        markdown.push('\n');
        markdown.push_str(&endpoint.description);
        markdown.push('\n');

        // Parameters
        markdown.push_str(
            &self.render_parameters(&endpoint.required_params, &endpoint.optional_params),
        );

        // Code example if present
        if let Some(ref example) = endpoint.python_example {
            markdown.push('\n');
            markdown.push_str(&self.render_code_example(example));
            markdown.push('\n');
        }

        // Request pattern if present
        if let Some(ref pattern) = endpoint.request_pattern {
            markdown.push_str(&self.render_request_pattern(pattern));
        }

        markdown
    }

    /// Render required and optional parameters as tables.
    ///
    /// Creates separate sections for required and optional parameters
    /// with proper markdown tables.
    pub fn render_parameters(&self, required: &[Parameter], optional: &[Parameter]) -> String {
        let mut markdown = String::new();

        // Required parameters section
        if !required.is_empty() {
            markdown.push_str("**Required Parameters:**\n\n");
            markdown.push_str(&self.render_parameter_table(required));
            markdown.push('\n');
        }

        // Optional parameters section
        if !optional.is_empty() {
            markdown.push_str("**Optional Parameters:**\n\n");
            markdown.push_str(&self.render_parameter_table(optional));
            markdown.push('\n');
        }

        markdown
    }

    /// Render a parameter table for a slice of parameters.
    ///
    /// Creates a markdown table with columns: Parameter, Type, Description, Default, Options.
    pub fn render_parameter_table(&self, params: &[Parameter]) -> String {
        let mut markdown = String::new();

        // Table header
        markdown.push_str("| Parameter | Type | Description | Default | Options |\n");
        markdown.push_str("|-----------|------|-------------|---------|---------|\n");

        // Table rows
        for param in params {
            markdown.push_str(&self.render_parameter_row(param));
        }

        markdown
    }

    /// Render a single parameter as a table row.
    ///
    /// Escapes pipe characters in description and options fields.
    pub fn render_parameter_row(&self, param: &Parameter) -> String {
        let name = &param.name;
        let value_type = &param.value_type;
        let description = param.description.replace('|', "\\|");
        let default = param.default.as_deref().unwrap_or("-");
        let options = if param.options.is_empty() {
            "-".to_string()
        } else {
            param.options.join(", ").replace('|', "\\|")
        };

        format!(
            "| {} | {} | {} | {} | {} |\n",
            name, value_type, description, default, options
        )
    }

    /// Render a code example as a markdown code block.
    ///
    /// Uses the language field for syntax highlighting.
    pub fn render_code_example(&self, example: &CodeExample) -> String {
        format!("```{}\n{}\n```", example.language, example.code)
    }

    /// Render a request pattern as a code block.
    ///
    /// Uses 'http' language for syntax highlighting of URLs.
    pub fn render_request_pattern(&self, pattern: &str) -> String {
        format!("**API Request Pattern:**\n\n```http\n{}\n```", pattern)
    }

    /// Optimize markdown output for LLM consumption.
    ///
    /// Adds frontmatter, table of contents, semantic markers, and optimizes whitespace
    /// for better LLM parsing and token efficiency.
    #[instrument(skip(self, metadata, categories))]
    pub fn optimize_for_llm(
        &self,
        markdown: String,
        metadata: &crate::domain::DocumentMetadata,
        categories: &[crate::domain::ApiCategory],
    ) -> String {
        let mut optimized = String::new();

        // Add YAML frontmatter
        optimized.push_str(&self.add_frontmatter(metadata));
        optimized.push('\n');

        // Add table of contents
        optimized.push_str(&self.add_table_of_contents(categories));
        optimized.push('\n');

        // Add semantic markers and optimize whitespace
        let with_markers = self.add_semantic_markers(markdown);
        let optimized_markdown = self.optimize_whitespace(with_markers);

        optimized.push_str(&optimized_markdown);
        optimized
    }

    /// Generate YAML frontmatter for document metadata.
    ///
    /// Creates a YAML block with metadata for better LLM context.
    pub fn add_frontmatter(&self, metadata: &crate::domain::DocumentMetadata) -> String {
        let endpoint_count = metadata.endpoint_count.unwrap_or(0);
        let category_count = metadata.category_count.unwrap_or(0);

        format!(
            "---\ntitle: \"{}\"\nsource: \"{}\"\nextracted_at: \"{}\"\nendpoint_count: {}\ncategory_count: {}\nformat_version: \"2.0\"\n---",
            metadata.title,
            metadata.source_url,
            metadata.extracted_at.format("%Y-%m-%dT%H:%M:%SZ"),
            endpoint_count,
            category_count
        )
    }

    /// Add semantic markers as HTML comments for better LLM parsing.
    ///
    /// Inserts HTML comments before major sections to help LLMs understand structure.
    pub fn add_semantic_markers(&self, markdown: String) -> String {
        let mut result = String::new();
        let lines: Vec<&str> = markdown.lines().collect();

        for line in lines {
            // Category markers
            if line.starts_with("## ") && !line.starts_with("## Table of Contents") {
                let category_name = line.strip_prefix("## ").unwrap_or("");
                result.push_str(&format!("<!-- CATEGORY: {} -->\n", category_name));
            }
            // Endpoint markers
            else if line.starts_with("### ") {
                let function_name = line
                    .strip_prefix("### ")
                    .unwrap_or("")
                    .split_whitespace()
                    .next()
                    .unwrap_or("");
                result.push_str(&format!("<!-- ENDPOINT: {} -->\n", function_name));
            }
            // Parameter table markers
            else if line.contains("**Required Parameters:**") {
                result.push_str("<!-- PARAMETERS:REQUIRED -->\n");
            } else if line.contains("**Optional Parameters:**") {
                result.push_str("<!-- PARAMETERS:OPTIONAL -->\n");
            }
            // Code example markers
            else if line.starts_with("```python") {
                result.push_str("<!-- CODE:PYTHON -->\n");
            }
            // Request pattern markers
            else if line.contains("**API Request Pattern:**") {
                result.push_str("<!-- REQUEST_PATTERN -->\n");
            }

            result.push_str(line);
            result.push('\n');
        }

        // Remove trailing newline
        result.trim_end().to_string()
    }

    /// Optimize whitespace for consistent formatting and token efficiency.
    ///
    /// Normalizes spacing between sections for better LLM parsing.
    pub fn optimize_whitespace(&self, markdown: String) -> String {
        // For now, just trim trailing whitespace from lines
        // This is a simplified implementation - the full whitespace optimization
        // would require more complex parsing of the markdown structure
        markdown
            .lines()
            .map(|line| line.trim_end())
            .collect::<Vec<&str>>()
            .join("\n")
            .trim_end()
            .to_string()
    }

    /// Generate a table of contents with GitHub-style anchor links.
    ///
    /// Creates a hierarchical TOC with proper anchor slugs.
    pub fn add_table_of_contents(&self, categories: &[crate::domain::ApiCategory]) -> String {
        let mut toc = String::from("## Table of Contents\n\n");

        for category in categories {
            let category_slug = self.create_slug(&category.name);
            toc.push_str(&format!("- [{}](#{})\n", category.name, category_slug));

            for endpoint in &category.endpoints {
                let endpoint_slug = self.create_slug(&endpoint.function_name);
                toc.push_str(&format!(
                    "  - [{}](#{})\n",
                    endpoint.function_name, endpoint_slug
                ));
            }

            toc.push('\n');
        }

        toc.trim_end().to_string()
    }

    /// Create a GitHub-style anchor slug from text.
    ///
    /// Converts text to lowercase, replaces spaces with hyphens,
    /// and removes special characters.
    pub fn create_slug(&self, text: &str) -> String {
        text.to_lowercase()
            .chars()
            .map(|c| {
                if c.is_alphanumeric() || c == ' ' || c == '-' {
                    if c == ' ' {
                        '-'
                    } else {
                        c
                    }
                } else {
                    '-' // Replace special chars with hyphens
                }
            })
            .collect::<String>()
            .split('-')
            .filter(|s| !s.is_empty())
            .collect::<Vec<&str>>()
            .join("-")
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::domain::{
        ApiCategory, ApiEndpoint, CodeExample, DocumentMetadata, DocumentStructure, Parameter,
        RawUrl, ValidatedUrl,
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
        let metadata =
            DocumentMetadata::new("Test API Documentation".to_string(), url).with_counts(1, 1);

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

        let result = renderer.render(&document);
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

        let endpoint1 = ApiEndpoint::new("ENDPOINT_ONE".to_string(), "First endpoint".to_string())
            .with_required_params(vec![Parameter::new(
                "param1".to_string(),
                "string".to_string(),
                "First parameter".to_string(),
            )]);

        let endpoint2 = ApiEndpoint::new("ENDPOINT_TWO".to_string(), "Second endpoint".to_string())
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
            Parameter::new(
                "api_key".to_string(),
                "string".to_string(),
                "API key".to_string(),
            ),
            Parameter::new(
                "symbol".to_string(),
                "string".to_string(),
                "Stock symbol".to_string(),
            ),
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
print(response.json())"#
                .to_string(),
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
            Parameter::new(
                "simple".to_string(),
                "string".to_string(),
                "Simple parameter".to_string(),
            ),
            Parameter::new(
                "with_default".to_string(),
                "integer".to_string(),
                "Has default".to_string(),
            )
            .with_default("100".to_string()),
            Parameter::new(
                "with_options".to_string(),
                "string".to_string(),
                "Has options".to_string(),
            )
            .with_options(vec!["option1".to_string(), "option2".to_string()]),
            Parameter::new(
                "with_all".to_string(),
                "boolean".to_string(),
                "Has default and options".to_string(),
            )
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
        assert!(result
            .contains("| with_all | boolean | Has default and options | true | true, false |"));
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
        assert_eq!(result, "```python\nprint('Hello, World!')\n```");
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

        let url =
            ValidatedUrl::validate(RawUrl::new("https://docs.example.com".to_string())).unwrap();
        let metadata = DocumentMetadata::new("Sample API Docs".to_string(), url).with_counts(42, 7);

        let result = renderer.render_document_header(&metadata);

        assert!(result.contains("# Sample API Docs"));
        assert!(result.contains("https://docs.example.com"));
        assert!(result.contains("**Endpoints:** 42"));
        assert!(result.contains("**Categories:** 7"));
        assert!(result.contains("---"));
    }

    /// Test YAML frontmatter generation
    #[test]
    fn test_add_frontmatter() {
        let renderer = MarkdownRenderer::new();

        let url =
            ValidatedUrl::validate(RawUrl::new("https://api.example.com".to_string())).unwrap();
        let metadata = DocumentMetadata::new("Test API".to_string(), url).with_counts(10, 3);

        let result = renderer.add_frontmatter(&metadata);

        assert!(result.starts_with("---"));
        assert!(result.ends_with("---"));
        assert!(result.contains("title: \"Test API\""));
        assert!(result.contains("source: \"https://api.example.com/\""));
        assert!(result.contains("endpoint_count: 10"));
        assert!(result.contains("category_count: 3"));
        assert!(result.contains("format_version: \"2.0\""));
    }

    /// Test semantic markers insertion
    #[test]
    fn test_add_semantic_markers() {
        let renderer = MarkdownRenderer::new();

        let markdown = "# Title\n\n## Stock Data\n\n### TIME_SERIES_DAILY\n\n**Required Parameters:**\n\n| Param | Type | Description |\n\n**Optional Parameters:**\n\n```python\nprint('hello')\n```\n\n**API Request Pattern:**\n\n```http\nGET /query\n```";

        let result = renderer.add_semantic_markers(markdown.to_string());

        assert!(result.contains("<!-- CATEGORY: Stock Data -->"));
        assert!(result.contains("<!-- ENDPOINT: TIME_SERIES_DAILY -->"));
        assert!(result.contains("<!-- PARAMETERS:REQUIRED -->"));
        assert!(result.contains("<!-- PARAMETERS:OPTIONAL -->"));
        assert!(result.contains("<!-- CODE:PYTHON -->"));
        assert!(result.contains("<!-- REQUEST_PATTERN -->"));
    }

    /// Test whitespace optimization
    #[test]
    fn test_optimize_whitespace() {
        let renderer = MarkdownRenderer::new();

        let markdown = "# Title  \n## Category\n### Endpoint\n";

        let result = renderer.optimize_whitespace(markdown.to_string());

        // Should trim trailing whitespace
        assert!(!result.contains("  \n"));
        assert!(result.contains("# Title\n"));
    }

    /// Test table of contents generation
    #[test]
    fn test_add_table_of_contents() {
        let renderer = MarkdownRenderer::new();

        let endpoint1 = ApiEndpoint::new("GET_DATA".to_string(), "Get data".to_string())
            .with_required_params(vec![Parameter::new(
                "key".to_string(),
                "string".to_string(),
                "API key".to_string(),
            )]);

        let endpoint2 = ApiEndpoint::new("POST_DATA".to_string(), "Post data".to_string())
            .with_required_params(vec![Parameter::new(
                "key".to_string(),
                "string".to_string(),
                "API key".to_string(),
            )]);

        let category1 = ApiCategory::new("Data Operations".to_string()).add_endpoint(endpoint1);

        let category2 = ApiCategory::new("Admin Operations".to_string()).add_endpoint(endpoint2);

        let categories = vec![category1, category2];

        let result = renderer.add_table_of_contents(&categories);

        assert!(result.starts_with("## Table of Contents"));
        assert!(result.contains("- [Data Operations](#data-operations)"));
        assert!(result.contains("  - [GET_DATA](#get-data)"));
        assert!(result.contains("- [Admin Operations](#admin-operations)"));
        assert!(result.contains("  - [POST_DATA](#post-data)"));
    }

    /// Test slug creation
    #[test]
    fn test_create_slug() {
        let renderer = MarkdownRenderer::new();

        assert_eq!(renderer.create_slug("Simple Text"), "simple-text");
        assert_eq!(renderer.create_slug("UPPER CASE"), "upper-case");
        assert_eq!(renderer.create_slug("Special@Chars!"), "special-chars");
        assert_eq!(renderer.create_slug("Multiple   Spaces"), "multiple-spaces");
        assert_eq!(
            renderer.create_slug("API_Function_Name"),
            "api-function-name"
        );
        assert_eq!(renderer.create_slug("123Numbers"), "123numbers");
    }

    /// Test full LLM optimization pipeline
    #[test]
    fn test_optimize_for_llm() {
        let renderer = MarkdownRenderer::new();

        let url = ValidatedUrl::validate(RawUrl::new("https://api.test.com".to_string())).unwrap();
        let metadata = DocumentMetadata::new("Test API".to_string(), url).with_counts(2, 1);

        let endpoint = ApiEndpoint::new("TEST_ENDPOINT".to_string(), "A test endpoint".to_string())
            .with_required_params(vec![Parameter::new(
                "key".to_string(),
                "string".to_string(),
                "API key".to_string(),
            )]);

        let category = ApiCategory::new("Test Category".to_string()).add_endpoint(endpoint);

        let categories = vec![category];

        let basic_markdown =
            "# Test API\n\n## Test Category\n\n### TEST_ENDPOINT\n\nA test endpoint".to_string();

        let result = renderer.optimize_for_llm(basic_markdown, &metadata, &categories);

        // Should contain frontmatter
        assert!(result.contains("---"));
        assert!(result.contains("title: \"Test API\""));
        assert!(result.contains("format_version: \"2.0\""));

        // Should contain TOC
        assert!(result.contains("## Table of Contents"));
        assert!(result.contains("- [Test Category](#test-category)"));

        // Should contain semantic markers
        assert!(result.contains("<!-- CATEGORY: Test Category -->"));
        assert!(result.contains("<!-- ENDPOINT: TEST_ENDPOINT -->"));
    }
}
