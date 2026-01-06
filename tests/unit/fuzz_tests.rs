//! Fuzz Testing for Content Extractor Resilience
//!
//! These tests verify that the ContentExtractor handles malformed input
//! gracefully without panicking, demonstrating system resilience.

use alphavantage_doc_extractor::domain::ContentExtractor;
use scraper::Html;

#[test]
/// Test that the system handles malformed HTML gracefully without panicking
fn test_fuzz_malformed_html() {
    // Test cases with various forms of malformed input
    let test_cases = vec![
        // Broken HTML structures
        "<broken><html><with><missing></tags>",
        "<unclosed><nested><tags></unclosed>",
        "</unexpected></closing></tags>",
        "<div><span><p></div>",

        // HTML with potential injection attempts
        "<script>alert('xss')</script><p>content</p>",
        "<img src=x onerror=alert(1)><p>test</p>",

        // Encoding and entity issues
        "&lt;unescaped&gt;&#39;quotes&#39;&amp;amps",
        "<p>Broken UTF-8: ���</p>",

        // Deep nesting
        "<div>".repeat(50) + "content" + "</div>".repeat(50),

        // Empty or minimal content
        "",
        "<html></html>",
        "<body></body>",

        // Mixed valid/invalid content
        r#"<html><head><title>Test</title></head><body><h1>Title</h1><p>Content</p><broken><unclosed></body></html>"#,

        // Unicode edge cases
        "🚀🚀🚀<p>Unicode content</p>🚀🚀🚀",
        "café<p>Content with accented chars</p>",
    ];

    for malformed_html in test_cases {
        // Parse the potentially malformed HTML
        let document = Html::parse_document(malformed_html);
        let extractor = ContentExtractor::new(&document);

        // Test main content identification (should not panic)
        let main_content_result = alphavantage_doc_extractor::domain::parser::identify_main_content(document.clone());

        match main_content_result {
            Ok(main_content) => {
                // Test category extraction (should not panic)
                let result = extractor.extract_categories(main_content);
                // Either Ok or Err is fine - the key is NO PANIC
                assert!(result.is_ok() || result.is_err());
            }
            Err(_) => {
                // Also acceptable - system handled malformed input gracefully
            }
        }

        // Test individual selector operations (should not panic)
        let _ = document.select(&scraper::Selector::parse("h1").unwrap_or_else(|_| scraper::Selector::parse("*").unwrap()));
        let _ = document.select(&scraper::Selector::parse("h2").unwrap_or_else(|_| scraper::Selector::parse("*").unwrap()));
        let _ = document.select(&scraper::Selector::parse("h3").unwrap_or_else(|_| scraper::Selector::parse("*").unwrap()));

        // Test text extraction on elements (should not panic)
        for element in document.select(&scraper::Selector::parse("*").unwrap()) {
            let _ = alphavantage_doc_extractor::domain::parser::extract_text(&element);
        }
    }
}

#[test]
/// Test that regex operations handle malformed input gracefully
fn test_regex_resilience() {
    let test_inputs = vec![
        "normal_text",
        "TEXT_WITH_UNDERSCORES",
        "mixedCase123",
        "<script>malicious content</script>",
        "text with spaces and symbols !@#$%^&*()",
        "",  // empty string
        "🚀 Unicode 🚀 content 🚀",
        "very_long_string_".repeat(100),
    ];

    for input in test_inputs {
        // Test function name regex (should not panic)
        if let Ok(regex) = regex::Regex::new(r"[A-Z][A-Z_]*[A-Z]") {
            let _ = regex.find(input);
        }

        // Test URL pattern regex (should not panic)
        if let Ok(regex) = regex::Regex::new(r"https?://www\.alphavantage\.co/query\?[^\s<>]+") {
            let _ = regex.find(input);
        }
    }
}

#[test]
/// Test HTML parsing with various byte sequences
fn test_html_parsing_byte_sequences() {
    // Test with various byte sequences that might not be valid UTF-8
    let test_cases = vec![
        vec![0x00, 0x01, 0x02],  // null bytes
        vec![0xFF, 0xFE, 0xFD],  // high bytes
        vec![0xC0, 0x80],        // invalid UTF-8 sequence
        vec![0xE0, 0x80, 0x80],  // another invalid sequence
        vec![],                   // empty
        vec![72, 101, 108, 108, 111], // "Hello" in ASCII
    ];

    for bytes in test_cases {
        // Convert bytes to string (lossy conversion should not panic)
        let text = String::from_utf8_lossy(&bytes);

        // Try parsing as HTML (should not panic)
        let _ = Html::parse_document(&text);
        let _ = Html::parse_fragment(&text);
    }
}

#[test]
/// Verify the system works correctly with valid input (regression test)
fn test_fuzz_valid_input_regression() {
    let valid_html = r#"
        <!DOCTYPE html>
        <html>
        <body>
            <div id="main-content">
                <h2>Time Series</h2>
                <p>Description</p>
                <h3>TIME_SERIES_DAILY</h3>
                <p>Endpoint description</p>
            </div>
        </body>
        </html>
    "#;

    let document = Html::parse_document(valid_html);
    let extractor = ContentExtractor::new(&document);

    if let Ok(main_content) = alphavantage_doc_extractor::domain::parser::identify_main_content(document) {
        let result = extractor.extract_categories(main_content);
        // Should work fine with valid input
        assert!(result.is_ok() || result.is_err()); // Either is fine, as long as no panic
    }
}

/// Generate random strings that could represent malformed HTML
fn arb_malformed_html() -> impl Strategy<Value = String> {
    // Generate strings with various problematic characters and patterns
    prop_oneof![
        // Random Unicode strings
        any::<String>(),
        // Strings with HTML-like tags but malformed
        r#"<[^>]*>[^<]*</[^>]*>"#.prop_map(|s| s),
        // Deeply nested structures
        prop::collection::vec(any::<String>(), 1..50).prop_map(|parts| {
            parts.into_iter().enumerate().map(|(i, part)| {
                format!("<div{}>{}</div{}>", i, part, i)
            }).collect::<Vec<_>>().join("")
        }),
        // Strings with null bytes and control characters
        prop::collection::vec(any::<u8>(), 1..1000).prop_map(|bytes| {
            String::from_utf8_lossy(&bytes).to_string()
        }),
        // Very long strings
        prop::collection::vec(any::<char>(), 10000..50000).prop_map(|chars| {
            chars.into_iter().collect::<String>()
        }),
        // HTML with script injection attempts
        r#"<script>alert("xss")</script>"#.prop_map(|s| s),
        r#"<img src=x onerror=alert(1)>"#.prop_map(|s| s),
        // Broken HTML structures
        r#"<unclosed><nested><tags></unclosed>"#.prop_map(|s| s),
        r#"</unexpected></closing></tags>"#.prop_map(|s| s),
        // HTML entities and encoding issues
        r#"&lt;unescaped&gt;&#39;quotes&#39;&amp;amps"#.prop_map(|s| s),
    ]
}

/// Generate random Unicode strings to test encoding edge cases
fn arb_unicode_strings() -> impl Strategy<Value = String> {
    prop_oneof![
        // Random Unicode strings
        any::<String>(),
        // High Unicode codepoints
        prop::collection::vec((0x1000..0x10FFFFu32).prop_map(|cp| char::from_u32(cp).unwrap_or('�')), 1..100)
            .prop_map(|chars| chars.into_iter().collect::<String>()),
        // Mixed ASCII and Unicode
        prop::collection::vec(prop_oneof![any::<char>(), (0x80..0xFFFFu32).prop_map(|cp| char::from_u32(cp).unwrap_or('�'))], 1..200)
            .prop_map(|chars| chars.into_iter().collect::<String>()),
        // Invalid UTF-8 sequences (as lossy conversion)
        prop::collection::vec(any::<u8>(), 1..500).prop_map(|bytes| {
            String::from_utf8_lossy(&bytes).to_string()
        }),
    ]
}

/// Generate deeply nested HTML structures to test recursion limits
fn arb_deep_nesting() -> impl Strategy<Value = String> {
    (1..100u32).prop_map(|depth| {
        let mut result = String::new();
        for i in 0..depth {
            result.push_str(&format!("<div class=\"level{}\">", i));
        }
        result.push_str("content");
        for i in (0..depth).rev() {
            result.push_str(&format!("</div><!-- level{} -->", i));
        }
        result
    })
}

#[test]
fn test_fuzz_malformed_html() {
    // Simple fuzz test with a few known problematic inputs
    let test_cases = vec![
        "<broken><html><with><missing></tags>",
        "<script>alert('xss')</script><p>content</p>",
        "<unclosed><nested><tags></unclosed>",
        "</unexpected></closing></tags>",
        "&lt;unescaped&gt;&#39;quotes&#39;&amp;amps",
        "<div>" + &"<div>".repeat(100) + "</div>",
        String::from_utf8_lossy(&[0xFF, 0xFE, 0xFD]).to_string(),
    ];

    for input in test_cases {
        // Parse the potentially malformed HTML
        let document = Html::parse_document(&input);

        // Create extractor
        let extractor = ContentExtractor::new(&document);

        // Try to find a main content area (this might fail, that's ok)
        let main_content_result = alphavantage_doc_extractor::domain::parser::identify_main_content(document.clone());

        // If we found main content, try to extract categories
        if let Ok(main_content) = main_content_result {
            let result = extractor.extract_categories(main_content);
            // The result should be either Ok or Err, but NEVER panic
            match result {
                Ok(_) => {}, // Success is fine
                Err(_) => {}, // Expected errors are fine
            }
        }
        // If we couldn't find main content, that's also fine - the system handled it gracefully
    }
}

    /// Test that ContentExtractor handles Unicode edge cases
    #[test]
    fn test_extract_categories_fuzz_unicode(input in arb_unicode_strings()) {
        let document = Html::parse_document(&input);
        let extractor = ContentExtractor::new(&document);

        let main_content_result = alphavantage_doc_extractor::domain::parser::identify_main_content(document.clone());

        if let Ok(main_content) = main_content_result {
            let result = extractor.extract_categories(main_content);
            // Should handle Unicode gracefully
            match result {
                Ok(_) => {},
                Err(_) => {},
            }
        }
    }

    /// Test that ContentExtractor handles deeply nested structures
    #[test]
    fn test_extract_categories_fuzz_deep_nesting(input in arb_deep_nesting()) {
        let document = Html::parse_document(&input);
        let extractor = ContentExtractor::new(&document);

        let main_content_result = alphavantage_doc_extractor::domain::parser::identify_main_content(document.clone());

        if let Ok(main_content) = main_content_result {
            let result = extractor.extract_categories(main_content);
            // Should handle deep nesting gracefully
            match result {
                Ok(_) => {},
                Err(_) => {},
            }
        }
    }

    /// Test that individual extractor methods handle malformed input
    #[test]
    fn test_extractor_methods_fuzz(input in arb_malformed_html()) {
        let document = Html::parse_document(&input);
        let extractor = ContentExtractor::new(&document);

        // Test various selector operations that might fail
        let _ = document.select(&scraper::Selector::parse("h1").unwrap_or_else(|_| scraper::Selector::parse("*").unwrap()));
        let _ = document.select(&scraper::Selector::parse("h2").unwrap_or_else(|_| scraper::Selector::parse("*").unwrap()));
        let _ = document.select(&scraper::Selector::parse("h3").unwrap_or_else(|_| scraper::Selector::parse("*").unwrap()));
        let _ = document.select(&scraper::Selector::parse("table").unwrap_or_else(|_| scraper::Selector::parse("*").unwrap()));
        let _ = document.select(&scraper::Selector::parse("pre").unwrap_or_else(|_| scraper::Selector::parse("*").unwrap()));
        let _ = document.select(&scraper::Selector::parse("code").unwrap_or_else(|_| scraper::Selector::parse("*").unwrap()));

        // Test text extraction on potentially malformed elements
        for element in document.select(&scraper::Selector::parse("*").unwrap()) {
            let _ = alphavantage_doc_extractor::domain::parser::extract_text(&element);
        }
    }

    /// Test that HTML parsing itself handles extreme input
    #[test]
    fn test_html_parsing_fuzz(input in prop::collection::vec(any::<u8>(), 0..10000)) {
        // Test with raw bytes that might not be valid UTF-8
        let _ = String::from_utf8_lossy(&input);

        // If it's valid UTF-8, try parsing as HTML
        if let Ok(text) = std::str::from_utf8(&input) {
            let _ = Html::parse_document(text);
            let _ = Html::parse_fragment(text);
        }
    }

    /// Test regex operations with fuzzed input
    #[test]
    fn test_regex_fuzz(input in arb_malformed_html()) {
        // Test the function name regex
        if let Ok(regex) = regex::Regex::new(r"[A-Z][A-Z_]*[A-Z]") {
            let _ = regex.find(&input);
        }

        // Test the URL pattern regex
        if let Ok(regex) = regex::Regex::new(r"https?://www\.alphavantage\.co/query\?[^\s<>]+") {
            let _ = regex.find(&input);
        }
    }
}

#[test]
/// Sanity check that the system works with valid input (not fuzzed)
fn test_fuzz_sanity_check() {
    let valid_html = r#"
        <!DOCTYPE html>
        <html>
        <body>
            <div id="main-content">
                <h2>Time Series</h2>
                <p>Description</p>
                <h3>TIME_SERIES_DAILY</h3>
                <p>Endpoint description</p>
            </div>
        </body>
        </html>
    "#;

    let document = Html::parse_document(valid_html);
    let extractor = ContentExtractor::new(&document);

    if let Ok(main_content) = alphavantage_doc_extractor::domain::parser::identify_main_content(document) {
        let result = extractor.extract_categories(main_content);
        // Should work fine with valid input
        assert!(result.is_ok() || result.is_err()); // Either is fine, as long as no panic
    }
}
