# Alpha Vantage Documentation Extractor

[![CI](https://github.com/TrantorVector/alphavantage-doc-extractor/actions/workflows/ci.yml/badge.svg)](https://github.com/TrantorVector/alphavantage-doc-extractor/actions/workflows/ci.yml)
[![License: MIT](https://img.shields.io/badge/License-MIT-yellow.svg)](https://opensource.org/licenses/MIT)
[![Rust 1.75+](https://img.shields.io/badge/rust-1.75+-000000.svg)](https://www.rust-lang.org/)

A robust, high-performance tool to extract and process API documentation from Alpha Vantage, generating LLM-optimized Markdown documentation with semantic markers and comprehensive validation.

## Features

- **🏗️ Hexagonal Architecture**: Clean separation between business logic and external dependencies
- **🔒 Type-Safe**: Uses Rust's type system for compile-time guarantees
- **⚡ High-Performance**: Built with Tokio for concurrent operations, zero-cost abstractions
- **🛡️ Error Resilient**: No panics, proper error propagation with context and correlation IDs
- **📊 Observable**: Structured logging with instrumentation and performance monitoring
- **🧪 Well-Tested**: Property-based testing, snapshot testing, golden master validation, and comprehensive integration tests
- **🤖 LLM-Optimized**: Semantic markers, frontmatter, and whitespace optimization for AI parsing
- **📋 Validation**: Comprehensive output validation ensuring quality and consistency

## Installation

### Prerequisites

- Rust 1.75 or later
- Cargo package manager

### Install from Source

```bash
git clone https://github.com/TrantorVector/alphavantage-doc-extractor.git
cd alphavantage-doc-extractor
cargo build --release
```

The binary will be available at `target/release/alphavantage-doc-extractor`.

### Install with Cargo

```bash
cargo install --git https://github.com/TrantorVector/alphavantage-doc-extractor.git
```

## Usage

### Basic Usage

Extract documentation from Alpha Vantage's official documentation:

```bash
alphavantage-doc-extractor --url https://www.alphavantage.co/documentation --output alphavantage-api.md
```

This extracts the complete API documentation and saves it as LLM-optimized Markdown.

### Advanced Usage

With all configuration options:

```bash
alphavantage-doc-extractor \
  --url https://www.alphavantage.co/documentation \
  --output alphavantage-api.md \
  --log-level info \
  --log-format pretty \
  --backup \
  --no-validation
```

### Configuration Options

| Option | Default | Description |
|--------|---------|-------------|
| `-u, --url <URL>` | Required | Source URL to extract documentation from |
| `-o, --output <OUTPUT>` | `output.md` | Output file path for generated markdown |
| `--log-level <LEVEL>` | `info` | Logging level (error, warn, info, debug, trace) |
| `--log-format <FORMAT>` | `pretty` | Log output format (pretty, json) |
| `--no-validation` | `false` | Skip output validation (faster but less safe) |
| `--backup` | `false` | Create backup of existing output file |
| `-h, --help` | | Display help information |
| `-V, --version` | | Display version information |

### Environment Variables

| Variable | Description |
|----------|-------------|
| `RUST_LOG` | Override default log level (same values as `--log-level`) |

## Output Format

The tool generates **LLM-optimized Markdown** with the following structure:

### Frontmatter
```yaml
---
format_version: "2.0"
source_url: "https://www.alphavantage.co/documentation"
extraction_date: "2024-01-07T12:00:00Z"
categories: 5
endpoints: 25
---

<!-- LLM Semantic Markers -->
<!-- CATEGORY_START: Stock Time Series Data -->
<!-- ENDPOINT_START: TIME_SERIES_INTRADAY -->
```

### Structure
1. **Document Header**: Title, metadata, and table of contents
2. **Categories**: Grouped API endpoints by functionality
3. **Endpoints**: Individual API documentation with:
   - Function name and description
   - Premium indicators (where applicable)
   - Parameter tables with types, defaults, and options
   - Code examples in multiple languages
   - Request pattern examples
4. **Semantic Markers**: HTML comments for LLM parsing guidance

### Validation Rules
- HTML artifacts removal (<script>, <div>, etc.)
- Proper heading hierarchy (H1 → H2 → H3)
- Balanced code fences
- Table formatting compliance
- Minimum content requirements
- Duplicate section detection
- Required sections presence

## Architecture

This project follows **Hexagonal Architecture** (Ports & Adapters) for maintainability and testability:

```
src/
├── adapters/          # External interface implementations
│   ├── cli.rs        # Command-line argument parsing
│   ├── http_client.rs # HTTP client implementation
│   └── file_writer.rs # File system operations
├── domain/           # Pure business logic (no external deps)
│   ├── models.rs     # Domain types and entities
│   ├── parser.rs     # HTML parsing and DOM traversal
│   ├── extractor.rs  # Content extraction logic
│   ├── renderer.rs   # Markdown generation
│   └── transformer.rs # HTML cleaning and transformation
├── ports/            # Abstract interfaces
│   ├── fetcher.rs    # HTTP client interface
│   └── writer.rs     # File writer interface
└── utils/            # Cross-cutting concerns
    ├── error.rs      # Error types and handling
    ├── logging.rs    # Structured logging
    └── validation.rs # Input validation utilities
```

### Design Principles

- **Parse, Don't Validate**: Inputs are immediately parsed into domain types
- **Type-State Pattern**: Use distinct types instead of boolean flags
- **Zero-Cost Abstractions**: Compile-time guarantees with runtime efficiency
- **No Panics**: All error cases return `Result` types with proper context
- **Domain Purity**: Business logic independent of external systems

## Development

### Testing

Run the complete test suite:

```bash
cargo test
```

Run end-to-end integration tests:

```bash
cargo test --test end_to_end
```

Run performance benchmarks:

```bash
cargo bench
```

### Golden Master Verification Workflow

The project uses **snapshot testing** with `insta` for output validation:

1. **First Run**: Tests will fail because no snapshots exist
   ```bash
   cargo test  # This will fail with missing snapshots
   ```

2. **Review Output**: Use `cargo insta review` to visually inspect generated markdown
   ```bash
   cargo insta review
   ```

3. **Accept Changes**: Press 'a' to accept if output looks correct
   - This creates the golden master snapshots
   - Future runs will compare against these baselines
   - CI will fail if output changes unexpectedly

### Linting and Formatting

```bash
cargo clippy -- -D warnings  # Lint with warnings as errors
cargo fmt --check           # Check formatting
cargo fmt                   # Apply formatting fixes
```

## Troubleshooting

### Common Issues

**"Permission denied" errors**
- Ensure you have write permissions for the output directory
- Use `--backup` flag to preserve existing files

**"URL validation failed" errors**
- Verify the URL starts with `http://` or `https://`
- Ensure the URL contains a valid domain structure

**"Output validation failed" errors**
- Check the generated markdown for formatting issues
- Use `--no-validation` flag to skip validation if needed

**High memory usage**
- The tool processes entire HTML documents in memory
- Ensure adequate RAM for large documentation pages

### Debug Mode

Enable debug logging for detailed processing information:

```bash
alphavantage-doc-extractor --url <URL> --log-level debug
```

### Performance Monitoring

Run benchmarks to check performance:

```bash
cargo bench
```

Expected performance (on modern hardware):
- Complete extraction: <30 seconds
- Memory usage: <100MB peak
- Categories/second: >10
- Endpoints/second: >50

## Contributing

We welcome contributions! Please see [CONTRIBUTING.md](CONTRIBUTING.md) for detailed guidelines.

### Quick Start

1. Fork the repository
2. Clone your fork: `git clone https://github.com/YOUR_USERNAME/alphavantage-doc-extractor.git`
3. Create a feature branch: `git checkout -b feature/amazing-feature`
4. Make your changes with tests
5. Run the full test suite: `cargo test && cargo clippy -- -D warnings && cargo fmt --check`
6. Submit a pull request

### Development Requirements

- Rust 1.75+
- All tests must pass
- Code must be formatted with `cargo fmt`
- No clippy warnings allowed
- New features require comprehensive tests
- Documentation updates for public APIs

## Repository

- **GitHub**: https://github.com/TrantorVector/alphavantage-doc-extractor
- **Issues**: https://github.com/TrantorVector/alphavantage-doc-extractor/issues
- **Documentation**: See `docs/` directory for detailed specifications

## License

This project is licensed under the MIT License - see the [LICENSE](LICENSE) file for details.

---

**Built with ❤️ in Rust for reliability, performance, and developer experience.**
