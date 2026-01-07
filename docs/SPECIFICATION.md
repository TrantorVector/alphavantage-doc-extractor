# Technical Specification

## Overview

The Alpha Vantage Documentation Extractor is a command-line tool that extracts API documentation from Alpha Vantage's web documentation and converts it into LLM-optimized Markdown format. The tool follows hexagonal architecture principles and provides comprehensive validation and error handling.

## Input Specification

### Source Input
- **Type**: URL string
- **Format**: Valid HTTP/HTTPS URL
- **Example**: `https://www.alphavantage.co/documentation`
- **Validation**: Must start with `http://` or `https://`, contain valid domain structure

### Command-Line Interface

The tool accepts the following command-line arguments:

```
USAGE:
    alphavantage-doc-extractor [OPTIONS] --url <URL>

OPTIONS:
    -u, --url <URL>              Source URL to extract documentation from
    -o, --output <OUTPUT>        Output file path [default: output.md]
        --log-level <LOG_LEVEL>  Logging level [default: info]
                                 [possible values: error, warn, info, debug, trace]
        --log-format <LOG_FORMAT> Log output format [default: pretty]
                                 [possible values: pretty, json]
        --no-validation          Skip output validation
        --backup                 Create backup of existing output file
    -h, --help                   Print help information
    -V, --version                Print version information
```

### Environment Variables

- `RUST_LOG`: Override default log level (same values as `--log-level`)

## Processing Pipeline

### Phase 1: Input Validation
1. Parse command-line arguments using clap
2. Validate URL format and structure
3. Validate output path permissions and format
4. Initialize logging system with correlation ID

### Phase 2: HTTP Fetching
1. Convert validated URL to HTTP request
2. Execute HTTP GET request with timeout
3. Handle network errors and retries
4. Validate response content type and size
5. Return raw HTML content

### Phase 3: HTML Cleaning
1. Parse HTML into DOM structure
2. Remove promotional content and advertisements
3. Clean anchor links and navigation elements
4. Remove empty elements and technical artifacts
5. Sanitize HTML entities and encoding issues
6. Return cleaned HTML string

### Phase 4: Content Identification
1. Parse cleaned HTML into DOM
2. Identify main content area using heuristics:
   - Bootstrap layout detection
   - Main tag presence
   - Content score calculation based on:
     - Header density
     - Link concentration
     - Text-to-HTML ratio
3. Extract main content element reference

### Phase 5: Content Extraction
1. Traverse DOM from main content element
2. Extract category sections (h2 elements)
3. For each category:
   - Extract category name and description
   - Find associated endpoints (h3 elements)
   - For each endpoint:
     - Extract function name and description
     - Detect premium indicators
     - Extract parameter tables
     - Extract code examples
     - Extract request patterns

### Phase 6: Markdown Rendering
1. Generate document header with metadata
2. Render table of contents with GitHub-style anchors
3. For each category:
   - Render category header and description
   - Render all endpoints in category
4. Apply LLM optimizations:
   - Add YAML frontmatter
   - Insert semantic markers
   - Optimize whitespace for token efficiency

### Phase 7: Output Validation
1. Check for HTML artifacts (<script>, <div>, etc.)
2. Validate heading hierarchy (H1 → H2 → H3)
3. Ensure balanced code fences
4. Verify table formatting compliance
5. Check minimum content requirements
6. Detect duplicate sections
7. Validate required sections presence

### Phase 8: File Output
1. Validate output path and permissions
2. Create backup if requested
3. Write content to file with UTF-8 encoding
4. Report file size and write statistics

## Output Format Specification

### Overall Structure

```markdown
---
format_version: "2.0"
source_url: "https://www.alphavantage.co/documentation"
extraction_date: "2024-01-07T12:00:00Z"
categories: 5
endpoints: 25
---

# Alpha Vantage API Documentation

<!-- LLM Semantic Markers -->
<!-- CATEGORY_START: Stock Time Series Data -->

## Table of Contents

- [Stock Time Series Data](#stock-time-series-data)
  - [TIME_SERIES_INTRADAY](#time_series_intraday)
  - [TIME_SERIES_DAILY](#time_series_daily)
- [Fundamental Data](#fundamental-data)
  - [OVERVIEW](#overview)
  - [INCOME_STATEMENT](#income_statement)

## Stock Time Series Data

This section contains APIs for retrieving stock time series data...

<!-- ENDPOINT_START: TIME_SERIES_INTRADAY -->

### TIME_SERIES_INTRADAY

This API returns intraday time series (timestamp, open, high, low, close, volume)...

#### Parameters

| Name | Type | Description | Default | Options |
|------|------|-------------|---------|---------|
| function | string | The time series of your choice | | TIME_SERIES_INTRADAY |
| symbol | string | The name of the equity | | |
| interval | string | Time interval between data points | 5min | 1min, 5min, 15min, 30min, 60min |

#### Python Code Example

```python
import requests

url = 'https://www.alphavantage.co/query?function=TIME_SERIES_INTRADAY&symbol=IBM&interval=5min&apikey=demo'
r = requests.get(url)
data = r.json()

print(data)
```

#### Request Pattern

```http
https://www.alphavantage.co/query?function=TIME_SERIES_INTRADAY&symbol=IBM&interval=5min&apikey=YOUR_API_KEY
```

<!-- ENDPOINT_END: TIME_SERIES_INTRADAY -->
<!-- CATEGORY_END: Stock Time Series Data -->
```

### Frontmatter Specification

The YAML frontmatter contains metadata for LLM processing:

```yaml
format_version: "2.0"          # Format version for compatibility
source_url: string            # Original documentation URL
extraction_date: datetime     # ISO 8601 timestamp
categories: integer          # Number of API categories found
endpoints: integer           # Total number of endpoints extracted
```

### Semantic Markers

HTML comments provide parsing guidance for LLMs:

- `<!-- CATEGORY_START: Category Name -->`
- `<!-- CATEGORY_END: Category Name -->`
- `<!-- ENDPOINT_START: FUNCTION_NAME -->`
- `<!-- ENDPOINT_END: FUNCTION_NAME -->`

### Content Sections

#### Document Header
- Title: "Alpha Vantage API Documentation"
- Metadata table with extraction statistics
- Table of contents with GitHub-style anchor links

#### Category Sections
- H2 heading with category name
- Category description paragraph(s)
- All endpoints belonging to the category

#### Endpoint Sections
- H3 heading with function name
- Premium indicator badge (if applicable)
- Endpoint description
- Parameters table (if present)
- Code examples (if present)
- Request pattern (if present)

### Parameter Table Format

Standard Markdown table with the following columns:
- **Name**: Parameter name (required parameters in bold)
- **Type**: Data type (string, integer, etc.)
- **Description**: Human-readable description
- **Default**: Default value (if any)
- **Options**: Comma-separated list of valid values (if applicable)

### Code Examples

- Language-specific fenced code blocks
- Supported languages: Python, R, VBA, MATLAB, JavaScript
- Clean, runnable code with proper indentation
- Include API key placeholder comments

### Request Patterns

- HTTP request format in code blocks
- Include all required parameters
- Use YOUR_API_KEY placeholder
- Show complete URL structure

## Validation Rules

### HTML Artifact Detection
- **Prohibited elements**: `<script>`, `<style>`, `<iframe>`, `<object>`
- **Prohibited attributes**: `onclick`, `onload`, `javascript:`
- **Prohibited text patterns**: HTML entities (`&lt;`, `&gt;`, `&amp;`)

### Heading Hierarchy
- Exactly one H1 element (document title)
- H2 elements for categories
- H3 elements for endpoints
- No skipping levels (no H3 without H2)

### Code Fence Balancing
- All code blocks must have opening and closing fences
- Language specifiers must be valid
- No nested code blocks

### Table Formatting
- Proper Markdown table syntax
- Aligned columns
- No empty header cells
- Consistent separator row

### Content Requirements
- Minimum 5KB output size
- At least 3 endpoints extracted
- All categories must have descriptions
- All endpoints must have descriptions

### Duplicate Detection
- No duplicate category names
- No duplicate endpoint names within categories
- No duplicate code examples

### Required Sections
- Document title present
- Table of contents present
- At least one category
- Categories contain endpoints

## Performance Requirements

### Timing Targets
- **Complete extraction**: < 30 seconds
- **HTML parsing**: < 5 seconds
- **Content extraction**: < 10 seconds
- **Markdown rendering**: < 5 seconds
- **Output validation**: < 2 seconds
- **File writing**: < 1 second

### Resource Limits
- **Peak memory usage**: < 100MB
- **CPU usage**: Single core optimized
- **Network requests**: 1 (documentation URL only)
- **File operations**: 1 write (plus optional backup)

### Scalability Metrics
- **Categories/second**: > 10
- **Endpoints/second**: > 50
- **HTML processing**: > 500KB/second
- **Markdown generation**: > 100KB/second

## Error Handling

### Error Types

#### Network Errors
- Connection failures
- Timeout errors
- HTTP status errors (4xx, 5xx)
- DNS resolution failures

#### Parsing Errors
- Invalid HTML structure
- Missing expected elements
- Malformed content
- Encoding errors

#### Validation Errors
- Schema violations
- Content requirement failures
- Format inconsistencies
- Quality threshold failures

#### File System Errors
- Permission denied
- Disk space insufficient
- Path validation failures
- Backup creation failures

### Error Context
All errors include:
- Correlation ID for request tracing
- Operation context (what was being attempted)
- Input parameters (sanitized)
- Timestamp
- Stack trace (in debug builds)

### Recovery Strategies
- **Retry**: Network failures (exponential backoff)
- **Fallback**: Missing content sections
- **Skip**: Optional content extraction failures
- **Partial Success**: Continue processing after non-critical errors

## Security Considerations

### Input Validation
- URL format validation
- Path traversal prevention
- File system access restrictions
- HTML sanitization

### Output Safety
- No HTML injection in Markdown
- Safe file operations
- Permission checks
- Backup integrity

### Network Security
- HTTPS-only URLs
- Timeout enforcement
- No arbitrary redirects
- Safe header handling

## Testing Requirements

### Unit Test Coverage
- **Domain logic**: 100% line coverage
- **Error handling**: All error paths tested
- **Edge cases**: Boundary conditions covered
- **Property-based**: Parser and validator testing

### Integration Testing
- **Component interaction**: All layer boundaries
- **External systems**: HTTP client, file system
- **CLI interface**: Complete argument parsing
- **Error propagation**: End-to-end error handling

### End-to-End Testing
- **Golden master**: Snapshot validation
- **Performance**: Benchmark regression detection
- **Real data**: Production documentation testing
- **CI validation**: Automated quality gates

### Test Fixtures
- **HTML fixtures**: Realistic Alpha Vantage documentation
- **Edge cases**: Malformed content, missing elements
- **Large datasets**: Performance testing
- **Error conditions**: Network failures, invalid inputs

## Deployment and Distribution

### Build Requirements
- Rust 1.75+ stable
- Cargo package manager
- Target platforms: Linux, macOS, Windows

### Packaging
- Single binary distribution
- No external dependencies required
- Static linking for portability
- Compressed distribution archives

### Installation Methods
- Cargo install from crates.io
- Pre-built binaries from GitHub releases
- Docker container (optional)
- Build from source

### Configuration
- Environment-based configuration
- Command-line overrides
- Sensible defaults
- Validation at startup

This specification ensures the tool is reliable, maintainable, and provides consistent high-quality output for LLM consumption.
