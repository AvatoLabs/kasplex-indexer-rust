use thiserror::Error;

/// KRC-20 operation error types
#[derive(Error, Debug, Clone, PartialEq)]
pub enum OperationError {
    #[error("Token '{tick}' not found")]
    TokenNotFound { tick: String },
    
    #[error("Token '{tick}' already exists")]
    TokenExists { tick: String },
    
    #[error("Token '{tick}' is reserved for address '{address}'")]
    TokenReserved { tick: String, address: String },
    
    #[error("Token '{tick}' is ignored")]
    TokenIgnored { tick: String },
    
    #[error("Insufficient balance: required {required}, available {available}")]
    InsufficientBalance { required: String, available: String },
    
    #[error("Invalid address: {address}")]
    InvalidAddress { address: String },
    
    #[error("Invalid amount: {amount}")]
    InvalidAmount { amount: String },
    
    #[error("Invalid token name: {tick}")]
    InvalidTokenName { tick: String },
    
    #[error("Fee not enough: required {required}, provided {provided}")]
    InsufficientFee { required: u64, provided: u64 },
    
    #[error("Address '{address}' is blacklisted for token '{tick}'")]
    AddressBlacklisted { address: String, tick: String },
    
    #[error("Operation '{operation}' not supported")]
    UnsupportedOperation { operation: String },
    
    #[error("Only token owner can perform this operation")]
    UnauthorizedOperation,
    
    #[error("Issue amount exceeds maximum supply: {amount} > {max_supply}")]
    ExceedsMaxSupply { amount: String, max_supply: String },
    
    #[error("Configuration error: {message}")]
    ConfigurationError { message: String },
    
    #[error("Storage error: {message}")]
    StorageError { message: String },
    
    #[error("Network error: {message}")]
    NetworkError { message: String },
    
    #[error("Internal error: {message}")]
    InternalError { message: String },
}

impl OperationError {
    /// Get error code for API responses
    pub fn error_code(&self) -> &'static str {
        match self {
            OperationError::TokenNotFound { .. } => "TOKEN_NOT_FOUND",
            OperationError::TokenExists { .. } => "TOKEN_EXISTS",
            OperationError::TokenReserved { .. } => "TOKEN_RESERVED",
            OperationError::TokenIgnored { .. } => "TOKEN_IGNORED",
            OperationError::InsufficientBalance { .. } => "INSUFFICIENT_BALANCE",
            OperationError::InvalidAddress { .. } => "INVALID_ADDRESS",
            OperationError::InvalidAmount { .. } => "INVALID_AMOUNT",
            OperationError::InvalidTokenName { .. } => "INVALID_TOKEN_NAME",
            OperationError::InsufficientFee { .. } => "INSUFFICIENT_FEE",
            OperationError::AddressBlacklisted { .. } => "ADDRESS_BLACKLISTED",
            OperationError::UnsupportedOperation { .. } => "UNSUPPORTED_OPERATION",
            OperationError::UnauthorizedOperation => "UNAUTHORIZED_OPERATION",
            OperationError::ExceedsMaxSupply { .. } => "EXCEEDS_MAX_SUPPLY",
            OperationError::ConfigurationError { .. } => "CONFIGURATION_ERROR",
            OperationError::StorageError { .. } => "STORAGE_ERROR",
            OperationError::NetworkError { .. } => "NETWORK_ERROR",
            OperationError::InternalError { .. } => "INTERNAL_ERROR",
        }
    }
    
    /// Get HTTP status code for API responses
    pub fn http_status_code(&self) -> u16 {
        match self {
            OperationError::TokenNotFound { .. } => 404,
            OperationError::TokenExists { .. } => 409,
            OperationError::TokenReserved { .. } => 403,
            OperationError::TokenIgnored { .. } => 403,
            OperationError::InsufficientBalance { .. } => 400,
            OperationError::InvalidAddress { .. } => 400,
            OperationError::InvalidAmount { .. } => 400,
            OperationError::InvalidTokenName { .. } => 400,
            OperationError::InsufficientFee { .. } => 400,
            OperationError::AddressBlacklisted { .. } => 403,
            OperationError::UnsupportedOperation { .. } => 400,
            OperationError::UnauthorizedOperation => 403,
            OperationError::ExceedsMaxSupply { .. } => 400,
            OperationError::ConfigurationError { .. } => 500,
            OperationError::StorageError { .. } => 500,
            OperationError::NetworkError { .. } => 503,
            OperationError::InternalError { .. } => 500,
        }
    }
}

/// Result type for operations
pub type OperationResult<T> = Result<T, OperationError>;

/// Convert anyhow::Error to OperationError
impl From<anyhow::Error> for OperationError {
    fn from(err: anyhow::Error) -> Self {
        OperationError::InternalError {
            message: err.to_string(),
        }
    }
}

/// Convert std::io::Error to OperationError
impl From<std::io::Error> for OperationError {
    fn from(err: std::io::Error) -> Self {
        OperationError::StorageError {
            message: err.to_string(),
        }
    }
}

/// Convert serde_json::Error to OperationError
impl From<serde_json::Error> for OperationError {
    fn from(err: serde_json::Error) -> Self {
        OperationError::InternalError {
            message: format!("JSON error: {}", err),
        }
    }
}

/// Convert rocksdb::Error to OperationError
impl From<rocksdb::Error> for OperationError {
    fn from(err: rocksdb::Error) -> Self {
        OperationError::StorageError {
            message: err.to_string(),
        }
    }
}
