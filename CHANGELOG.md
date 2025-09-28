# Changelog

All notable changes to this project will be documented in this file.

The format is based on [Keep a Changelog](https://keepachangelog.com/en/1.0.0/),
and this project adheres to [Semantic Versioning](https://semver.org/spec/v2.0.0.html).

## [Unreleased]

### Added
- Initial release of x402 Rust implementation
- HTTP-native micropayment protocol support
- EIP-3009 token transfer integration
- Web framework middleware for Axum, Actix Web, and Warp
- Comprehensive facilitator client for payment verification
- Real wallet integration with EIP-712 signing
- Blockchain client for network interactions
- Template system for payment UI
- Discovery client for finding x402 resources
- Extensive test coverage (97+ tests)
- Performance optimizations and error handling

### Features
- **Core Types**: PaymentRequirements, PaymentPayload, ExactEvmPayload
- **HTTP Client**: X402Client with automatic payment handling
- **Middleware**: Framework-agnostic payment protection
- **Cryptography**: EIP-712 signing, signature verification, JWT authentication
- **Blockchain**: Support for Base and Avalanche networks
- **Templates**: Responsive HTML templates for payment UI
- **Examples**: Complete working examples for all frameworks

### Supported Networks
- Base Mainnet and Sepolia Testnet
- Avalanche Mainnet and Fuji Testnet

### Supported Frameworks
- Axum (default)
- Actix Web
- Warp

## [0.1.0] - 2025-01-27

### Added
- Initial release
- Complete x402 protocol implementation
- All core functionality and examples
- Comprehensive documentation
- CI/CD pipeline setup
