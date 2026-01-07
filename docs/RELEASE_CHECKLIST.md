# Release Checklist

## Pre-Release
- [ ] `cargo test` passing
- [ ] `cargo test --test end_to_end` passing
- [ ] `cargo fmt --check` passing
- [ ] `cargo clippy -- -D warnings` passing
- [ ] `cargo audit` clean
- [ ] `cargo bench` run and checked
- [ ] Version bumped in `Cargo.toml`
- [ ] `CHANGELOG.md` updated

## Release
- [ ] Create tag `vX.Y.Z`
- [ ] Push tag to trigger release workflow
- [ ] Verify GitHub Release created
- [ ] Verify crate published to crates.io

