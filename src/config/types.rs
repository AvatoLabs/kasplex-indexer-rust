use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct StartupConfig {
    pub hysteresis: u32,
    #[serde(rename = "daaScoreRange")]
    pub daa_score_range: Vec<[u64; 2]>,
    #[serde(rename = "tickReserved")]
    pub tick_reserved: Vec<String>,
    #[serde(rename = "kaspaNodeURL")]
    pub kaspa_node_url: String,
    #[serde(rename = "isTestnet")]
    pub is_testnet: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RocksConfig {
    pub path: String,
}

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct RestConfig {
    /// Kaspa public REST base URL
    #[serde(
        rename = "kaspaRestBaseURL",
        default = "default_kaspa_rest_base_url"
    )]
    pub kaspa_rest_base_url: String,
}

fn default_kaspa_rest_base_url() -> String { "https://api-tn10.kaspa.org".to_string() }

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct HttpConfig {
    /// Listen address
    #[serde(default = "default_bind_addr")]
    pub bind: String,
    /// Listen port
    #[serde(default = "default_http_port")]
    pub port: u16,
}

fn default_bind_addr() -> String {
    "0.0.0.0".to_string()
}
fn default_http_port() -> u16 {
    8080
}

/// Distributed storage node configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DistributedNodeConfig {
    /// Node ID
    #[serde(rename = "nodeId")]
    pub node_id: String,
    /// Data directory
    #[serde(rename = "dataDir")]
    pub data_dir: String,
    /// Number of shards
    #[serde(rename = "shardCount")]
    pub shard_count: u32,
    /// Replication factor
    #[serde(rename = "replicationFactor")]
    pub replication_factor: u32,
    /// List of other node addresses
    pub nodes: Vec<String>,
    /// Whether to enable distributed mode
    pub enabled: bool,
    /// Node role (primary, replica, standalone)
    pub role: String,
    /// Listen port
    pub port: u16,
    /// Maximum number of connections
    #[serde(rename = "maxConnections")]
    pub max_connections: u32,
}

/// Shard configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ShardConfig {
    /// Shard ID
    #[serde(rename = "shardId")]
    pub shard_id: u32,
    /// Shard data directory
    #[serde(rename = "dataDir")]
    pub data_dir: String,
    /// Whether this is the primary shard
    #[serde(rename = "isPrimary")]
    pub is_primary: bool,
    /// List of replica nodes
    pub replicas: Vec<String>,
    /// Shard weight
    pub weight: f64,
    /// Maximum data size (MB)
    #[serde(rename = "maxDataSize")]
    pub max_data_size: u64,
}

/// Replication configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ReplicationConfig {
    /// Replication strategy (sync, async, semi-sync)
    pub strategy: String,
    /// Replication timeout (seconds)
    pub timeout: u64,
    /// Maximum number of retries
    #[serde(rename = "maxRetries")]
    pub max_retries: u32,
    /// Retry interval (seconds)
    #[serde(rename = "retryInterval")]
    pub retry_interval: u64,
    /// Whether to enable compression
    #[serde(rename = "enableCompression")]
    pub enable_compression: bool,
    /// Compression level (1-9)
    #[serde(rename = "compressionLevel")]
    pub compression_level: u8,
}

/// Consistent hashing configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct HashRingConfig {
    /// Number of virtual nodes
    #[serde(rename = "virtualNodes")]
    pub virtual_nodes: u32,
    /// Hash algorithm (blake3, sha256, md5)
    #[serde(rename = "hashAlgorithm")]
    pub hash_algorithm: String,
    /// Whether to enable consistent hashing
    pub enabled: bool,
    /// Hash ring size
    #[serde(rename = "ringSize")]
    pub ring_size: u32,
}

/// Performance configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PerformanceConfig {
    /// Write buffer size (MB)
    #[serde(rename = "writeBufferSize")]
    pub write_buffer_size: u64,
    /// Maximum number of write buffers
    #[serde(rename = "maxWriteBufferNumber")]
    pub max_write_buffer_number: u32,
    /// Target file size (MB)
    #[serde(rename = "targetFileSizeBase")]
    pub target_file_size_base: u64,
    /// Maximum number of background jobs
    #[serde(rename = "maxBackgroundJobs")]
    pub max_background_jobs: u32,
    /// Maximum number of open files
    #[serde(rename = "maxOpenFiles")]
    pub max_open_files: u32,
    /// Whether to enable compression
    #[serde(rename = "enableCompression")]
    pub enable_compression: bool,
    /// Compression type (snappy, lz4, zstd)
    #[serde(rename = "compressionType")]
    pub compression_type: String,
    /// Batch write size
    #[serde(rename = "batchSize")]
    pub batch_size: u32,
    /// Number of concurrent reads
    #[serde(rename = "concurrentReads")]
    pub concurrent_reads: u32,
    /// Number of concurrent writes
    #[serde(rename = "concurrentWrites")]
    pub concurrent_writes: u32,
}

/// Monitoring configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MonitoringConfig {
    /// Whether to enable monitoring
    pub enabled: bool,
    /// Health check interval (seconds)
    #[serde(rename = "healthCheckInterval")]
    pub health_check_interval: u64,
    /// Metrics collection interval (seconds)
    #[serde(rename = "metricsInterval")]
    pub metrics_interval: u64,
    /// Log level (debug, info, warn, error)
    #[serde(rename = "logLevel")]
    pub log_level: String,
    /// Whether to enable performance metrics
    #[serde(rename = "enableMetrics")]
    pub enable_metrics: bool,
    /// Whether to enable distributed tracing
    #[serde(rename = "enableTracing")]
    pub enable_tracing: bool,
}

/// Security configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SecurityConfig {
    /// Whether to enable authentication
    #[serde(rename = "enableAuth")]
    pub enable_auth: bool,
    /// Authentication key
    #[serde(rename = "authKey")]
    pub auth_key: String,
    /// Whether to enable encryption
    #[serde(rename = "enableEncryption")]
    pub enable_encryption: bool,
    /// Encryption key
    #[serde(rename = "encryptionKey")]
    pub encryption_key: String,
    /// Whether to enable SSL/TLS
    #[serde(rename = "enableSSL")]
    pub enable_ssl: bool,
    /// SSL certificate path
    #[serde(rename = "sslCertPath")]
    pub ssl_cert_path: String,
    /// SSL private key path
    #[serde(rename = "sslKeyPath")]
    pub ssl_key_path: String,
}

/// Complete distributed storage configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DistributedConfig {
    /// Node configuration
    pub node: DistributedNodeConfig,
    /// List of shard configurations
    pub shards: Vec<ShardConfig>,
    /// Replication configuration
    pub replication: ReplicationConfig,
    /// Consistent hashing configuration
    #[serde(rename = "hashRing")]
    pub hash_ring: HashRingConfig,
    /// Performance configuration
    pub performance: PerformanceConfig,
    /// Monitoring configuration
    pub monitoring: MonitoringConfig,
    /// Security configuration
    pub security: SecurityConfig,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Config {
    pub startup: StartupConfig,
    pub rocksdb: RocksConfig,
    pub distributed: DistributedConfig,
    #[serde(default)]
    pub http: HttpConfig,
    #[serde(default)]
    pub rest: RestConfig,
    pub debug: u8,
    pub testnet: bool,
    #[serde(rename = "isTestnet")]
    pub is_testnet: bool,
}

impl Default for StartupConfig {
    fn default() -> Self {
        Self {
            hysteresis: 3,
            daa_score_range: vec![],
            tick_reserved: vec![],
            kaspa_node_url: "http://localhost:16110".to_string(),
            is_testnet: false,
        }
    }
}

impl Default for RocksConfig {
    fn default() -> Self {
        Self {
            path: "./data".to_string(),
        }
    }
}

impl Default for HttpConfig {
    fn default() -> Self {
        Self {
            bind: default_bind_addr(),
            port: default_http_port(),
        }
    }
}

impl Default for DistributedNodeConfig {
    fn default() -> Self {
        Self {
            node_id: "node_1".to_string(),
            data_dir: "./data/distributed".to_string(),
            shard_count: 8,
            replication_factor: 3,
            nodes: vec![],
            enabled: true,
            role: "primary".to_string(),
            port: 8080,
            max_connections: 1000,
        }
    }
}

impl Default for ShardConfig {
    fn default() -> Self {
        Self {
            shard_id: 0,
            data_dir: "./data/shard_0".to_string(),
            is_primary: true,
            replicas: vec![],
            weight: 1.0,
            max_data_size: 1024, // 1GB
        }
    }
}

impl Default for ReplicationConfig {
    fn default() -> Self {
        Self {
            strategy: "async".to_string(),
            timeout: 30,
            max_retries: 3,
            retry_interval: 5,
            enable_compression: true,
            compression_level: 6,
        }
    }
}

impl Default for HashRingConfig {
    fn default() -> Self {
        Self {
            virtual_nodes: 150,
            hash_algorithm: "blake3".to_string(),
            enabled: true,
            ring_size: 1024,
        }
    }
}

impl Default for PerformanceConfig {
    fn default() -> Self {
        Self {
            write_buffer_size: 64, // 64MB
            max_write_buffer_number: 3,
            target_file_size_base: 64, // 64MB
            max_background_jobs: 4,
            max_open_files: 1000,
            enable_compression: true,
            compression_type: "lz4".to_string(),
            batch_size: 1000,
            concurrent_reads: 8,
            concurrent_writes: 4,
        }
    }
}

impl Default for MonitoringConfig {
    fn default() -> Self {
        Self {
            enabled: true,
            health_check_interval: 30,
            metrics_interval: 60,
            log_level: "info".to_string(),
            enable_metrics: true,
            enable_tracing: false,
        }
    }
}

impl Default for SecurityConfig {
    fn default() -> Self {
        Self {
            enable_auth: false,
            auth_key: "".to_string(),
            enable_encryption: false,
            encryption_key: "".to_string(),
            enable_ssl: false,
            ssl_cert_path: "".to_string(),
            ssl_key_path: "".to_string(),
        }
    }
}

impl Default for DistributedConfig {
    fn default() -> Self {
        Self {
            node: DistributedNodeConfig::default(),
            shards: vec![ShardConfig::default()],
            replication: ReplicationConfig::default(),
            hash_ring: HashRingConfig::default(),
            performance: PerformanceConfig::default(),
            monitoring: MonitoringConfig::default(),
            security: SecurityConfig::default(),
        }
    }
}

impl Default for Config {
    fn default() -> Self {
        Self {
            startup: StartupConfig::default(),
            rocksdb: RocksConfig::default(),
            distributed: DistributedConfig::default(),
            http: HttpConfig::default(),
            rest: RestConfig::default(),
            debug: 2,
            testnet: false,
            is_testnet: false,
        }
    }
}
