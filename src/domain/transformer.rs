//! HTML Sanitization and Cleaning
//!
//! This module provides functionality to clean and sanitize HTML content
//! by removing navigation, promotional content, technical artifacts, and
//! other unwanted elements that are not part of the main documentation.

use crate::utils::{error::ExtractionResult, logging::Timer};
use regex::Regex;
use scraper::{Html, Selector};
use tracing::{debug, info};

/// Constants for HTML cleaning selectors and phrases
const NAVIGATION_SELECTORS: &[&str] = &[
    "nav",
    ".sidebar",
    "#sidebar",
    ".navigation",
    ".nav-menu",
    ".toc",
    "aside",
    "[role='navigation']",
];

const PROMOTIONAL_SELECTORS: &[&str] = &[
    "footer",
    ".footer",
    "#footer",
    ".banner",
    ".cta",
    ".newsletter",
    ".social-links",
];

const TECHNICAL_ARTIFACT_SELECTORS: &[&str] = &["script", "style", "noscript", "iframe"];

const PROMOTIONAL_PHRASES: &[&str] = &[
    "looking for more programming languages?",
    "claim your free api key",
    "premium membership",
    "subscribe",
    "follow us",
];

/// HtmlCleaner provides methods to sanitize and clean HTML content.
#[derive(Debug, Default)]
pub struct HtmlCleaner;

impl HtmlCleaner {
    /// Create a new HtmlCleaner instance.
    pub fn new() -> Self {
        Self
    }

    /// Clean HTML content by removing unwanted elements and artifacts.
    ///
    /// This method applies a comprehensive cleaning pipeline:
    /// 1. Remove technical artifacts (scripts, styles, etc.)
    /// 2. Remove navigation elements
    /// 3. Remove promotional content
    /// 4. Remove elements by text content
    /// 5. Clean anchor links
    /// 6. Remove empty elements
    ///
    /// # Arguments
    /// * `html_content` - The raw HTML content to clean
    ///
    /// # Returns
    /// Cleaned HTML string or an extraction error
    pub fn clean(&self, html_content: &str) -> ExtractionResult<String> {
        let _timer = Timer::new("html_cleaning");
        info!("Starting HTML cleaning process");

        let original_size = html_content.len();
        let mut cleaned = html_content.to_string();

        // Remove technical artifacts first (scripts, styles, etc.)
        cleaned = self.remove_by_selectors(&cleaned, TECHNICAL_ARTIFACT_SELECTORS);
        debug!("Removed technical artifacts");

        // Remove navigation elements
        cleaned = self.remove_by_selectors(&cleaned, NAVIGATION_SELECTORS);
        debug!("Removed navigation elements");

        // Remove promotional content by selectors
        cleaned = self.remove_by_selectors(&cleaned, PROMOTIONAL_SELECTORS);
        debug!("Removed promotional content by selectors");

        // Remove promotional content by text phrases
        cleaned = self.remove_by_text_content(&cleaned, PROMOTIONAL_PHRASES);
        debug!("Removed promotional content by text");

        // Clean anchor links
        cleaned = self.clean_anchor_links(&cleaned);
        debug!("Cleaned anchor links");

        // Remove empty elements
        cleaned = self.remove_empty_elements(&cleaned);
        debug!("Removed empty elements");

        // Validate the cleaned HTML
        if let Err(errors) = self.validate_cleaned(&cleaned) {
            info!("Validation found remaining artifacts: {:?}", errors);
        }

        let final_size = cleaned.len();
        let reduction = original_size.saturating_sub(final_size);
        info!(
            "HTML cleaning completed: {} -> {} bytes ({} bytes removed)",
            original_size, final_size, reduction
        );

        Ok(cleaned)
    }

    /// Remove elements matching the given CSS selectors.
    fn remove_by_selectors(&self, html: &str, selectors: &[&str]) -> String {
        let document = Html::parse_document(html);
        let mut html_string = html.to_string();

        for selector_str in selectors {
            if let Ok(selector) = Selector::parse(selector_str) {
                for element in document.select(&selector) {
                    let html_repr = element.html();
                    html_string = html_string.replace(&html_repr, "");
                }
            }
        }

        html_string
    }

    /// Remove elements containing specific text phrases (case-insensitive).
    fn remove_by_text_content(&self, html: &str, phrases: &[&str]) -> String {
        let document = Html::parse_document(html);
        let mut html_string = html.to_string();

        // Select text-containing elements that we want to check for promotional content
        // Only check leaf elements, not containers that hold other content
        let text_selectors = [
            "p", "span", "h1", "h2", "h3", "h4", "h5", "h6", "div", "section",
        ];

        for selector_str in &text_selectors {
            if let Ok(selector) = Selector::parse(selector_str) {
                for element in document.select(&selector) {
                    // Only process elements that don't have significant child elements with content
                    let child_elements: Vec<_> = element.children().collect();
                    let has_content_children = child_elements.iter().any(|child| {
                        child.value().is_element()
                            && !child.value().as_element().unwrap().name().is_empty()
                    });

                    // Skip elements that have other elements as children (containers)
                    if has_content_children && *selector_str != "div" && *selector_str != "section"
                    {
                        continue;
                    }

                    let text_content = super::extract_text(&element);
                    let lower_text = text_content.to_lowercase();
                    let trimmed_text = lower_text.trim();

                    // Skip if text is too short or empty
                    if trimmed_text.len() < 3 {
                        continue;
                    }

                    for phrase in phrases {
                        if trimmed_text.contains(&phrase.to_lowercase()) {
                            let html_repr = element.html();
                            html_string = html_string.replace(&html_repr, "");
                            break; // Remove the first matching phrase
                        }
                    }
                }
            }
        }

        html_string
    }

    /// Remove empty HTML elements (div, p, span, etc. with only whitespace).
    fn remove_empty_elements(&self, html: &str) -> String {
        let empty_patterns = [
            r#"<div[^>]*>\s*</div>"#,
            r#"<p[^>]*>\s*</p>"#,
            r#"<span[^>]*>\s*</span>"#,
            r#"<section[^>]*>\s*</section>"#,
            r#"<article[^>]*>\s*</article>"#,
            r#"<aside[^>]*>\s*</aside>"#,
            r#"<nav[^>]*>\s*</nav>"#,
            r#"<header[^>]*>\s*</header>"#,
            r#"<footer[^>]*>\s*</footer>"#,
        ];

        let mut cleaned = html.to_string();

        for pattern in &empty_patterns {
            if let Ok(regex) = Regex::new(pattern) {
                cleaned = regex.replace_all(&cleaned, "").to_string();
            }
        }

        cleaned
    }

    /// Clean anchor links by removing internal page links and "back to top" links.
    fn clean_anchor_links(&self, html: &str) -> String {
        let document = Html::parse_document(html);
        let mut html_string = html.to_string();

        if let Ok(selector) = Selector::parse("a") {
            for element in document.select(&selector) {
                // Check for href="#..." links
                if let Some(href) = element.value().attr("href") {
                    if href.starts_with('#') {
                        let html_repr = element.html();
                        html_string = html_string.replace(&html_repr, "");
                        continue;
                    }
                }

                // Check for "back to top" links (text content)
                let text_content = super::extract_text(&element);
                let lower_text = text_content.to_lowercase();

                if lower_text.contains("back to top")
                    || lower_text.contains('↑')
                    || lower_text.contains('⬆')
                    || lower_text.contains("top")
                        && (lower_text.contains("back") || lower_text.contains("return"))
                {
                    let html_repr = element.html();
                    html_string = html_string.replace(&html_repr, "");
                }
            }
        }

        html_string
    }

    /// Validate that cleaned HTML doesn't contain unwanted artifacts.
    ///
    /// Returns Ok(()) if validation passes, or Err(Vec<String>) with error messages.
    pub fn validate_cleaned(&self, html: &str) -> Result<(), Vec<String>> {
        let mut errors = Vec::new();

        // Check for remaining navigation elements
        for selector in NAVIGATION_SELECTORS {
            if html.contains(&format!("<{} ", selector.trim_start_matches('.')))
                || html.contains(&format!("<{}", selector.trim_start_matches('.')))
            {
                errors.push(format!("Found remaining navigation element: {}", selector));
            }
        }

        // Check for remaining promotional elements
        for selector in PROMOTIONAL_SELECTORS {
            if html.contains(&format!("<{} ", selector.trim_start_matches('.')))
                || html.contains(&format!("<{}", selector.trim_start_matches('.')))
            {
                errors.push(format!("Found remaining promotional element: {}", selector));
            }
        }

        // Check for technical artifacts
        for selector in TECHNICAL_ARTIFACT_SELECTORS {
            if html.contains(&format!("<{} ", selector)) || html.contains(&format!("<{}", selector))
            {
                errors.push(format!("Found remaining technical artifact: {}", selector));
            }
        }

        // Check for promotional phrases
        let lower_html = html.to_lowercase();
        for phrase in PROMOTIONAL_PHRASES {
            if lower_html.contains(phrase) {
                errors.push(format!("Found remaining promotional phrase: {}", phrase));
            }
        }

        if errors.is_empty() {
            Ok(())
        } else {
            Err(errors)
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_remove_by_selectors() {
        let cleaner = HtmlCleaner::new();
        let html = r#"<div><script>alert('test')</script><p>Content</p></div>"#;

        let cleaned = cleaner.remove_by_selectors(html, &["script"]);
        assert!(!cleaned.contains("<script>"));
        assert!(cleaned.contains("<p>Content</p>"));
    }

    #[test]
    fn test_remove_by_text_content() {
        let cleaner = HtmlCleaner::new();
        let html = r#"<div><p>Subscribe to our newsletter</p><p>Real content</p></div>"#;

        let cleaned = cleaner.remove_by_text_content(html, &["subscribe"]);
        assert!(!cleaned.contains("Subscribe to our newsletter"));
        assert!(cleaned.contains("Real content"));
    }

    #[test]
    fn test_remove_empty_elements() {
        let cleaner = HtmlCleaner::new();
        let html = r#"<div><p>Content</p><div>   </div><span></span></div>"#;

        let cleaned = cleaner.remove_empty_elements(html);
        assert!(!cleaned.contains("<div>   </div>"));
        assert!(!cleaned.contains("<span></span>"));
        assert!(cleaned.contains("<p>Content</p>"));
    }

    #[test]
    fn test_clean_anchor_links() {
        let cleaner = HtmlCleaner::new();
        let html = r##"<div><a href="#top">Back to top ↑</a><a href="/page">Valid link</a></div>"##;

        let cleaned = cleaner.clean_anchor_links(html);
        assert!(!cleaned.contains("Back to top"));
        assert!(cleaned.contains("Valid link"));
    }

    #[test]
    fn test_validate_cleaned_success() {
        let cleaner = HtmlCleaner::new();
        let clean_html = r#"<div><p>Clean content without artifacts</p></div>"#;

        assert!(cleaner.validate_cleaned(clean_html).is_ok());
    }

    #[test]
    fn test_validate_cleaned_with_artifacts() {
        let cleaner = HtmlCleaner::new();
        let dirty_html = r#"<div><script>alert('test')</script><p>Clean content</p></div>"#;

        let result = cleaner.validate_cleaned(dirty_html);
        assert!(result.is_err());
        let errors = result.unwrap_err();
        assert!(errors.iter().any(|e| e.contains("script")));
    }

    #[test]
    fn test_full_cleaning_pipeline() {
        let cleaner = HtmlCleaner::new();
        let dirty_html = r##"<div><nav>Navigation</nav><script>alert('test')</script><div class="newsletter">Subscribe to our newsletter</div><p>Real content here</p><a href="#top">Back to top</a><div></div></div>"##;

        let cleaned = cleaner.clean(dirty_html).unwrap();

        // Should not contain removed elements
        assert!(!cleaned.contains("<nav>"));
        assert!(!cleaned.contains("<script>"));
        assert!(!cleaned.contains("Subscribe"));
        assert!(!cleaned.contains("Back to top"));
        assert!(!cleaned.contains("<div></div>"));

        // Should contain real content
        assert!(cleaned.contains("Real content here"));
    }
}
