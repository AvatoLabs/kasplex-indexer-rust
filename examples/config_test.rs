use kaspa_indexer_rust::config::{load_config, types::Config, validate_config};

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    println!("Fun20 Client Configuration Test");
    println!("Version: {}", kaspa_indexer_rust::config::VERSION);

    // Load configuration
    let mut config = Config::default();
    match load_config(&mut config) {
        Ok(_) => {
            println!("✅ Configuration loaded successfully");

            // Validate configuration
            match validate_config(&config) {
                Ok(_) => println!("✅ Configuration validation passed"),
                Err(e) => println!("❌ Configuration validation failed: {}", e),
            }

            // Print configuration information
            println!("\n📋 Configuration Summary:");
            println!("  Kaspa Node URL: {}", config.startup.kaspa_node_url);
            println!("  Is Testnet: {}", config.startup.is_testnet);
            println!("  RocksDB Path: {}", config.rocksdb.path);
            println!("  Debug Level: {}", config.debug);
            println!("  Distributed Enabled: {}", config.distributed.node.enabled);
            println!("  Shard Count: {}", config.distributed.node.shard_count);
            println!(
                "  Replication Factor: {}",
                config.distributed.node.replication_factor
            );

            println!("\n🔧 Distributed Configuration:");
            println!("  Node ID: {}", config.distributed.node.node_id);
            println!("  Data Directory: {}", config.distributed.node.data_dir);
            println!("  Role: {}", config.distributed.node.role);
            println!("  Port: {}", config.distributed.node.port);
            println!(
                "  Max Connections: {}",
                config.distributed.node.max_connections
            );

            println!("\n📊 Performance Configuration:");
            println!(
                "  Write Buffer Size: {} MB",
                config.distributed.performance.write_buffer_size
            );
            println!(
                "  Max Write Buffer Number: {}",
                config.distributed.performance.max_write_buffer_number
            );
            println!(
                "  Target File Size Base: {} MB",
                config.distributed.performance.target_file_size_base
            );
            println!(
                "  Max Background Jobs: {}",
                config.distributed.performance.max_background_jobs
            );
            println!(
                "  Max Open Files: {}",
                config.distributed.performance.max_open_files
            );
            println!(
                "  Enable Compression: {}",
                config.distributed.performance.enable_compression
            );
            println!(
                "  Compression Type: {}",
                config.distributed.performance.compression_type
            );
            println!(
                "  Batch Size: {}",
                config.distributed.performance.batch_size
            );
            println!(
                "  Concurrent Reads: {}",
                config.distributed.performance.concurrent_reads
            );
            println!(
                "  Concurrent Writes: {}",
                config.distributed.performance.concurrent_writes
            );

            println!("\n🔒 Security Configuration:");
            println!("  Enable Auth: {}", config.distributed.security.enable_auth);
            println!(
                "  Enable Encryption: {}",
                config.distributed.security.enable_encryption
            );
            println!("  Enable SSL: {}", config.distributed.security.enable_ssl);

            println!("\n📈 Monitoring Configuration:");
            println!("  Enabled: {}", config.distributed.monitoring.enabled);
            println!(
                "  Health Check Interval: {} seconds",
                config.distributed.monitoring.health_check_interval
            );
            println!(
                "  Metrics Interval: {} seconds",
                config.distributed.monitoring.metrics_interval
            );
            println!("  Log Level: {}", config.distributed.monitoring.log_level);
            println!(
                "  Enable Metrics: {}",
                config.distributed.monitoring.enable_metrics
            );
            println!(
                "  Enable Tracing: {}",
                config.distributed.monitoring.enable_tracing
            );

            println!("\n🎯 Shards Configuration:");
            for (i, shard) in config.distributed.shards.iter().enumerate() {
                println!(
                    "  Shard {}: ID={}, Primary={}, Weight={}, MaxDataSize={}MB",
                    i, shard.shard_id, shard.is_primary, shard.weight, shard.max_data_size
                );
            }
        }
        Err(e) => {
            println!("❌ Failed to load configuration: {}", e);
            println!("Using default configuration...");

            // Use default configuration
            match validate_config(&config) {
                Ok(_) => println!("✅ Default configuration validation passed"),
                Err(e) => println!("❌ Default configuration validation failed: {}", e),
            }
        }
    }

    Ok(())
}
