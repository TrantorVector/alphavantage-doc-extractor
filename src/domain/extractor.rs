//! Content extraction logic for Alpha Vantage documentation.
//!
//! This module implements category and endpoint extraction from parsed HTML documents.
//! It follows the parse-don't-validate principle and uses structured error handling.

use crate::domain::{ApiCategory, ApiEndpoint, CodeExample, Parameter};
use crate::utils::{error::ContentExtractionError, error::ExtractionResult};
use regex::Regex;
use scraper::{ElementRef, Html, Selector};
use tracing::{instrument, warn};

/// Content extractor for Alpha Vantage documentation pages.
///
/// This struct holds a reference to the parsed HTML document and provides
/// methods to extract categories and endpoints from the main content area.
pub struct ContentExtractor<'a> {
    /// Reference to the parsed HTML document (reserved for future use)
    #[allow(dead_code)]
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
    /// This method finds all `<h2>` and `<h3>` elements in the main content and attempts
    /// to parse them into `ApiCategory` instances with their endpoints.
    /// It uses a linear scan of headings to handle nested structures where
    /// endpoints might not be direct siblings of categories.
    ///
    /// # Arguments
    /// * `main_content` - The main content element to extract categories from
    ///
    /// # Returns
    /// A vector of successfully extracted categories, or an error if no categories found
    #[instrument(skip(self, main_content), fields(request_id = %uuid::Uuid::new_v4()))]
    pub fn extract_categories(
        &self,
        main_content: ElementRef,
    ) -> ExtractionResult<Vec<ApiCategory>> {
        let heading_selector = Selector::parse("h2, h3, h4").map_err(|e| {
            ContentExtractionError::CategoryExtractionFailed(
                "selector".to_string(),
                format!("Failed to parse heading selector: {}", e),
            )
        })?;

        let elements: Vec<ElementRef> = main_content.select(&heading_selector).collect();

        if elements.is_empty() {
            return Err(ContentExtractionError::CategoryExtractionFailed(
                "no_headings_found".to_string(),
                "No h2, h3, or h4 elements found in main content".to_string(),
            ));
        }

        let mut categories = Vec::new();
        let mut current_category: Option<ApiCategory> = None;

        for element in elements {
            let tag_name = element.value().name();

            if tag_name == "h2" {
                // Save previous category if it exists
                if let Some(cat) = current_category {
                    if !cat.endpoints.is_empty() {
                        categories.push(cat);
                    } else {
                        warn!("Skipping empty category: {}", cat.name);
                    }
                }

                // Start new category
                match self.parse_category(element) {
                    Ok(cat) => current_category = Some(cat),
                    Err(e) => {
                        warn!("Failed to parse category: {}", e);
                        current_category = None;
                    }
                }
            } else if tag_name == "h3" || tag_name == "h4" {
                // Add endpoint to current category
                if let Some(mut cat) = current_category.take() {
                    match self.parse_endpoint(element) {
                        Ok(endpoint) => {
                            cat = cat.add_endpoint(endpoint);
                        }
                        Err(e) => {
                            warn!("Failed to parse endpoint: {}", e);
                        }
                    }
                    current_category = Some(cat);
                } else {
                    // warn!("Found endpoint ({}) before any category (h2): {:?}", tag_name, crate::domain::parser::extract_text(&element));
                }
            }
        }


        // Push the final category
        if let Some(cat) = current_category {
            if !cat.endpoints.is_empty() {
                categories.push(cat);
            } else {
                warn!("Skipping empty category: {}", cat.name);
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

    /// Parse a single category from an h2 element (metadata only).
    ///
    /// This method extracts the category name and optional description.
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

        let mut category = ApiCategory::new(name);
        if let Some(desc) = description {
            category = category.with_description(desc);
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
            .filter(|c| {
                c.is_alphanumeric() || c.is_whitespace() || *c == '(' || *c == ')' || *c == '_'
            })
            .collect();

        // Collapse multiple spaces and trim
        filtered.split_whitespace().collect::<Vec<&str>>().join(" ")
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
        let (required_params, optional_params) = self.extract_parameters(h3_element);
        let python_example = self.extract_code_example(h3_element);

        // Combine all parameters for pattern construction
        let all_params = [&required_params[..], &optional_params[..]].concat();

        // Extract or construct request pattern
        let request_pattern =
            self.extract_request_pattern(h3_element, Some(&function_name), Some(&all_params));

        let mut endpoint = ApiEndpoint::new(function_name, description)
            .set_premium(premium_only)
            .with_required_params(required_params)
            .with_optional_params(optional_params)
            .with_request_pattern(request_pattern);

        if let Some(example) = python_example {
            endpoint = endpoint.with_python_example(example);
        }

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
        let function_pattern = Regex::new(r"[A-Z][A-Z_]*[A-Z]").map_err(|e| {
            ContentExtractionError::CategoryExtractionFailed(
                "regex_compilation".to_string(),
                format!("Failed to compile function name regex: {}", e),
            )
        })?;

        if let Some(captures) = function_pattern.find(&h3_text) {
            let function_name = captures.as_str().to_string();

            // Validate that it matches the expected format (no lowercase, proper underscores)
            if function_name.chars().all(|c| c.is_uppercase() || c == '_') {
                Ok(function_name)
            } else {
                Err(ContentExtractionError::CategoryExtractionFailed(
                    "invalid_function_format".to_string(),
                    format!(
                        "Function name '{}' contains invalid characters",
                        function_name
                    ),
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
        if h3_text.contains("premium")
            || h3_text.contains("paid")
            || h3_text.contains("subscription")
        {
            return true;
        }

        // Check nearby content for premium indicators
        let mut current = h3_element.next_sibling();
        let mut check_count = 0;

        while let Some(sibling) = current {
            if check_count >= 3 {
                // Only check next 3 elements
                break;
            }

            if let Some(element) = sibling.value().as_element() {
                if element.name() == "p" {
                    let text = extract_text(&ElementRef::wrap(sibling).unwrap()).to_lowercase();
                    if text.contains("premium")
                        || text.contains("paid")
                        || text.contains("subscription")
                    {
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

    /// Extract parameters from tables following an endpoint heading.
    ///
    /// This method looks for parameter tables after an endpoint heading and
    /// parses them into required and optional parameters.
    /// It also supports extracting parameters from paragraphs (new Alpha Vantage format).
    ///
    /// # Arguments
    /// * `h3_element` - The heading element representing the endpoint (h3 or h4)
    ///
    /// # Returns
    /// A tuple of (required_parameters, optional_parameters)
    #[instrument(skip(self, h3_element))]
    pub fn extract_parameters(&self, h3_element: ElementRef) -> (Vec<Parameter>, Vec<Parameter>) {
        let mut required_params = Vec::new();
        let mut optional_params = Vec::new();

        let mut current = h3_element.next_sibling();

        // Traverse siblings looking for parameter tables
        while let Some(sibling) = current {
            if let Some(element) = sibling.value().as_element() {
                // Stop if we hit another heading
                let tag = element.name();
                if tag == "h2" || tag == "h3" || tag == "h4" {
                    break;
                }

                // Look for table elements
                if tag == "table" {
                    let parameters = self.parse_parameter_table(ElementRef::wrap(sibling).unwrap());

                    // Separate into required and optional
                    for param in parameters {
                        if self
                            .is_required_parameter(ElementRef::wrap(sibling).unwrap(), &param.name)
                        {
                            required_params.push(param);
                        } else {
                            optional_params.push(param);
                        }
                    }
                }
            }

            current = sibling.next_sibling();
        }

        // If we found parameters in tables, return them
        if !required_params.is_empty() || !optional_params.is_empty() {
            return (required_params, optional_params);
        }

        // Fallback: Try to extract from paragraphs (new format)
        self.extract_parameters_from_paragraphs(h3_element)
    }

    /// Extract parameters from paragraph-based layout.
    ///
    /// # Arguments
    /// * `heading` - The endpoint heading element
    ///
    /// # Returns
    /// A tuple of (required_parameters, optional_parameters)
    fn extract_parameters_from_paragraphs(&self, heading: ElementRef) -> (Vec<Parameter>, Vec<Parameter>) {
        use crate::domain::parser::extract_text;

        let mut required_params = Vec::new();
        let mut optional_params = Vec::new();
        
        let mut current = heading.next_sibling();
        let mut pending_param: Option<(String, bool)> = None; // (name, is_required)

        while let Some(sibling) = current {
            if let Some(element) = sibling.value().as_element() {
                let tag_name = element.name();
                if tag_name == "h2" || tag_name == "h3" || tag_name == "h4" {
                    break;
                }

                if tag_name == "p" {
                    let text = extract_text(&ElementRef::wrap(sibling).unwrap());
                    
                    // Check for parameter definition line
                    // Format: "❚ Required: function" or "❚ Optional: outputsize"
                    if text.contains("Required:") || text.contains("Optional:") {
                         // Parse definition
                         let p_element = ElementRef::wrap(sibling).unwrap();
                         let code_selector = Selector::parse("code").unwrap();
                         
                         // Try to find code tag first
                         let param_name_opt = if let Some(code_el) = p_element.select(&code_selector).next() {
                             Some(extract_text(&code_el))
                         } else {
                             // Fallback: extract last word?
                             None
                         };

                         if let Some(param_name) = param_name_opt {
                             let is_required = text.contains("Required:");
                             
                             // If we had a pending param, save it with default description
                             if let Some((name, req)) = pending_param.take() {
                                 let p = Parameter::new(name, "string".to_string(), "No description available".to_string());
                                 if req { required_params.push(p); } else { optional_params.push(p); }
                             }
                             
                             pending_param = Some((param_name, is_required));
                         }
                    } else if let Some((name, is_required)) = pending_param.take() {
                         // This paragraph is likely the description for the pending param
                         let description = text;
                         let mut param = Parameter::new(name, "string".to_string(), description.clone());
                         
                         // Basic heuristic for default values
                         if description.contains("default=") || description.contains("default is") {
                             // Extract simple default if possible, or just leave as is
                             if let Some(idx) = description.find("default=") {
                                 let rest = &description[idx + 8..];
                                 let default_val = rest.split_whitespace().next().unwrap_or("").trim_matches('.').to_string();
                                 if !default_val.is_empty() {
                                     param = param.with_default(default_val);
                                 }
                             } else if description.contains("compact") {
                                 param = param.with_default("compact".to_string());
                             }
                         }
                         
                         if is_required {
                             required_params.push(param);
                         } else {
                             optional_params.push(param);
                         }
                    }
                }
            }
            current = sibling.next_sibling();
        }
        
        // Handle last pending param
        if let Some((name, req)) = pending_param {
             let p = Parameter::new(name, "string".to_string(), "No description available".to_string());
             if req { required_params.push(p); } else { optional_params.push(p); }
        }
        
        (required_params, optional_params)
    }

    /// Parse a parameter table into a vector of Parameter objects.
    ///
    /// # Arguments
    /// * `table` - The table element containing parameter definitions
    ///
    /// # Returns
    /// A vector of successfully parsed parameters
    #[instrument(skip(self, table))]
    fn parse_parameter_table(&self, table: ElementRef) -> Vec<Parameter> {
        let mut parameters = Vec::new();

        // Find all table rows
        let tr_selector = Selector::parse("tr").unwrap();
        let rows: Vec<ElementRef> = table.select(&tr_selector).collect();

        // Skip the first row (header)
        for row in rows.iter().skip(1) {
            if let Some(param) = self.parse_parameter_from_row(*row) {
                parameters.push(param);
            }
        }

        parameters
    }

    /// Parse a single parameter from a table row.
    ///
    /// Expected table structure:
    /// - Column 1: Parameter name
    /// - Column 2: Data type
    /// - Column 3: Description
    /// - Column 4 (optional): Default value
    /// - Column 5 (optional): Allowed values/options
    ///
    /// # Arguments
    /// * `row` - The table row element
    ///
    /// # Returns
    /// Some(Parameter) if parsing succeeds, None otherwise
    #[instrument(skip(self, row))]
    fn parse_parameter_from_row(&self, row: ElementRef) -> Option<Parameter> {
        use crate::domain::parser::extract_text;

        // Find all table cells
        let td_selector = Selector::parse("td").unwrap();
        let cells: Vec<ElementRef> = row.select(&td_selector).collect();

        if cells.len() < 3 {
            // Need at least name, type, and description
            return None;
        }

        let name = extract_text(&cells[0]).trim().to_string();
        let value_type = extract_text(&cells[1]).trim().to_string();
        let description = extract_text(&cells[2]).trim().to_string();

        if name.is_empty() || value_type.is_empty() || description.is_empty() {
            return None;
        }

        let mut param = Parameter::new(name, value_type, description);

        // Optional: default value (column 4)
        if cells.len() > 3 {
            let default_text = extract_text(&cells[3]).trim().to_string();
            if !default_text.is_empty()
                && default_text != "-"
                && default_text.to_lowercase() != "none"
            {
                param = param.with_default(default_text);
            }
        }

        // Optional: options (column 5)
        if cells.len() > 4 {
            let options_text = extract_text(&cells[4]);
            let options_text = options_text.trim();
            if !options_text.is_empty() && options_text != "-" {
                let options = self.parse_options(options_text);
                if !options.is_empty() {
                    param = param.with_options(options);
                }
            }
        }

        Some(param)
    }

    /// Parse options from text (comma-separated or bullet lists).
    ///
    /// # Arguments
    /// * `options_text` - The text containing option values
    ///
    /// # Returns
    /// A vector of parsed option strings
    fn parse_options(&self, options_text: &str) -> Vec<String> {
        // Handle comma-separated values
        if options_text.contains(',') {
            options_text
                .split(',')
                .map(|s| s.trim().to_string())
                .filter(|s| !s.is_empty())
                .collect()
        } else if options_text.contains('\n') {
            // Handle newline-separated values
            options_text
                .lines()
                .map(|s| s.trim().trim_start_matches("•").trim().to_string())
                .filter(|s| !s.is_empty())
                .collect()
        } else {
            // Single option
            vec![options_text.to_string()]
        }
    }

    /// Determine if a parameter is required based on row content and parameter name.
    ///
    /// # Arguments
    /// * `table` - The table element (for context)
    /// * `param_name` - The parameter name to check
    ///
    /// # Returns
    /// True if the parameter is required, false if optional
    #[instrument(skip(self, table))]
    fn is_required_parameter(&self, table: ElementRef, param_name: &str) -> bool {
        use crate::domain::parser::extract_text;

        // Always required parameters
        let always_required = ["function", "apikey", "symbol"];
        if always_required.contains(&param_name.to_lowercase().as_str()) {
            return true;
        }

        // Check table content for required indicators
        let table_text = extract_text(&table).to_lowercase();

        // Look for required indicators in the table
        if table_text.contains("required") {
            // This is a simplified check - in practice, we'd need more sophisticated parsing
            // For now, assume most parameters in tables are required unless explicitly optional
            return true;
        }

        // Default: assume required (most API parameters are required)
        true
    }

    /// Extract Python code example from content following an h3 element.
    ///
    /// This method looks for <pre> or <code> blocks after an h3 endpoint heading,
    /// verifies they contain Python code, and returns a cleaned CodeExample.
    ///
    /// # Arguments
    /// * `h3_element` - The h3 element representing the endpoint
    ///
    /// # Returns
    /// A Python CodeExample if found and valid, None otherwise
    #[instrument(skip(self, h3_element))]
    pub fn extract_code_example(&self, h3_element: ElementRef) -> Option<CodeExample> {
        let mut current = h3_element.next_sibling();

        // Traverse siblings looking for code blocks
        while let Some(sibling) = current {
            if let Some(element) = sibling.value().as_element() {
                // Stop if we hit another heading
                if element.name() == "h2" || element.name() == "h3" {
                    break;
                }

                // Check for <pre> or <code> elements
                if element.name() == "pre" || element.name() == "code" {
                    let code_element = ElementRef::wrap(sibling).unwrap();
                    let raw_code = crate::domain::parser::extract_text(&code_element);

                    // Check if this looks like Python code
                    if self.is_python_code(&raw_code) {
                        let cleaned_code = self.clean_code_block(&raw_code);
                        return Some(CodeExample::python(cleaned_code));
                    }
                } else if element.name() == "div" {
                    // Check if div contains pre/code (common in layout)
                    let div_element = ElementRef::wrap(sibling).unwrap();
                    let pre_selector = Selector::parse("pre").unwrap();
                    if let Some(pre) = div_element.select(&pre_selector).next() {
                        let raw_code = crate::domain::parser::extract_text(&pre);
                        if self.is_python_code(&raw_code) {
                            let cleaned_code = self.clean_code_block(&raw_code);
                            return Some(CodeExample::python(cleaned_code));
                        }
                    }
                }
            }

            current = sibling.next_sibling();
        }

        None
    }

    /// Determine if the given code text appears to be Python code.
    ///
    /// This method checks for Python-specific keywords and patterns to distinguish
    /// Python code from other languages like R, VBA, MATLAB, or JSON responses.
    ///
    /// # Arguments
    /// * `code_text` - The raw code text to analyze
    ///
    /// # Returns
    /// True if the code appears to be Python, false otherwise
    #[instrument(skip(self))]
    fn is_python_code(&self, code_text: &str) -> bool {
        let text = code_text.to_lowercase();

        // Exclude other languages first
        if text.contains("<-") || text.contains("library(") || text.contains("data.frame") {
            return false; // R code
        }
        if text.contains("sub ") || text.contains("end sub") || text.contains("worksheets") {
            return false; // Excel VBA
        }
        if text.contains("function [") || text.contains("end;") || text.starts_with('%') {
            return false; // MATLAB
        }
        if text.trim().starts_with('{') && text.trim().ends_with('}') {
            return false; // JSON response
        }

        // Check for Python keywords and patterns
        let python_keywords = [
            "import", "def", "class", "print", "requests", "pandas", "numpy", "json", "urllib",
            "os", "sys", "datetime",
        ];

        let mut python_indicators = 0;

        // Count Python keywords
        for keyword in &python_keywords {
            if text.contains(keyword) {
                python_indicators += 1;
            }
        }

        // Check for Python-specific patterns
        if text.contains("):") || text.contains("):\n") {
            // function definitions
            python_indicators += 1;
        }
        if text.contains("if __name__") {
            // common Python pattern
            python_indicators += 2;
        }
        if text.contains("requests.get(") || text.contains("requests.post(") {
            python_indicators += 2; // HTTP requests (common in API examples)
        }

        // Require at least 2 indicators to be considered Python
        python_indicators >= 2
    }

    /// Clean a code block by removing HTML entities and formatting artifacts.
    ///
    /// # Arguments
    /// * `code` - The raw code text to clean
    ///
    /// # Returns
    /// The cleaned code text
    #[instrument(skip(self))]
    fn clean_code_block(&self, code: &str) -> String {
        let mut cleaned = code.to_string();

        // Replace common HTML entities
        cleaned = cleaned.replace("&lt;", "<");
        cleaned = cleaned.replace("&gt;", ">");
        cleaned = cleaned.replace("&amp;", "&");
        cleaned = cleaned.replace("&quot;", "\"");
        cleaned = cleaned.replace("&#39;", "'");
        cleaned = cleaned.replace("&nbsp;", " ");

        // Remove syntax highlighting artifacts (common in code blocks)
        // Remove line numbers like "1  ", "2  ", etc.
        let line_number_pattern = Regex::new(r"^\d+\s+").unwrap();
        cleaned = cleaned
            .lines()
            .map(|line| line_number_pattern.replace(line, "").to_string())
            .collect::<Vec<String>>()
            .join("\n");

        // Normalize line endings
        cleaned = cleaned.replace("\r\n", "\n").replace('\r', "\n");

        // Trim leading/trailing whitespace while preserving internal indentation
        cleaned = cleaned.trim().to_string();

        cleaned
    }

    /// Deduplicate a vector of code examples by removing exact duplicates and near-duplicates.
    ///
    /// # Arguments
    /// * `examples` - Vector of code examples to deduplicate
    ///
    /// # Returns
    /// A deduplicated vector with duplicates removed
    #[instrument(skip(self))]
    pub fn deduplicate_code_examples(&self, examples: Vec<CodeExample>) -> Vec<CodeExample> {
        let mut unique_examples: Vec<CodeExample> = Vec::new();
        let mut seen_hashes = std::collections::HashSet::new();

        for example in examples {
            // Simple hash-based deduplication (exact duplicates)
            use std::collections::hash_map::DefaultHasher;
            use std::hash::{Hash, Hasher};

            let mut hasher = DefaultHasher::new();
            example.code.hash(&mut hasher);
            let hash = hasher.finish();

            if seen_hashes.contains(&hash) {
                continue; // Skip exact duplicate
            }

            // Check for near-duplicates (simple similarity check)
            let mut is_near_duplicate = false;
            for unique in &unique_examples {
                if self.code_similarity(&example.code, &unique.code) > 0.95 {
                    is_near_duplicate = true;
                    break;
                }
            }

            if !is_near_duplicate {
                seen_hashes.insert(hash);
                unique_examples.push(example);
            }
        }

        unique_examples
    }

    /// Calculate similarity between two code strings (0.0 to 1.0).
    ///
    /// This is a simple implementation that compares normalized code.
    /// In production, you might want a more sophisticated diff algorithm.
    ///
    /// # Arguments
    /// * `code1` - First code string
    /// * `code2` - Second code string
    ///
    /// # Returns
    /// Similarity score between 0.0 and 1.0
    fn code_similarity(&self, code1: &str, code2: &str) -> f64 {
        let normalized1 = self.normalize_code_for_comparison(code1);
        let normalized2 = self.normalize_code_for_comparison(code2);

        if normalized1 == normalized2 {
            return 1.0; // Exact match after normalization
        }

        // Simple character-based similarity
        let longer = normalized1.len().max(normalized2.len());
        if longer == 0 {
            return 1.0;
        }

        let shorter = normalized1.len().min(normalized2.len());
        (shorter as f64) / (longer as f64)
    }

    /// Normalize code for similarity comparison by removing whitespace differences.
    ///
    /// # Arguments
    /// * `code` - Code to normalize
    ///
    /// # Returns
    /// Normalized code string
    fn normalize_code_for_comparison(&self, code: &str) -> String {
        code.lines()
            .map(|line| line.trim())
            .filter(|line| !line.is_empty())
            .collect::<Vec<&str>>()
            .join("\n")
            .to_lowercase()
    }

    /// Extract API request pattern from endpoint documentation.
    ///
    /// This method attempts to find an actual URL pattern in the documentation,
    /// or constructs one from the endpoint function name and parameters if none found.
    ///
    /// # Arguments
    /// * `h3_element` - The h3 element representing the endpoint
    /// * `function_name` - Optional function name for pattern construction
    /// * `params` - Optional parameters for pattern construction
    ///
    /// # Returns
    /// A URL pattern string for the API request
    #[instrument(skip(self, h3_element, params))]
    pub fn extract_request_pattern(
        &self,
        h3_element: ElementRef,
        function_name: Option<&str>,
        params: Option<&[Parameter]>,
    ) -> String {
        // First try to find an actual URL pattern in the documentation
        if let Some(pattern) = self.find_url_pattern(h3_element) {
            return pattern;
        }

        // If no pattern found and we have function name and params, construct one
        if let (Some(func), Some(parameters)) = (function_name, params) {
            return self.construct_pattern_from_endpoint(func, parameters);
        }

        // Fallback
        "https://www.alphavantage.co/query".to_string()
    }

    /// Find URL pattern in the documentation following an h3 element.
    ///
    /// Searches for Alpha Vantage API URL patterns using regex.
    ///
    /// # Arguments
    /// * `h3_element` - The h3 element to search after
    ///
    /// # Returns
    /// The first URL pattern found, or None
    #[instrument(skip(self, h3_element))]
    fn find_url_pattern(&self, h3_element: ElementRef) -> Option<String> {
        use crate::domain::parser::extract_text;

        // Regex pattern for Alpha Vantage URLs
        let url_pattern = Regex::new(r"https?://www\.alphavantage\.co/query\?[^\s<>]+").ok()?;

        let mut current = h3_element.next_sibling();

        // Search through following elements for URL patterns
        while let Some(sibling) = current {
            if let Some(element) = sibling.value().as_element() {
                // Stop if we hit another heading
                if element.name() == "h2" || element.name() == "h3" {
                    break;
                }

                // Check various element types for URL patterns
                let text_to_search = if element.name() == "a" {
                    // For links, check the href attribute
                    if let Some(href) = element.attr("href") {
                        href.to_string()
                    } else {
                        extract_text(&ElementRef::wrap(sibling).unwrap())
                    }
                } else {
                    extract_text(&ElementRef::wrap(sibling).unwrap())
                };

                // Search for URL pattern in the text
                if let Some(captures) = url_pattern.find(&text_to_search) {
                    let url = captures.as_str().to_string();
                    // Validate that it contains at least function parameter
                    if url.contains("function=") {
                        return Some(url);
                    }
                }
            }

            current = sibling.next_sibling();
        }

        None
    }

    /// Construct a request pattern from endpoint function name and parameters.
    ///
    /// # Arguments
    /// * `function_name` - The API function name
    /// * `params` - List of parameters for the endpoint
    ///
    /// # Returns
    /// A constructed URL pattern string
    #[instrument(skip(self))]
    pub fn construct_pattern_from_endpoint(
        &self,
        function_name: &str,
        params: &[Parameter],
    ) -> String {
        let url = "https://www.alphavantage.co/query?".to_string();
        let mut query_parts = Vec::new();

        // Always include function parameter
        query_parts.push(format!("function={}", function_name));

        // Add required parameters
        for param in params {
            if param.name == "function" || param.name == "apikey" {
                // Skip function (already added) and apikey (added at end)
                continue;
            }
            query_parts.push(format!("{}={}", param.name, param.name.to_uppercase()));
        }

        // Add common optional parameters with defaults
        if !params.iter().any(|p| p.name == "outputsize") {
            query_parts.push("outputsize=compact".to_string());
        }
        if !params.iter().any(|p| p.name == "datatype") {
            query_parts.push("datatype=json".to_string());
        }

        // Always include apikey at the end
        query_parts.push("apikey=YOUR_API_KEY".to_string());

        url + &query_parts.join("&")
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

        assert_eq!(extractor.normalize_category_name("   "), "");
    }

    #[test]
    fn test_extract_category_name_success() {
        let html = r#"<h2>Time Series Data</h2>"#;
        let document = Html::parse_fragment(html);
        let extractor = ContentExtractor::new(&document);

        let h2 = document
            .select(&Selector::parse("h2").unwrap())
            .next()
            .unwrap();
        let result = extractor.extract_category_name(h2);

        assert!(result.is_ok());
        assert_eq!(result.unwrap(), "Time Series Data");
    }

    #[test]
    fn test_extract_category_name_empty() {
        let html = r#"<h2>   </h2>"#;
        let document = Html::parse_fragment(html);
        let extractor = ContentExtractor::new(&document);

        let h2 = document
            .select(&Selector::parse("h2").unwrap())
            .next()
            .unwrap();
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

        let h2 = document
            .select(&Selector::parse("h2").unwrap())
            .next()
            .unwrap();
        let result = extractor.extract_category_description(h2);

        assert!(result.is_some());
        assert_eq!(
            result.unwrap(),
            "This is a description of time series data."
        );
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

        let h2 = document
            .select(&Selector::parse("h2").unwrap())
            .next()
            .unwrap();
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

        let h2 = document
            .select(&Selector::parse("h2").unwrap())
            .next()
            .unwrap();
        let result = extractor.extract_category_description(h2);

        assert!(result.is_some());
        assert_eq!(
            result.unwrap(),
            "This is the real description with enough length."
        );
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

        let h2 = document
            .select(&Selector::parse("h2").unwrap())
            .next()
            .unwrap();
        let result = extractor.extract_category_description(h2);

        assert!(result.is_some());
        assert_eq!(result.unwrap(), "This is the description.");
    }

    #[test]
    fn test_extract_category_description_none() {
        let html = r#"<h2>Time Series</h2>"#;
        let document = Html::parse_fragment(html);
        let extractor = ContentExtractor::new(&document);

        let h2 = document
            .select(&Selector::parse("h2").unwrap())
            .next()
            .unwrap();
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

        let h2 = document
            .select(&Selector::parse("h2").unwrap())
            .next()
            .unwrap();
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

        let h2 = document
            .select(&Selector::parse("h2").unwrap())
            .next()
            .unwrap();
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

        let h3 = document
            .select(&Selector::parse("h3").unwrap())
            .next()
            .unwrap();
        let result = extractor.extract_function_name(h3);

        assert!(result.is_ok());
        assert_eq!(result.unwrap(), "TIME_SERIES_INTRADAY");
    }

    #[test]
    fn test_extract_function_name_with_text() {
        let html = r#"<h3>Get TIME_SERIES_DAILY data</h3>"#;
        let document = Html::parse_fragment(html);
        let extractor = ContentExtractor::new(&document);

        let h3 = document
            .select(&Selector::parse("h3").unwrap())
            .next()
            .unwrap();
        let result = extractor.extract_function_name(h3);

        assert!(result.is_ok());
        assert_eq!(result.unwrap(), "TIME_SERIES_DAILY");
    }

    #[test]
    fn test_extract_function_name_no_match() {
        let html = r#"<h3>Get time series data</h3>"#;
        let document = Html::parse_fragment(html);
        let extractor = ContentExtractor::new(&document);

        let h3 = document
            .select(&Selector::parse("h3").unwrap())
            .next()
            .unwrap();
        let result = extractor.extract_function_name(h3);

        assert!(result.is_err());
    }

    #[test]
    fn test_extract_function_name_no_valid_pattern() {
        let html = r#"<h3>function_name</h3>"#;
        let document = Html::parse_fragment(html);
        let extractor = ContentExtractor::new(&document);

        let h3 = document
            .select(&Selector::parse("h3").unwrap())
            .next()
            .unwrap();
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

        let h3 = document
            .select(&Selector::parse("h3").unwrap())
            .next()
            .unwrap();
        let result = extractor.extract_endpoint_description(h3);

        assert_eq!(
            result,
            "This endpoint returns daily time series data for stocks."
        );
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

        let h3 = document
            .select(&Selector::parse("h3").unwrap())
            .next()
            .unwrap();
        let result = extractor.extract_endpoint_description(h3);

        assert_eq!(result, "This is the real description.");
    }

    #[test]
    fn test_extract_endpoint_description_none() {
        let html = r#"<h3>TIME_SERIES_DAILY</h3>"#;
        let document = Html::parse_fragment(html);
        let extractor = ContentExtractor::new(&document);

        let h3 = document
            .select(&Selector::parse("h3").unwrap())
            .next()
            .unwrap();
        let result = extractor.extract_endpoint_description(h3);

        assert_eq!(result, "No description available");
    }

    #[test]
    fn test_detect_premium_endpoint_in_title() {
        let html = r#"<h3>TIME_SERIES_DAILY (Premium)</h3>"#;
        let document = Html::parse_fragment(html);
        let extractor = ContentExtractor::new(&document);

        let h3 = document
            .select(&Selector::parse("h3").unwrap())
            .next()
            .unwrap();
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

        let h3 = document
            .select(&Selector::parse("h3").unwrap())
            .next()
            .unwrap();
        let result = extractor.detect_premium_endpoint(h3);

        assert!(result);
    }

    #[test]
    fn test_detect_premium_endpoint_not_premium() {
        let html = r#"<h3>TIME_SERIES_DAILY</h3>"#;
        let document = Html::parse_fragment(html);
        let extractor = ContentExtractor::new(&document);

        let h3 = document
            .select(&Selector::parse("h3").unwrap())
            .next()
            .unwrap();
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

        let h3 = document
            .select(&Selector::parse("h3").unwrap())
            .next()
            .unwrap();
        let result = extractor.parse_endpoint(h3);

        assert!(result.is_ok());
        let endpoint = result.unwrap();
        assert_eq!(endpoint.function_name, "TIME_SERIES_DAILY");
        assert_eq!(
            endpoint.description,
            "This endpoint returns daily time series data."
        );
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

        let h3 = document
            .select(&Selector::parse("h3").unwrap())
            .next()
            .unwrap();
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

        let h2 = document
            .select(&Selector::parse("h2").unwrap())
            .next()
            .unwrap();
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

        let h2 = document
            .select(&Selector::parse("h2").unwrap())
            .next()
            .unwrap();
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

        let h2 = document
            .select(&Selector::parse("h2").unwrap())
            .next()
            .unwrap();
        let endpoints = extractor.extract_endpoints(h2);

        assert_eq!(endpoints.len(), 1);
        assert_eq!(endpoints[0].function_name, "TIME_SERIES_DAILY");
    }

    #[test]
    fn test_parse_category_with_endpoints() {
        let html = r#"
            <div id="main">
                <h2>Time Series Data</h2>
                <p>Time series category description.</p>
                <h3>TIME_SERIES_DAILY</h3>
                <p>Daily time series endpoint.</p>
                <h3>TIME_SERIES_INTRADAY</h3>
                <p>Intraday time series endpoint.</p>
            </div>
        "#;
        let document = Html::parse_fragment(html);
        let extractor = ContentExtractor::new(&document);

        let main = document
            .select(&Selector::parse("#main").unwrap())
            .next()
            .unwrap();
        
        // Use extract_categories instead of parse_category since parse_category no longer extracts endpoints
        let result = extractor.extract_categories(main);

        assert!(result.is_ok());
        let categories = result.unwrap();
        assert_eq!(categories.len(), 1);
        let category = &categories[0];
        
        assert_eq!(category.name, "Time Series Data");
        assert_eq!(
            category.description,
            Some("Time series category description.".to_string())
        );
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
                <h3>TIME_SERIES_DAILY</h3>
                <p>Daily data.</p>
                <h2>Fundamental Data</h2>
                <p>Description of fundamental data.</p>
                <h3>OVERVIEW</h3>
                <p>Company overview.</p>
            </div>
        "#;
        let document = Html::parse_fragment(html);
        let extractor = ContentExtractor::new(&document);

        let main = document
            .select(&Selector::parse("#main").unwrap())
            .next()
            .unwrap();
        let result = extractor.extract_categories(main);

        assert!(result.is_ok());
        let categories = result.unwrap();
        assert_eq!(categories.len(), 2);
        assert_eq!(categories[0].name, "Time Series");
        assert_eq!(categories[0].endpoints.len(), 1);
        assert_eq!(categories[1].name, "Fundamental Data");
        assert_eq!(categories[1].endpoints.len(), 1);
    }

    #[test]
    fn test_extract_categories_no_h2() {
        let html = r#"<div id="main"><p>No headings here</p></div>"#;
        let document = Html::parse_fragment(html);
        let extractor = ContentExtractor::new(&document);

        let main = document
            .select(&Selector::parse("#main").unwrap())
            .next()
            .unwrap();
        let result = extractor.extract_categories(main);

        assert!(result.is_err());
    }

    #[test]
    fn test_extract_categories_all_fail() {
        let html = r#"<div id="main"><h2>   </h2></div>"#;
        let document = Html::parse_fragment(html);
        let extractor = ContentExtractor::new(&document);

        let main = document
            .select(&Selector::parse("#main").unwrap())
            .next()
            .unwrap();
        let result = extractor.extract_categories(main);

        assert!(result.is_err());
    }

    #[test]
    fn test_parse_parameter_from_row_basic() {
        let html = r#"
            <table>
                <tr>
                    <td>symbol</td>
                    <td>string</td>
                    <td>The stock symbol to query</td>
                </tr>
            </table>
        "#;
        let document = Html::parse_fragment(html);
        let extractor = ContentExtractor::new(&document);

        let row = document
            .select(&Selector::parse("tr").unwrap())
            .next()
            .unwrap();
        let param = extractor.parse_parameter_from_row(row);

        assert!(param.is_some());
        let param = param.unwrap();
        assert_eq!(param.name, "symbol");
        assert_eq!(param.value_type, "string");
        assert_eq!(param.description, "The stock symbol to query");
        assert!(param.default.is_none());
        assert!(param.options.is_empty());
    }

    #[test]
    fn test_parse_parameter_from_row_with_default() {
        let html = r#"
            <table>
                <tr>
                    <td>outputsize</td>
                    <td>string</td>
                    <td>Number of data points</td>
                    <td>compact</td>
                </tr>
            </table>
        "#;
        let document = Html::parse_fragment(html);
        let extractor = ContentExtractor::new(&document);

        let row = document
            .select(&Selector::parse("tr").unwrap())
            .next()
            .unwrap();
        let param = extractor.parse_parameter_from_row(row);

        assert!(param.is_some());
        let param = param.unwrap();
        assert_eq!(param.name, "outputsize");
        assert_eq!(param.default, Some("compact".to_string()));
    }

    #[test]
    fn test_parse_parameter_from_row_with_options() {
        let html = r#"
            <table>
                <tr>
                    <td>datatype</td>
                    <td>string</td>
                    <td>Data format</td>
                    <td>json</td>
                    <td>json, csv, xml</td>
                </tr>
            </table>
        "#;
        let document = Html::parse_fragment(html);
        let extractor = ContentExtractor::new(&document);

        let row = document
            .select(&Selector::parse("tr").unwrap())
            .next()
            .unwrap();
        let param = extractor.parse_parameter_from_row(row);

        assert!(param.is_some());
        let param = param.unwrap();
        assert_eq!(param.name, "datatype");
        assert_eq!(param.options, vec!["json", "csv", "xml"]);
    }

    #[test]
    fn test_parse_parameter_from_row_insufficient_columns() {
        let html = r#"
            <table>
                <tr>
                    <td>symbol</td>
                    <td>string</td>
                </tr>
            </table>
        "#;
        let document = Html::parse_fragment(html);
        let extractor = ContentExtractor::new(&document);

        let row = document
            .select(&Selector::parse("tr").unwrap())
            .next()
            .unwrap();
        let param = extractor.parse_parameter_from_row(row);

        assert!(param.is_none());
    }

    #[test]
    fn test_parse_parameter_from_row_empty_cells() {
        let html = r#"
            <table>
                <tr>
                    <td></td>
                    <td>string</td>
                    <td>Description</td>
                </tr>
            </table>
        "#;
        let document = Html::parse_fragment(html);
        let extractor = ContentExtractor::new(&document);

        let row = document
            .select(&Selector::parse("tr").unwrap())
            .next()
            .unwrap();
        let param = extractor.parse_parameter_from_row(row);

        assert!(param.is_none());
    }

    #[test]
    fn test_parse_options_comma_separated() {
        let html = Html::parse_fragment("");
        let extractor = ContentExtractor::new(&html);
        let options = extractor.parse_options("json, csv, xml");

        assert_eq!(options, vec!["json", "csv", "xml"]);
    }

    #[test]
    fn test_parse_options_newline_separated() {
        let html = Html::parse_fragment("");
        let extractor = ContentExtractor::new(&html);
        let options = extractor.parse_options("• json\n• csv\n• xml");

        assert_eq!(options, vec!["json", "csv", "xml"]);
    }

    #[test]
    fn test_parse_options_single_value() {
        let html = Html::parse_fragment("");
        let extractor = ContentExtractor::new(&html);
        let options = extractor.parse_options("json");

        assert_eq!(options, vec!["json"]);
    }

    #[test]
    fn test_is_required_parameter_always_required() {
        let html = r#"<table><tr><td>function</td></tr></table>"#;
        let document = Html::parse_fragment(html);
        let extractor = ContentExtractor::new(&document);

        let table = document
            .select(&Selector::parse("table").unwrap())
            .next()
            .unwrap();
        assert!(extractor.is_required_parameter(table, "function"));
        assert!(extractor.is_required_parameter(table, "apikey"));
        assert!(extractor.is_required_parameter(table, "symbol"));
    }

    #[test]
    fn test_is_required_parameter_optional() {
        let html = r#"<table><tr><td>outputsize</td></tr></table>"#;
        let document = Html::parse_fragment(html);
        let extractor = ContentExtractor::new(&document);

        let table = document
            .select(&Selector::parse("table").unwrap())
            .next()
            .unwrap();
        // For now, default is required - in practice this would be more sophisticated
        assert!(extractor.is_required_parameter(table, "outputsize"));
    }

    #[test]
    fn test_parse_parameter_table() {
        let html = r#"
            <table>
                <tr><th>Name</th><th>Type</th><th>Description</th></tr>
                <tr><td>symbol</td><td>string</td><td>Stock symbol</td></tr>
                <tr><td>function</td><td>string</td><td>API function</td></tr>
            </table>
        "#;
        let document = Html::parse_fragment(html);
        let extractor = ContentExtractor::new(&document);

        let table = document
            .select(&Selector::parse("table").unwrap())
            .next()
            .unwrap();
        let params = extractor.parse_parameter_table(table);

        assert_eq!(params.len(), 2);
        assert_eq!(params[0].name, "symbol");
        assert_eq!(params[1].name, "function");
    }

    #[test]
    fn test_extract_parameters_with_table() {
        let html = r#"
            <h3>TIME_SERIES_DAILY</h3>
            <p>Description</p>
            <table>
                <tr><th>Name</th><th>Type</th><th>Description</th></tr>
                <tr><td>function</td><td>string</td><td>API function</td></tr>
                <tr><td>symbol</td><td>string</td><td>Stock symbol</td></tr>
                <tr><td>outputsize</td><td>string</td><td>Output size</td></tr>
            </table>
        "#;
        let document = Html::parse_fragment(html);
        let extractor = ContentExtractor::new(&document);

        let h3 = document
            .select(&Selector::parse("h3").unwrap())
            .next()
            .unwrap();
        let (required, optional) = extractor.extract_parameters(h3);

        // All parameters are currently treated as required by default
        assert_eq!(required.len(), 3);
        assert_eq!(optional.len(), 0);
        assert_eq!(required[0].name, "function");
        assert_eq!(required[1].name, "symbol");
        assert_eq!(required[2].name, "outputsize");
    }

    #[test]
    fn test_parse_endpoint_with_parameters() {
        let html = r#"
            <h3>TIME_SERIES_DAILY</h3>
            <p>This endpoint returns daily time series data.</p>
            <table>
                <tr><th>Name</th><th>Type</th><th>Description</th></tr>
                <tr><td>function</td><td>string</td><td>API function</td></tr>
                <tr><td>symbol</td><td>string</td><td>Stock symbol</td></tr>
            </table>
        "#;
        let document = Html::parse_fragment(html);
        let extractor = ContentExtractor::new(&document);

        let h3 = document
            .select(&Selector::parse("h3").unwrap())
            .next()
            .unwrap();
        let result = extractor.parse_endpoint(h3);

        assert!(result.is_ok());
        let endpoint = result.unwrap();
        assert_eq!(endpoint.function_name, "TIME_SERIES_DAILY");
        assert_eq!(endpoint.required_params.len(), 2);
        assert_eq!(endpoint.optional_params.len(), 0);
        assert_eq!(endpoint.required_params[0].name, "function");
        assert_eq!(endpoint.required_params[1].name, "symbol");
    }

    #[test]
    fn test_is_python_code_python() {
        let html = Html::parse_fragment("");
        let extractor = ContentExtractor::new(&html);

        assert!(extractor.is_python_code("import requests\nresponse = requests.get(url)"));
        assert!(extractor.is_python_code("def get_data():\n    return requests.get('api')"));
        assert!(extractor.is_python_code("import pandas as pd\ndf = pd.DataFrame(data)"));
        assert!(extractor.is_python_code("if __name__ == '__main__':\n    main()"));
    }

    #[test]
    fn test_is_python_code_not_python() {
        let html = Html::parse_fragment("");
        let extractor = ContentExtractor::new(&html);

        // R code
        assert!(!extractor.is_python_code("library(httr)\ndata <- GET(url)"));
        assert!(!extractor.is_python_code("df <- data.frame(x = 1:10)"));

        // VBA code
        assert!(!extractor
            .is_python_code("Sub GetData()\n    Worksheets(1).Range(\"A1\") = \"data\"\nEnd Sub"));

        // MATLAB code
        assert!(!extractor
            .is_python_code("function [output] = getData(input)\n    output = input * 2;\nend"));

        // JSON
        assert!(!extractor.is_python_code("{\n    \"key\": \"value\",\n    \"data\": [1, 2, 3]\n}"));

        // Random text
        assert!(!extractor.is_python_code("This is just some random text without any code."));
    }

    #[test]
    fn test_clean_code_block() {
        let html = Html::parse_fragment("");
        let extractor = ContentExtractor::new(&html);

        let dirty_code = "1  import requests\n2  response = requests.get(&quot;url&quot;)\n3  print(response.text)";
        let clean_code = extractor.clean_code_block(dirty_code);

        assert_eq!(
            clean_code,
            "import requests\nresponse = requests.get(\"url\")\nprint(response.text)"
        );
    }

    #[test]
    fn test_extract_code_example_found() {
        let html = r#"
            <h3>TIME_SERIES_DAILY</h3>
            <p>Description</p>
            <pre><code>import requests
response = requests.get(url)
print(response.json())</code></pre>
        "#;
        let document = Html::parse_fragment(html);
        let extractor = ContentExtractor::new(&document);

        let h3 = document
            .select(&Selector::parse("h3").unwrap())
            .next()
            .unwrap();
        let example = extractor.extract_code_example(h3);

        assert!(example.is_some());
        let example = example.unwrap();
        assert_eq!(example.language, "python");
        assert!(example.code.contains("import requests"));
    }

    #[test]
    fn test_extract_code_example_not_python() {
        let html = r#"
            <h3>TIME_SERIES_DAILY</h3>
            <p>Description</p>
            <pre><code>library(httr)
response <- GET(url)
print(content(response))</code></pre>
        "#;
        let document = Html::parse_fragment(html);
        let extractor = ContentExtractor::new(&document);

        let h3 = document
            .select(&Selector::parse("h3").unwrap())
            .next()
            .unwrap();
        let example = extractor.extract_code_example(h3);

        assert!(example.is_none());
    }

    #[test]
    fn test_extract_code_example_no_code() {
        let html = r#"<h3>TIME_SERIES_DAILY</h3><p>Description only</p>"#;
        let document = Html::parse_fragment(html);
        let extractor = ContentExtractor::new(&document);

        let h3 = document
            .select(&Selector::parse("h3").unwrap())
            .next()
            .unwrap();
        let example = extractor.extract_code_example(h3);

        assert!(example.is_none());
    }

    #[test]
    fn test_deduplicate_code_examples() {
        let html = Html::parse_fragment("");
        let extractor = ContentExtractor::new(&html);

        let examples = vec![
            CodeExample::python("import requests\nprint('hello')".to_string()),
            CodeExample::python("import requests\nprint('hello')".to_string()), // duplicate
            CodeExample::python("import pandas\nprint('world')".to_string()),
        ];

        let deduplicated = extractor.deduplicate_code_examples(examples);
        assert_eq!(deduplicated.len(), 2);
        assert!(deduplicated.iter().any(|e| e.code.contains("pandas")));
        assert!(deduplicated.iter().any(|e| e.code.contains("requests")));
    }

    #[test]
    fn test_parse_endpoint_with_code_example() {
        let html = r#"
            <h3>TIME_SERIES_DAILY</h3>
            <p>This endpoint returns daily time series data.</p>
            <table>
                <tr><th>Name</th><th>Type</th><th>Description</th></tr>
                <tr><td>function</td><td>string</td><td>API function</td></tr>
            </table>
            <pre><code>import requests
response = requests.get(url)
data = response.json()</code></pre>
        "#;
        let document = Html::parse_fragment(html);
        let extractor = ContentExtractor::new(&document);

        let h3 = document
            .select(&Selector::parse("h3").unwrap())
            .next()
            .unwrap();
        let result = extractor.parse_endpoint(h3);

        assert!(result.is_ok());
        let endpoint = result.unwrap();
        assert_eq!(endpoint.function_name, "TIME_SERIES_DAILY");
        assert!(endpoint.python_example.is_some());
        let example = endpoint.python_example.unwrap();
        assert_eq!(example.language, "python");
        assert!(example.code.contains("import requests"));
    }

    #[test]
    fn test_code_similarity() {
        let html = Html::parse_fragment("");
        let extractor = ContentExtractor::new(&html);

        // Exact match
        assert_eq!(
            extractor.code_similarity("import requests", "import requests"),
            1.0
        );

        // Similar but not exact
        assert!(extractor.code_similarity("import requests", "import pandas") < 1.0);

        // Empty strings
        assert_eq!(extractor.code_similarity("", ""), 1.0);
    }

    #[test]
    fn test_extract_request_pattern_constructs_fallback() {
        let html = r#"<h3>TIME_SERIES_DAILY</h3>"#;
        let document = Html::parse_fragment(html);
        let extractor = ContentExtractor::new(&document);

        let h3 = document
            .select(&Selector::parse("h3").unwrap())
            .next()
            .unwrap();
        let pattern = extractor.extract_request_pattern(h3, None, None);

        // Should return base URL when no pattern found
        assert_eq!(pattern, "https://www.alphavantage.co/query");
    }

    #[test]
    fn test_find_url_pattern_found() {
        let html = r#"
            <h3>TIME_SERIES_DAILY</h3>
            <p>Example: https://www.alphavantage.co/query?function=TIME_SERIES_DAILY&symbol=IBM&apikey=demo</p>
        "#;
        let document = Html::parse_fragment(html);
        let extractor = ContentExtractor::new(&document);

        let h3 = document
            .select(&Selector::parse("h3").unwrap())
            .next()
            .unwrap();
        let pattern = extractor.find_url_pattern(h3);

        assert!(pattern.is_some());
        assert_eq!(
            pattern.unwrap(),
            "https://www.alphavantage.co/query?function=TIME_SERIES_DAILY&symbol=IBM&apikey=demo"
        );
    }

    #[test]
    fn test_find_url_pattern_not_found() {
        let html = r#"<h3>TIME_SERIES_DAILY</h3><p>No URL here</p>"#;
        let document = Html::parse_fragment(html);
        let extractor = ContentExtractor::new(&document);

        let h3 = document
            .select(&Selector::parse("h3").unwrap())
            .next()
            .unwrap();
        let pattern = extractor.find_url_pattern(h3);

        assert!(pattern.is_none());
    }

    #[test]
    fn test_construct_pattern_from_endpoint() {
        let html = Html::parse_fragment("");
        let extractor = ContentExtractor::new(&html);

        let params = vec![
            Parameter::new(
                "function".to_string(),
                "string".to_string(),
                "API function".to_string(),
            ),
            Parameter::new(
                "symbol".to_string(),
                "string".to_string(),
                "Stock symbol".to_string(),
            ),
        ];

        let pattern = extractor.construct_pattern_from_endpoint("TIME_SERIES_DAILY", &params);

        assert!(pattern.starts_with("https://www.alphavantage.co/query?"));
        assert!(pattern.contains("function=TIME_SERIES_DAILY"));
        assert!(pattern.contains("symbol=SYMBOL"));
        assert!(pattern.contains("outputsize=compact"));
        assert!(pattern.contains("datatype=json"));
        assert!(pattern.contains("apikey=YOUR_API_KEY"));
    }

    #[test]
    fn test_construct_pattern_with_existing_optionals() {
        let html = Html::parse_fragment("");
        let extractor = ContentExtractor::new(&html);

        let params = vec![
            Parameter::new(
                "function".to_string(),
                "string".to_string(),
                "API function".to_string(),
            ),
            Parameter::new(
                "symbol".to_string(),
                "string".to_string(),
                "Stock symbol".to_string(),
            ),
            Parameter::new(
                "outputsize".to_string(),
                "string".to_string(),
                "Output size".to_string(),
            ),
        ];

        let pattern = extractor.construct_pattern_from_endpoint("TIME_SERIES_DAILY", &params);

        // Should not add default outputsize since it exists in params
        assert!(!pattern.contains("outputsize=compact")); // Default should not be added
        assert!(pattern.contains("datatype=json"));
    }
}
