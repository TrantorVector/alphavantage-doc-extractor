# Alpha Vantage Documentation Extractor

A robust, high-performance tool to extract and process API documentation from Alpha Vantage, generating structured Markdown documentation.

## Features

- **Hexagonal Architecture**: Clean separation between business logic and external dependencies
- **Type-Safe**: Uses Rust's type system for compile-time guarantees
- **Async/Await**: Built with Tokio for high-performance concurrent operations
- **Comprehensive Error Handling**: No panics, proper error propagation with context
- **Observable**: Structured logging with correlation IDs and instrumentation
- **Testable**: Property-based testing, snapshot testing, and integration tests

## Installation

### Prerequisites

- Rust 1.75 or later
- Cargo package manager

### Build from Source

```bash
git clone https://github.com/TrantorVector/alphavantage-doc-extractor.git
cd alphavantage-doc-extractor
cargo build --release
```

## Usage

### Basic Usage

Extract documentation from Alpha Vantage:

```bash
./target/release/alphavantage-doc-extractor extract --output docs.md
```

### Advanced Usage

With custom API key and rate limiting:

```bash
export ALPHA_VANTAGE_API_KEY=your_api_key_here
./target/release/alphavantage-doc-extractor extract \
  --output alphavantage-api-docs.md \
  --rate-limit 5 \
  --timeout 30
```

### Validation Mode

Validate existing documentation without extraction:

```bash
./target/release/alphavantage-doc-extractor validate docs.md
```

### Render Mode

Convert extracted data to different formats:

```bash
./target/release/alphavantage-doc-extractor render \
  --input extracted.json \
  --output docs.md \
  --format markdown
```

## Architecture

This project follows **Hexagonal Architecture** (Ports & Adapters):

```
src/
├── adapters/     # External interface implementations (CLI, HTTP, Filesystem)
├── domain/       # Pure business logic (parsing, extraction, transformation)
├── ports/        # Abstract interfaces for external dependencies
└── utils/        # Shared utilities (errors, logging, validation)
```

### Design Principles

- **Parse, Don't Validate**: Inputs are immediately parsed into domain types
- **Type-State Pattern**: Use distinct types instead of boolean flags
- **Zero-Cost Abstractions**: Compile-time guarantees with runtime efficiency
- **No Panics**: All error cases return `Result` types

## Development

### Testing

Run the full test suite:

```bash
cargo test
```

Run with coverage:

```bash
cargo tarpaulin
```

### Benchmarks

Run performance benchmarks:

```bash
cargo bench
```

### Linting and Formatting

```bash
cargo clippy -- -D warnings
cargo fmt --check
```

## Contributing

1. Fork the repository
2. Create a feature branch
3. Add tests for new functionality
4. Ensure all checks pass: `cargo test && cargo clippy && cargo fmt`
5. Submit a pull request

## License

This project is licensed under the MIT License - see the [LICENSE](LICENSE) file for details.
