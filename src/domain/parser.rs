//! HTML parser and DOM traversal utilities.
//!
//! This module provides functionality to parse HTML documents and traverse
//! the DOM tree for content extraction. It includes utilities for selecting
//! elements, extracting text, and working with HTML attributes.

use crate::domain::{DocumentMetadata, ParsedDocument, RawHtml};
use crate::utils::{ParseError, ParseResult};
use scraper::{ElementRef, Html, Selector};
use tracing::info;

/// Implement HTML document parsing from RawHtml.
///
/// This converts raw HTML content into a structured ParsedDocument with
/// extracted metadata and a traversable DOM.
impl TryFrom<RawHtml> for ParsedDocument {
    type Error = String;

    fn try_from(raw: RawHtml) -> Result<Self, Self::Error> {
        // Parse HTML using scraper
        let dom = Html::parse_document(&raw.content);

        // Extract document title with fallbacks
        let title = extract_document_title(&dom);

        // Create document metadata
        let metadata = DocumentMetadata::new(title, raw.source_url.clone());

        Ok(ParsedDocument { dom, metadata })
    }
}

/// Extract document title with fallback strategy.
///
/// Tries to find title in this order:
/// 1. <title> tag content
/// 2. First <h1> tag content
/// 3. Default to "Untitled Document"
fn extract_document_title(dom: &Html) -> String {
    // Try <title> tag first
    if let Some(title_elem) = select_first(dom, "title") {
        let title_text = extract_text(&title_elem);
        if !title_text.trim().is_empty() {
            return title_text.trim().to_string();
        }
    }

    // Fallback to first <h1> tag
    if let Some(h1_elem) = select_first(dom, "h1") {
        let h1_text = extract_text(&h1_elem);
        if !h1_text.trim().is_empty() {
            return h1_text.trim().to_string();
        }
    }

    // Default fallback
    "Untitled Document".to_string()
}

/// Select the first element matching the CSS selector.
///
/// Returns None if no element matches or if the selector is invalid.
pub fn select_first<'a>(doc: &'a Html, selector: &str) -> Option<ElementRef<'a>> {
    let selector = Selector::parse(selector).ok()?;
    doc.select(&selector).next()
}

/// Select all elements matching the CSS selector.
///
/// Returns an empty vector if no elements match or if the selector is invalid.
pub fn select_all<'a>(doc: &'a Html, selector: &str) -> Vec<ElementRef<'a>> {
    let selector = match Selector::parse(selector) {
        Ok(sel) => sel,
        Err(_) => return Vec::new(),
    };
    doc.select(&selector).collect()
}

/// Extract all text content from an element and its descendants.
///
/// Joins text nodes with spaces and normalizes whitespace to create readable text.
pub fn extract_text(element: &ElementRef) -> String {
    element
        .text()
        .collect::<Vec<_>>()
        .join(" ")
        // Normalize whitespace: collapse multiple spaces into single space
        .split_whitespace()
        .collect::<Vec<_>>()
        .join(" ")
}

/// Check if an element has a specific CSS class.
pub fn has_class(element: &ElementRef, class: &str) -> bool {
    element.value().classes().any(|c| c == class)
}

/// Get the value of an attribute from an element.
pub fn get_attribute(element: &ElementRef, attr: &str) -> Option<String> {
    element.value().attr(attr).map(|s| s.to_string())
}

/// Count child elements matching a CSS selector.
pub fn count_children(element: &ElementRef, selector: &str) -> usize {
    let selector = match Selector::parse(selector) {
        Ok(sel) => sel,
        Err(_) => return 0,
    };

    element.select(&selector).count()
}

/// Identify the main content area of an HTML document.
///
/// Uses a three-phase algorithm:
/// 1. Try explicit selectors (main, article, etc.)
/// 2. Use heuristic scoring if no explicit match
/// 3. Fallback to <body> element
pub fn identify_main_content(document: &Html) -> ParseResult<ElementRef<'_>> {
    // Phase A: Explicit selectors
    let explicit_selectors = [
        "main",
        "article",
        "[role='main']",
        ".main-content",
        "#main-content",
        ".container > .row > .col-md-9",
        ".col-md-9",
    ];

    for selector in &explicit_selectors {
        if let Some(element) = select_first(document, selector) {
            info!("Found main content using explicit selector: {}", selector);
            return Ok(element);
        }
    }

    // Phase B: Heuristic scoring
    info!("No explicit content selectors found, using heuristic scoring");
    let candidates = select_all(document, "div, section, article");

    if !candidates.is_empty() {
        let mut best_element = &candidates[0];
        let mut best_score = 0;

        for candidate in &candidates {
            let score = calculate_content_score(candidate);
            if score > best_score {
                best_score = score;
                best_element = candidate;
            }
        }

        info!(
            "Selected main content with heuristic score: {} (text: {}, headings: {}, code: {}, paragraphs: {})",
            best_score,
            extract_text(best_element).len(),
            count_children(best_element, "h2, h3, h4"),
            count_children(best_element, "pre, code"),
            count_children(best_element, "p")
        );

        return Ok(*best_element);
    }

    // If no candidates found at all, continue to Phase C (body fallback)

    // Phase C: Fallback to body
    if let Some(body) = select_first(document, "body") {
        info!("Using fallback: <body> element as main content");
        Ok(body)
    } else {
        Err(ParseError::InvalidHtml(
            "MissingElement: Could not find body element in document".to_string(),
        ))
    }
}

/// Calculate a content score for an element to determine if it's likely main content.
///
/// Higher scores indicate more likely main content areas.
/// Formula: text_length + (heading_count * 100) + (code_block_count * 50) + (paragraph_count * 20)
pub fn calculate_content_score(element: &ElementRef) -> usize {
    let text_length = extract_text(element).len();
    let heading_count = count_children(element, "h2, h3, h4");
    let code_block_count = count_children(element, "pre, code");
    let paragraph_count = count_children(element, "p");

    text_length + (heading_count * 100) + (code_block_count * 50) + (paragraph_count * 20)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::domain::RawUrl;

    #[test]
    fn test_document_parsing_with_title() {
        let html = r#"
            <!DOCTYPE html>
            <html>
            <head><title>Test Document</title></head>
            <body><h1>Heading</h1></body>
            </html>
        "#;

        let raw_html = RawHtml::new(
            html.to_string(),
            ValidatedUrl::try_from(RawUrl::new("https://example.com".to_string())).unwrap(),
        );

        let parsed = ParsedDocument::try_from(raw_html).unwrap();
        assert_eq!(parsed.metadata.title, "Test Document");
    }

    #[test]
    fn test_document_parsing_h1_fallback() {
        let html = r#"
            <!DOCTYPE html>
            <html>
            <body><h1>Fallback Title</h1></body>
            </html>
        "#;

        let raw_html = RawHtml::new(
            html.to_string(),
            ValidatedUrl::try_from(RawUrl::new("https://example.com".to_string())).unwrap(),
        );

        let parsed = ParsedDocument::try_from(raw_html).unwrap();
        assert_eq!(parsed.metadata.title, "Fallback Title");
    }

    #[test]
    fn test_document_parsing_default_title() {
        let html = r#"
            <!DOCTYPE html>
            <html>
            <body><p>No title or h1</p></body>
            </html>
        "#;

        let raw_html = RawHtml::new(
            html.to_string(),
            ValidatedUrl::try_from(RawUrl::new("https://example.com".to_string())).unwrap(),
        );

        let parsed = ParsedDocument::try_from(raw_html).unwrap();
        assert_eq!(parsed.metadata.title, "Untitled Document");
    }

    #[test]
    fn test_select_first() {
        let html = r#"<div><p>First</p><p>Second</p></div>"#;
        let doc = Html::parse_fragment(html);

        let first_p = select_first(&doc, "p");
        assert!(first_p.is_some());
        assert_eq!(extract_text(&first_p.unwrap()), "First");
    }

    #[test]
    fn test_select_all() {
        let html = r#"<div><p>First</p><p>Second</p><span>Third</span></div>"#;
        let doc = Html::parse_fragment(html);

        let all_p = select_all(&doc, "p");
        assert_eq!(all_p.len(), 2);

        let all_span = select_all(&doc, "span");
        assert_eq!(all_span.len(), 1);
    }

    #[test]
    fn test_extract_text() {
        let html = r#"<p>Hello <strong>world</strong>!</p>"#;
        let doc = Html::parse_fragment(html);
        let p = select_first(&doc, "p").unwrap();

        assert_eq!(extract_text(&p), "Hello world !");
    }

    #[test]
    fn test_has_class() {
        let html = r#"<div class="foo bar">Content</div>"#;
        let doc = Html::parse_fragment(html);
        let div = select_first(&doc, "div").unwrap();

        assert!(has_class(&div, "foo"));
        assert!(has_class(&div, "bar"));
        assert!(!has_class(&div, "baz"));
    }

    #[test]
    fn test_get_attribute() {
        let html = r#"<a href="https://example.com" title="Link">Click</a>"#;
        let doc = Html::parse_fragment(html);
        let a = select_first(&doc, "a").unwrap();

        assert_eq!(
            get_attribute(&a, "href"),
            Some("https://example.com".to_string())
        );
        assert_eq!(get_attribute(&a, "title"), Some("Link".to_string()));
        assert_eq!(get_attribute(&a, "nonexistent"), None);
    }

    #[test]
    fn test_count_children() {
        let html = r#"<div><h2>Title 1</h2><p>Para 1</p><h2>Title 2</h2><p>Para 2</p><span>Other</span></div>"#;
        let doc = Html::parse_fragment(html);
        let div = select_first(&doc, "div").unwrap();

        assert_eq!(count_children(&div, "h2"), 2);
        assert_eq!(count_children(&div, "p"), 2);
        assert_eq!(count_children(&div, "span"), 1);
        assert_eq!(count_children(&div, "nonexistent"), 0);
    }

    #[test]
    fn test_invalid_selector() {
        let html = r#"<div><p>Test</p></div>"#;
        let doc = Html::parse_fragment(html);

        // Invalid selectors should return None/empty results gracefully
        let result = select_first(&doc, "invalid[selector");
        assert!(result.is_none());

        let results = select_all(&doc, "invalid[selector");
        assert!(results.is_empty());
    }

    #[test]
    fn test_identify_main_content_with_main_tag() {
        let html_content = std::fs::read_to_string("tests/fixtures/main_tag.html")
            .expect("Failed to read main_tag.html");

        let doc = Html::parse_document(&html_content);
        let main_content = identify_main_content(&doc).unwrap();

        // Should find the <main> element
        assert_eq!(main_content.value().name(), "main");

        // Should contain the expected content
        let text = extract_text(&main_content);
        assert!(text.contains("Main Content Area"));
        assert!(text.contains("API Documentation"));
    }

    #[test]
    fn test_identify_main_content_with_bootstrap_layout() {
        let html_content = std::fs::read_to_string("tests/fixtures/bootstrap_layout.html")
            .expect("Failed to read bootstrap_layout.html");

        let doc = Html::parse_document(&html_content);
        let main_content = identify_main_content(&doc).unwrap();

        // Should find the #main-content element (higher priority than .col-md-9)
        assert_eq!(main_content.value().attr("id"), Some("main-content"));

        // Should contain the expected content
        let text = extract_text(&main_content);
        assert!(text.contains("API Documentation"));
        assert!(text.contains("TIME_SERIES_DAILY"));
    }

    #[test]
    fn test_identify_main_content_with_heuristic_scoring() {
        let html_content = std::fs::read_to_string("tests/fixtures/ambiguous_layout.html")
            .expect("Failed to read ambiguous_layout.html");

        let doc = Html::parse_document(&html_content);
        let main_content = identify_main_content(&doc).unwrap();

        // Should select the .large-content div with highest score
        assert!(has_class(&main_content, "large-content"));

        // Should contain extensive content
        let text = extract_text(&main_content);
        assert!(text.contains("Main Documentation"));
        assert!(text.contains("TIME_SERIES_INTRADAY"));
        assert!(text.contains("CURRENCY_EXCHANGE_RATE"));
        assert!(text.len() > 1000); // Should be substantial content
    }

    #[test]
    fn test_calculate_content_score() {
        // Test with a content-rich element
        let html = r#"
            <div>
                <h2>Title</h2>
                <h3>Subtitle</h3>
                <p>This is a paragraph with some content.</p>
                <p>Another paragraph here.</p>
                <pre><code>some code here</code></pre>
                <p>More content to increase text length significantly.</p>
            </div>
        "#;
        let doc = Html::parse_fragment(html);
        let div = select_first(&doc, "div").unwrap();

        let score = calculate_content_score(&div);

        // Calculate expected score manually
        let text_length = extract_text(&div).len();
        let heading_count = count_children(&div, "h2, h3, h4"); // 2 headings
        let code_block_count = count_children(&div, "pre, code"); // 2 code elements
        let paragraph_count = count_children(&div, "p"); // 3 paragraphs

        let expected_score =
            text_length + (heading_count * 100) + (code_block_count * 50) + (paragraph_count * 20);

        assert_eq!(score, expected_score);
        assert!(score > 500); // Should be a reasonably high score
    }

    #[test]
    fn test_calculate_content_score_empty_element() {
        let html = r#"<div></div>"#;
        let doc = Html::parse_fragment(html);
        let div = select_first(&doc, "div").unwrap();

        let score = calculate_content_score(&div);
        assert_eq!(score, 0); // Empty element should have zero score
    }

    #[test]
    fn test_identify_main_content_fallback_to_body() {
        // Test with content that has no div/section/article elements
        let html = r#"
            <!DOCTYPE html>
            <html>
            <head><title>Test</title></head>
            <body>
                <p>Direct paragraph content</p>
                <span>Some span content</span>
            </body>
            </html>
        "#;

        let doc = Html::parse_document(html);
        let main_content = identify_main_content(&doc).unwrap();

        assert_eq!(main_content.value().name(), "body");
    }
}
