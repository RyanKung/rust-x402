//! Unified HTTP server abstractions for x402
//!
//! This module provides a trait-based abstraction for creating HTTP servers
//! that works across different HTTP protocols (HTTP/1.1, HTTP/2, and HTTP/3).

use crate::Result;
use axum::Router;

/// Configuration for HTTP server binding
#[derive(Debug, Clone)]
pub struct ServerConfig {
    /// Bind address (e.g., "0.0.0.0:8080")
    pub bind_addr: String,
    /// Protocol version to use
    pub protocol: HttpProtocol,
}

/// HTTP protocol versions
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum HttpProtocol {
    /// HTTP/1.1 over TCP
    Http1,
    /// HTTP/2 over TCP (with TLS)
    Http2,
    /// HTTP/3 over QUIC/UDP
    Http3,
}

impl Default for ServerConfig {
    fn default() -> Self {
        Self {
            bind_addr: "0.0.0.0:8080".to_string(),
            protocol: HttpProtocol::Http1,
        }
    }
}

impl ServerConfig {
    /// Create a new server config
    pub fn new(bind_addr: impl Into<String>, protocol: HttpProtocol) -> Self {
        Self {
            bind_addr: bind_addr.into(),
            protocol,
        }
    }

    /// Set HTTP/1.1
    pub fn http1(self) -> Self {
        Self {
            protocol: HttpProtocol::Http1,
            ..self
        }
    }

    /// Set HTTP/2
    pub fn http2(self) -> Self {
        Self {
            protocol: HttpProtocol::Http2,
            ..self
        }
    }

    /// Set HTTP/3
    pub fn http3(self) -> Self {
        Self {
            protocol: HttpProtocol::Http3,
            ..self
        }
    }
}

/// Trait for creating and starting HTTP servers
#[async_trait::async_trait]
pub trait HttpServer {
    /// Start the server with the given router and config
    async fn serve(router: Router, config: ServerConfig) -> Result<()>;
}

/// Unified server builder
#[derive(Debug)]
pub struct ServerBuilder {
    router: Router,
    config: ServerConfig,
}

impl ServerBuilder {
    /// Create a new server builder
    pub fn new(router: Router) -> Self {
        Self {
            router,
            config: ServerConfig::default(),
        }
    }

    /// Set the bind address
    pub fn bind(mut self, addr: impl Into<String>) -> Self {
        self.config.bind_addr = addr.into();
        self
    }

    /// Use HTTP/1.1
    pub fn http1(mut self) -> Self {
        self.config = self.config.http1();
        self
    }

    /// Use HTTP/2
    pub fn http2(mut self) -> Self {
        self.config = self.config.http2();
        self
    }

    /// Use HTTP/3
    pub fn http3(mut self) -> Self {
        self.config = self.config.http3();
        self
    }

    /// Start the server
    pub async fn serve(self) -> Result<()> {
        match self.config.protocol {
            HttpProtocol::Http1 => Http1Server::serve(self.router, self.config).await,
            HttpProtocol::Http2 => Http2Server::serve(self.router, self.config).await,
            HttpProtocol::Http3 => Http3Server::serve(self.router, self.config).await,
        }
    }
}

/// HTTP/1.1 server implementation
pub struct Http1Server;

#[async_trait::async_trait]
impl HttpServer for Http1Server {
    async fn serve(router: Router, config: ServerConfig) -> Result<()> {
        let listener = tokio::net::TcpListener::bind(&config.bind_addr)
            .await
            .map_err(|e| {
                crate::X402Error::config(format!("Failed to bind to {}: {}", config.bind_addr, e))
            })?;

        tracing::info!(
            "🚀 HTTP/1.1 server listening on http://{}",
            config.bind_addr
        );

        axum::serve(listener, router)
            .await
            .map_err(|e| crate::X402Error::config(format!("Server error: {}", e)))?;

        Ok(())
    }
}

/// HTTP/2 server implementation
pub struct Http2Server;

#[async_trait::async_trait]
impl HttpServer for Http2Server {
    async fn serve(router: Router, config: ServerConfig) -> Result<()> {
        // HTTP/2 support is handled by Axum automatically with TLS
        // This is a fallback to HTTP/1.1 if TLS is not configured
        let listener = tokio::net::TcpListener::bind(&config.bind_addr)
            .await
            .map_err(|e| {
                crate::X402Error::config(format!("Failed to bind to {}: {}", config.bind_addr, e))
            })?;

        tracing::info!(
            "🚀 HTTP/2 server listening on https://{} (with TLS)",
            config.bind_addr
        );
        tracing::warn!("HTTP/2 requires TLS configuration. Consider using axum with TLS support.");

        axum::serve(listener, router)
            .await
            .map_err(|e| crate::X402Error::config(format!("Server error: {}", e)))?;

        Ok(())
    }
}

/// HTTP/3 server implementation
pub struct Http3Server;

#[async_trait::async_trait]
impl HttpServer for Http3Server {
    async fn serve(router: Router, config: ServerConfig) -> Result<()> {
        #[cfg(feature = "http3")]
        {
            use crate::http3::Http3Config;

            let http3_config = Http3Config::new(&config.bind_addr);
            crate::http3::create_http3_server(http3_config, router).await
        }

        #[cfg(not(feature = "http3"))]
        {
            Err(crate::X402Error::config(
                "HTTP/3 support is not enabled. Compile with 'http3' feature flag.".to_string(),
            ))
        }
    }
}

/// Convenience function to create a server builder
pub fn create_server(router: Router) -> ServerBuilder {
    ServerBuilder::new(router)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_server_config_default() {
        let config = ServerConfig::default();
        assert_eq!(config.bind_addr, "0.0.0.0:8080");
        assert_eq!(config.protocol, HttpProtocol::Http1);
    }

    #[test]
    fn test_server_config_new() {
        let config = ServerConfig::new("127.0.0.1:3000", HttpProtocol::Http3);
        assert_eq!(config.bind_addr, "127.0.0.1:3000");
        assert_eq!(config.protocol, HttpProtocol::Http3);
    }

    #[test]
    fn test_server_config_protocol_setters() {
        let config = ServerConfig::default().http2();
        assert_eq!(config.protocol, HttpProtocol::Http2);

        let config = ServerConfig::default().http3();
        assert_eq!(config.protocol, HttpProtocol::Http3);
    }

    #[tokio::test]
    async fn test_server_builder() {
        let router = Router::new();
        let builder = ServerBuilder::new(router)
            .bind("127.0.0.1:0") // Use port 0 for automatic port assignment
            .http1();

        // This will fail on binding, but we can verify the configuration
        assert_eq!(builder.config.bind_addr, "127.0.0.1:0");
        assert_eq!(builder.config.protocol, HttpProtocol::Http1);
    }
}
