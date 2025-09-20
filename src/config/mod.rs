pub mod types;

use anyhow::Result;
use once_cell::sync::Lazy;
use std::collections::HashMap;
use std::sync::Mutex;
use tracing::info;

// Version constant corresponding to Go version
pub const VERSION: &str = "0.1";

// Global reserved token mapping, corresponding to Go version TickReserved
static TICK_RESERVED: Lazy<Mutex<HashMap<String, String>>> =
    Lazy::new(|| Mutex::new(HashMap::new()));

/// Load configuration file, corresponding to Go version's config.Load
pub fn load_config(config: &mut crate::config::types::Config) -> Result<()> {
    // Try to load configuration file, only use TOML format
    let config_paths = [
        "mainnet.toml",
        "testnet.toml",
        "config.toml",
        "config/testnet.toml",
        "config/config.toml",
    ];

    info!("Searching for configuration files...");
    for config_path in &config_paths {
        info!("Checking path: {}", config_path);
        if std::path::Path::new(config_path).exists() {
            info!("Loading configuration from: {}", config_path);
            let content = std::fs::read_to_string(config_path)?;
            info!("Configuration file content length: {} bytes", content.len());

            // Only use TOML parser
            *config = toml::from_str(&content)?;
            info!("Configuration loaded successfully from: {}", config_path);

            // Apply reserved token list
            if !config.startup.tick_reserved.is_empty() {
                apply_tick_reserved(&config.startup.tick_reserved);
            }

            return Ok(());
        } else {
            info!("Configuration file not found: {}", config_path);
        }
    }

    // If no configuration file found, return error
    Err(anyhow::anyhow!("No TOML configuration file found"))
}

/// Apply reserved token list, corresponding to Go version's ApplyTickReserved
pub fn apply_tick_reserved(reserved_list: &[String]) {
    let mut tick_reserved = TICK_RESERVED.lock().unwrap();

    for reserved_item in reserved_list {
        let parts: Vec<&str> = reserved_item.split('_').collect();
        if parts.len() >= 2 {
            let tick = parts[0].to_uppercase();
            let address = parts[1..].join("_");
            tick_reserved.insert(tick, address);
        }
    }

    // Apply default reserved tokens if none are configured
    if reserved_list.is_empty() {
        apply_default_reserved_tokens(&mut tick_reserved);
    }

    info!("Applied {} reserved tokens", tick_reserved.len());
}

/// Apply default reserved tokens (previously hardcoded)
fn apply_default_reserved_tokens(tick_reserved: &mut std::collections::HashMap<String, String>) {
    let default_reserved = [
        ("NACHO", "kaspa:qzrsq2mfj9sf7uye3u5q7juejzlr0axk5jz9fpg4vqe76erdyvxxze84k9nk7"),
        ("KCATS", "kaspa:qq8guq855gxkfrj2w25skwgj7cp4hy08x6a8mz70tdtmgv5p2ngwqxpj4cknc"),
        ("KASTOR", "kaspa:qr8vt54764aaddejhjfwtsh07jcjr49v38vrw2vtmxxtle7j2uepynwy57ufg"),
        ("KASPER", "kaspa:qppklkx2zyr2g2djg3uy2y2tsufwsqjk36pt27vt2xfu8uqm24pskk4p7tq5n"),
        ("FUSUN", "kaspa:qzp30gu5uty8jahu9lq5vtplw2ca8m2k7p45ez3y8jf9yrm5qdxquq5nl45t5"),
        ("KPAW", "kaspa:qpp0y685frmnlvhmnz5t6qljatumqm9zmppwnhwu9vyyl6w8nt30qjedekmdw"),
        ("PPKAS", "kaspa:qrlx9377yje3gvj9qxvwnn697d209lshgcrvge3yzlxnvyrfyk3q583jh3cmz"),
        ("GHOAD", "kaspa:qpkty3ymqs67t0z3g7l457l79f9k6drl55uf2qeq5tlkrpf3zwh85es0xtaj9"),
        ("KEPE", "kaspa:qq45gur2grn80uuegg9qgewl0wg2ahz5n4qm9246laej9533f8e22x3xe6hkm"),
        ("WORI", "kaspa:qzhgepc7mjscszkteeqhy99d3v96ftpg2wyy6r85nd0kg9m8rfmusqpp7mxkq"),
        ("KEKE", "kaspa:qqq9m42mdcvlz8c7r9kmpqj59wkfx3nppqte8ay20m4p46x3z0lsyzz34h8uf"),
        ("DOGK", "kaspa:qpsj64nxtlwceq4e7jvrsrkl0y6dayfyrqr49pep7pd2tq2uzvk7ks7n0qwxc"),
        ("BTAI", "kaspa:qp0na29g4lysnaep5pmg9xkdzcn4xm4a35ha5naq79ns9mcgc3pccnf225qma"),
        ("KASBOT", "kaspa:qrrcpdaev9augqwy8jnnp20skplyswa7ezz3m9ex3ryxw22frpzpj2xx99scq"),
        ("SOMPS", "kaspa:qry7xqy6s7d449gqyl0dkr99x6df0q5jlj6u52p84tfv6rddxjrucnn066237"),
        ("KREP", "kaspa:qzaclsmr5vttzlt0rz0x3shnudny8lnz5zpmjr4lp9v7aa7u7zvexh05eqwq0"),
    ];

    for (tick, address) in &default_reserved {
        tick_reserved.insert(tick.to_string(), address.to_string());
    }
}

/// Check if token is reserved, corresponding to Go version's TickReserved check
pub fn is_tick_reserved(tick: &str) -> bool {
    let tick_reserved = TICK_RESERVED.lock().unwrap();
    tick_reserved.contains_key(&tick.to_uppercase())
}

/// Get reserved token address, corresponding to Go version's TickReserved[tick]
pub fn get_reserved_tick_address(tick: &str) -> Option<String> {
    let tick_reserved = TICK_RESERVED.lock().unwrap();
    tick_reserved.get(&tick.to_uppercase()).cloned()
}

/// Validate configuration, corresponding to Go version's configuration validation
pub fn validate_config(config: &crate::config::types::Config) -> Result<()> {
    // Validate startup configuration
    validate_startup_config(&config.startup)?;
    
    // Validate RocksDB configuration
    validate_rocksdb_config(&config.rocksdb)?;
    
    // Validate HTTP configuration
    validate_http_config(&config.http)?;
    
    // Validate REST configuration
    validate_rest_config(&config.rest)?;
    
    // Validate distributed configuration if enabled
    if config.distributed.node.enabled {
        validate_distributed_config(&config.distributed)?;
    }

    info!("Configuration validation passed");
    Ok(())
}

fn validate_startup_config(startup: &crate::config::types::StartupConfig) -> Result<()> {
    if startup.kaspa_node_url.is_empty() {
        return Err(anyhow::anyhow!("kaspa_node_url is required"));
    }
    
    // Validate URL format
    if !startup.kaspa_node_url.starts_with("http://") && !startup.kaspa_node_url.starts_with("https://") {
        return Err(anyhow::anyhow!("kaspa_node_url must start with http:// or https://"));
    }
    
    // Validate hysteresis value
    if startup.hysteresis == 0 {
        return Err(anyhow::anyhow!("hysteresis must be greater than 0"));
    }
    
    // Validate DAA score range
    for range in &startup.daa_score_range {
        if range.len() != 2 {
            return Err(anyhow::anyhow!("Invalid daa_score_range format"));
        }
        if range[0] >= range[1] {
            return Err(anyhow::anyhow!(
                "Invalid daa_score_range: start must be less than end"
            ));
        }
    }
    
    // Validate reserved tokens format
    for reserved_token in &startup.tick_reserved {
        if !reserved_token.contains('_') {
            return Err(anyhow::anyhow!(
                "Invalid reserved token format '{}': must be in format 'TICK_ADDRESS'", 
                reserved_token
            ));
        }
    }
    
    Ok(())
}

fn validate_rocksdb_config(rocksdb: &crate::config::types::RocksConfig) -> Result<()> {
    if rocksdb.path.is_empty() {
        return Err(anyhow::anyhow!("rocksdb.path is required"));
    }
    
    // Check if path is writable
    let path = std::path::Path::new(&rocksdb.path);
    if let Some(parent) = path.parent() {
        if !parent.exists() {
            return Err(anyhow::anyhow!(
                "RocksDB parent directory does not exist: {}", 
                parent.display()
            ));
        }
    }
    
    Ok(())
}

fn validate_http_config(http: &crate::config::types::HttpConfig) -> Result<()> {
    if http.port == 0 {
        return Err(anyhow::anyhow!("HTTP port must be greater than 0"));
    }
    
    if http.port > 65535 {
        return Err(anyhow::anyhow!("HTTP port must be less than or equal to 65535"));
    }
    
    // Validate bind address
    if http.bind.parse::<std::net::IpAddr>().is_err() {
        return Err(anyhow::anyhow!("Invalid HTTP bind address: {}", http.bind));
    }
    
    Ok(())
}

fn validate_rest_config(rest: &crate::config::types::RestConfig) -> Result<()> {
    if rest.kaspa_rest_base_url.is_empty() {
        return Err(anyhow::anyhow!("kaspa_rest_base_url is required"));
    }
    
    // Validate URL format
    if !rest.kaspa_rest_base_url.starts_with("http://") && !rest.kaspa_rest_base_url.starts_with("https://") {
        return Err(anyhow::anyhow!("kaspa_rest_base_url must start with http:// or https://"));
    }
    
    Ok(())
}

fn validate_distributed_config(distributed: &crate::config::types::DistributedConfig) -> Result<()> {
    // Validate node configuration
    if distributed.node.node_id.is_empty() {
        return Err(anyhow::anyhow!("distributed.node.node_id is required"));
    }
    
    if distributed.node.data_dir.is_empty() {
        return Err(anyhow::anyhow!("distributed.node.data_dir is required"));
    }
    
    if distributed.node.shard_count == 0 {
        return Err(anyhow::anyhow!("distributed.node.shard_count must be greater than 0"));
    }
    
    if distributed.node.replication_factor == 0 {
        return Err(anyhow::anyhow!("distributed.node.replication_factor must be greater than 0"));
    }
    
    if distributed.node.replication_factor > distributed.node.shard_count {
        return Err(anyhow::anyhow!(
            "distributed.node.replication_factor ({}) cannot be greater than shard_count ({})",
            distributed.node.replication_factor,
            distributed.node.shard_count
        ));
    }
    
    // Validate hash ring configuration
    if distributed.hash_ring.virtual_nodes == 0 {
        return Err(anyhow::anyhow!("distributed.hash_ring.virtual_nodes must be greater than 0"));
    }
    
    if !["blake3", "sha256", "md5"].contains(&distributed.hash_ring.hash_algorithm.as_str()) {
        return Err(anyhow::anyhow!(
            "distributed.hash_ring.hash_algorithm must be one of: blake3, sha256, md5"
        ));
    }
    
    // Validate replication configuration
    if !["sync", "async", "semi-sync"].contains(&distributed.replication.strategy.as_str()) {
        return Err(anyhow::anyhow!(
            "distributed.replication.strategy must be one of: sync, async, semi-sync"
        ));
    }
    
    if distributed.replication.timeout == 0 {
        return Err(anyhow::anyhow!("distributed.replication.timeout must be greater than 0"));
    }
    
    // Validate performance configuration
    if distributed.performance.write_buffer_size == 0 {
        return Err(anyhow::anyhow!("distributed.performance.write_buffer_size must be greater than 0"));
    }
    
    if distributed.performance.max_write_buffer_number == 0 {
        return Err(anyhow::anyhow!("distributed.performance.max_write_buffer_number must be greater than 0"));
    }
    
    if !["snappy", "lz4", "zstd"].contains(&distributed.performance.compression_type.as_str()) {
        return Err(anyhow::anyhow!(
            "distributed.performance.compression_type must be one of: snappy, lz4, zstd"
        ));
    }
    
    Ok(())
}

/// Get current configuration's testnet status
pub fn is_testnet(config: &crate::config::types::Config) -> bool {
    config.testnet || config.is_testnet
}
