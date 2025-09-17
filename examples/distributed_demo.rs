use anyhow::Result;
use kaspa_indexer_rust::config::types::{
    Config, DistributedConfig, DistributedNodeConfig, HashRingConfig, ReplicationConfig,
    ShardConfig,
};
use kaspa_indexer_rust::storage::StorageManager;
use tracing::{info, warn};

#[tokio::main]
async fn main() -> Result<()> {
    tracing_subscriber::fmt::init();

    info!("🚀 Distributed storage demo started");

    // Create default configuration
    let config = Config {
        distributed: DistributedConfig {
            node: DistributedNodeConfig {
                node_id: "node_1".to_string(),
                data_dir: "./data/distributed".to_string(),
                shard_count: 8,
                replication_factor: 2,
                nodes: vec![],
                enabled: true,
                role: "primary".to_string(),
                port: 8080,
                max_connections: 1000,
            },
            shards: vec![
                ShardConfig {
                    shard_id: 0,
                    data_dir: "./data/distributed/shard_0".to_string(),
                    is_primary: true,
                    replicas: vec![],
                    weight: 1.0,
                    max_data_size: 1024,
                },
                ShardConfig {
                    shard_id: 1,
                    data_dir: "./data/distributed/shard_1".to_string(),
                    is_primary: false,
                    replicas: vec![],
                    weight: 1.0,
                    max_data_size: 1024,
                },
            ],
            hash_ring: HashRingConfig {
                virtual_nodes: 150,
                hash_algorithm: "blake3".to_string(),
                enabled: true,
                ring_size: 1024,
            },
            replication: ReplicationConfig {
                strategy: "async".to_string(),
                timeout: 30,
                max_retries: 3,
                retry_interval: 5,
                enable_compression: true,
                compression_level: 6,
            },
            performance: Default::default(),
            monitoring: Default::default(),
            security: Default::default(),
        },
        ..Default::default()
    };

    info!("✅ Using default distributed configuration");

    // Display distributed configuration information
    info!("📊 Distributed configuration information:");
    info!("  Node ID: {}", config.distributed.node.node_id);
    info!("  Data directory: {}", config.distributed.node.data_dir);
    info!("  Shard count: {}", config.distributed.shards.len());
    info!("  Replication factor: {}", config.distributed.node.replication_factor);
    info!(
        "  Hash algorithm: {}",
        config.distributed.hash_ring.hash_algorithm
    );
    info!(
        "  Virtual nodes: {}",
        config.distributed.hash_ring.virtual_nodes
    );
    info!("  Replication strategy: {}", config.distributed.replication.strategy);

    // Initialize storage manager
    let storage = StorageManager::new(config.rocksdb, Some(config.distributed)).await?;

    info!("✅ Storage manager initialized successfully");

    // Check if distributed mode is enabled
    if storage.is_distributed_enabled() {
        info!("🎯 Distributed mode enabled");

        // Get shard information
        let shard_info = storage.get_shard_info();
        info!("📋 Shard information:");
        for shard in shard_info {
            info!(
                "  Shard {}: {} (Primary: {})",
                shard.shard_id, shard.data_dir, shard.is_primary
            );
        }

        // Get hash ring information
        let hash_ring_info = storage.get_hash_ring_info();
        if !hash_ring_info.is_empty() {
            info!("🔄 Hash ring information:");
            info!("  Shard count: {}", hash_ring_info.len());
            for (shard_id, virtual_nodes) in &hash_ring_info {
                info!("  Shard {}: {} virtual nodes", shard_id, virtual_nodes);
            }
        } else {
            info!("🔄 No hash ring information");
        }

        // Execute some distributed operations
        info!("🔧 Executing distributed operations test...");

        // Insert data
        let test_data = vec![
            ("user:1", b"Alice".to_vec()),
            ("user:2", b"Bob".to_vec()),
            ("user:3", b"Charlie".to_vec()),
            ("token:KAS", b"Kaspa Token".to_vec()),
            ("token:KSP", b"Kaspa Script".to_vec()),
            ("balance:addr1:KAS", b"1000".to_vec()),
            ("balance:addr2:KSP", b"500".to_vec()),
        ];

        for (key, value) in &test_data {
            match storage.distributed_insert(key, value).await {
                Ok(_) => info!("✅ Insert successful: {}", key),
                Err(e) => warn!("❌ Insert failed: {} - {}", key, e),
            }
        }

        // Read data
        info!("📖 Reading data test...");
        for (key, _) in &test_data {
            match storage.distributed_get(key).await {
                Ok(Some(value)) => info!(
                    "✅ Read successful: {} = {:?}",
                    key,
                    String::from_utf8_lossy(&value)
                ),
                Ok(None) => warn!("⚠️ Data does not exist: {}", key),
                Err(e) => warn!("❌ Read failed: {} - {}", key, e),
            }
        }

        // Health check
        info!("🏥 Executing health check...");
        match storage.health_check().await {
            Ok(health) => {
                info!(
                    "Health status: {}",
                    if health.is_healthy {
                        "✅ Healthy"
                    } else {
                        "❌ Unhealthy"
                    }
                );
                info!(
                    "Healthy shards: {}/{}",
                    health.healthy_shards, health.total_shards
                );
                info!("Health ratio: {:.2}%", health.health_ratio * 100.0);

                // Display shard details
                for (shard_id, shard_health) in health.shard_details {
                    info!(
                        "  Shard {}: {} (Response time: {:?})",
                        shard_id,
                        if shard_health.is_healthy {
                            "✅"
                        } else {
                            "❌"
                        },
                        shard_health.response_time
                    );
                }
            }
            Err(e) => warn!("❌ Health check failed: {}", e),
        }

        // Get metrics
        if let Some(metrics) = storage.get_distributed_metrics().await {
            info!("📈 Performance metrics:");
            info!("  Total operations: {}", metrics.total_operations);
            info!("  Successful operations: {}", metrics.successful_operations);
            info!("  Failed operations: {}", metrics.failed_operations);
            info!("  Replication operations: {}", metrics.replication_operations);
            info!("  Replication failures: {}", metrics.replication_failures);
            info!("  Average response time: {:?}", metrics.average_response_time);
        }

        // Get configuration information
        if let Some(distributed_config) = storage.get_distributed_config() {
            info!("⚙️ Current configuration:");
            info!("  Node role: {}", distributed_config.node.role);
            info!("  Listen port: {}", distributed_config.node.port);
            info!("  Max connections: {}", distributed_config.node.max_connections);
            info!("  Monitoring enabled: {}", distributed_config.monitoring.enabled);
            info!("  Security authentication: {}", distributed_config.security.enable_auth);
        }
    } else {
        info!("⚠️ Distributed mode not enabled, using standalone mode");

        // Standalone mode test
        info!("🔧 Executing standalone operations test...");

        // Insert data
        match storage.distributed_insert("test_key", b"test_value").await {
            Ok(_) => info!("✅ Standalone insert successful"),
            Err(e) => warn!("❌ Standalone insert failed: {}", e),
        }

        // Read data
        match storage.distributed_get("test_key").await {
            Ok(Some(value)) => info!("✅ Standalone read successful: {:?}", String::from_utf8_lossy(&value)),
            Ok(None) => warn!("⚠️ Standalone data does not exist"),
            Err(e) => warn!("❌ Standalone read failed: {}", e),
        }
    }

    // Close storage
    storage.shutdown().await?;
    info!("✅ Storage closed");

    info!("🎉 Distributed storage demo completed");
    Ok(())
}
