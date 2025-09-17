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

    info!("Applied {} reserved tokens", reserved_list.len());
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
    // Validate necessary configuration items
    if config.startup.kaspa_node_url.is_empty() {
        return Err(anyhow::anyhow!("kaspa_node_url is required"));
    }

    if config.rocksdb.path.is_empty() {
        return Err(anyhow::anyhow!("rocksdb.path is required"));
    }

    // Validate DAA score range
    for range in &config.startup.daa_score_range {
        if range.len() != 2 {
            return Err(anyhow::anyhow!("Invalid daa_score_range format"));
        }
        if range[0] >= range[1] {
            return Err(anyhow::anyhow!(
                "Invalid daa_score_range: start must be less than end"
            ));
        }
    }

    info!("Configuration validation passed");
    Ok(())
}

/// Get current configuration's testnet status
pub fn is_testnet(config: &crate::config::types::Config) -> bool {
    config.testnet || config.is_testnet
}
