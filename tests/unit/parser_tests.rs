//! Unit tests for HTML parser and DOM utilities.
//!
//! Tests document parsing, title extraction, and all DOM utility functions
//! using test fixtures and generated HTML fragments.

use alphavantage_doc_extractor::domain::{
    count_children, extract_text, get_attribute, has_class, select_all, select_first, DocumentMetadata,
    ParsedDocument, RawHtml, ValidatedUrl,
};
use alphavantage_doc_extractor::domain::RawUrl;
use scraper::Html;
use std::fs;

#[test]
fn test_parse_valid_document_with_title() {
    let html_content = fs::read_to_string("tests/fixtures/valid_document.html")
        .expect("Failed to read valid_document.html");

    let raw_html = RawHtml::new(
        html_content,
        ValidatedUrl::try_from(RawUrl::new("https://example.com".to_string())).unwrap(),
    );

    let parsed = ParsedDocument::try_from(raw_html).unwrap();
    assert_eq!(parsed.metadata.title, "Alpha Vantage API Documentation");
}

#[test]
fn test_parse_document_with_h1_fallback() {
    let html_content = fs::read_to_string("tests/fixtures/no_title.html")
        .expect("Failed to read no_title.html");

    let raw_html = RawHtml::new(
        html_content,
        ValidatedUrl::try_from(RawUrl::new("https://example.com".to_string())).unwrap(),
    );

    let parsed = ParsedDocument::try_from(raw_html).unwrap();
    assert_eq!(parsed.metadata.title, "Alpha Vantage API Reference");
}

#[test]
fn test_parse_malformed_document() {
    let html_content = fs::read_to_string("tests/fixtures/malformed.html")
        .expect("Failed to read malformed.html");

    let raw_html = RawHtml::new(
        html_content,
        ValidatedUrl::try_from(RawUrl::new("https://example.com".to_string())).unwrap(),
    );

    // Parser should handle malformed HTML gracefully
    let parsed = ParsedDocument::try_from(raw_html).unwrap();
    assert_eq!(parsed.metadata.title, "Malformed HTML Document");
}

#[test]
fn test_select_first_returns_first_match() {
    let html = r#"
        <div>
            <p>First paragraph</p>
            <p>Second paragraph</p>
            <p>Third paragraph</p>
        </div>
    "#;

    let doc = Html::parse_fragment(html);
    let first_p = select_first(&doc, "p");

    assert!(first_p.is_some());
    assert_eq!(extract_text(&first_p.unwrap()), "First paragraph");
}

#[test]
fn test_select_all_returns_multiple_matches() {
    let html = r#"
        <article>
            <p>First paragraph with <em>emphasis</em></p>
            <p>Second paragraph</p>
            <p>Third paragraph</p>
            <div>Some div content</div>
            <p>Fourth paragraph</p>
        </article>
    "#;

    let doc = Html::parse_fragment(html);
    let all_p = select_all(&doc, "p");

    assert_eq!(all_p.len(), 4);
    assert_eq!(extract_text(&all_p[0]), "First paragraph with emphasis");
    assert_eq!(extract_text(&all_p[1]), "Second paragraph");
    assert_eq!(extract_text(&all_p[2]), "Third paragraph");
    assert_eq!(extract_text(&all_p[3]), "Fourth paragraph");
}

#[test]
fn test_has_class_identifies_classes() {
    let html = r#"
        <div class="primary active">Primary content</div>
        <div class="secondary">Secondary content</div>
        <span>No classes</span>
    "#;

    let doc = Html::parse_fragment(html);
    let primary_div = select_first(&doc, "div.primary").unwrap();
    let secondary_div = select_first(&doc, "div.secondary").unwrap();
    let span = select_first(&doc, "span").unwrap();

    assert!(has_class(&primary_div, "primary"));
    assert!(has_class(&primary_div, "active"));
    assert!(!has_class(&primary_div, "secondary"));

    assert!(has_class(&secondary_div, "secondary"));
    assert!(!has_class(&secondary_div, "primary"));

    assert!(!has_class(&span, "any-class"));
}

#[test]
fn test_get_attribute_extracts_values() {
    let html = r#"
        <a href="https://example.com" title="Example Link" data-id="123">Click here</a>
        <img src="/image.jpg" alt="Test image">
        <div>No attributes</div>
    "#;

    let doc = Html::parse_fragment(html);
    let link = select_first(&doc, "a").unwrap();
    let img = select_first(&doc, "img").unwrap();
    let div = select_first(&doc, "div").unwrap();

    assert_eq!(get_attribute(&link, "href"), Some("https://example.com".to_string()));
    assert_eq!(get_attribute(&link, "title"), Some("Example Link".to_string()));
    assert_eq!(get_attribute(&link, "data-id"), Some("123".to_string()));
    assert_eq!(get_attribute(&link, "nonexistent"), None);

    assert_eq!(get_attribute(&img, "src"), Some("/image.jpg".to_string()));
    assert_eq!(get_attribute(&img, "alt"), Some("Test image".to_string()));

    assert_eq!(get_attribute(&div, "any"), None);
}

#[test]
fn test_count_children_counts_elements() {
    let html = r#"
        <article>
            <header>
                <h1>Main Title</h1>
                <p>Subtitle</p>
            </header>
            <section>
                <h2>Section 1</h2>
                <p>Content 1</p>
                <h3>Subsection 1.1</h3>
                <p>More content</p>
            </section>
            <section>
                <h2>Section 2</h2>
                <p>Content 2</p>
                <h3>Subsection 2.1</h3>
                <p>Even more content</p>
                <h3>Subsection 2.2</h3>
                <p>Final content</p>
            </section>
            <aside>
                <h3>Sidebar</h3>
                <p>Sidebar content</p>
            </aside>
        </article>
    "#;

    let doc = Html::parse_fragment(html);
    let article = select_first(&doc, "article").unwrap();

    // Count all h2 elements (should be 2)
    assert_eq!(count_children(&article, "h2"), 2);

    // Count all h3 elements (should be 4)
    assert_eq!(count_children(&article, "h3"), 4);

    // Count all p elements (should be 7)
    assert_eq!(count_children(&article, "p"), 7);

    // Count section elements (should be 2)
    assert_eq!(count_children(&article, "section"), 2);

    // Count non-existent elements (should be 0)
    assert_eq!(count_children(&article, "table"), 0);
}

#[test]
fn test_extract_text_with_complex_content() {
    let html = r#"
        <div>
            <h1>Welcome to <em>Alpha Vantage</em></h1>
            <p>This is a <strong>test</strong> paragraph with <a href="#">links</a> and <code>code</code>.</p>
            <ul>
                <li>First item</li>
                <li>Second item with <span>nested</span> content</li>
            </ul>
        </div>
    "#;

    let doc = Html::parse_fragment(html);
    let div = select_first(&doc, "div").unwrap();

    let extracted = extract_text(&div);
    assert!(extracted.contains("Welcome to Alpha Vantage"));
    assert!(extracted.contains("This is a test paragraph"));
    assert!(extracted.contains("First item"));
    assert!(extracted.contains("Second item with nested content"));
}

#[test]
fn test_parser_handles_empty_content() {
    let html = r#"<html><head></head><body></body></html>"#;

    let raw_html = RawHtml::new(
        html.to_string(),
        ValidatedUrl::try_from(RawUrl::new("https://example.com".to_string())).unwrap(),
    );

    let parsed = ParsedDocument::try_from(raw_html).unwrap();
    assert_eq!(parsed.metadata.title, "Untitled Document");
}

#[test]
fn test_select_first_with_no_matches() {
    let html = r#"<div><p>Content</p></div>"#;
    let doc = Html::parse_fragment(html);

    let result = select_first(&doc, "h1");
    assert!(result.is_none());
}

#[test]
fn test_select_all_with_no_matches() {
    let html = r#"<div><p>Content</p></div>"#;
    let doc = Html::parse_fragment(html);

    let results = select_all(&doc, "table");
    assert!(results.is_empty());
}

#[test]
fn test_count_children_with_no_matches() {
    let html = r#"<div><p>Content</p></div>"#;
    let doc = Html::parse_fragment(html);
    let div = select_first(&doc, "div").unwrap();

    assert_eq!(count_children(&div, "span"), 0);
}

#[test]
fn test_extract_text_from_empty_element() {
    let html = r#"<div></div>"#;
    let doc = Html::parse_fragment(html);
    let div = select_first(&doc, "div").unwrap();

    assert_eq!(extract_text(&div), "");
}

    #[test]
    fn test_fixture_files_exist() {
        // Ensure our test fixtures exist
        assert!(fs::metadata("tests/fixtures/valid_document.html").is_ok());
        assert!(fs::metadata("tests/fixtures/no_title.html").is_ok());
        assert!(fs::metadata("tests/fixtures/malformed.html").is_ok());
        assert!(fs::metadata("tests/fixtures/main_tag.html").is_ok());
        assert!(fs::metadata("tests/fixtures/bootstrap_layout.html").is_ok());
        assert!(fs::metadata("tests/fixtures/ambiguous_layout.html").is_ok());
    }
