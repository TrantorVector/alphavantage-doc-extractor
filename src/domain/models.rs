//! Domain models for the Alpha Vantage Documentation Extractor.
//!
//! This module contains type-safe domain models using the builder pattern,
//! newtypes for strong typing, and comprehensive validation.
//!
//! All models follow parse-don't-validate principle and use type-state pattern
//! where appropriate.

use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use std::fmt;

/// Type-safe newtype for unvalidated URL strings.
/// Cannot be used directly - must be validated into ValidatedUrl.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct RawUrl(pub String);

impl RawUrl {
    /// Create a new RawUrl from a string.
    pub fn new(url: String) -> Self {
        Self(url)
    }

    /// Get the inner string value.
    pub fn into_inner(self) -> String {
        self.0
    }
}

impl fmt::Display for RawUrl {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.0)
    }
}

/// Type-safe newtype for validated URLs.
/// Can only be created through validation, ensuring URL validity.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ValidatedUrl {
    inner: url::Url,
}

impl ValidatedUrl {
    /// Validate and create a ValidatedUrl from a RawUrl.
    /// Returns an error if the URL is invalid or uses unsupported scheme.
    pub fn validate(raw: RawUrl) -> Result<Self, String> {
        let url = url::Url::parse(&raw.0).map_err(|e| format!("Invalid URL '{}': {}", raw.0, e))?;

        // Only allow http and https schemes
        if url.scheme() != "http" && url.scheme() != "https" {
            return Err(format!(
                "Unsupported URL scheme '{}', only http/https supported",
                url.scheme()
            ));
        }

        // Must have a host
        if url.host_str().is_none() {
            return Err(format!("URL missing host: {}", url));
        }

        Ok(Self { inner: url })
    }

    /// Get the URL as a string.
    pub fn as_str(&self) -> &str {
        self.inner.as_str()
    }

    /// Get the inner Url for advanced operations.
    pub fn inner(&self) -> &url::Url {
        &self.inner
    }
}

impl fmt::Display for ValidatedUrl {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.inner)
    }
}

impl Serialize for ValidatedUrl {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        serializer.serialize_str(self.as_str())
    }
}

impl<'de> Deserialize<'de> for ValidatedUrl {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        let url_str = String::deserialize(deserializer)?;
        let raw_url = RawUrl::new(url_str);
        ValidatedUrl::validate(raw_url).map_err(serde::de::Error::custom)
    }
}

/// Raw HTML content with metadata about when and where it was fetched.
#[derive(Debug, Clone)]
pub struct RawHtml {
    pub content: String,
    pub source_url: ValidatedUrl,
    pub fetched_at: DateTime<Utc>,
}

impl RawHtml {
    /// Create new RawHtml from content and source URL.
    pub fn new(content: String, source_url: ValidatedUrl) -> Self {
        Self {
            content,
            source_url,
            fetched_at: Utc::now(),
        }
    }

    /// Get the content length in bytes.
    pub fn content_length(&self) -> usize {
        self.content.len()
    }
}

/// Parsed HTML document with DOM and metadata.
#[derive(Debug)]
pub struct ParsedDocument {
    pub dom: scraper::Html,
    pub metadata: DocumentMetadata,
}

impl ParsedDocument {
    /// Create a new ParsedDocument from HTML and metadata.
    pub fn new(dom: scraper::Html, metadata: DocumentMetadata) -> Self {
        Self { dom, metadata }
    }

    /// Get the document title from metadata.
    pub fn title(&self) -> &str {
        &self.metadata.title
    }
}

/// Metadata about a document's extraction process.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DocumentMetadata {
    pub title: String,
    pub source_url: String, // Stored as string for serialization
    pub extracted_at: DateTime<Utc>,
    pub endpoint_count: Option<usize>,
    pub category_count: Option<usize>,
}

impl DocumentMetadata {
    /// Create new DocumentMetadata with title and source URL.
    pub fn new(title: String, source_url: ValidatedUrl) -> Self {
        Self {
            title,
            source_url: source_url.as_str().to_string(),
            extracted_at: Utc::now(),
            endpoint_count: None,
            category_count: None,
        }
    }

    /// Builder method to set endpoint and category counts.
    pub fn with_counts(mut self, endpoint_count: usize, category_count: usize) -> Self {
        self.endpoint_count = Some(endpoint_count);
        self.category_count = Some(category_count);
        self
    }
}

/// Complete document structure with metadata and categories.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DocumentStructure {
    pub metadata: DocumentMetadata,
    pub categories: Vec<ApiCategory>,
}

impl DocumentStructure {
    /// Create new DocumentStructure.
    pub fn new(metadata: DocumentMetadata, categories: Vec<ApiCategory>) -> Self {
        Self {
            metadata,
            categories,
        }
    }

    /// Get total number of endpoints across all categories.
    pub fn total_endpoints(&self) -> usize {
        self.categories.iter().map(|cat| cat.endpoints.len()).sum()
    }

    /// Validate the document structure.
    pub fn validate(&self) -> Result<(), String> {
        if self.categories.is_empty() {
            return Err("Document must have at least one category".to_string());
        }

        for category in &self.categories {
            category.validate()?;
        }

        Ok(())
    }
}

/// API category containing multiple endpoints.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ApiCategory {
    pub name: String,
    pub description: Option<String>,
    pub endpoints: Vec<ApiEndpoint>,
}

impl ApiCategory {
    /// Create new ApiCategory with name.
    pub fn new(name: String) -> Self {
        Self {
            name,
            description: None,
            endpoints: Vec::new(),
        }
    }

    /// Builder method to set description.
    pub fn with_description(mut self, description: String) -> Self {
        self.description = Some(description);
        self
    }

    /// Builder method to add an endpoint.
    pub fn add_endpoint(mut self, endpoint: ApiEndpoint) -> Self {
        self.endpoints.push(endpoint);
        self
    }

    /// Validate the category.
    pub fn validate(&self) -> Result<(), String> {
        if self.name.trim().is_empty() {
            return Err("Category name cannot be empty".to_string());
        }

        if self.endpoints.is_empty() {
            return Err(format!(
                "Category '{}' must have at least one endpoint",
                self.name
            ));
        }

        for endpoint in &self.endpoints {
            endpoint.validate()?;
        }

        Ok(())
    }
}

/// API endpoint with parameters and examples.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ApiEndpoint {
    pub function_name: String,
    pub description: String,
    pub required_params: Vec<Parameter>,
    pub optional_params: Vec<Parameter>,
    pub request_pattern: Option<String>,
    pub python_example: Option<CodeExample>,
    pub premium_only: bool,
}

impl ApiEndpoint {
    /// Create new ApiEndpoint with function name and description.
    pub fn new(function_name: String, description: String) -> Self {
        Self {
            function_name,
            description,
            required_params: Vec::new(),
            optional_params: Vec::new(),
            request_pattern: None,
            python_example: None,
            premium_only: false,
        }
    }

    /// Builder method to set required parameters.
    pub fn with_required_params(mut self, params: Vec<Parameter>) -> Self {
        self.required_params = params;
        self
    }

    /// Builder method to set optional parameters.
    pub fn with_optional_params(mut self, params: Vec<Parameter>) -> Self {
        self.optional_params = params;
        self
    }

    /// Builder method to set request pattern.
    pub fn with_request_pattern(mut self, pattern: String) -> Self {
        self.request_pattern = Some(pattern);
        self
    }

    /// Builder method to set Python example.
    pub fn with_python_example(mut self, example: CodeExample) -> Self {
        self.python_example = Some(example);
        self
    }

    /// Builder method to set premium status.
    pub fn set_premium(mut self, premium: bool) -> Self {
        self.premium_only = premium;
        self
    }

    /// Validate the endpoint.
    pub fn validate(&self) -> Result<(), String> {
        // Function name must be UPPERCASE_WITH_UNDERSCORES
        if !self
            .function_name
            .chars()
            .all(|c| c.is_ascii_uppercase() || c == '_')
        {
            return Err(format!(
                "Function name '{}' must be UPPERCASE_WITH_UNDERSCORES",
                self.function_name
            ));
        }

        if self.description.trim().is_empty() {
            return Err(format!(
                "Endpoint '{}' must have a description",
                self.function_name
            ));
        }

        if self.required_params.is_empty() {
            return Err(format!(
                "Endpoint '{}' must have at least one required parameter",
                self.function_name
            ));
        }

        // Validate all parameters
        for param in &self.required_params {
            param
                .validate()
                .map_err(|e| format!("Required parameter error: {}", e))?;
        }
        for param in &self.optional_params {
            param
                .validate()
                .map_err(|e| format!("Optional parameter error: {}", e))?;
        }

        // Validate Python example if present
        if let Some(ref example) = self.python_example {
            example
                .validate()
                .map_err(|e| format!("Python example error: {}", e))?;
        }

        Ok(())
    }
}

/// API parameter with type and validation information.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Parameter {
    pub name: String,
    pub value_type: String,
    pub default: Option<String>,
    pub options: Vec<String>,
    pub description: String,
}

impl Parameter {
    /// Create new Parameter.
    pub fn new(name: String, value_type: String, description: String) -> Self {
        Self {
            name,
            value_type,
            default: None,
            options: Vec::new(),
            description,
        }
    }

    /// Builder method to set default value.
    pub fn with_default(mut self, default: String) -> Self {
        self.default = Some(default);
        self
    }

    /// Builder method to set options.
    pub fn with_options(mut self, options: Vec<String>) -> Self {
        self.options = options;
        self
    }

    /// Validate the parameter.
    pub fn validate(&self) -> Result<(), String> {
        if self.name.trim().is_empty() {
            return Err("Parameter name cannot be empty".to_string());
        }

        if self.value_type.trim().is_empty() {
            return Err(format!("Parameter '{}' must have a value type", self.name));
        }

        if self.description.trim().is_empty() {
            return Err(format!("Parameter '{}' must have a description", self.name));
        }

        Ok(())
    }
}

/// Code example in a specific programming language.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CodeExample {
    pub language: String,
    pub code: String,
}

impl CodeExample {
    /// Create a Python code example.
    pub fn python(code: String) -> Self {
        Self {
            language: "python".to_string(),
            code,
        }
    }

    /// Create a Bash code example.
    pub fn bash(code: String) -> Self {
        Self {
            language: "bash".to_string(),
            code,
        }
    }

    /// Validate the code example.
    pub fn validate(&self) -> Result<(), String> {
        if self.code.trim().is_empty() {
            return Err(format!("{} code example cannot be empty", self.language));
        }

        Ok(())
    }
}

/// Metadata about the final output.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct OutputMetadata {
    pub source_title: String,
    pub source_url: String,
    pub extracted_at: DateTime<Utc>,
    pub endpoint_count: usize,
    pub category_count: usize,
    pub output_size_bytes: usize,
}

impl OutputMetadata {
    /// Create new OutputMetadata.
    pub fn new(
        source_title: String,
        source_url: String,
        endpoint_count: usize,
        category_count: usize,
        output_size_bytes: usize,
    ) -> Self {
        Self {
            source_title,
            source_url,
            extracted_at: Utc::now(),
            endpoint_count,
            category_count,
            output_size_bytes,
        }
    }
}
