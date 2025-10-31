//! # x402 Rust Implementation
//!
//! A **high-performance, type-safe** Rust implementation of the x402 HTTP-native micropayment protocol.
//!
//! ## Features
//!
//! - 🚀 **HTTP-native micropayments**: Leverage the HTTP 402 status code for payment requirements
//! - ⛓️ **Blockchain integration**: Support for EIP-3009 token transfers with real wallet integration
//! - 🌐 **Web framework support**: Middleware for Axum, Actix Web, and Warp
//! - 💰 **Facilitator integration**: Built-in support for payment verification and settlement
//! - 🔒 **Type safety**: Strongly typed Rust implementation with comprehensive error handling
//! - 🧪 **Comprehensive testing**: 70+ tests with 100% pass rate covering all real implementations
//! - 🏗️ **Real implementations**: Production-ready wallet, blockchain, and facilitator clients
//! - 🌊 **Multipart & Streaming**: Full support for large file uploads and streaming responses
//! - 📡 **HTTP/3 Support**: Optional HTTP/3 (QUIC) support for modern high-performance networking
//!
//! ## Quick Start
//!
//! ### Creating a Payment Server with Axum
//!
//! ```rust,no_run
//! use axum::{response::Json, routing::get};
//! use rust_x402::{
//!     axum::{create_payment_app, AxumPaymentConfig},
//!     types::FacilitatorConfig,
//! };
//! use std::str::FromStr;
//!
//! #[tokio::main]
//! async fn main() -> Result<(), Box<dyn std::error::Error>> {
//!     // Create facilitator config
//!     let facilitator_config = FacilitatorConfig::default();
//!     
//!     // Create payment configuration
//!     let payment_config = AxumPaymentConfig::new(
//!         rust_decimal::Decimal::from_str("0.0001")?,
//!         "0x209693Bc6afc0C5328bA36FaF03C514EF312287C",
//!     )
//!     .with_description("Premium API access")
//!     .with_facilitator_config(facilitator_config)
//!     .with_testnet(true);
//!
//!     // Create the application with payment middleware
//!     let app = create_payment_app(payment_config, |router| {
//!         router.route("/joke", get(joke_handler))
//!     });
//!
//!     // Start server
//!     let listener = tokio::net::TcpListener::bind("0.0.0.0:4021").await?;
//!     axum::serve(listener, app).await?;
//!
//!     Ok(())
//! }
//!
//! async fn joke_handler() -> Json<serde_json::Value> {
//!     Json(serde_json::json!({
//!         "joke": "Why do programmers prefer dark mode? Because light attracts bugs!"
//!     }))
//! }
//! ```
//!
//! ## Architecture
//!
//! The Rust implementation is organized into several modules:
//!
//! - **`types`**: Core data structures and type definitions
//! - **`client`**: HTTP client with x402 payment support
//! - **`facilitator`**: Payment verification and settlement
//! - **`middleware`**: Web framework middleware implementations
//! - **`crypto`**: Cryptographic utilities for payment signing
//! - **`error`**: Comprehensive error handling
//! - **`wallet`**: Real wallet integration with EIP-712 signing
//! - **`blockchain`**: Blockchain client for network interactions
//! - **`blockchain_facilitator`**: Blockchain-based facilitator implementation
//! - **`http3`**: HTTP/3 (QUIC) support (feature-gated)
//! - **`proxy`**: Reverse proxy with streaming support
//!
//! ## HTTP Protocol Support
//!
//! - **HTTP/1.1**: Full support with chunked transfer encoding
//! - **HTTP/2**: Full support with multiplexing
//! - **Multipart**: Support for `multipart/form-data` uploads (via `multipart` feature)
//! - **Streaming**: Chunked and streaming responses (via `streaming` feature)
//! - **HTTP/3** (optional): QUIC-based HTTP/3 via `http3` feature flag
//!
//! ## Optional Features
//!
//! x402 supports optional features for a modular build:
//!
//! ```toml
//! [dependencies]
//! rust-x402 = { version = "0.1.2", features = ["http3", "streaming", "multipart"] }
//! ```
//!
//! - **`http3`**: Enable HTTP/3 (QUIC) support
//! - **`streaming`**: Enable chunked and streaming responses
//! - **`multipart`**: Enable `multipart/form-data` upload support (requires `streaming`)
//! - **`axum`**: Enable Axum web framework integration (default)
//! - **`actix-web`**: Enable Actix Web framework integration
//! - **`warp`**: Enable Warp web framework integration
//!
//! ## Blockchain Support
//!
//! Currently supports:
//! - **Base**: Base mainnet and testnet
//! - **Avalanche**: Avalanche mainnet and Fuji testnet
//! - **EIP-3009**: Transfer with Authorization standard

pub mod blockchain;
pub mod blockchain_facilitator;
pub mod client;
pub mod crypto;
pub mod error;
pub mod facilitator;
pub mod facilitator_storage;
pub mod middleware;
pub mod proxy;
pub mod template;
pub mod types;
pub mod wallet;

// HTTP/3 support (feature-gated)
#[cfg(feature = "http3")]
pub mod http3;

// Re-exports for convenience
pub use blockchain::{BlockchainClient, BlockchainClientFactory};
pub use blockchain_facilitator::{
    BlockchainFacilitatorClient, BlockchainFacilitatorConfig, BlockchainFacilitatorFactory,
};
pub use client::X402Client;
pub use error::{Result, X402Error};
pub use types::*;
pub use wallet::{Wallet, WalletFactory};

// Feature-gated framework support
#[cfg(feature = "axum")]
pub mod axum;

#[cfg(feature = "actix-web")]
pub mod actix_web;

#[cfg(feature = "warp")]
pub mod warp;

/// Current version of the x402 library
pub const VERSION: &str = env!("CARGO_PKG_VERSION");

/// x402 protocol version
pub const X402_VERSION: u32 = 1;

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_version_constants() {
        assert_eq!(X402_VERSION, 1);
        // VERSION is a const string, so it's never empty
        assert!(!VERSION.is_empty());
    }

    #[test]
    fn test_payment_requirements_creation() {
        let requirements = PaymentRequirements::new(
            "exact",
            "base-sepolia",
            "1000000",
            "0x036CbD53842c5426634e7929541eC2318f3dCF7e",
            "0x209693Bc6afc0C5328bA36FaF03C514EF312287C",
            "https://example.com/test",
            "Test payment",
        );

        assert_eq!(requirements.scheme, "exact");
        assert_eq!(requirements.network, "base-sepolia");
        assert_eq!(requirements.max_amount_required, "1000000");
        assert_eq!(
            requirements.asset,
            "0x036CbD53842c5426634e7929541eC2318f3dCF7e"
        );
        assert_eq!(
            requirements.pay_to,
            "0x209693Bc6afc0C5328bA36FaF03C514EF312287C"
        );
        assert_eq!(requirements.resource, "https://example.com/test");
        assert_eq!(requirements.description, "Test payment");
    }

    #[test]
    fn test_payment_requirements_usdc_info() {
        let mut requirements = PaymentRequirements::new(
            "exact",
            "base-sepolia",
            "1000000",
            "0x036CbD53842c5426634e7929541eC2318f3dCF7e",
            "0x209693Bc6afc0C5328bA36FaF03C514EF312287C",
            "https://example.com/test",
            "Test payment",
        );

        requirements
            .set_usdc_info(crate::types::Network::Testnet)
            .unwrap();
        assert!(requirements.extra.is_some());

        let extra = requirements.extra.as_ref().unwrap();
        assert_eq!(extra["name"], "USDC");
        assert_eq!(extra["version"], "2");
    }

    #[test]
    fn test_payment_payload_creation() {
        let authorization = ExactEvmPayloadAuthorization::new(
            "0x857b06519E91e3A54538791bDbb0E22373e36b66",
            "0x209693Bc6afc0C5328bA36FaF03C514EF312287C",
            "1000000",
            "1745323800",
            "1745323985",
            "0xf3746613c2d920b5fdabc0856f2aeb2d4f88ee6037b8cc5d04a71a4462f13480",
        );

        let payload = ExactEvmPayload {
            signature: "0x2d6a7588d6acca505cbf0d9a4a227e0c52c6c34008c8e8986a1283259764173608a2ce6496642e377d6da8dbbf5836e9bd15092f9ecab05ded3d6293af148b571c".to_string(),
            authorization,
        };

        let payment_payload = PaymentPayload::new("exact", "base-sepolia", payload);

        assert_eq!(payment_payload.x402_version, X402_VERSION);
        assert_eq!(payment_payload.scheme, "exact");
        assert_eq!(payment_payload.network, "base-sepolia");
    }

    #[test]
    fn test_payment_payload_base64_encoding() {
        let authorization = ExactEvmPayloadAuthorization::new(
            "0x857b06519E91e3A54538791bDbb0E22373e36b66",
            "0x209693Bc6afc0C5328bA36FaF03C514EF312287C",
            "1000000",
            "1745323800",
            "1745323985",
            "0xf3746613c2d920b5fdabc0856f2aeb2d4f88ee6037b8cc5d04a71a4462f13480",
        );

        let payload = ExactEvmPayload {
            signature: "0x2d6a7588d6acca505cbf0d9a4a227e0c52c6c34008c8e8986a1283259764173608a2ce6496642e377d6da8dbbf5836e9bd15092f9ecab05ded3d6293af148b571c".to_string(),
            authorization,
        };

        let payment_payload = PaymentPayload::new("exact", "base-sepolia", payload);
        let encoded = payment_payload.to_base64().unwrap();
        let decoded = PaymentPayload::from_base64(&encoded).unwrap();

        assert_eq!(payment_payload.x402_version, decoded.x402_version);
        assert_eq!(payment_payload.scheme, decoded.scheme);
        assert_eq!(payment_payload.network, decoded.network);
    }

    #[test]
    fn test_authorization_validity() {
        let now = chrono::Utc::now().timestamp();
        let valid_after = (now - 100).to_string();
        let valid_before = (now + 100).to_string();

        let authorization = ExactEvmPayloadAuthorization::new(
            "0x857b06519E91e3A54538791bDbb0E22373e36b66",
            "0x209693Bc6afc0C5328bA36FaF03C514EF312287C",
            "1000000",
            valid_after,
            valid_before,
            "0xf3746613c2d920b5fdabc0856f2aeb2d4f88ee6037b8cc5d04a71a4462f13480",
        );

        assert!(authorization.is_valid_now().unwrap());
    }

    #[test]
    fn test_authorization_expired() {
        let now = chrono::Utc::now().timestamp();
        let valid_after = (now - 200).to_string();
        let valid_before = (now - 100).to_string(); // Expired

        let authorization = ExactEvmPayloadAuthorization::new(
            "0x857b06519E91e3A54538791bDbb0E22373e36b66",
            "0x209693Bc6afc0C5328bA36FaF03C514EF312287C",
            "1000000",
            valid_after,
            valid_before,
            "0xf3746613c2d920b5fdabc0856f2aeb2d4f88ee6037b8cc5d04a71a4462f13480",
        );

        assert!(!authorization.is_valid_now().unwrap());
    }

    #[test]
    fn test_facilitator_config() {
        let config = FacilitatorConfig {
            url: "https://example.com/facilitator".to_string(),
            timeout: Some(std::time::Duration::from_secs(30)),
            create_auth_headers: None,
        };

        assert_eq!(config.url, "https://example.com/facilitator".to_string());
        assert_eq!(config.timeout, Some(std::time::Duration::from_secs(30)));
    }

    #[test]
    fn test_blockchain_facilitator_config() {
        let config = BlockchainFacilitatorConfig {
            rpc_url: Some("https://example.com/facilitator".to_string()),
            network: "base-sepolia".to_string(),
            verification_timeout: std::time::Duration::from_secs(30),
            confirmation_blocks: 1,
            max_retries: 3,
            retry_delay: std::time::Duration::from_secs(1),
        };

        assert_eq!(
            config.rpc_url,
            Some("https://example.com/facilitator".to_string())
        );
        assert_eq!(
            config.verification_timeout,
            std::time::Duration::from_secs(30)
        );
    }

    #[test]
    fn test_networks() {
        assert_eq!(networks::BASE_MAINNET, "base");
        assert_eq!(networks::BASE_SEPOLIA, "base-sepolia");
        assert_eq!(networks::AVALANCHE_MAINNET, "avalanche");
        assert_eq!(networks::AVALANCHE_FUJI, "avalanche-fuji");

        assert!(networks::is_supported("base-sepolia"));
        assert!(networks::is_supported("base"));
        assert!(!networks::is_supported("unsupported-network"));

        assert_eq!(
            networks::get_usdc_address("base-sepolia"),
            Some("0x036CbD53842c5426634e7929541eC2318f3dCF7e")
        );
        assert_eq!(
            networks::get_usdc_address("base"),
            Some("0x833589fCD6eDb6E08f4c7C32D4f71b54bdA02913")
        );
    }

    #[test]
    fn test_schemes() {
        assert_eq!(schemes::EXACT, "exact");
    }
}
