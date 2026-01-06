# Test Fixtures

This directory contains test fixtures used for integration and unit testing.

## HTML Fixtures

- `alphavantage_docs.html` - Sample Alpha Vantage documentation page
- `endpoint_examples.html` - Examples of different API endpoint documentation formats

## JSON Fixtures

- `expected_endpoints.json` - Expected parsed endpoint data
- `validation_cases.json` - Edge cases for validation testing

## Markdown Fixtures

- `golden_master.md` - Expected markdown output for snapshot testing
- `error_cases.md` - Examples of malformed output for error testing

## Usage

Fixtures should be used in tests to ensure consistent and reproducible results.
When updating fixtures, ensure all dependent tests are updated accordingly.
