# Technical Specification Document: Alpha Vantage Documentation Extractor

**Project Name:** `alphavantage-doc-extractor`
**Version:** 2.0.0
**Status:** Approved for Development
**Classification:** Open Source Utility
**License:** MIT
**Last Updated:** January 5, 2026

***

## 1. Executive Summary

`alphavantage-doc-extractor` is a high-performance, command-line utility that transforms the Alpha Vantage API documentation from a single-page HTML application into a structured, offline-ready Markdown reference optimized for Large Language Model (LLM) consumption and developer workflows.[^1]

The tool addresses a critical gap: while Alpha Vantage provides comprehensive API documentation at https://www.alphavantage.co/documentation/, developers need a clean, portable, version-controlled format suitable for LLM ingestion, offline reference, and integration into documentation pipelines.

**Core Value Proposition:**

- **LLM-Ready Output:** Structured markdown optimized for embedding generation and RAG systems
- **Zero Noise:** Removes all HTML artifacts, navigation, promotional content, and redundant elements
- **Developer-Focused:** Clean code examples and parameter tables in markdown format
- **Maintainable:** Well-tested, documented codebase following FAANG-grade engineering principles

***

## 2. Project Scope

### 2.1 In Scope

**Content Extraction:**

- All API endpoint documentation including:
    - Time Series Stock Data APIs
    - US Options Data APIs
    - Alpha Intelligence
    - Fundamental Data APIs
    - Forex (FX) Data APIs
    - Cryptocurrency APIs
    - Commodities APIs
    - Economic Indicators
    - Technical Indicators
- Endpoint descriptions and functionality
- Required and optional parameter tables
- Python code examples only
- Request URL patterns

**Content Transformation:**

- HTML to Markdown conversion
- Parameter tables to Markdown table format
- Code blocks to fenced Markdown syntax
- Hierarchical heading structure preservation
- Automatic table of contents generation

**Quality Assurance:**

- Navigation element exclusion
- Footer and promotional content removal
- HTML artifact sanitization
- Duplicate content deduplication
- Markdown syntax validation


### 2.2 Out of Scope

- **Live API response fetching:** JSON example responses will not be fetched from live API endpoints
- **Multi-language support:** Only Python examples included (JavaScript, PHP, C\#, R examples excluded)
- **Real-time updates:** Not a monitoring tool; one-time extraction utility
- **API key management:** No credential storage or API authentication
- **Documentation hosting:** Output generation only; deployment not included


### 2.3 Success Criteria

| Metric | Target | Measurement |
| :-- | :-- | :-- |
| Content Accuracy | >99% fidelity to source | Manual validation against 10 random endpoints |
| HTML Artifact Removal | 100% clean | Zero HTML tags in output |
| LLM Readability Score | >90% | Token efficiency (content:noise ratio) |
| Execution Time | <30 seconds | Complete extraction on standard hardware |
| Memory Footprint | <100MB peak | Runtime monitoring |
| Output Size | 2-4MB | File size measurement |
| Test Coverage | >85% | Code coverage tools |


***

## 3. Functional Requirements

### 3.1 Content Selection Strategy

#### Primary Content Container

**Target:** The main documentation content area, excluding all navigation and ancillary elements.

**CSS Selector Strategy:**

```rust
// Primary content area (typical Bootstrap layout)
let main_content = Selector::parse("body > .container > .row > .col-md-9").unwrap();

// Fallback selectors (if layout changes)
let fallbacks = vec![
    "main",
    "article",
    ".main-content",
    "#content",
    "[role='main']"
];
```


#### Exclusion Rules

**Elements to Remove Entirely:**

```
Navigation Elements:
- <nav>
- .sidebar, #sidebar
- .navigation, .nav-menu
- .table-of-contents, .toc
- a[href^="#"] (in-page anchor links)

Promotional/Footer Content:
- <footer>
- .footer, #footer
- Elements containing text: "Looking for more programming languages?"
- Elements containing text: "Want to integrate with LLMs?"
- Elements containing text: "Claim your free API key"
- .banner, .cta, .call-to-action
- .newsletter-signup, .email-capture

Technical Artifacts:
- <script>
- <style>
- <noscript>
- <iframe>
- <!-- HTML comments -->
- Empty elements: <div></div>, <p></p>, <span></span>

Language-Specific Examples (Non-Python):
- Code blocks containing "// JavaScript"
- Code blocks containing "<?php"
- Code blocks containing "using System;" (C#)
- Code blocks containing "library(" (R)
```


#### Content Deduplication Strategy

**Redundancy Identification:**
The Alpha Vantage documentation contains systematic redundancy across endpoints:

1. **Repeated Boilerplate:** Standard descriptions appearing in multiple sections
2. **Footer Promotions:** Identical CTAs after each API section
3. **Navigation Elements:** Repeated sidebar content

**Deduplication Approach:**

```rust
// Track unique content by section
let mut seen_content: HashMap<String, bool> = HashMap::new();

// For each content block
if let Some(text) = normalize_text(&element.text()) {
    // Create content hash
    let hash = compute_hash(&text);
    
    // Skip if seen before (with context awareness)
    if seen_content.contains_key(&hash) {
        // Exception: Parameter tables may look similar but differ in values
        if !is_parameter_table(&element) {
            continue; // Skip duplicate
        }
    }
    
    seen_content.insert(hash, true);
}
```

**Context-Aware Deduplication:**

- Keep endpoint-specific parameter descriptions even if similar structure
- Remove identical promotional footers across sections
- Preserve similar-looking code examples if they demonstrate different endpoints
- Eliminate repeated navigation instructions


### 3.2 LLM Optimization Requirements

#### Structural Optimization

**Chunking Strategy:**
Each API endpoint should be a self-contained, semantically complete unit for embedding generation.

**Structure Per Endpoint:**

```markdown
## [API_FUNCTION_NAME]

### Description
[Single paragraph, 2-4 sentences describing purpose and use case]

### Parameters

#### Required Parameters
| Parameter | Value | Description |
|-----------|-------|-------------|
| ... | ... | ... |

#### Optional Parameters
| Parameter | Default | Options | Description |
|-----------|---------|---------|-------------|
| ... | ... | ... | ... |

### Request Format
[Standard URL pattern with placeholder values]

### Python Implementation
[Clean, executable code example]

---
```

**LLM-Specific Enhancements:**

1. **Semantic Markers:**
```markdown
<!-- ENDPOINT: TIME_SERIES_INTRADAY -->
<!-- CATEGORY: Time Series Stock Data -->
<!-- PREMIUM: false -->
```

Hidden HTML comments provide metadata for embedding systems without cluttering visual presentation.

2. **Consistent Terminology:**

- Standardize parameter names (e.g., always use `apikey` not `api_key` or `API_KEY`)
- Use consistent case (e.g., `TIME_SERIES_DAILY` not `Time_Series_Daily`)
- Normalize spacing and punctuation

3. **Token Efficiency:**

- Remove verbose explanations that don't add semantic value
- Eliminate marketing language
- Keep descriptions factual and concise
- Remove redundant examples showing the same concept

4. **Hierarchical Structure:**
```
Document Level (H1)
├── Category Level (H2)
│   ├── Endpoint Level (H3)
│   │   ├── Description
│   │   ├── Parameters (H4)
│   │   │   ├── Required (H5)
│   │   │   └── Optional (H5)
│   │   ├── Request Format (H4)
│   │   └── Python Implementation (H4)
```

Consistent depth allows LLMs to understand document hierarchy.

### 3.3 Content Transformation Rules

#### HTML to Markdown Mapping

| HTML Element | Markdown Output | Notes |
| :-- | :-- | :-- |
| `<h1>` | `# Heading` | Document title only |
| `<h2>` | `## Heading` | Category sections |
| `<h3>` | `### Heading` | Endpoint names |
| `<h4>` | `#### Heading` | Subsections |
| `<p>` | Plain text with newlines | Double newline between paragraphs |
| `<strong>`, `<b>` | `**bold**` | Emphasis preserved |
| `<em>`, `<i>` | `*italic*` | Emphasis preserved |
| `<code>` | ```code``` | Inline code |
| `<pre><code>` | ````language\ncode\n```` | Fenced code blocks |
| `<table>` | Markdown table | With alignment |
| `<a href="external">` | `[text](url)` | External links only |
| `<a href="#">` | `text` | Remove anchor links |
| `<ul>`, `<li>` | `- item` | Bullet lists |
| `<ol>`, `<li>` | `1. item` | Numbered lists |
| `<br>` | Double newline | Paragraph break |

#### Code Block Processing

**Detection:**

```rust
// Identify code blocks
if element.value().name() == "pre" {
    if let Some(code_element) = element.select(&code_selector).next() {
        let code_text = code_element.text().collect::<String>();
        
        // Detect language (Python only in scope)
        let language = if code_text.contains("import ") || code_text.contains("def ") {
            "python"
        } else if code_text.starts_with("http") {
            "bash"  // For request URLs
        } else {
            ""  // Generic code block
        };
        
        // Skip non-Python examples
        if is_non_python_example(&code_text) {
            continue;
        }
        
        markdown.push_str(&format!("```{}\n{}\n```\n\n", language, code_text));
    }
}
```

**Non-Python Detection Patterns:**

```rust
fn is_non_python_example(code: &str) -> bool {
    // JavaScript patterns
    if code.contains("const ") || code.contains("let ") 
        || code.contains("function(") || code.contains("=>") {
        return true;
    }
    
    // PHP patterns
    if code.contains("<?php") || code.starts_with("<?") {
        return true;
    }
    
    // C# patterns
    if code.contains("using System") || code.contains("namespace ") {
        return true;
    }
    
    // R patterns
    if code.contains("library(") || code.contains("<-") {
        return true;
    }
    
    false
}
```


#### Table Extraction

**Parameter Tables:**

```rust
// Extract table structure
fn convert_table_to_markdown(table: &ElementRef) -> String {
    let mut markdown = String::new();
    
    // Extract headers
    let headers: Vec<String> = table.select(&th_selector)
        .map(|th| th.text().collect::<String>().trim())
        .collect();
    
    // Build header row
    markdown.push_str(&format!("| {} |\n", headers.join(" | ")));
    markdown.push_str(&format!("|{}|\n", vec!["---"; headers.len()].join("|")));
    
    // Extract data rows
    for row in table.select(&tr_selector) {
        let cells: Vec<String> = row.select(&td_selector)
            .map(|td| {
                // Clean cell content
                td.text().collect::<String>()
                    .trim()
                    .replace("|", "\\|")  // Escape pipes
                    .replace("\n", " ")   // Single line
            })
            .collect();
        
        if !cells.is_empty() {
            markdown.push_str(&format!("| {} |\n", cells.join(" | ")));
        }
    }
    
    markdown
}
```


### 3.4 Navigation and Boundary Detection

#### Content Area Identification Algorithm

**Phase 1: Main Content Locator**

```rust
fn identify_main_content(document: &Html) -> Result<ElementRef, ExtractionError> {
    // Strategy 1: Explicit selectors
    let explicit_selectors = vec![
        "main",
        "article", 
        "[role='main']",
        ".main-content",
        "#main-content",
    ];
    
    for selector_str in explicit_selectors {
        if let Ok(selector) = Selector::parse(selector_str) {
            if let Some(element) = document.select(&selector).next() {
                return Ok(element);
            }
        }
    }
    
    // Strategy 2: Heuristic-based (largest content area)
    let content_candidates = document.select(
        &Selector::parse(".container > .row > div").unwrap()
    );
    
    let main_content = content_candidates
        .max_by_key(|el| calculate_content_score(el))
        .ok_or(ExtractionError::NoMainContentFound)?;
    
    Ok(main_content)
}

fn calculate_content_score(element: &ElementRef) -> usize {
    let text_length = element.text().collect::<String>().len();
    let heading_count = element.select(&Selector::parse("h2, h3").unwrap()).count();
    let code_block_count = element.select(&Selector::parse("pre, code").unwrap()).count();
    
    // Score formula: prioritize text + structure
    text_length + (heading_count * 100) + (code_block_count * 50)
}
```

**Phase 2: Navigation Boundary Detection**

```rust
fn remove_navigation_elements(element: &mut ElementRef) {
    let navigation_selectors = vec![
        "nav",
        ".sidebar",
        "#sidebar", 
        ".navigation",
        ".nav-menu",
        ".toc",
        ".table-of-contents",
        "[role='navigation']",
        "aside",
    ];
    
    for selector_str in navigation_selectors {
        if let Ok(selector) = Selector::parse(selector_str) {
            for nav_element in element.select(&selector) {
                // Mark for removal
                nav_element.remove();
            }
        }
    }
}
```

**Phase 3: Footer and Promotional Content Removal**

```rust
fn remove_promotional_content(html: &mut Html) {
    // Text-based removal
    let promotional_phrases = vec![
        "Looking for more programming languages?",
        "Want to integrate with LLMs?",
        "Claim your free API key",
        "Premium membership",
        "Subscribe to our newsletter",
        "Follow us on",
    ];
    
    // Element-based removal
    let promotional_selectors = vec![
        "footer",
        ".footer",
        ".banner",
        ".cta",
        ".call-to-action",
        ".newsletter",
        ".social-links",
    ];
    
    // Remove by text content
    for element in html.select(&Selector::parse("div, section, p").unwrap()) {
        let text = element.text().collect::<String>();
        for phrase in &promotional_phrases {
            if text.contains(phrase) {
                element.remove();
                break;
            }
        }
    }
    
    // Remove by selector
    for selector_str in promotional_selectors {
        if let Ok(selector) = Selector::parse(selector_str) {
            for element in html.select(&selector) {
                element.remove();
            }
        }
    }
}
```


***

## 4. Technical Architecture

### 4.1 System Design Principles

**Core Tenets:**

1. **Type Safety:** Invalid states are unrepresentable at compile time[^1]
2. **Explicit Error Handling:** No panics; all errors typed and recoverable
3. **Separation of Concerns:** Domain logic independent of I/O
4. **Testability:** Pure functions with dependency injection
5. **Performance:** Zero-cost abstractions; efficient memory usage
6. **Observability:** Structured logging with correlation IDs

### 4.2 Architecture Pattern: Hexagonal (Ports \& Adapters)

```
┌─────────────────────────────────────────────────────────────┐
│                     ADAPTERS (Outside)                       │
│  ┌──────────┐    ┌──────────┐    ┌──────────┐             │
│  │   CLI    │    │  Network │    │   File   │             │
│  │ Interface│    │  Client  │    │  Writer  │             │
│  └────┬─────┘    └────┬─────┘    └────┬─────┘             │
│       │               │               │                     │
└───────┼───────────────┼───────────────┼─────────────────────┘
        │               │               │
        ▼               ▼               ▼
┌─────────────────────────────────────────────────────────────┐
│                        PORTS (Interface)                     │
│    InputPort       FetchPort       OutputPort               │
└─────────────────────────────────────────────────────────────┘
        │               │               │
        ▼               ▼               ▼
┌─────────────────────────────────────────────────────────────┐
│                   CORE DOMAIN (Pure Logic)                   │
│                                                              │
│  ┌──────────────┐   ┌──────────────┐   ┌──────────────┐   │
│  │    Parser    │──▶│ Transformer  │──▶│   Renderer   │   │
│  │              │   │              │   │              │   │
│  │ HTML → AST   │   │ AST → Model  │   │ Model → MD   │   │
│  └──────────────┘   └──────────────┘   └──────────────┘   │
│                                                              │
│  ┌──────────────────────────────────────────────────────┐  │
│  │              Domain Models (Types)                   │  │
│  │  • DocumentStructure                                 │  │
│  │  • ApiEndpoint                                       │  │
│  │  • ParameterTable                                    │  │
│  │  • CodeExample                                       │  │
│  └──────────────────────────────────────────────────────┘  │
└─────────────────────────────────────────────────────────────┘
```


### 4.3 Data Flow \& Type System

#### Typestate Pattern Implementation

**Phase 1: URL Validation**

```rust
// Raw input (untrusted)
struct RawUrl(String);

// Validated URL (trusted)
struct ValidatedUrl {
    inner: url::Url,
}

impl TryFrom<RawUrl> for ValidatedUrl {
    type Error = UrlParseError;
    
    fn try_from(raw: RawUrl) -> Result<Self, Self::Error> {
        let url = url::Url::parse(&raw.0)
            .map_err(|e| UrlParseError::InvalidFormat(e))?;
        
        // Validate scheme
        if url.scheme() != "https" && url.scheme() != "http" {
            return Err(UrlParseError::InvalidScheme);
        }
        
        Ok(ValidatedUrl { inner: url })
    }
}
```

**Phase 2: HTML Fetching**

```rust
// Raw HTML (unprocessed)
#[derive(Debug)]
struct RawHtml {
    content: String,
    source_url: ValidatedUrl,
    fetched_at: DateTime<Utc>,
}

impl RawHtml {
    async fn fetch(url: ValidatedUrl) -> Result<Self, NetworkError> {
        let response = reqwest::get(url.inner.as_str())
            .await
            .map_err(NetworkError::RequestFailed)?;
        
        if !response.status().is_success() {
            return Err(NetworkError::HttpError(response.status()));
        }
        
        let content = response.text()
            .await
            .map_err(NetworkError::BodyReadFailed)?;
        
        Ok(RawHtml {
            content,
            source_url: url,
            fetched_at: Utc::now(),
        })
    }
}
```

**Phase 3: HTML Parsing**

```rust
// Parsed DOM (structured)
struct ParsedDocument {
    dom: scraper::Html,
    metadata: DocumentMetadata,
}

struct DocumentMetadata {
    title: String,
    source_url: String,
    parsed_at: DateTime<Utc>,
}

impl TryFrom<RawHtml> for ParsedDocument {
    type Error = ParseError;
    
    fn try_from(raw: RawHtml) -> Result<Self, Self::Error> {
        let dom = scraper::Html::parse_document(&raw.content);
        
        // Extract metadata
        let title = extract_title(&dom)
            .ok_or(ParseError::MissingTitle)?;
        
        Ok(ParsedDocument {
            dom,
            metadata: DocumentMetadata {
                title,
                source_url: raw.source_url.inner.to_string(),
                parsed_at: Utc::now(),
            },
        })
    }
}
```

**Phase 4: Content Extraction**

```rust
// Extracted structure (domain model)
#[derive(Debug, Clone, Serialize)]
struct DocumentStructure {
    metadata: DocumentMetadata,
    categories: Vec<ApiCategory>,
}

#[derive(Debug, Clone, Serialize)]
struct ApiCategory {
    name: String,
    description: Option<String>,
    endpoints: Vec<ApiEndpoint>,
}

#[derive(Debug, Clone, Serialize)]
struct ApiEndpoint {
    function_name: String,
    description: String,
    required_params: Vec<Parameter>,
    optional_params: Vec<Parameter>,
    request_pattern: String,
    python_example: Option<CodeExample>,
    premium_only: bool,
}

#[derive(Debug, Clone, Serialize)]
struct Parameter {
    name: String,
    value_type: String,
    default: Option<String>,
    options: Vec<String>,
    description: String,
}

#[derive(Debug, Clone, Serialize)]
struct CodeExample {
    language: String,  // Always "python" in this scope
    code: String,
}

impl TryFrom<ParsedDocument> for DocumentStructure {
    type Error = ExtractionError;
    
    fn try_from(doc: ParsedDocument) -> Result<Self, Self::Error> {
        let extractor = ContentExtractor::new(&doc.dom);
        
        // Extract main content area
        let main_content = extractor.identify_main_content()?;
        
        // Remove navigation and promotional content
        let cleaned_content = extractor.clean_content(main_content)?;
        
        // Parse into structured model
        let categories = extractor.extract_categories(cleaned_content)?;
        
        Ok(DocumentStructure {
            metadata: doc.metadata,
            categories,
        })
    }
}
```

**Phase 5: Markdown Generation**

```rust
// Final output (validated markdown)
struct MarkdownDocument {
    content: String,
    metadata: OutputMetadata,
}

struct OutputMetadata {
    source_title: String,
    source_url: String,
    extracted_at: DateTime<Utc>,
    endpoint_count: usize,
    category_count: usize,
    output_size_bytes: usize,
}

impl From<DocumentStructure> for MarkdownDocument {
    fn from(structure: DocumentStructure) -> Self {
        let renderer = MarkdownRenderer::new();
        let content = renderer.render(&structure);
        
        MarkdownDocument {
            metadata: OutputMetadata {
                source_title: structure.metadata.title,
                source_url: structure.metadata.source_url,
                extracted_at: Utc::now(),
                endpoint_count: structure.categories.iter()
                    .map(|c| c.endpoints.len())
                    .sum(),
                category_count: structure.categories.len(),
                output_size_bytes: content.len(),
            },
            content,
        }
    }
}
```


### 4.4 Module Structure

```
alphavantage-doc-extractor/
├── src/
│   ├── main.rs                 # CLI entry point
│   ├── lib.rs                  # Public library interface
│   │
│   ├── adapters/               # External world interfaces
│   │   ├── mod.rs
│   │   ├── cli.rs              # Command-line argument parsing
│   │   ├── http_client.rs     # Network fetching
│   │   └── file_writer.rs     # File system operations
│   │
│   ├── domain/                 # Core business logic (pure)
│   │   ├── mod.rs
│   │   ├── models.rs           # Domain types
│   │   ├── parser.rs           # HTML → AST
│   │   ├── extractor.rs        # AST → Domain Model
│   │   ├── transformer.rs      # Content cleaning & transformation
│   │   └── renderer.rs         # Domain Model → Markdown
│   │
│   ├── ports/                  # Interface definitions
│   │   ├── mod.rs
│   │   ├── fetcher.rs          # Fetch port trait
│   │   └── writer.rs           # Output port trait
│   │
│   └── utils/                  # Shared utilities
│       ├── mod.rs
│       ├── error.rs            # Error types
│       ├── logging.rs          # Structured logging setup
│       └── validation.rs       # Input validation helpers
│
├── tests/
│   ├── integration/
│   │   ├── end_to_end.rs       # Full pipeline tests
│   │   └── snapshot.rs         # Output validation
│   │
│   └── fixtures/
│       ├── sample_page.html    # Test HTML samples
│       └── expected_output.md  # Expected results
│
├── benches/
│   └── extraction_bench.rs     # Performance benchmarks
│
├── docs/
│   ├── architecture.md         # This document
│   ├── usage.md                # User guide
│   └── development.md          # Contributor guide
│
├── Cargo.toml
├── README.md
├── LICENSE
└── .github/
    └── workflows/
        ├── ci.yml              # Continuous integration
        └── release.yml         # Release automation
```


### 4.5 Dependency Management

**Cargo.toml:**

```toml
[package]
name = "alphavantage-doc-extractor"
version = "2.0.0"
edition = "2021"
rust-version = "1.75"
authors = ["Your Name <email@example.com>"]
license = "MIT"
description = "Extract Alpha Vantage API documentation to clean, LLM-optimized Markdown"
repository = "https://github.com/yourusername/alphavantage-doc-extractor"
keywords = ["alphavantage", "documentation", "markdown", "api", "scraper"]
categories = ["command-line-utilities", "web-programming"]

[dependencies]
# CLI & Configuration
clap = { version = "4.4", features = ["derive", "env", "wrap_help"] }
serde = { version = "1.0", features = ["derive"] }
serde_json = "1.0"

# HTTP Client
reqwest = { version = "0.11", features = ["rustls-tls"], default-features = false }

# HTML Parsing
scraper = "0.18"
html5ever = "0.26"

# Error Handling
thiserror = "1.0"
anyhow = "1.0"

# Async Runtime
tokio = { version = "1.35", features = ["rt-multi-thread", "macros"] }

# Logging
tracing = "0.1"
tracing-subscriber = { version = "0.3", features = ["json", "env-filter", "fmt"] }

# Utilities
chrono = { version = "0.4", features = ["serde"] }
uuid = { version = "1.6", features = ["v4", "serde"] }
regex = "1.10"

[dev-dependencies]
# Testing
mockito = "1.2"
assert_cmd = "2.0"
predicates = "3.0"
tempfile = "3.8"
insta = "1.34"  # Snapshot testing

# Benchmarking
criterion = "0.5"

[profile.release]
opt-level = 3
lto = true
codegen-units = 1
strip = true

[profile.dev]
opt-level = 0

[profile.test]
opt-level = 1
```


***

## 5. Implementation Specifications

### 5.1 CLI Interface

**Command Structure:**

```bash
alphavantage-doc-extractor [OPTIONS]
```

**Options:**

```
OPTIONS:
    -u, --url <URL>
            Source URL to extract documentation from
            [default: https://www.alphavantage.co/documentation/]
    
    -o, --output <FILE>
            Output file path for generated Markdown
            [default: alpha_vantage_api.md]
    
    -f, --format <FORMAT>
            Output format
            [default: markdown]
            [possible values: markdown, json]
    
    --log-level <LEVEL>
            Logging verbosity
            [default: info]
            [possible values: trace, debug, info, warn, error]
    
    --log-format <FORMAT>
            Log output format
            [default: pretty]
            [possible values: pretty, json]
    
    --validate
            Validate output Markdown syntax after generation
    
    --dry-run
            Parse and extract without writing output file
    
    -h, --help
            Print help information
    
    -V, --version
            Print version information
```

**Example Usage:**

```bash
# Basic extraction
alphavantage-doc-extractor

# Custom output path
alphavantage-doc-extractor -o ./docs/alphavantage-api.md

# Debug mode with JSON logs
alphavantage-doc-extractor --log-level debug --log-format json

# Validate output
alphavantage-doc-extractor --validate

# Dry run (parse only, no output)
alphavantage-doc-extractor --dry-run
```


### 5.2 Output Format Specification

**Markdown Template:**

```markdown
---
title: Alpha Vantage API Documentation
version: 2.0.0
source_url: https://www.alphavantage.co/documentation/
extracted_at: 2026-01-05T13:03:00Z
extractor_version: 2.0.0
endpoint_count: 73
category_count: 8
language_examples: python
optimization: llm-ready
---

# Alpha Vantage API Documentation

> **📋 Offline Documentation**  
> This is a structured, LLM-optimized snapshot of the Alpha Vantage API documentation.  
> **Source:** [Alpha Vantage Documentation](https://www.alphavantage.co/documentation/)  
> **Generated:** January 5, 2026

## Table of Contents

- [Time Series Stock Data APIs](#time-series-stock-data-apis)
  - [TIME_SERIES_INTRADAY](#time_series_intraday)
  - [TIME_SERIES_DAILY](#time_series_daily)
  - [TIME_SERIES_DAILY_ADJUSTED](#time_series_daily_adjusted)
  - [TIME_SERIES_WEEKLY](#time_series_weekly)
  - [TIME_SERIES_WEEKLY_ADJUSTED](#time_series_weekly_adjusted)
  - [TIME_SERIES_MONTHLY](#time_series_monthly)
  - [TIME_SERIES_MONTHLY_ADJUSTED](#time_series_monthly_adjusted)
  - [GLOBAL_QUOTE](#global_quote)
  - [SYMBOL_SEARCH](#symbol_search)
- [US Options Data APIs](#us-options-data-apis)
  - [REALTIME_OPTIONS](#realtime_options)
  - [HISTORICAL_OPTIONS](#historical_options)
- [Fundamental Data](#fundamental-data)
- [Forex (FX)](#forex-fx)
- [Cryptocurrencies](#cryptocurrencies)
- [Commodities](#commodities)
- [Economic Indicators](#economic-indicators)
- [Technical Indicators](#technical-indicators)

---

## Time Series Stock Data APIs

<!-- CATEGORY: Time Series Stock Data -->
<!-- CATEGORY_ID: time-series -->

### TIME_SERIES_INTRADAY

<!-- ENDPOINT: TIME_SERIES_INTRADAY -->
<!-- PREMIUM: false -->

Returns intraday time series data (OHLCV) for equity symbols with up to 20 years of historical depth. Supports pre-market and post-market hours for comprehensive market coverage.

#### Parameters

##### Required Parameters

| Parameter | Value | Description |
|-----------|-------|-------------|
| `function` | `TIME_SERIES_INTRADAY` | API function identifier |
| `symbol` | Stock ticker (e.g., `IBM`) | Equity symbol to query |
| `interval` | `1min`, `5min`, `15min`, `30min`, `60min` | Time interval between data points |
| `apikey` | Your API key | Authentication credential |

##### Optional Parameters

| Parameter | Default | Options | Description |
|-----------|---------|---------|-------------|
| `adjusted` | `true` | `true`, `false` | Adjust prices for splits and dividends |
| `extended_hours` | `true` | `true`, `false` | Include pre-market and after-hours data |
| `month` | Current month | `YYYY-MM` format | Query specific historical month |
| `outputsize` | `compact` | `compact`, `full` | Data range: 100 latest points vs 30 days full |
| `datatype` | `json` | `json`, `csv` | Response format |

#### Request Format

```bash
https://www.alphavantage.co/query?function=TIME_SERIES_INTRADAY&symbol=IBM&interval=5min&apikey=YOUR_API_KEY
```


#### Python Implementation

```python
import requests

url = 'https://www.alphavantage.co/query'
params = {
    'function': 'TIME_SERIES_INTRADAY',
    'symbol': 'IBM',
    'interval': '5min',
    'apikey': 'YOUR_API_KEY'
}

response = requests.get(url, params=params)
data = response.json()

# Access time series data
time_series = data['Time Series (5min)']
for timestamp, values in time_series.items():
    print(f"{timestamp}: Open={values['1. open']}, Close={values['4. close']}")
```


---

### TIME_SERIES_DAILY

<!-- ENDPOINT: TIME_SERIES_DAILY -->
<!-- PREMIUM: false -->
Returns daily time series data (open, high, low, close, volume) for equity symbols with up to 20 years of historical data.

#### Parameters

##### Required Parameters

| Parameter | Value | Description |
| :-- | :-- | :-- |
| `function` | `TIME_SERIES_DAILY` | API function identifier |
| `symbol` | Stock ticker (e.g., `MSFT`) | Equity symbol to query |
| `apikey` | Your API key | Authentication credential |

##### Optional Parameters

| Parameter | Default | Options | Description |
| :-- | :-- | :-- | :-- |
| `outputsize` | `compact` | `compact`, `full` | 100 latest days vs 20 years full history |
| `datatype` | `json` | `json`, `csv` | Response format |

#### Request Format

```bash
https://www.alphavantage.co/query?function=TIME_SERIES_DAILY&symbol=MSFT&apikey=YOUR_API_KEY
```


#### Python Implementation

```python
import requests

url = 'https://www.alphavantage.co/query'
params = {
    'function': 'TIME_SERIES_DAILY',
    'symbol': 'MSFT',
    'outputsize': 'full',
    'apikey': 'YOUR_API_KEY'
}

response = requests.get(url, params=params)
data = response.json()

# Process daily time series
daily_data = data['Time Series (Daily)']
for date, values in daily_data.items():
    print(f"{date}: Close={values['4. close']}, Volume={values['5. volume']}")
```


---

[... continues for all 73 endpoints ...]

```

**Key Features:**
- YAML frontmatter with extraction metadata
- Hierarchical heading structure (H1 → H2 → H3 → H4 → H5)
- Semantic HTML comments for LLM parsing
- Consistent parameter table format
- Clean Python code examples only
- Request URLs with placeholders
- No promotional content
- No navigation artifacts
- Optimized for embedding generation

### 5.3 Error Handling Strategy

**Error Type Hierarchy:**
```rust
// src/utils/error.rs

use thiserror::Error;

#[derive(Error, Debug)]
pub enum ExtractionError {
    #[error("Network error: {0}")]
    Network(#[from] NetworkError),
    
    #[error("Parsing error: {0}")]
    Parse(#[from] ParseError),
    
    #[error("Content extraction error: {0}")]
    ContentExtraction(#[from] ContentExtractionError),
    
    #[error("Rendering error: {0}")]
    Render(#[from] RenderError),
    
    #[error("I/O error: {0}")]
    Io(#[from] std::io::Error),
    
    #[error("Configuration error: {0}")]
    Config(String),
}

#[derive(Error, Debug)]
pub enum NetworkError {
    #[error("Failed to send request: {0}")]
    RequestFailed(#[from] reqwest::Error),
    
    #[error("HTTP error: {0}")]
    HttpError(reqwest::StatusCode),
    
    #[error("Failed to read response body: {0}")]
    BodyReadFailed(reqwest::Error),
    
    #[error("Request timeout after {0}s")]
    Timeout(u64),
}

#[derive(Error, Debug)]
pub enum ParseError {
    #[error("Invalid HTML structure")]
    InvalidHtml,
    
    #[error("Missing document title")]
    MissingTitle,
    
    #[error("Failed to parse HTML: {0}")]
    HtmlParseFailed(String),
}

#[derive(Error, Debug)]
pub enum ContentExtractionError {
    #[error("No main content area found")]
    NoMainContentFound,
    
    #[error("Failed to extract category: {0}")]
    CategoryExtractionFailed(String),
    
    #[error("Failed to extract endpoint: {0}")]
    EndpointExtractionFailed(String),
    
    #[error("Invalid parameter table structure")]
    InvalidParameterTable,
    
    #[error("No content extracted")]
    EmptyContent,
}

#[derive(Error, Debug)]
pub enum RenderError {
    #[error("Failed to render markdown: {0}")]
    MarkdownGenerationFailed(String),
    
    #[error("Invalid output path: {0}")]
    InvalidOutputPath(String),
}
```

**Error Recovery Strategy:**

```rust
// Graceful degradation example
fn extract_endpoint(element: &ElementRef) -> Result<ApiEndpoint, ContentExtractionError> {
    // Try to extract all components
    let function_name = extract_function_name(element)?;
    let description = extract_description(element)?;
    
    // Non-critical components: use defaults on failure
    let required_params = extract_required_params(element)
        .unwrap_or_else(|e| {
            tracing::warn!("Failed to extract required params: {}", e);
            Vec::new()
        });
    
    let optional_params = extract_optional_params(element)
        .unwrap_or_else(|e| {
            tracing::warn!("Failed to extract optional params: {}", e);
            Vec::new()
        });
    
    let python_example = extract_python_example(element)
        .ok();  // Option: None if extraction fails
    
    Ok(ApiEndpoint {
        function_name,
        description,
        required_params,
        optional_params,
        python_example,
        request_pattern: generate_request_pattern(&function_name),
        premium_only: false,
    })
}
```


### 5.4 Logging \& Observability

**Structured Logging Setup:**

```rust
// src/utils/logging.rs

use tracing::{info, warn, error, debug};
use tracing_subscriber::{layer::SubscriberExt, util::SubscriberInitExt, EnvFilter};
use uuid::Uuid;

pub fn init_logging(log_level: &str, log_format: &str) -> anyhow::Result<String> {
    let request_id = Uuid::new_v4().to_string();
    
    // Create env filter
    let env_filter = EnvFilter::try_from_default_env()
        .unwrap_or_else(|_| EnvFilter::new(log_level));
    
    // Setup subscriber based on format
    match log_format {
        "json" => {
            tracing_subscriber::registry()
                .with(env_filter)
                .with(tracing_subscriber::fmt::layer().json())
                .init();
        }
        _ => {
            tracing_subscriber::registry()
                .with(env_filter)
                .with(tracing_subscriber::fmt::layer().pretty())
                .init();
        }
    }
    
    info!(
        request_id = %request_id,
        "Logging initialized"
    );
    
    Ok(request_id)
}
```

**Instrumentation Example:**

```rust
use tracing::{instrument, info, warn};

#[instrument(skip(html))]
async fn extract_documentation(html: RawHtml) -> Result<DocumentStructure, ExtractionError> {
    info!("Starting documentation extraction");
    
    let parsed = ParsedDocument::try_from(html)?;
    info!(title = %parsed.metadata.title, "Document parsed successfully");
    
    let extractor = ContentExtractor::new(&parsed.dom);
    
    let main_content = extractor.identify_main_content()?;
    info!("Main content area identified");
    
    let cleaned = extractor.clean_content(main_content)?;
    info!("Content cleaned and sanitized");
    
    let structure = extractor.extract_structure(cleaned)?;
    info!(
        categories = structure.categories.len(),
        endpoints = structure.total_endpoints(),
        "Content extraction complete"
    );
    
    Ok(structure)
}
```


***

## 6. Quality Assurance

### 6.1 Testing Strategy

#### Unit Tests

```rust
#[cfg(test)]
mod tests {
    use super::*;
    
    #[test]
    fn test_navigation_removal() {
        let html = r#"
            <div class="main">
                <nav>Skip this</nav>
                <h2>Keep this</h2>
                <p>Content</p>
            </div>
        "#;
        
        let doc = Html::parse_document(html);
        let cleaned = remove_navigation(&doc);
        
        assert!(!cleaned.contains("Skip this"));
        assert!(cleaned.contains("Keep this"));
    }
    
    #[test]
    fn test_python_example_extraction() {
        let html = r#"
            <pre><code>import requests
r = requests.get('https://api.example.com')
print(r.json())</code></pre>
        "#;
        
        let doc = Html::parse_document(html);
        let example = extract_python_example(&doc).unwrap();
        
        assert_eq!(example.language, "python");
        assert!(example.code.contains("import requests"));
    }
    
    #[test]
    fn test_non_python_filtering() {
        let javascript_code = "const data = fetch('https://api.example.com');";
        assert!(is_non_python_example(javascript_code));
        
        let python_code = "import requests\ndata = requests.get()";
        assert!(!is_non_python_example(python_code));
    }
}
```


#### Integration Tests

```rust
// tests/integration/end_to_end.rs

#[tokio::test]
async fn test_full_extraction_pipeline() {
    // Load fixture
    let html = std::fs::read_to_string("tests/fixtures/sample_page.html")
        .expect("Failed to load fixture");
    
    let raw_html = RawHtml {
        content: html,
        source_url: ValidatedUrl::try_from(
            RawUrl("https://example.com".to_string())
        ).unwrap(),
        fetched_at: Utc::now(),
    };
    
    // Run full pipeline
    let structure = extract_documentation(raw_html).await
        .expect("Extraction failed");
    
    let markdown = MarkdownDocument::from(structure);
    
    // Assertions
    assert!(!markdown.content.is_empty());
    assert!(!markdown.content.contains("<div"));
    assert!(!markdown.content.contains("Looking for more programming languages"));
    assert!(markdown.content.contains("```python"));
    assert!(markdown.metadata.endpoint_count > 0);
}
```


#### Snapshot Tests

```rust
// tests/integration/snapshot.rs

use insta::assert_snapshot;

#[test]
fn test_markdown_output_format() {
    let structure = create_sample_structure();
    let markdown = MarkdownDocument::from(structure);
    
    // Compare against saved snapshot
    assert_snapshot!(markdown.content);
}
```


### 6.2 Validation

**Markdown Syntax Validation:**

```rust
fn validate_markdown_syntax(content: &str) -> Result<(), ValidationError> {
    let mut issues = Vec::new();
    
    // Check for HTML artifacts
    if content.contains("<div") || content.contains("<script") {
        issues.push("HTML artifacts found in output");
    }
    
    // Check for unclosed code blocks
    let code_fence_count = content.matches("```").count();
    if code_fence_count % 2 != 0 {
        issues.push("Unclosed code fence detected");
    }
    
    // Check table formatting
    for (i, line) in content.lines().enumerate() {
        if line.starts_with("|") {
            let pipe_count = line.chars().filter(|&c| c == '|').count();
            if pipe_count < 2 {
                issues.push(&format!("Invalid table row at line {}", i + 1));
            }
        }
    }
    
    // Check heading hierarchy
    let mut prev_level = 0;
    for line in content.lines() {
        if let Some(level) = get_heading_level(line) {
            if level > prev_level + 1 {
                issues.push(&format!("Heading hierarchy skip detected: {} to {}", prev_level, level));
            }
            prev_level = level;
        }
    }
    
    if issues.is_empty() {
        Ok(())
    } else {
        Err(ValidationError::SyntaxIssues(issues))
    }
}
```


***

## 7. Performance Requirements

### 7.1 Performance Targets

| Metric | Target | Measurement Method |
| :-- | :-- | :-- |
| Extraction Time | < 30 seconds | End-to-end execution |
| Peak Memory | < 100 MB | Runtime profiling |
| Binary Size | < 10 MB | Stripped release build |
| CPU Usage | < 50% single core | System monitoring |
| Output Generation | < 1 second | Markdown rendering phase |

### 7.2 Optimization Strategies

**Memory Efficiency:**

- Stream HTML parsing (no full DOM in memory)
- Incremental markdown generation
- Lazy evaluation of content transformations
- Drop intermediate structures after processing

**CPU Efficiency:**

- Compile-time regex optimization
- Minimize string allocations (use `Cow<str>`)
- Efficient DOM traversal (CSS selectors over manual iteration)
- Single-pass transformations where possible

**Benchmarking:**

```rust
// benches/extraction_bench.rs

use criterion::{black_box, criterion_group, criterion_main, Criterion};

fn benchmark_html_parsing(c: &mut Criterion) {
    let html = std::fs::read_to_string("tests/fixtures/sample_page.html").unwrap();
    
    c.bench_function("parse_html", |b| {
        b.iter(|| {
            let doc = Html::parse_document(black_box(&html));
            black_box(doc);
        });
    });
}

fn benchmark_content_extraction(c: &mut Criterion) {
    let html = std::fs::read_to_string("tests/fixtures/sample_page.html").unwrap();
    let doc = Html::parse_document(&html);
    
    c.bench_function("extract_content", |b| {
        b.iter(|| {
            let extractor = ContentExtractor::new(&doc);
            let structure = extractor.extract_structure(black_box(&doc)).unwrap();
            black_box(structure);
        });
    });
}

criterion_group!(benches, benchmark_html_parsing, benchmark_content_extraction);
criterion_main!(benches);
```


***

## 8. Development Workflow

### 8.1 Setup

```bash
# Clone repository
git clone https://github.com/yourusername/alphavantage-doc-extractor.git
cd alphavantage-doc-extractor

# Build
cargo build --release

# Run tests
cargo test

# Run benchmarks
cargo bench

# Check code quality
cargo clippy -- -D warnings

# Format code
cargo fmt
```


### 8.2 CI/CD Pipeline

**GitHub Actions (.github/workflows/ci.yml):**

```yaml
name: CI

on: [push, pull_request]

jobs:
  test:
    runs-on: ubuntu-latest
    steps:
      - uses: actions/checkout@v4
      - uses: actions-rs/toolchain@v1
        with:
          toolchain: stable
      
      - name: Run tests
        run: cargo test --all-features
      
      - name: Run clippy
        run: cargo clippy -- -D warnings
      
      - name: Check formatting
        run: cargo fmt -- --check
  
  build:
    runs-on: ${{ matrix.os }}
    strategy:
      matrix:
        os: [ubuntu-latest, macos-latest, windows-latest]
    steps:
      - uses: actions/checkout@v4
      - uses: actions-rs/toolchain@v1
        with:
          toolchain: stable
      
      - name: Build release binary
        run: cargo build --release
      
      - name: Upload artifact
        uses: actions/upload-artifact@v3
        with:
          name: binary-${{ matrix.os }}
          path: target/release/alphavantage-doc-extractor*
```


***

## 9. Documentation

### 9.1 README.md Structure

```markdown
# Alpha Vantage Documentation Extractor

[![CI](https://github.com/user/repo/actions/workflows/ci.yml/badge.svg)](...)
[![License: MIT](https://img.shields.io/badge/License-MIT-yellow.svg)](...)

Extract Alpha Vantage API documentation to clean, LLM-optimized Markdown.

## Features
- ✅ Clean HTML-to-Markdown conversion
- ✅ LLM-optimized structure
- ✅ Python code examples
- ✅ Zero noise output

## Installation
[Details]

## Usage
[Examples]

## Development
[Contributing guide]

## License
MIT
```


***

## 10. Project Timeline

| Phase | Duration | Deliverables |
| :-- | :-- | :-- |
| **Phase 1: Core** | Week 1 | HTML fetching, parsing, basic navigation removal |
| **Phase 2: Extraction** | Week 2 | Content extraction, table parsing, Python filtering |
| **Phase 3: Rendering** | Week 3 | Markdown generation, LLM optimization |
| **Phase 4: Quality** | Week 4 | Testing, validation, documentation |
| **Phase 5: Release** | Week 5 | CI/CD, GitHub release, community docs |


***

## Appendix A: LLM Optimization Checklist

- ✅ Consistent heading hierarchy (H1 → H2 → H3 → H4)
- ✅ Semantic HTML comments for metadata
- ✅ Self-contained endpoint sections
- ✅ Standardized parameter tables
- ✅ Clean code examples (Python only)
- ✅ No promotional content
- ✅ No navigation artifacts
- ✅ Factual, concise descriptions
- ✅ Token-efficient formatting
- ✅ Proper markdown syntax throughout

***

## Appendix B: Example Output Excerpt

See Section 5.2 for complete output format specification with full example.

***

**Document Status:** ✅ Ready for Implementation
**Approval:** Pending
**Next Steps:** Begin Phase 1 development

<div align="center">⁂</div>
