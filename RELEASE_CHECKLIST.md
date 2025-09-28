# x402 Rust Release Checklist

## Pre-Release Checklist

### 1. Code Quality
- [ ] All tests pass: `cargo test --all-features`
- [ ] Clippy passes: `cargo clippy --all-features -- -D warnings`
- [ ] Format check passes: `cargo fmt --all -- --check`
- [ ] Security audit passes: `cargo audit`

### 2. Documentation
- [ ] README.md is up to date
- [ ] All public APIs are documented
- [ ] Examples are working and documented
- [ ] Architecture documentation is complete

### 3. Version Management
- [ ] Update version in Cargo.toml
- [ ] Update CHANGELOG.md (if exists)
- [ ] Tag the release: `git tag v0.1.0`
- [ ] Push tags: `git push origin v0.1.0`

### 4. Publishing
- [ ] Login to crates.io: `cargo login`
- [ ] Publish to crates.io: `cargo publish`
- [ ] Verify publication on crates.io
- [ ] Create GitHub release

## Post-Release Checklist

### 1. Verification
- [ ] Test installation: `cargo install x402`
- [ ] Verify documentation on docs.rs
- [ ] Test examples with published version
- [ ] Update any dependent projects

### 2. Communication
- [ ] Announce release on relevant channels
- [ ] Update project documentation
- [ ] Monitor for issues and feedback

## Commands

```bash
# Run all checks
cargo test --all-features
cargo clippy --all-features -- -D warnings
cargo fmt --all -- --check
cargo audit

# Publish
cargo login
cargo publish

# Test installation
cargo install x402
```

## Notes

- The package name is `x402` (as specified in Cargo.toml)
- Repository: https://github.com/RyanKung/x402_rs
- Documentation: https://docs.rs/x402
- Homepage: https://x402.org
