# Installation Guide

## Quick Start

Add x402 to your `Cargo.toml`:

```toml
[dependencies]
rust-x402 = "0.1.0"
```

## Feature Flags

x402 supports several feature flags to include only what you need:

### Web Framework Support
- `axum` (default) - Axum web framework middleware
- `actix-web` - Actix Web framework middleware  
- `warp` - Warp web framework middleware

### Example Configuration

```toml
[dependencies]
# Basic usage with Axum (default)
rust-x402 = "0.1.0"

# Or specify features explicitly
rust-x402 = { version = "0.1.0", features = ["axum"] }

# Multiple frameworks
rust-x402 = { version = "0.1.0", features = ["axum", "actix-web", "warp"] }

# No framework features (core only)
rust-x402 = { version = "0.1.0", default-features = false }
```

## Requirements

- Rust 1.70 or later
- Tokio async runtime
- For blockchain features: access to Ethereum-compatible RPC endpoints

## Development Setup

1. Clone the repository:
```bash
git clone https://github.com/RyanKung/x402_rs.git
cd x402_rs
```

2. Run tests:
```bash
cargo test --all-features
```

3. Run examples:
```bash
# Axum server example
cargo run --example axum_server --features axum

# Client example
cargo run --example client
```

## Publishing

To publish a new version:

1. Update version in `Cargo.toml`
2. Run the release script:
```bash
./scripts/release.sh
```

Or manually:
```bash
cargo test --all-features
cargo clippy --all-features -- -D warnings
cargo fmt --all -- --check
cargo publish
```

## Troubleshooting

### Common Issues

1. **Build errors with features**: Make sure you're using the correct feature flags for your framework
2. **Network errors**: Check your RPC endpoint configuration
3. **Authentication errors**: Verify your private key and network configuration

### Getting Help

- [Documentation](https://docs.rs/rust-x402)
- [GitHub Issues](https://github.com/RyanKung/x402_rs/issues)
- [Examples](examples/)
