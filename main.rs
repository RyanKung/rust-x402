//! X402 Facilitator Server
//!
//! A standalone facilitator server for verifying and settling x402 micropayments.
//! This server can be run as a binary for production deployment.
//!
//! ## Storage Backends
//!
//! - **In-Memory**: Default storage (data lost on restart)
//! - **Redis**: Persistent storage (enable with `redis` feature)

use axum::{
    extract::{Query, State},
    http::StatusCode,
    response::Json,
    routing::{get, post},
    Router,
};
use serde::Deserialize;
use std::env;
use std::sync::Arc;

use rand::Rng;
use rust_x402::{
    facilitator_storage::{InMemoryStorage, NonceStorage},
    types::*,
    Result, X402Error,
};

#[cfg(feature = "redis")]
use rust_x402::facilitator_storage::redis_storage::RedisStorage;

/// Facilitator implementation with pluggable storage
#[derive(Debug, Clone)]
struct Facilitator<S: NonceStorage> {
    storage: Arc<S>,
}

impl<S: NonceStorage> Facilitator<S> {
    /// Create a new facilitator with the given storage backend
    fn new(storage: S) -> Self {
        Self {
            storage: Arc::new(storage),
        }
    }

    /// Verify a payment payload
    async fn verify_payment(
        &self,
        payload: &PaymentPayload,
        requirements: &PaymentRequirements,
    ) -> Result<VerifyResponse> {
        // Check if nonce has been used before (replay protection)
        let nonce = &payload.payload.authorization.nonce;
        if self.storage.has_nonce(nonce).await? {
            return Ok(VerifyResponse {
                is_valid: false,
                invalid_reason: Some("nonce_already_used".to_string()),
                payer: Some(payload.payload.authorization.from.clone()),
            });
        }

        // Verify authorization timing
        if !payload.payload.authorization.is_valid_now()? {
            return Ok(VerifyResponse {
                is_valid: false,
                invalid_reason: Some("authorization_expired".to_string()),
                payer: Some(payload.payload.authorization.from.clone()),
            });
        }

        // Verify amount meets requirements
        let payment_amount: u128 = payload
            .payload
            .authorization
            .value
            .parse()
            .map_err(|_| X402Error::invalid_payment_requirements("Invalid payment amount"))?;
        let required_amount: u128 = requirements
            .max_amount_required
            .parse()
            .map_err(|_| X402Error::invalid_payment_requirements("Invalid required amount"))?;

        if payment_amount < required_amount {
            return Ok(VerifyResponse {
                is_valid: false,
                invalid_reason: Some("insufficient_amount".to_string()),
                payer: Some(payload.payload.authorization.from.clone()),
            });
        }

        // Verify recipient matches
        if payload.payload.authorization.to != requirements.pay_to {
            return Ok(VerifyResponse {
                is_valid: false,
                invalid_reason: Some("recipient_mismatch".to_string()),
                payer: Some(payload.payload.authorization.from.clone()),
            });
        }

        // Mark nonce as processed
        self.storage.mark_nonce(nonce).await?;

        Ok(VerifyResponse {
            is_valid: true,
            invalid_reason: None,
            payer: Some(payload.payload.authorization.from.clone()),
        })
    }

    /// Settle a verified payment
    ///
    /// Note: This is a mock implementation that generates a simulated transaction hash.
    /// For production use, integrate with BlockchainFacilitatorClient to perform
    /// real blockchain transactions.
    async fn settle_payment(
        &self,
        payload: &PaymentPayload,
        _requirements: &PaymentRequirements,
    ) -> Result<SettleResponse> {
        // TODO: Integrate with BlockchainFacilitatorClient for real blockchain settlement
        // 1. Call the blockchain to execute the transfer
        // 2. Wait for transaction confirmation
        // 3. Return the transaction hash

        // For now, we'll simulate a successful settlement
        let mock_transaction_hash = format!("0x{:064x}", rand::thread_rng().gen::<u128>());

        Ok(SettleResponse {
            success: true,
            error_reason: None,
            transaction: mock_transaction_hash,
            network: payload.network.clone(),
            payer: Some(payload.payload.authorization.from.clone()),
        })
    }
}

// Type alias for facilitator with in-memory storage
type InMemoryFacilitator = Facilitator<InMemoryStorage>;

#[cfg(feature = "redis")]
type RedisFacilitator = Facilitator<RedisStorage>;

/// Request types for the facilitator API
#[derive(Debug, Deserialize)]
struct VerifyRequest {
    x402_version: u32,
    payment_payload: PaymentPayload,
    payment_requirements: PaymentRequirements,
}

#[derive(Debug, Deserialize)]
struct SettleRequest {
    x402_version: u32,
    payment_payload: PaymentPayload,
    payment_requirements: PaymentRequirements,
}

/// Supported networks query
#[derive(Debug, Deserialize)]
struct SupportedQuery {
    #[serde(default)]
    #[allow(dead_code)]
    format: Option<String>,
}

#[tokio::main]
async fn main() -> std::result::Result<(), Box<dyn std::error::Error>> {
    // Initialize tracing
    tracing_subscriber::fmt::init();

    // Get bind address from environment or use default
    let bind_address = env::var("BIND_ADDRESS").unwrap_or_else(|_| "0.0.0.0:3000".to_string());

    // Get storage backend from environment
    let storage_type = env::var("STORAGE_BACKEND").unwrap_or_else(|_| "memory".to_string());

    let app = if storage_type == "redis" {
        #[cfg(not(feature = "redis"))]
        {
            eprintln!("Error: Redis storage requested but 'redis' feature is not enabled");
            eprintln!("Please build with: cargo build --features redis");
            std::process::exit(1);
        }

        #[cfg(feature = "redis")]
        {
            let redis_url =
                env::var("REDIS_URL").unwrap_or_else(|_| "redis://localhost:6379".to_string());
            let key_prefix = env::var("REDIS_KEY_PREFIX").ok();

            println!("🔴 Using Redis storage: {}", redis_url);
            let storage = RedisStorage::new(&redis_url, key_prefix.as_deref()).await?;
            let facilitator = Facilitator::new(storage);

            Router::new()
                .route("/verify", post(verify_handler_redis))
                .route("/settle", post(settle_handler_redis))
                .route("/supported", get(supported_handler))
                .route("/health", get(health_handler))
                .with_state(facilitator)
        }
    } else {
        println!("💾 Using in-memory storage");
        let storage = InMemoryStorage::new();
        let facilitator = Facilitator::new(storage);

        Router::new()
            .route("/verify", post(verify_handler_memory))
            .route("/settle", post(settle_handler_memory))
            .route("/supported", get(supported_handler))
            .route("/health", get(health_handler))
            .with_state(facilitator)
    };

    // Start the server
    let listener = tokio::net::TcpListener::bind(&bind_address).await?;
    println!(
        "🔧 X402 Facilitator server running on http://{}",
        bind_address
    );
    println!("📋 Available endpoints:");
    println!("   POST /verify - Verify payment authorization");
    println!("   POST /settle - Settle verified payment");
    println!("   GET /supported - Get supported payment schemes");
    println!("   GET /health - Health check endpoint");
    println!("\nEnvironment variables:");
    println!("   BIND_ADDRESS - Server bind address (default: 0.0.0.0:3000)");
    println!("   STORAGE_BACKEND - Storage backend: 'memory' or 'redis' (default: memory)");
    #[cfg(feature = "redis")]
    {
        println!("   REDIS_URL - Redis connection URL (default: redis://localhost:6379)");
        println!("   REDIS_KEY_PREFIX - Redis key prefix (default: x402:nonce:)");
    }

    axum::serve(listener, app).await?;

    Ok(())
}

/// Handle payment verification requests (in-memory storage)
async fn verify_handler_memory(
    State(facilitator): State<InMemoryFacilitator>,
    Json(request): Json<VerifyRequest>,
) -> std::result::Result<Json<VerifyResponse>, StatusCode> {
    if request.x402_version != X402_VERSION {
        return Err(StatusCode::BAD_REQUEST);
    }

    match facilitator
        .verify_payment(&request.payment_payload, &request.payment_requirements)
        .await
    {
        Ok(response) => Ok(Json(response)),
        Err(e) => {
            eprintln!("Verification error: {}", e);
            Err(StatusCode::INTERNAL_SERVER_ERROR)
        }
    }
}

/// Handle payment settlement requests (in-memory storage)
async fn settle_handler_memory(
    State(facilitator): State<InMemoryFacilitator>,
    Json(request): Json<SettleRequest>,
) -> std::result::Result<Json<SettleResponse>, StatusCode> {
    if request.x402_version != X402_VERSION {
        return Err(StatusCode::BAD_REQUEST);
    }

    match facilitator
        .settle_payment(&request.payment_payload, &request.payment_requirements)
        .await
    {
        Ok(response) => Ok(Json(response)),
        Err(e) => {
            eprintln!("Settlement error: {}", e);
            Err(StatusCode::INTERNAL_SERVER_ERROR)
        }
    }
}

#[cfg(feature = "redis")]
async fn verify_handler_redis(
    State(facilitator): State<RedisFacilitator>,
    Json(request): Json<VerifyRequest>,
) -> std::result::Result<Json<VerifyResponse>, StatusCode> {
    if request.x402_version != X402_VERSION {
        return Err(StatusCode::BAD_REQUEST);
    }

    match facilitator
        .verify_payment(&request.payment_payload, &request.payment_requirements)
        .await
    {
        Ok(response) => Ok(Json(response)),
        Err(e) => {
            eprintln!("Verification error: {}", e);
            Err(StatusCode::INTERNAL_SERVER_ERROR)
        }
    }
}

#[cfg(feature = "redis")]
async fn settle_handler_redis(
    State(facilitator): State<RedisFacilitator>,
    Json(request): Json<SettleRequest>,
) -> std::result::Result<Json<SettleResponse>, StatusCode> {
    if request.x402_version != X402_VERSION {
        return Err(StatusCode::BAD_REQUEST);
    }

    match facilitator
        .settle_payment(&request.payment_payload, &request.payment_requirements)
        .await
    {
        Ok(response) => Ok(Json(response)),
        Err(e) => {
            eprintln!("Settlement error: {}", e);
            Err(StatusCode::INTERNAL_SERVER_ERROR)
        }
    }
}

/// Handle supported payment schemes requests
async fn supported_handler(Query(_query): Query<SupportedQuery>) -> Json<SupportedKinds> {
    Json(SupportedKinds {
        kinds: vec![
            SupportedKind {
                x402_version: X402_VERSION,
                scheme: schemes::EXACT.to_string(),
                network: networks::BASE_SEPOLIA.to_string(),
                metadata: None,
            },
            SupportedKind {
                x402_version: X402_VERSION,
                scheme: schemes::EXACT.to_string(),
                network: networks::BASE_MAINNET.to_string(),
                metadata: None,
            },
            SupportedKind {
                x402_version: X402_VERSION,
                scheme: schemes::EXACT.to_string(),
                network: networks::AVALANCHE_FUJI.to_string(),
                metadata: None,
            },
            SupportedKind {
                x402_version: X402_VERSION,
                scheme: schemes::EXACT.to_string(),
                network: networks::AVALANCHE_MAINNET.to_string(),
                metadata: None,
            },
        ],
    })
}

/// Health check endpoint
async fn health_handler() -> Json<serde_json::Value> {
    Json(serde_json::json!({
        "status": "healthy",
        "version": rust_x402::VERSION,
        "x402_version": X402_VERSION,
    }))
}
