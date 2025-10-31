//! HTTP/3 support for x402 payments using QUIC protocol
//!
//! This module provides HTTP/3 server and client implementations for x402.
//! HTTP/3 uses QUIC over UDP instead of TCP, providing better performance
//! and connection migration capabilities.

// Common types and configuration - always available regardless of feature flag
mod config {
    /// HTTP/3 server configuration
    #[derive(Debug, Clone)]
    pub struct Http3Config {
        /// UDP bind address
        pub bind_addr: String,
        /// Certificate path (PEM format)
        pub cert_path: Option<String>,
        /// Private key path (PEM format)
        pub key_path: Option<String>,
        /// Maximum concurrent bidirectional streams
        pub max_concurrent_bidi_streams: u32,
        /// Maximum concurrent unidirectional streams
        pub max_concurrent_uni_streams: u32,
        /// Connection idle timeout in seconds
        pub max_idle_timeout_secs: u64,
    }

    impl Default for Http3Config {
        fn default() -> Self {
            Self {
                bind_addr: "0.0.0.0:4433".to_string(),
                cert_path: None,
                key_path: None,
                max_concurrent_bidi_streams: 100,
                max_concurrent_uni_streams: 100,
                max_idle_timeout_secs: 60,
            }
        }
    }

    impl Http3Config {
        /// Create a new HTTP/3 config
        pub fn new(bind_addr: impl Into<String>) -> Self {
            Self {
                bind_addr: bind_addr.into(),
                ..Default::default()
            }
        }

        /// Set certificate and key paths
        pub fn with_tls(
            mut self,
            cert_path: impl Into<String>,
            key_path: impl Into<String>,
        ) -> Self {
            self.cert_path = Some(cert_path.into());
            self.key_path = Some(key_path.into());
            self
        }

        /// Set maximum concurrent bidirectional streams
        pub fn with_max_concurrent_bidi_streams(mut self, max: u32) -> Self {
            self.max_concurrent_bidi_streams = max;
            self
        }

        /// Set maximum concurrent unidirectional streams
        pub fn with_max_concurrent_uni_streams(mut self, max: u32) -> Self {
            self.max_concurrent_uni_streams = max;
            self
        }

        /// Set connection idle timeout in seconds
        pub fn with_max_idle_timeout(mut self, timeout_secs: u64) -> Self {
            self.max_idle_timeout_secs = timeout_secs;
            self
        }
    }

    #[cfg(test)]
    mod tests {
        use super::*;

        #[test]
        fn test_http3_config_default() {
            let config = Http3Config::default();
            assert_eq!(config.bind_addr, "0.0.0.0:4433");
            assert_eq!(config.max_concurrent_bidi_streams, 100);
            assert_eq!(config.max_concurrent_uni_streams, 100);
            assert_eq!(config.max_idle_timeout_secs, 60);
        }

        #[test]
        fn test_http3_config_new() {
            let config = Http3Config::new("127.0.0.1:8080");
            assert_eq!(config.bind_addr, "127.0.0.1:8080");
        }

        #[test]
        fn test_http3_config_with_tls() {
            let config = Http3Config::new("127.0.0.1:8080")
                .with_tls("/path/to/cert.pem", "/path/to/key.pem");
            assert_eq!(config.cert_path, Some("/path/to/cert.pem".to_string()));
            assert_eq!(config.key_path, Some("/path/to/key.pem".to_string()));
        }

        #[test]
        fn test_http3_config_stream_limits() {
            let config = Http3Config::default()
                .with_max_concurrent_bidi_streams(200)
                .with_max_concurrent_uni_streams(150);
            assert_eq!(config.max_concurrent_bidi_streams, 200);
            assert_eq!(config.max_concurrent_uni_streams, 150);
        }

        #[test]
        fn test_http3_config_timeout() {
            let config = Http3Config::default().with_max_idle_timeout(120);
            assert_eq!(config.max_idle_timeout_secs, 120);
        }
    }
}

#[cfg(feature = "http3")]
mod implementation {
    use crate::Result;
    use axum::Router;
    use bytes::Bytes;
    use h3::client::{builder, SendRequest};
    use h3::server::RequestResolver;
    use h3_axum::serve_h3_with_axum;
    use h3_quinn::Connection as H3Connection;
    use quinn::crypto::rustls::{QuicClientConfig, QuicServerConfig};
    use quinn::{Connection, Endpoint, Incoming};
    use rcgen::generate_simple_self_signed;
    use rustls::pki_types::{CertificateDer, PrivateKeyDer};
    use rustls::{ClientConfig, RootCertStore, ServerConfig};
    use std::marker::PhantomData;
    use std::{
        net::{SocketAddr, ToSocketAddrs},
        sync::Arc,
        time::Duration,
    };

    use super::config::Http3Config;

    // Common HTTP/3 constants
    const ALPN_PROTOCOL: &[u8] = b"h3";
    const DEFAULT_SNI: &str = "localhost";
    const DEFAULT_CLIENT_BIND: &str = "0.0.0.0:0";

    /// Configure TLS with ALPN for HTTP/3
    fn configure_tls_alpn(tls_config: &mut rustls::ServerConfig) {
        tls_config.alpn_protocols = vec![ALPN_PROTOCOL.to_vec()];
        tls_config.max_early_data_size = u32::MAX;
    }

    /// Configure client TLS with ALPN for HTTP/3
    fn configure_client_tls_alpn(client_config: &mut ClientConfig) {
        client_config.alpn_protocols = vec![ALPN_PROTOCOL.to_vec()];
    }

    /// Convert rustls error to X402Error
    fn quic_config_error(e: impl std::fmt::Display) -> crate::X402Error {
        crate::X402Error::config(format!("Failed to create QUIC config: {}", e))
    }

    /// Convert network error to X402Error
    fn network_config_error(msg: impl Into<String>) -> crate::X402Error {
        crate::X402Error::network_error(msg)
    }

    /// Create an HTTP/3 server with x402 payment middleware
    ///
    /// This function starts an HTTP/3 server using QUIC protocol over UDP.
    /// It accepts the axum Router and serves it over HTTP/3.
    ///
    /// # Example
    ///
    /// ```no_run
    /// use axum::Router;
    /// use rust_x402::http3::{create_http3_server, Http3Config};
    ///
    /// # #[cfg(feature = "http3")]
    /// # async fn example() -> rust_x402::Result<()> {
    /// let app = Router::new(); // Your axum router
    /// let config = Http3Config::new("127.0.0.1:4433");
    /// create_http3_server(config, app).await?;
    /// # Ok(())
    /// # }
    /// ```
    pub async fn create_http3_server(config: Http3Config, router: Router) -> Result<()> {
        // Generate or load TLS certificate
        let (certs, key) = load_certificate(&config)?;

        // Configure TLS with ALPN for HTTP/3
        let mut tls_config = ServerConfig::builder()
            .with_no_client_auth()
            .with_single_cert(certs, key)?;

        configure_tls_alpn(&mut tls_config);

        // Configure QUIC transport
        let mut server_config = quinn::ServerConfig::with_crypto(Arc::new(
            QuicServerConfig::try_from(tls_config).map_err(quic_config_error)?,
        ));

        // Configure QUIC transport parameters
        let transport_config = Arc::get_mut(&mut server_config.transport).unwrap();
        transport_config
            .max_concurrent_bidi_streams(config.max_concurrent_bidi_streams.into())
            .max_concurrent_uni_streams(config.max_concurrent_uni_streams.into());

        // Set timeout - this can fail with VarIntBoundsExceeded
        if let Ok(timeout) = Duration::from_secs(config.max_idle_timeout_secs).try_into() {
            transport_config.max_idle_timeout(Some(timeout));
        }

        // Bind and listen
        let addr: SocketAddr = config.bind_addr.parse().map_err(|e| {
            crate::X402Error::config(format!("Invalid bind address: {}: {}", config.bind_addr, e))
        })?;
        let endpoint = Endpoint::server(server_config, addr)?;

        tracing::info!("🚀 HTTP/3 server listening on https://{}", addr);

        // Accept connections
        while let Some(incoming) = endpoint.accept().await {
            let router = router.clone();
            tokio::spawn(async move {
                if let Err(e) = handle_connection(incoming, router).await {
                    tracing::error!("Connection error: {}", e);
                }
            });
        }

        Ok(())
    }

    /// Handle an incoming HTTP/3 connection
    async fn handle_connection(incoming: Incoming, router: Router) -> Result<()> {
        let conn = incoming.await?;
        let remote_addr = conn.remote_address();

        tracing::debug!("New HTTP/3 connection from {}", remote_addr);

        // Build H3 connection
        let h3_conn = h3::server::builder().build(H3Connection::new(conn)).await?;

        tokio::pin!(h3_conn);

        // Accept H3 requests
        loop {
            match h3_conn.accept().await {
                Ok(Some(resolver)) => {
                    let router = router.clone();
                    tokio::spawn(async move {
                        if let Err(e) = handle_request(resolver, router).await {
                            tracing::error!("Request error: {}", e);
                        }
                    });
                }
                Ok(None) => {
                    tracing::debug!("Connection closed by peer: {}", remote_addr);
                    break;
                }
                Err(e) => {
                    // Distinguish graceful closes from errors
                    if h3_axum::is_graceful_h3_close(&e) {
                        tracing::debug!("Connection closed gracefully: {}", remote_addr);
                    } else {
                        tracing::error!("H3 connection error: {:?}", e);
                    }
                    break;
                }
            }
        }

        Ok(())
    }

    /// Handle an individual HTTP/3 request
    async fn handle_request(
        resolver: RequestResolver<H3Connection, Bytes>,
        router: Router,
    ) -> Result<()> {
        // Use h3-axum to serve Axum over HTTP/3
        serve_h3_with_axum(router, resolver)
            .await
            .map_err(|e| crate::X402Error::unexpected(format!("HTTP/3 request error: {}", e)))
    }

    /// Load or generate TLS certificate
    fn load_certificate(
        config: &Http3Config,
    ) -> Result<(Vec<CertificateDer<'static>>, PrivateKeyDer<'static>)> {
        // If custom certificate paths are provided, load them
        if let (Some(_cert_path), Some(_key_path)) = (&config.cert_path, &config.key_path) {
            // TODO: Load custom certificates from files
            // For now, fall back to self-signed certificate
            tracing::warn!(
                "Custom certificate loading not yet implemented, generating self-signed certificate"
            );
            generate_self_signed_cert()
        } else {
            // Generate self-signed certificate for development
            generate_self_signed_cert()
        }
    }

    /// Generate a self-signed certificate for development/testing
    fn generate_self_signed_cert() -> Result<(Vec<CertificateDer<'static>>, PrivateKeyDer<'static>)>
    {
        let cert = generate_simple_self_signed(vec!["localhost".into()])?;
        let key = PrivateKeyDer::Pkcs8(cert.key_pair.serialize_der().into());
        let cert = CertificateDer::from(cert.cert);

        Ok((vec![cert], key))
    }

    /// HTTP/3 client for making requests
    ///
    /// This client provides basic HTTP/3 functionality using QUIC protocol.
    /// For production use, consider using a more complete HTTP/3 client library.
    #[derive(Debug)]
    pub struct Http3Client {
        _phantom: PhantomData<()>,
    }

    impl Http3Client {
        /// Create a new HTTP/3 client
        pub fn new() -> Result<Self> {
            Ok(Self {
                _phantom: PhantomData,
            })
        }

        /// Create an HTTP/3 client with custom configuration
        pub fn with_config(_config: Http3Config) -> Result<Self> {
            Ok(Self {
                _phantom: PhantomData,
            })
        }

        /// Connect to an HTTP/3 server and establish a connection
        ///
        /// Returns a SendRequest that can be used to make HTTP requests.
        ///
        /// # Example
        ///
        /// ```no_run
        /// # use std::result::Result;
        /// use http::Request;
        /// use rust_x402::http3::Http3Client;
        ///
        /// # async fn example() -> Result<(), Box<dyn std::error::Error>> {
        /// let client = Http3Client::new()?;
        /// let (_conn, mut send_request) = client.connect("127.0.0.1:4433").await?;
        ///
        /// let request = Request::get("https://127.0.0.1/test").body(()).map_err(|e| format!("Invalid request: {}", e))?;
        /// let mut stream = send_request.send_request(request).await.map_err(|e| format!("Failed to send: {}", e))?;
        /// stream.finish().await.map_err(|e| format!("Failed to finish: {}", e))?;
        /// let _response = stream.recv_response().await.map_err(|e| format!("Failed to receive: {}", e))?;
        /// # Ok(())
        /// # }
        /// ```
        pub async fn connect(
            &self,
            remote: impl ToSocketAddrs + 'static + Send + Sync + Clone,
        ) -> Result<(
            Connection,
            SendRequest<<H3Connection as h3::quic::Connection<bytes::Bytes>>::OpenStreams, Bytes>,
        )> {
            // TODO: Support loading CA certificates
            let roots = RootCertStore::empty();
            // For development, accept self-signed certificates by leaving empty
            // In production, load proper CA certificates

            let mut client_crypto = ClientConfig::builder()
                .with_root_certificates(Arc::new(roots))
                .with_no_client_auth();

            configure_client_tls_alpn(&mut client_crypto);

            let client_config = quinn::ClientConfig::new(Arc::new(
                QuicClientConfig::try_from(client_crypto).map_err(quic_config_error)?,
            ));

            // Bind to any available UDP port
            let bind_addr: SocketAddr = DEFAULT_CLIENT_BIND.parse().unwrap();
            let mut endpoint = Endpoint::client(bind_addr)?;
            endpoint.set_default_client_config(client_config);

            // Connect to remote
            let remote_addr = remote
                .to_socket_addrs()
                .map_err(|e| network_config_error(format!("Failed to resolve address: {}", e)))?
                .next()
                .ok_or_else(|| network_config_error("No address found"))?;

            let conn = endpoint
                .connect(remote_addr, DEFAULT_SNI)
                .map_err(|e| network_config_error(format!("Failed to initiate connection: {}", e)))?
                .await?;

            // Build HTTP/3 client
            let h3_conn = builder()
                .max_field_section_size(8192)
                .build(H3Connection::new(conn.clone()))
                .await
                .map_err(|e| {
                    network_config_error(format!("Failed to build H3 connection: {}", e))
                })?;

            Ok((conn.clone(), h3_conn.1))
        }
    }

    impl Clone for Http3Client {
        fn clone(&self) -> Self {
            Self {
                _phantom: PhantomData,
            }
        }
    }

    impl Default for Http3Client {
        fn default() -> Self {
            Self {
                _phantom: PhantomData,
            }
        }
    }

    #[cfg(test)]
    mod tests {
        use super::*;

        #[test]
        fn test_http3_client_creation() {
            let _client = Http3Client::new();
        }

        #[test]
        fn test_http3_client_with_config() {
            let config = Http3Config::default();
            let _client = Http3Client::with_config(config);
        }
    }
}

#[cfg(not(feature = "http3"))]
mod implementation {
    use super::config::Http3Config;
    use crate::{Result, X402Error};
    use axum::Router;

    /// Create an HTTP/3 server with x402 payment middleware
    pub async fn create_http3_server(_config: Http3Config, _router: Router) -> Result<()> {
        Err(X402Error::config(
            "HTTP/3 support is not enabled. Compile with 'http3' feature flag.",
        ))
    }

    /// HTTP/3 client for making requests
    #[derive(Debug, Clone, Default)]
    pub struct Http3Client;

    impl Http3Client {
        /// Create a new HTTP/3 client
        pub fn new() -> Self {
            Self::default()
        }

        /// Create an HTTP/3 client with custom configuration
        pub fn with_config(_config: Http3Config) -> Self {
            Self::default()
        }
    }

    #[cfg(test)]
    mod tests {
        use super::*;

        #[test]
        fn test_http3_client_creation() {
            let _client = Http3Client::new();
        }
    }
}

// Re-export common types
pub use config::Http3Config;

// Re-export implementation-specific types and functions
pub use implementation::*;
