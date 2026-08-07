//! SCMessenger core-lite error types.
//!
//! Simplified error hierarchy for core messaging operations.

use thiserror::Error;

/// Top-level error for core-lite operations.
#[derive(Error, Debug)]
pub enum MeshError {
    /// Transport layer failure.
    #[error("transport layer failure: {0}")]
    Transport(#[from] TransportError),

    /// Serialization or deserialization failure.
    #[error("serialization failure: {0}")]
    Serialization(#[from] SerializationError),

    /// Peer authentication failed.
    #[error("peer authentication failed: {0}")]
    Auth(String),

    /// Rate limit exceeded for peer.
    #[error("rate limited: {peer_id}")]
    RateLimited { peer_id: String },

    /// Invalid state for the requested operation.
    #[error("invalid state: {0}")]
    InvalidState(String),

    /// Resource not found.
    #[error("not found: {0}")]
    NotFound(String),

    /// Cryptographic operation failed.
    #[error("cryptographic error: {0}")]
    Crypto(String),

    /// Configuration error.
    #[error("configuration error: {0}")]
    Config(String),

    /// Invalid input provided.
    #[error("invalid input: {0}")]
    InvalidInput(String),

    /// Storage operation failed.
    #[error("storage error: {0}")]
    Storage(String),

    /// Identity operation failed.
    #[error("identity error: {0}")]
    Identity(String),

    /// Message operation failed.
    #[error("message error: {0}")]
    Message(String),

    /// Identity not initialized
    #[error("Identity not initialized")]
    NotInitialized,

    /// Service already running
    #[error("Service already running")]
    AlreadyRunning,

    /// Peer is blocked
    #[error("Peer is blocked")]
    Blocked,

    /// Consent required
    #[error("Consent required")]
    ConsentRequired,

    /// Internal error
    #[error("Internal error")]
    Internal,

    /// Data corruption detected
    #[error("Data corruption detected")]
    CorruptionDetected,

    /// Dial self
    #[error("Dial self")]
    DialSelf,

    /// No addresses
    #[error("No addresses")]
    NoAddresses,

    /// Connection limit reached
    #[error("Connection limit reached")]
    ConnectionLimit,

    /// Multiaddress not supported
    #[error("Multiaddress not supported")]
    MultiaddrNotSupported,

    /// IO error
    #[error("IO error")]
    IoError,

    /// Onion routing disabled
    #[error("Onion routing disabled")]
    OnionRoutingDisabled,
}

/// Transport layer errors.
#[derive(Error, Debug)]
pub enum TransportError {
    /// Noise protocol handshake failed.
    #[error("noise handshake failed: {0}")]
    NoiseHandshake(String),

    /// Connection reset by peer.
    #[error("connection reset by peer: {peer_id}")]
    ConnectionReset { peer_id: String },

    /// I/O error during transport operation.
    #[error("I/O error: {0}")]
    Io(#[from] std::io::Error),

    /// Connection timeout.
    #[error("connection timeout: {0}")]
    Timeout(String),

    /// Dial failure.
    #[error("dial failed: {peer_id}: {reason}")]
    DialFailed { peer_id: String, reason: String },

    /// No active connection to peer.
    #[error("not connected to peer: {peer_id}")]
    NotConnected { peer_id: String },

    /// Invalid multiaddress.
    #[error("invalid multiaddress: {0}")]
    InvalidMultiaddr(String),

    /// Transport protocol error.
    #[error("transport protocol error: {0}")]
    ProtocolError(String),
}

/// Serialization and deserialization errors.
#[derive(Error, Debug)]
pub enum SerializationError {
    /// Bincode encoding failed.
    #[error("bincode encode failed: {0}")]
    Encode(#[from] Box<bincode::ErrorKind>),

    /// Schema version not supported.
    #[error("schema version {version} not supported")]
    UnsupportedVersion { version: u16 },

    /// JSON serialization error.
    #[error("JSON error: {0}")]
    Json(#[from] serde_json::Error),

    /// Invalid UTF-8 string.
    #[error("invalid UTF-8: {0}")]
    InvalidUtf8(String),

    /// Unexpected data format.
    #[error("unexpected format: expected {expected}, got {got}")]
    UnexpectedFormat { expected: String, got: String },

    /// Data too large.
    #[error("data too large: {size} bytes (max: {max})")]
    TooLarge { size: usize, max: usize },

    /// Missing required field.
    #[error("missing required field: {0}")]
    MissingField(String),
}

/// Result type alias for mesh operations.
pub type MeshResult<T> = Result<T, MeshError>;

/// Result type alias for transport operations.
pub type TransportResult<T> = Result<T, TransportError>;

/// Result type alias for serialization operations.
pub type SerializationResult<T> = Result<T, SerializationError>;