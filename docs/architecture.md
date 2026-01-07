# Architecture Documentation

## Overview

The Alpha Vantage Documentation Extractor follows **Hexagonal Architecture** (also known as Ports & Adapters) to achieve clean separation of concerns, high testability, and maintainability. This architecture treats the domain logic as the core of the application, with external concerns adapted to the domain's needs.

## Architectural Principles

### Hexagonal Architecture

```
┌─────────────────────────────────────────────────┐
│                 EXTERNAL WORLD                  │
│  ┌─────────────────────────────────────────────┐ │
│  │                ADAPTERS                     │ │
│  │  ┌─────────────────────────────────────────┐ │ │
│  │  │               PORTS                     │ │ │
│  │  │  ┌─────────────────────────────────────┐ │ │ │
│  │  │  │           DOMAIN                    │ │ │ │
│  │  │  │                                     │ │ │ │
│  │  │  │  • Business Logic                   │ │ │ │
│  │  │  │  • Domain Models                    │ │ │ │
│  │  │  │  • Pure Functions                   │ │ │ │
│  │  │  │                                     │ │ │ │
│  │  │  └─────────────────────────────────────┘ │ │ │
│  │  └─────────────────────────────────────────┘ │ │
│  └─────────────────────────────────────────────┘ │
└─────────────────────────────────────────────────┘
```

### Key Benefits

- **Domain Isolation**: Business logic knows nothing about external systems
- **Testability**: Each layer can be tested independently
- **Technology Independence**: External systems can be swapped without affecting business logic
- **Maintainability**: Clear boundaries make the codebase easier to understand and modify

## Module Structure

```
src/
├── adapters/          # External interface implementations
│   ├── cli.rs        # Command-line argument parsing (clap)
│   ├── http_client.rs # HTTP client implementation (reqwest)
│   └── file_writer.rs # File system operations (std::fs)
├── domain/           # Pure business logic (no external dependencies)
│   ├── models.rs     # Domain types and entities
│   ├── parser.rs     # HTML parsing and DOM traversal (scraper)
│   ├── extractor.rs  # Content extraction logic
│   ├── renderer.rs   # Markdown generation and LLM optimization
│   └── transformer.rs # HTML cleaning and transformation
├── ports/            # Abstract interface definitions
│   ├── fetcher.rs    # HTTP client interface (async trait)
│   └── writer.rs     # File writer interface (async trait)
├── utils/            # Cross-cutting concerns
│   ├── error.rs      # Error types and handling (thiserror, anyhow)
│   ├── logging.rs    # Structured logging (tracing)
│   └── validation.rs # Input validation utilities
└── main.rs           # Application entry point and orchestration
```

## Data Flow

### Complete Pipeline

```
URL Input
    ↓
CLI Adapter → URL Validation → RawUrl → ValidatedUrl
    ↓
HTTP Client Adapter → RawHtml
    ↓
HTML Cleaner → Cleaned HTML String
    ↓
HTML Parser → ParsedDocument (with DOM)
    ↓
Content Identifier → Main Content ElementRef
    ↓
Content Extractor → Vec<ApiCategory>
    ↓
Document Structure Builder → DocumentStructure
    ↓
Markdown Renderer (+ LLM Optimization) → String
    ↓
Output Validator → ValidationReport
    ↓
File Writer Adapter → Markdown File
```

### Layer Responsibilities

#### Adapters Layer

**Purpose**: Implement external interfaces and adapt external systems to domain needs.

- **CLI Adapter**: Parses command-line arguments using clap derive macros
- **HTTP Client Adapter**: Makes HTTP requests using reqwest, handles retries and timeouts
- **File Writer Adapter**: Manages file system operations with backup and permission handling

**Characteristics**:
- Knows about external libraries and frameworks
- Implements port interfaces
- Contains no business logic
- Handles external system failures and maps to domain errors

#### Domain Layer

**Purpose**: Contains all business logic, completely independent of external systems.

- **Models**: Core domain types (ApiCategory, ApiEndpoint, Parameter, etc.)
- **Parser**: HTML DOM traversal and element selection
- **Extractor**: API documentation extraction from HTML
- **Renderer**: Markdown generation with LLM optimizations
- **Transformer**: HTML cleaning and sanitization

**Characteristics**:
- Pure functions with no side effects
- No external dependencies (except serde for serialization)
- Comprehensive unit test coverage
- Type-safe with compile-time guarantees

#### Ports Layer

**Purpose**: Define abstract interfaces for external dependencies.

- **Fetcher**: Async trait for HTTP operations
- **Writer**: Async trait for file operations

**Characteristics**:
- Trait definitions only
- No implementations
- Technology-agnostic interfaces
- Easy to mock for testing

#### Utils Layer

**Purpose**: Cross-cutting concerns shared across layers.

- **Error**: Custom error types with proper error chaining
- **Logging**: Structured logging with correlation IDs
- **Validation**: Input validation utilities

**Characteristics**:
- Shared utilities
- No business logic
- Framework and library agnostic where possible

## Design Decisions

### Type-State Pattern

Instead of boolean flags, we use distinct types to represent different states:

```rust
// Instead of: struct Url { value: String, validated: bool }

// We use:
pub struct RawUrl(String);
pub struct ValidatedUrl { inner: url::Url }

// Construction validates immediately
impl RawUrl {
    pub fn new(url: String) -> Self { /* basic validation */ }
}

impl ValidatedUrl {
    pub fn validate(raw: RawUrl) -> Result<Self, ValidationError> { /* full validation */ }
}
```

**Benefits**:
- Compile-time state guarantees
- Impossible to use unvalidated data
- Clear API contracts

### Parse, Don't Validate

Inputs are immediately parsed into domain types rather than stored as raw strings:

```rust
// Good: Parse immediately
pub fn extract_documentation(url: ValidatedUrl) -> Result<DocumentStructure, Error>

// Bad: Store raw string and validate later
pub fn extract_documentation(url: String) -> Result<DocumentStructure, Error> {
    validate_url(&url)?;
    // ... rest of function
}
```

**Benefits**:
- Invalid data cannot propagate through the system
- Errors caught at boundaries
- Cleaner function signatures

### Error Handling Strategy

- **Domain Errors**: Pure domain logic errors (parsing failures, validation errors)
- **Infrastructure Errors**: External system failures (network, filesystem)
- **Error Context**: All errors include correlation IDs and operation context
- **No Panics**: All error cases return `Result` types

### Testing Strategy

#### Unit Tests
- Located in `tests/unit/` and co-located with implementation
- Test individual functions and modules
- Use property-based testing for parsers and validators
- Mock external dependencies using trait objects

#### Integration Tests
- Located in `tests/integration/`
- Test component interactions
- Use real external systems where safe (filesystem, logging)
- Test CLI interface with `assert_cmd`

#### End-to-End Tests
- Located in `tests/end_to_end.rs`
- Test complete pipeline with real data
- Golden master validation with `insta`
- Performance benchmarking with `criterion`

#### Test Fixtures
- Realistic HTML fixtures in `tests/fixtures/`
- Pre-recorded API responses for consistent testing
- Edge cases and error conditions

## Performance Considerations

### Zero-Cost Abstractions

- **Compile-time guarantees**: Type safety without runtime overhead
- **Iterator chains**: Efficient data processing without intermediate allocations
- **Stack allocation**: Small objects allocated on stack where possible

### Async/Await Design

- **Non-blocking I/O**: HTTP requests don't block the event loop
- **Concurrent processing**: Multiple operations can proceed simultaneously
- **Resource efficiency**: Tokio's work-stealing scheduler optimizes CPU usage

### Memory Management

- **Streaming processing**: Large HTML documents processed without full buffering
- **RAII patterns**: Resources automatically cleaned up
- **Borrowing over ownership**: Minimize unnecessary allocations

## Evolution and Maintenance

### Adding New Features

1. **Identify the layer**: Determine if it's domain logic, external interface, or utility
2. **Define ports first**: If external interface, define the abstract interface
3. **Implement domain logic**: Pure functions with comprehensive tests
4. **Create adapters**: Implement external interfaces
5. **Update orchestration**: Wire everything together in main.rs

### Technology Changes

- **HTTP client**: Replace reqwest implementation, keep Fetcher trait unchanged
- **CLI framework**: Replace clap with another library, keep Config struct interface
- **File operations**: Change filesystem implementation, keep Writer trait

### Testing Evolution

- **New test fixtures**: Add realistic data for new functionality
- **Property-based tests**: Ensure robustness against edge cases
- **Performance regression tests**: Catch performance degradation early

This architecture ensures the codebase remains maintainable, testable, and adaptable to future requirements while providing high performance and reliability.
