//! Content extraction logic for Alpha Vantage documentation.
//!
//! This module implements category and endpoint extraction from parsed HTML documents.
//! It follows the parse-don't-validate principle and uses structured error handling.

use crate::domain::{ApiCategory, ApiEndpoint};
use crate::utils::{error::ExtractionResult, error::ContentExtractionError};
use scraper::{ElementRef, Html, Selector};
use tracing::{instrument, warn};
use regex::Regex;

/// Content extractor for Alpha Vantage documentation pages.
///
/// This struct holds a reference to the parsed HTML document and provides
/// methods to extract categories and endpoints from the main content area.
pub struct ContentExtractor<'a> {
    /// Reference to the parsed HTML document
    document: &'a Html,
}

impl<'a> ContentExtractor<'a> {
    /// Create a new ContentExtractor for the given HTML document.
    ///
    /// # Arguments
    /// * `document` - Reference to the parsed HTML document
    ///
    /// # Returns
    /// A new ContentExtractor instance
    pub fn new(document: &'a Html) -> Self {
        Self { document }
    }

    /// Extract all API categories from the main content area.
    ///
    /// This method finds all `<h2>` elements in the main content and attempts
    /// to parse them into `ApiCategory` instances. Failed extractions are logged
    /// as warnings but don't stop the overall extraction process.
    ///
    /// # Arguments
    /// * `main_content` - The main content element to extract categories from
    ///
    /// # Returns
    /// A vector of successfully extracted categories, or an error if no categories found
    #[instrument(skip(self, main_content), fields(request_id = %uuid::Uuid::new_v4()))]
    pub fn extract_categories(&self, main_content: ElementRef) -> ExtractionResult<Vec<ApiCategory>> {
        let h2_selector = Selector::parse("h2").map_err(|e| {
            ContentExtractionError::CategoryExtractionFailed(
                "selector".to_string(),
                format!("Failed to parse h2 selector: {}", e),
            )
        })?;

        let h2_elements: Vec<ElementRef> = main_content.select(&h2_selector).collect();

        if h2_elements.is_empty() {
            return Err(ContentExtractionError::CategoryExtractionFailed(
                "no_h2_found".to_string(),
                "No h2 elements found in main content".to_string(),
            ));
        }

        let mut categories = Vec::new();

        for h2_element in h2_elements {
            match self.parse_category(h2_element) {
                Ok(category) => categories.push(category),
                Err(e) => {
                    warn!("Failed to parse category: {}", e);
                    // Continue with other categories
                }
            }
        }

        if categories.is_empty() {
            return Err(ContentExtractionError::CategoryExtractionFailed(
                "no_categories".to_string(),
                "No categories could be successfully extracted".to_string(),
            ));
        }

        Ok(categories)
    }

    /// Parse a single category from an h2 element.
    ///
    /// This method extracts the category name, optional description, and all
    /// endpoints from an h2 element and its following siblings.
    ///
    /// # Arguments
    /// * `h2_element` - The h2 element to parse
    ///
    /// # Returns
    /// A successfully parsed ApiCategory, or an error
    #[instrument(skip(self, h2_element))]
    fn parse_category(&self, h2_element: ElementRef) -> ExtractionResult<ApiCategory> {
        let name = self.extract_category_name(h2_element)?;
        let description = self.extract_category_description(h2_element);
        let endpoints = self.extract_endpoints(h2_element);

        let mut category = ApiCategory::new(name);
        if let Some(desc) = description {
            category = category.with_description(desc);
        }

        // Add all extracted endpoints to the category
        for endpoint in endpoints {
            category = category.add_endpoint(endpoint);
        }

        Ok(category)
    }

    /// Extract and normalize the category name from an h2 element.
    ///
    /// # Arguments
    /// * `h2_element` - The h2 element containing the category name
    ///
    /// # Returns
    /// The normalized category name, or an error if extraction fails
    #[instrument(skip(self, h2_element))]
    fn extract_category_name(&self, h2_element: ElementRef) -> ExtractionResult<String> {
        use crate::domain::parser::extract_text;

        let raw_name = extract_text(&h2_element);
        let normalized_name = self.normalize_category_name(&raw_name);

        if normalized_name.trim().is_empty() {
            return Err(ContentExtractionError::CategoryExtractionFailed(
                "empty_name".to_string(),
                "Category name is empty after normalization".to_string(),
            ));
        }

        Ok(normalized_name)
    }

    /// Normalize a category name by cleaning and standardizing it.
    ///
    /// This method:
    /// - Trims whitespace
    /// - Keeps only alphanumeric characters, spaces, parentheses, and underscores
    /// - Collapses multiple spaces into single spaces
    ///
    /// # Arguments
    /// * `name` - The raw category name to normalize
    ///
    /// # Returns
    /// The normalized category name
    fn normalize_category_name(&self, name: &str) -> String {
        // Keep only alphanumeric, spaces, parentheses, and underscores
        let filtered: String = name
            .chars()
            .filter(|c| c.is_alphanumeric() || c.is_whitespace() || *c == '(' || *c == ')' || *c == '_')
            .collect();

        // Collapse multiple spaces and trim
        filtered
            .split_whitespace()
            .collect::<Vec<&str>>()
            .join(" ")
    }

    /// Extract the category description from content following an h2 element.
    ///
    /// This method looks for the first `<p>` element after the h2, skipping
    /// promotional content and limiting the length to 500 characters.
    ///
    /// # Arguments
    /// * `h2_element` - The h2 element to find description for
    ///
    /// # Returns
    /// The category description if found and valid, None otherwise
    #[instrument(skip(self, h2_element))]
    fn extract_category_description(&self, h2_element: ElementRef) -> Option<String> {
        use crate::domain::parser::extract_text;

        // Find the next <p> element after this h2
        let mut current = h2_element.next_sibling();

        while let Some(sibling) = current {
            if let Some(element) = sibling.value().as_element() {
                // Stop if we hit another heading
                if element.name() == "h2" || element.name() == "h3" {
                    break;
                }

                // Check if this is a <p> element
                if element.name() == "p" {
                    let text = extract_text(&ElementRef::wrap(sibling).unwrap());

                    // Skip if too short
                    if text.len() < 20 {
                        current = sibling.next_sibling();
                        continue;
                    }

                    // Skip promotional content
                    let lower_text = text.to_lowercase();
                    if lower_text.contains("claim your")
                        || lower_text.contains("subscribe")
                        || lower_text.contains("follow us")
                    {
                        current = sibling.next_sibling();
                        continue;
                    }

                    // Limit to 500 characters
                    let description = if text.len() > 500 {
                        text.chars().take(500).collect()
                    } else {
                        text
                    };

                    return Some(description);
                }
            }

            current = sibling.next_sibling();
        }

        None
    }

    /// Extract all endpoints from a category (h2 element and its content).
    ///
    /// This method traverses siblings after the h2 element to find all h3 elements
    /// representing API endpoints, then parses each one into an ApiEndpoint.
    ///
    /// # Arguments
    /// * `h2_element` - The h2 element representing the category
    ///
    /// # Returns
    /// A vector of successfully parsed endpoints
    #[instrument(skip(self, h2_element))]
    pub fn extract_endpoints(&self, h2_element: ElementRef) -> Vec<ApiEndpoint> {
        let mut endpoints = Vec::new();
        let mut current = h2_element.next_sibling();

        // Traverse siblings until we hit another h2 (next category)
        while let Some(sibling) = current {
            if let Some(element) = sibling.value().as_element() {
                // Stop if we hit another h2
                if element.name() == "h2" {
                    break;
                }

                // Process h3 elements (endpoints)
                if element.name() == "h3" {
                    match self.parse_endpoint(ElementRef::wrap(sibling).unwrap()) {
                        Ok(endpoint) => endpoints.push(endpoint),
                        Err(e) => {
                            warn!("Failed to parse endpoint: {}", e);
                            // Continue with other endpoints
                        }
                    }
                }
            }

            current = sibling.next_sibling();
        }

        endpoints
    }

    /// Parse a single endpoint from an h3 element.
    ///
    /// # Arguments
    /// * `h3_element` - The h3 element representing the endpoint
    ///
    /// # Returns
    /// A successfully parsed ApiEndpoint, or an error
    #[instrument(skip(self, h3_element))]
    fn parse_endpoint(&self, h3_element: ElementRef) -> ExtractionResult<ApiEndpoint> {
        let function_name = self.extract_function_name(h3_element)?;
        let description = self.extract_endpoint_description(h3_element);
        let premium_only = self.detect_premium_endpoint(h3_element);

        let endpoint = ApiEndpoint::new(function_name, description)
            .set_premium(premium_only);

        Ok(endpoint)
    }

    /// Extract the function name from an h3 element using regex pattern.
    ///
    /// Looks for uppercase patterns like FUNCTION_NAME using regex [A-Z][A-Z_]*[A-Z]
    ///
    /// # Arguments
    /// * `h3_element` - The h3 element containing the function name
    ///
    /// # Returns
    /// The extracted function name, or an error if not found or invalid
    #[instrument(skip(self, h3_element))]
    fn extract_function_name(&self, h3_element: ElementRef) -> ExtractionResult<String> {
        use crate::domain::parser::extract_text;

        let h3_text = extract_text(&h3_element);

        // Regex pattern for uppercase function names: [A-Z][A-Z_]*[A-Z]
        // This matches patterns like TIME_SERIES_DAILY, OVERVIEW, etc.
        let function_pattern = Regex::new(r"[A-Z][A-Z_]*[A-Z]")
            .map_err(|e| ContentExtractionError::CategoryExtractionFailed(
                "regex_compilation".to_string(),
                format!("Failed to compile function name regex: {}", e),
            ))?;

        if let Some(captures) = function_pattern.find(&h3_text) {
            let function_name = captures.as_str().to_string();

            // Validate that it matches the expected format (no lowercase, proper underscores)
            if function_name.chars().all(|c| c.is_uppercase() || c == '_') {
                Ok(function_name)
            } else {
                Err(ContentExtractionError::CategoryExtractionFailed(
                    "invalid_function_format".to_string(),
                    format!("Function name '{}' contains invalid characters", function_name),
                ))
            }
        } else {
            Err(ContentExtractionError::CategoryExtractionFailed(
                "no_function_name".to_string(),
                format!("No valid function name found in h3 text: '{}'", h3_text),
            ))
        }
    }

    /// Extract the endpoint description from content following an h3 element.
    ///
    /// # Arguments
    /// * `h3_element` - The h3 element to find description for
    ///
    /// # Returns
    /// The endpoint description, or a default message if none found
    #[instrument(skip(self, h3_element))]
    fn extract_endpoint_description(&self, h3_element: ElementRef) -> String {
        use crate::domain::parser::extract_text;

        let mut current = h3_element.next_sibling();

        // Traverse siblings looking for a <p> element
        while let Some(sibling) = current {
            if let Some(element) = sibling.value().as_element() {
                // Stop if we hit another heading
                if element.name() == "h2" || element.name() == "h3" {
                    break;
                }

                // Check if this is a <p> element
                if element.name() == "p" {
                    let text = extract_text(&ElementRef::wrap(sibling).unwrap());

                    // Skip if too short or promotional
                    if text.len() < 20 {
                        current = sibling.next_sibling();
                        continue;
                    }

                    let lower_text = text.to_lowercase();
                    if lower_text.contains("claim your")
                        || lower_text.contains("subscribe")
                        || lower_text.contains("follow us")
                    {
                        current = sibling.next_sibling();
                        continue;
                    }

                    // Limit to 500 characters
                    return if text.len() > 500 {
                        text.chars().take(500).collect()
                    } else {
                        text
                    };
                }
            }

            current = sibling.next_sibling();
        }

        "No description available".to_string()
    }

    /// Detect if an endpoint is premium-only by checking for premium indicators.
    ///
    /// # Arguments
    /// * `h3_element` - The h3 element to check
    ///
    /// # Returns
    /// True if premium indicators are found, false otherwise
    #[instrument(skip(self, h3_element))]
    fn detect_premium_endpoint(&self, h3_element: ElementRef) -> bool {
        use crate::domain::parser::extract_text;

        let h3_text = extract_text(&h3_element).to_lowercase();

        // Check h3 text for premium indicators
        if h3_text.contains("premium") || h3_text.contains("paid") || h3_text.contains("subscription") {
            return true;
        }

        // Check nearby content for premium indicators
        let mut current = h3_element.next_sibling();
        let mut check_count = 0;

        while let Some(sibling) = current {
            if check_count >= 3 { // Only check next 3 elements
                break;
            }

            if let Some(element) = sibling.value().as_element() {
                if element.name() == "p" {
                    let text = extract_text(&ElementRef::wrap(sibling).unwrap()).to_lowercase();
                    if text.contains("premium") || text.contains("paid") || text.contains("subscription") {
                        return true;
                    }
                } else if element.name() == "h2" || element.name() == "h3" {
                    break; // Stop at next heading
                }
            }

            current = sibling.next_sibling();
            check_count += 1;
        }

        false
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use scraper::Html;

    #[test]
    fn test_normalize_category_name() {
        let html = Html::parse_fragment("");
        let extractor = ContentExtractor::new(&html);

        assert_eq!(
            extractor.normalize_category_name("Time Series Data"),
            "Time Series Data"
        );

        assert_eq!(
            extractor.normalize_category_name("  Time   Series   "),
            "Time Series"
        );

        assert_eq!(
            extractor.normalize_category_name("Stock (TIME_SERIES_INTRADAY)"),
            "Stock (TIME_SERIES_INTRADAY)"
        );

        assert_eq!(
            extractor.normalize_category_name("Special@#$%Chars"),
            "SpecialChars"
        );

        assert_eq!(
            extractor.normalize_category_name("   "),
            ""
        );
    }

    #[test]
    fn test_extract_category_name_success() {
        let html = r#"<h2>Time Series Data</h2>"#;
        let document = Html::parse_fragment(html);
        let extractor = ContentExtractor::new(&document);

        let h2 = document.select(&Selector::parse("h2").unwrap()).next().unwrap();
        let result = extractor.extract_category_name(h2);

        assert!(result.is_ok());
        assert_eq!(result.unwrap(), "Time Series Data");
    }

    #[test]
    fn test_extract_category_name_empty() {
        let html = r#"<h2>   </h2>"#;
        let document = Html::parse_fragment(html);
        let extractor = ContentExtractor::new(&document);

        let h2 = document.select(&Selector::parse("h2").unwrap()).next().unwrap();
        let result = extractor.extract_category_name(h2);

        assert!(result.is_err());
    }

    #[test]
    fn test_extract_category_description_found() {
        let html = r#"
            <h2>Time Series</h2>
            <p>This is a description of time series data.</p>
        "#;
        let document = Html::parse_fragment(html);
        let extractor = ContentExtractor::new(&document);

        let h2 = document.select(&Selector::parse("h2").unwrap()).next().unwrap();
        let result = extractor.extract_category_description(h2);

        assert!(result.is_some());
        assert_eq!(result.unwrap(), "This is a description of time series data.");
    }

    #[test]
    fn test_extract_category_description_promotional() {
        let html = r#"
            <h2>Time Series</h2>
            <p>Claim your free API key today!</p>
            <p>This is the real description.</p>
        "#;
        let document = Html::parse_fragment(html);
        let extractor = ContentExtractor::new(&document);

        let h2 = document.select(&Selector::parse("h2").unwrap()).next().unwrap();
        let result = extractor.extract_category_description(h2);

        assert!(result.is_some());
        assert_eq!(result.unwrap(), "This is the real description.");
    }

    #[test]
    fn test_extract_category_description_too_short() {
        let html = r#"
            <h2>Time Series</h2>
            <p>Short</p>
            <p>This is the real description with enough length.</p>
        "#;
        let document = Html::parse_fragment(html);
        let extractor = ContentExtractor::new(&document);

        let h2 = document.select(&Selector::parse("h2").unwrap()).next().unwrap();
        let result = extractor.extract_category_description(h2);

        assert!(result.is_some());
        assert_eq!(result.unwrap(), "This is the real description with enough length.");
    }

    #[test]
    fn test_extract_category_description_stops_at_heading() {
        let html = r#"
            <h2>Time Series</h2>
            <p>This is the description.</p>
            <h3>Next Section</h3>
            <p>This should not be included.</p>
        "#;
        let document = Html::parse_fragment(html);
        let extractor = ContentExtractor::new(&document);

        let h2 = document.select(&Selector::parse("h2").unwrap()).next().unwrap();
        let result = extractor.extract_category_description(h2);

        assert!(result.is_some());
        assert_eq!(result.unwrap(), "This is the description.");
    }

    #[test]
    fn test_extract_category_description_none() {
        let html = r#"<h2>Time Series</h2>"#;
        let document = Html::parse_fragment(html);
        let extractor = ContentExtractor::new(&document);

        let h2 = document.select(&Selector::parse("h2").unwrap()).next().unwrap();
        let result = extractor.extract_category_description(h2);

        assert!(result.is_none());
    }

    #[test]
    fn test_parse_category_with_description() {
        let html = r#"
            <h2>Time Series Data</h2>
            <p>This is a description of time series functionality.</p>
        "#;
        let document = Html::parse_fragment(html);
        let extractor = ContentExtractor::new(&document);

        let h2 = document.select(&Selector::parse("h2").unwrap()).next().unwrap();
        let result = extractor.parse_category(h2);

        assert!(result.is_ok());
        let category = result.unwrap();
        assert_eq!(category.name, "Time Series Data");
        assert_eq!(
            category.description,
            Some("This is a description of time series functionality.".to_string())
        );
        assert!(category.endpoints.is_empty());
    }

    #[test]
    fn test_parse_category_without_description() {
        let html = r#"<h2>Time Series Data</h2>"#;
        let document = Html::parse_fragment(html);
        let extractor = ContentExtractor::new(&document);

        let h2 = document.select(&Selector::parse("h2").unwrap()).next().unwrap();
        let result = extractor.parse_category(h2);

        assert!(result.is_ok());
        let category = result.unwrap();
        assert_eq!(category.name, "Time Series Data");
        assert!(category.description.is_none());
        assert!(category.endpoints.is_empty());
    }

    #[test]
    fn test_extract_function_name_success() {
        let html = r#"<h3>TIME_SERIES_INTRADAY</h3>"#;
        let document = Html::parse_fragment(html);
        let extractor = ContentExtractor::new(&document);

        let h3 = document.select(&Selector::parse("h3").unwrap()).next().unwrap();
        let result = extractor.extract_function_name(h3);

        assert!(result.is_ok());
        assert_eq!(result.unwrap(), "TIME_SERIES_INTRADAY");
    }

    #[test]
    fn test_extract_function_name_with_text() {
        let html = r#"<h3>Get TIME_SERIES_DAILY data</h3>"#;
        let document = Html::parse_fragment(html);
        let extractor = ContentExtractor::new(&document);

        let h3 = document.select(&Selector::parse("h3").unwrap()).next().unwrap();
        let result = extractor.extract_function_name(h3);

        assert!(result.is_ok());
        assert_eq!(result.unwrap(), "TIME_SERIES_DAILY");
    }

    #[test]
    fn test_extract_function_name_no_match() {
        let html = r#"<h3>Get time series data</h3>"#;
        let document = Html::parse_fragment(html);
        let extractor = ContentExtractor::new(&document);

        let h3 = document.select(&Selector::parse("h3").unwrap()).next().unwrap();
        let result = extractor.extract_function_name(h3);

        assert!(result.is_err());
    }

    #[test]
    fn test_extract_function_name_no_valid_pattern() {
        let html = r#"<h3>function_name</h3>"#;
        let document = Html::parse_fragment(html);
        let extractor = ContentExtractor::new(&document);

        let h3 = document.select(&Selector::parse("h3").unwrap()).next().unwrap();
        let result = extractor.extract_function_name(h3);

        assert!(result.is_err());
    }

    #[test]
    fn test_extract_endpoint_description_found() {
        let html = r#"
            <h3>TIME_SERIES_DAILY</h3>
            <p>This endpoint returns daily time series data for stocks.</p>
        "#;
        let document = Html::parse_fragment(html);
        let extractor = ContentExtractor::new(&document);

        let h3 = document.select(&Selector::parse("h3").unwrap()).next().unwrap();
        let result = extractor.extract_endpoint_description(h3);

        assert_eq!(result, "This endpoint returns daily time series data for stocks.");
    }

    #[test]
    fn test_extract_endpoint_description_promotional() {
        let html = r#"
            <h3>TIME_SERIES_DAILY</h3>
            <p>Claim your free API key today!</p>
            <p>This is the real description.</p>
        "#;
        let document = Html::parse_fragment(html);
        let extractor = ContentExtractor::new(&document);

        let h3 = document.select(&Selector::parse("h3").unwrap()).next().unwrap();
        let result = extractor.extract_endpoint_description(h3);

        assert_eq!(result, "This is the real description.");
    }

    #[test]
    fn test_extract_endpoint_description_none() {
        let html = r#"<h3>TIME_SERIES_DAILY</h3>"#;
        let document = Html::parse_fragment(html);
        let extractor = ContentExtractor::new(&document);

        let h3 = document.select(&Selector::parse("h3").unwrap()).next().unwrap();
        let result = extractor.extract_endpoint_description(h3);

        assert_eq!(result, "No description available");
    }

    #[test]
    fn test_detect_premium_endpoint_in_title() {
        let html = r#"<h3>TIME_SERIES_DAILY (Premium)</h3>"#;
        let document = Html::parse_fragment(html);
        let extractor = ContentExtractor::new(&document);

        let h3 = document.select(&Selector::parse("h3").unwrap()).next().unwrap();
        let result = extractor.detect_premium_endpoint(h3);

        assert!(result);
    }

    #[test]
    fn test_detect_premium_endpoint_in_description() {
        let html = r#"
            <h3>TIME_SERIES_DAILY</h3>
            <p>This is a premium endpoint requiring subscription.</p>
        "#;
        let document = Html::parse_fragment(html);
        let extractor = ContentExtractor::new(&document);

        let h3 = document.select(&Selector::parse("h3").unwrap()).next().unwrap();
        let result = extractor.detect_premium_endpoint(h3);

        assert!(result);
    }

    #[test]
    fn test_detect_premium_endpoint_not_premium() {
        let html = r#"<h3>TIME_SERIES_DAILY</h3>"#;
        let document = Html::parse_fragment(html);
        let extractor = ContentExtractor::new(&document);

        let h3 = document.select(&Selector::parse("h3").unwrap()).next().unwrap();
        let result = extractor.detect_premium_endpoint(h3);

        assert!(!result);
    }

    #[test]
    fn test_parse_endpoint_success() {
        let html = r#"
            <h3>TIME_SERIES_DAILY</h3>
            <p>This endpoint returns daily time series data.</p>
        "#;
        let document = Html::parse_fragment(html);
        let extractor = ContentExtractor::new(&document);

        let h3 = document.select(&Selector::parse("h3").unwrap()).next().unwrap();
        let result = extractor.parse_endpoint(h3);

        assert!(result.is_ok());
        let endpoint = result.unwrap();
        assert_eq!(endpoint.function_name, "TIME_SERIES_DAILY");
        assert_eq!(endpoint.description, "This endpoint returns daily time series data.");
        assert!(!endpoint.premium_only);
    }

    #[test]
    fn test_parse_endpoint_premium() {
        let html = r#"
            <h3>TIME_SERIES_DAILY (Premium)</h3>
            <p>This is a premium endpoint.</p>
        "#;
        let document = Html::parse_fragment(html);
        let extractor = ContentExtractor::new(&document);

        let h3 = document.select(&Selector::parse("h3").unwrap()).next().unwrap();
        let result = extractor.parse_endpoint(h3);

        assert!(result.is_ok());
        let endpoint = result.unwrap();
        assert_eq!(endpoint.function_name, "TIME_SERIES_DAILY");
        assert!(endpoint.premium_only);
    }

    #[test]
    fn test_extract_endpoints_single() {
        let html = r#"
            <h2>Time Series</h2>
            <h3>TIME_SERIES_DAILY</h3>
            <p>Description of daily data.</p>
            <h2>Next Category</h2>
        "#;
        let document = Html::parse_fragment(html);
        let extractor = ContentExtractor::new(&document);

        let h2 = document.select(&Selector::parse("h2").unwrap()).next().unwrap();
        let endpoints = extractor.extract_endpoints(h2);

        assert_eq!(endpoints.len(), 1);
        assert_eq!(endpoints[0].function_name, "TIME_SERIES_DAILY");
    }

    #[test]
    fn test_extract_endpoints_multiple() {
        let html = r#"
            <h2>Time Series</h2>
            <h3>TIME_SERIES_DAILY</h3>
            <p>Daily data endpoint.</p>
            <h3>TIME_SERIES_INTRADAY</h3>
            <p>Intraday data endpoint.</p>
            <h2>Next Category</h2>
        "#;
        let document = Html::parse_fragment(html);
        let extractor = ContentExtractor::new(&document);

        let h2 = document.select(&Selector::parse("h2").unwrap()).next().unwrap();
        let endpoints = extractor.extract_endpoints(h2);

        assert_eq!(endpoints.len(), 2);
        assert_eq!(endpoints[0].function_name, "TIME_SERIES_DAILY");
        assert_eq!(endpoints[1].function_name, "TIME_SERIES_INTRADAY");
    }

    #[test]
    fn test_extract_endpoints_stops_at_next_h2() {
        let html = r#"
            <h2>Time Series</h2>
            <h3>TIME_SERIES_DAILY</h3>
            <p>Daily data.</p>
            <h2>Fundamental Data</h2>
            <h3>OVERVIEW</h3>
            <p>Company overview.</p>
        "#;
        let document = Html::parse_fragment(html);
        let extractor = ContentExtractor::new(&document);

        let h2 = document.select(&Selector::parse("h2").unwrap()).next().unwrap();
        let endpoints = extractor.extract_endpoints(h2);

        assert_eq!(endpoints.len(), 1);
        assert_eq!(endpoints[0].function_name, "TIME_SERIES_DAILY");
    }

    #[test]
    fn test_parse_category_with_endpoints() {
        let html = r#"
            <h2>Time Series Data</h2>
            <p>Time series category description.</p>
            <h3>TIME_SERIES_DAILY</h3>
            <p>Daily time series endpoint.</p>
            <h3>TIME_SERIES_INTRADAY</h3>
            <p>Intraday time series endpoint.</p>
        "#;
        let document = Html::parse_fragment(html);
        let extractor = ContentExtractor::new(&document);

        let h2 = document.select(&Selector::parse("h2").unwrap()).next().unwrap();
        let result = extractor.parse_category(h2);

        assert!(result.is_ok());
        let category = result.unwrap();
        assert_eq!(category.name, "Time Series Data");
        assert_eq!(category.description, Some("Time series category description.".to_string()));
        assert_eq!(category.endpoints.len(), 2);
        assert_eq!(category.endpoints[0].function_name, "TIME_SERIES_DAILY");
        assert_eq!(category.endpoints[1].function_name, "TIME_SERIES_INTRADAY");
    }

    #[test]
    fn test_extract_categories_success() {
        let html = r#"
            <div id="main">
                <h2>Time Series</h2>
                <p>Description of time series.</p>
                <h2>Fundamental Data</h2>
                <p>Description of fundamental data.</p>
            </div>
        "#;
        let document = Html::parse_fragment(html);
        let extractor = ContentExtractor::new(&document);

        let main = document.select(&Selector::parse("#main").unwrap()).next().unwrap();
        let result = extractor.extract_categories(main);

        assert!(result.is_ok());
        let categories = result.unwrap();
        assert_eq!(categories.len(), 2);
        assert_eq!(categories[0].name, "Time Series");
        assert_eq!(categories[1].name, "Fundamental Data");
    }

    #[test]
    fn test_extract_categories_no_h2() {
        let html = r#"<div id="main"><p>No headings here</p></div>"#;
        let document = Html::parse_fragment(html);
        let extractor = ContentExtractor::new(&document);

        let main = document.select(&Selector::parse("#main").unwrap()).next().unwrap();
        let result = extractor.extract_categories(main);

        assert!(result.is_err());
    }

    #[test]
    fn test_extract_categories_all_fail() {
        let html = r#"<div id="main"><h2>   </h2></div>"#;
        let document = Html::parse_fragment(html);
        let extractor = ContentExtractor::new(&document);

        let main = document.select(&Selector::parse("#main").unwrap()).next().unwrap();
        let result = extractor.extract_categories(main);

        assert!(result.is_err());
    }
}
