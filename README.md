<div align="center">

![x402 Logo](https://raw.githubusercontent.com/RyanKung/rust-x402/master/logo.png)

# x402 Rust Implementation

[![Crates.io](https://img.shields.io/crates/v/rust-x402)](https://crates.io/crates/rust-x402)
[![Documentation](https://docs.rs/rust-x402/badge.svg)](https://docs.rs/rust-x402)
[![License](https://img.shields.io/badge/license-GPL%20v3-blue.svg)](LICENSE)
[![Rust](https://img.shields.io/badge/rust-1.70%2B-orange.svg)](https://www.rust-lang.org)
[![GitHub](https://img.shields.io/github/stars/RyanKung/x402_rs?style=social)](https://github.com/RyanKung/x402_rs)
[![Build Status](https://img.shields.io/github/actions/workflow/status/RyanKung/x402_rs/ci.yml?branch=main)](https://github.com/RyanKung/x402_rs/actions)
[![Coverage](https://img.shields.io/codecov/c/github/RyanKung/x402_rs)](https://codecov.io/gh/RyanKung/x402_rs)
[![Downloads](https://img.shields.io/crates/d/rust-x402)](https://crates.io/crates/rust-x402)
[![Dependencies](https://img.shields.io/librariesio/release/crates/rust-x402)](https://libraries.io/crate/rust-x402)
[![Security](https://img.shields.io/security-headers?url=https%3A%2F%2Fdocs.rs%2Frust-x402)](https://docs.rs/rust-x402)

A **high-performance, type-safe** Rust implementation of the x402 HTTP-native micropayment protocol.

[Features](#features) • [Quick Start](#quick-start) • [Examples](#examples) • [Documentation](#architecture)

</div>

## 📦 Installation

Add this to your `Cargo.toml`:

```toml
[dependencies]
rust-x402 = "0.1.0"
```

## ✨ Features

- 🚀 **HTTP-native micropayments**: Leverage the HTTP 402 status code for payment requirements
- ⛓️ **Blockchain integration**: Support for EIP-3009 token transfers with real wallet integration
- 🌐 **Web framework support**: Middleware for Axum, Actix Web, and Warp
- 💰 **Facilitator integration**: Built-in support for payment verification and settlement
- 🔒 **Type safety**: Strongly typed Rust implementation with comprehensive error handling
- 🧪 **Comprehensive testing**: 62+ tests with 100% pass rate covering all real implementations
- 🏗️ **Real implementations**: Production-ready wallet, blockchain, and facilitator clients

## 🚀 Quick Start

### Creating a Payment Server with Axum

```rust
use axum::{
    extract::State,
    http::StatusCode,
    response::Json,
    routing::get,
    Router,
};
use rust_x402::{
    axum::PaymentMiddleware,
    types::{PaymentRequirements, FacilitatorConfig},
};
use std::sync::Arc;

#[tokio::main]
async fn main() {
    // Create facilitator config
    let facilitator_config = FacilitatorConfig::default();
    
    // Create payment middleware
    let payment_middleware = PaymentMiddleware::new(
        rust_decimal::Decimal::from_str("0.0001").unwrap(),
        "0x209693Bc6afc0C5328bA36FaF03C514EF312287C".to_string(),
    )
    .with_facilitator_config(facilitator_config)
    .with_description("Premium API access".to_string());

    // Create router with payment middleware
    let app = Router::new()
        .route("/joke", get(joke_handler))
        .layer(payment_middleware);

    // Start server
    let listener = tokio::net::TcpListener::bind("0.0.0.0:4021").await.unwrap();
    axum::serve(listener, app).await.unwrap();
}

async fn joke_handler() -> Result<Json<serde_json::Value>, StatusCode> {
    Ok(Json(serde_json::json!({
        "joke": "Why do programmers prefer dark mode? Because light attracts bugs!"
    })))
}
```

### 💳 Making Payments with a Client

```rust
use rust_x402::client::X402Client;
use rust_x402::types::{PaymentPayload, PaymentRequirements};

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let client = X402Client::new();
    
    // Make a request to a protected resource
    let response = client.get("http://localhost:4021/joke").await?;
    
    if response.status() == 402 {
        // Handle payment required response
        let payment_req = response.json::<PaymentRequirements>().await?;
        
        // Create and sign payment payload (implementation depends on wallet integration)
        let payment_payload = create_payment_payload(&payment_req)?;
        
        // Retry request with payment
        let final_response = client
            .get("http://localhost:4021/joke")
            .header("X-PAYMENT", encode_payment_payload(&payment_payload)?)
            .send()
            .await?;
            
        println!("Response: {}", final_response.text().await?);
    }
    
    Ok(())
}
```

## 🏗️ Architecture

The Rust implementation is organized into several modules:

- 📦 **`types`**: Core data structures and type definitions
- 🌐 **`client`**: HTTP client with x402 payment support
- 💰 **`facilitator`**: Payment verification and settlement
- 🔧 **`middleware`**: Web framework middleware implementations
- 🔐 **`crypto`**: Cryptographic utilities for payment signing
- ❌ **`error`**: Comprehensive error handling
- 🏦 **`wallet`**: Real wallet integration with EIP-712 signing
- ⛓️ **`blockchain`**: Blockchain client for network interactions
- 🏭 **`real_facilitator`**: Production-ready facilitator implementation

## 🌐 Supported Web Frameworks

- 🚀 **Axum**: Modern, ergonomic web framework
- ⚡ **Actix Web**: High-performance actor-based framework
- 🪶 **Warp**: Lightweight, composable web server

## ⛓️ Blockchain Support

Currently supports:
- 🏛️ **Base**: Base mainnet and testnet
- ❄️ **Avalanche**: Avalanche mainnet and Fuji testnet
- 📜 **EIP-3009**: Transfer with Authorization standard

## 📚 Examples

See the `examples/` directory for complete working examples:
- 🚀 `axum_server.rs`: Payment server using Axum
- 💳 `client.rs`: Client making payments
- 💰 `facilitator.rs`: Custom facilitator implementation
- 🏦 `real_implementation_demo.rs`: Real wallet and blockchain integration
- 🔐 `real_wallet_integration.rs`: Production-ready wallet integration

## 📊 Testing

- ✅ **62+ tests** with 100% pass rate
- 🧪 **Comprehensive coverage** of all real implementations
- 🔍 **Integration tests** for end-to-end workflows
- 🛡️ **Error handling tests** for robust error scenarios

## 📄 License

Licensed under the Apache License, Version 2.0. See LICENSE for details.
