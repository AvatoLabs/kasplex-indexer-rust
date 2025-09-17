use anyhow::Result;
use kaspa_indexer_rust::config::types::RocksConfig;
use kaspa_indexer_rust::storage::StorageManager;
use tracing::{info, warn};

#[tokio::main]
async fn main() -> Result<()> {
    // Initialize logging
    tracing_subscriber::fmt::init();

    info!("🚀 Single RocksDB demo started");

    // Configure single RocksDB
    let rocks_config = RocksConfig {
        path: "./data/single_rocksdb".to_string(),
    };

    info!("📊 Single RocksDB configuration:");
    info!("  Data directory: {}", rocks_config.path);

    // Initialize single storage manager
    let mut storage = StorageManager::new_single_node(rocks_config).await?;

    info!("✅ Single RocksDB storage manager initialized successfully");

    // Check storage mode
    if storage.is_distributed_enabled() {
        warn!("⚠️ Distributed mode unexpectedly enabled");
    } else {
        info!("🎯 Single mode enabled");
    }

    // Execute some single operations
    info!("🔧 Executing single operations test...");

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
    } else {
        info!("📈 Single mode has no distributed metrics");
    }

    // Get shard information
    let shard_info = storage.get_shard_info();
    if shard_info.is_empty() {
        info!("📋 Single mode has no shard information");
    } else {
        info!("📋 Shard information:");
        for shard in shard_info {
            info!(
                "  Shard {}: {} (Primary: {})",
                shard.shard_id, shard.data_dir, shard.is_primary
            );
        }
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
        info!("🔄 Single mode has no hash ring information");
    }

    // Close storage
    storage.shutdown().await?;
    info!("✅ Storage closed");

    info!("🎉 Single RocksDB demo completed");
    Ok(())
}
