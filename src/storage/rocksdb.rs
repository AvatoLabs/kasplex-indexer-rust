use crate::config::types::RocksConfig;
use crate::storage::types::*;
use anyhow::Result;
use rocksdb::{DB, Options, WriteBatch};
use std::path::Path;
use std::sync::Arc;
use tracing::{debug, info};

#[derive(Debug, Clone)]
pub struct RocksDBClient {
    config: RocksConfig,
    db: Arc<rocksdb::DB>,
}

impl RocksDBClient {
    pub fn new(config: RocksConfig) -> Result<Self> {
        let db_path = Path::new(&config.path);
        if !db_path.exists() {
            std::fs::create_dir_all(db_path)?;
        }

        let mut opts = Options::default();
        opts.create_if_missing(true);
        opts.set_max_open_files(10000);
        opts.set_use_fsync(true);
        opts.set_bytes_per_sync(1024 * 1024);
        // Configuration corresponding to Go version
        opts.set_write_buffer_size(256 * 1024 * 1024); // 256MB
        opts.set_max_write_buffer_number(4);
        // Note: set_max_background_compactions is deprecated, RocksDB automatically decides this
        // opts.set_max_background_compactions(4);

        let db = DB::open(&opts, db_path)?;
        info!("RocksDB initialized at: {}", config.path);

        Ok(Self {
            config,
            db: Arc::new(db),
        })
    }

    pub fn init(&self) -> Result<()> {
        // Initialize default state if needed
        let runtime_state = RuntimeState {
            last_processed_block: "".to_string(),
            last_processed_daa_score: 0,
            is_syncing: false,
            sync_start_time: 0,
            total_blocks_processed: 0,
            total_operations_processed: 0,
        };

        self.set_runtime_state(&runtime_state)?;
        info!("RocksDB state initialized");
        Ok(())
    }

    pub fn shutdown(&self) -> Result<()> {
        // RocksDB will be closed when Arc is dropped
        info!("RocksDB shutdown completed");
        Ok(())
    }

    /// Public method: directly store key-value pair
    pub fn put_raw(&self, key: &str, value: &[u8]) -> Result<()> {
        self.db.put(key.as_bytes(), value)?;
        Ok(())
    }

    /// Public method: directly get key-value pair
    pub fn get_raw(&self, key: &str) -> Result<Option<Vec<u8>>> {
        Ok(self.db.get(key.as_bytes())?.map(|v| v.to_vec()))
    }

    /// Public method: delete key-value pair
    pub fn delete_raw(&self, key: &str) -> Result<()> {
        self.db.delete(key.as_bytes())?;
        Ok(())
    }

    /// Prefix scan method
    pub fn scan_prefix(&self, prefix: &str) -> Result<Vec<(String, Vec<u8>)>> {
        let mut results = Vec::new();
        let iter = self.db.iterator(rocksdb::IteratorMode::From(
            prefix.as_bytes(),
            rocksdb::Direction::Forward,
        ));

        for item in iter {
            let (key, value) = item?;
            let key_str = String::from_utf8(key.to_vec())?;

            // Check if still starts with specified prefix
            if key_str.starts_with(prefix) {
                results.push((key_str, value.to_vec()));
            } else {
                // If no longer matches prefix, stop scanning
                break;
            }
        }

        Ok(results)
    }

    // Token operations
    pub fn set_token(&self, token: &TokenData) -> Result<()> {
        let key = format!("token:{}", token.tick);
        let value = serde_json::to_string(token)?;
        self.db.put(key.as_bytes(), value.as_bytes())?;
        debug!("Stored token: {}", token.tick);
        Ok(())
    }

    pub fn get_token(&self, tick: &str) -> Result<Option<TokenData>> {
        let key = format!("token:{}", tick);
        if let Some(value) = self.db.get(key.as_bytes())? {
            let token: TokenData = serde_json::from_slice(&value)?;
            Ok(Some(token))
        } else {
            Ok(None)
        }
    }

    // Balance operations
    pub fn set_balance(&self, balance: &BalanceData) -> Result<()> {
        let key = format!("balance:{}:{}", balance.address, balance.tick);
        let value = serde_json::to_string(balance)?;
        self.db.put(key.as_bytes(), value.as_bytes())?;
        debug!("Stored balance: {} for {}", balance.tick, balance.address);
        Ok(())
    }

    pub fn get_balance(&self, address: &str, tick: &str) -> Result<Option<BalanceData>> {
        let key = format!("balance:{}:{}", address, tick);
        if let Some(value) = self.db.get(key.as_bytes())? {
            let balance: BalanceData = serde_json::from_slice(&value)?;
            Ok(Some(balance))
        } else {
            Ok(None)
        }
    }

    // Market operations
    pub fn set_market(&self, market: &MarketData) -> Result<()> {
        let key = format!("market:{}", market.tick);
        let value = serde_json::to_string(market)?;
        self.db.put(key.as_bytes(), value.as_bytes())?;
        debug!("Stored market data: {}", market.tick);
        Ok(())
    }

    pub fn get_market(&self, tick: &str) -> Result<Option<MarketData>> {
        let key = format!("market:{}", tick);
        if let Some(value) = self.db.get(key.as_bytes())? {
            let market: MarketData = serde_json::from_slice(&value)?;
            Ok(Some(market))
        } else {
            Ok(None)
        }
    }

    // Runtime state operations
    pub fn set_runtime_state(&self, state: &RuntimeState) -> Result<()> {
        let key = "runtime:state";
        let value = serde_json::to_string(state)?;
        self.db.put(key.as_bytes(), value.as_bytes())?;
        debug!("Updated runtime state");
        Ok(())
    }

    pub fn get_runtime_state(&self) -> Result<Option<RuntimeState>> {
        let key = "runtime:state";
        if let Some(value) = self.db.get(key.as_bytes())? {
            let state: RuntimeState = serde_json::from_slice(&value)?;
            Ok(Some(state))
        } else {
            Ok(None)
        }
    }

    // Runtime data management - corresponding to Go version's SetRuntimeRocks and GetRuntimeRocks
    pub fn set_runtime_data(&self, key: &str, value: &str) -> Result<()> {
        let full_key = format!("RTA_{}", key); // Corresponding to Go version keyPrefixRuntime
        self.db.put(full_key.as_bytes(), value.as_bytes())?;
        Ok(())
    }

    pub fn get_runtime_data(&self, key: &str) -> Result<Option<String>> {
        let full_key = format!("RTA_{}", key); // Corresponding to Go version keyPrefixRuntime
        if let Some(data) = self.db.get(full_key.as_bytes())? {
            let value = String::from_utf8(data.to_vec())?;
            Ok(Some(value))
        } else {
            Ok(None)
        }
    }

    // Blacklist operations
    pub fn set_blacklist(&self, entry: &BlacklistEntry) -> Result<()> {
        let key = format!("blacklist:{}", entry.tick);
        let value = serde_json::to_string(entry)?;
        self.db.put(key.as_bytes(), value.as_bytes())?;
        debug!("Added to blacklist: {}", entry.tick);
        Ok(())
    }

    pub fn get_blacklist(&self, tick: &str) -> Result<Option<BlacklistEntry>> {
        let key = format!("blacklist:{}", tick);
        if let Some(value) = self.db.get(key.as_bytes())? {
            let entry: BlacklistEntry = serde_json::from_slice(&value)?;
            Ok(Some(entry))
        } else {
            Ok(None)
        }
    }

    // Reserved token operations
    pub fn set_reserved_token(&self, reserved: &ReservedToken) -> Result<()> {
        let key = format!("reserved:{}", reserved.tick);
        let value = serde_json::to_string(reserved)?;
        self.db.put(key.as_bytes(), value.as_bytes())?;
        debug!("Added reserved token: {}", reserved.tick);
        Ok(())
    }

    pub fn get_reserved_token(&self, tick: &str) -> Result<Option<ReservedToken>> {
        let key = format!("reserved:{}", tick);
        if let Some(value) = self.db.get(key.as_bytes())? {
            let reserved: ReservedToken = serde_json::from_slice(&value)?;
            Ok(Some(reserved))
        } else {
            Ok(None)
        }
    }

    // Batch operations
    pub fn batch_write(&self, operations: &[StorageOperation]) -> Result<()> {
        let mut batch = WriteBatch::default();

        for operation in operations {
            match operation {
                StorageOperation::UpdateToken(token) => {
                    let key = format!("token:{}", token.tick);
                    let value = serde_json::to_string(token)?;
                    batch.put(key.as_bytes(), value.as_bytes());
                }
                StorageOperation::UpdateBalance(balance) => {
                    let key = format!("balance:{}:{}", balance.address, balance.tick);
                    let value = serde_json::to_string(balance)?;
                    batch.put(key.as_bytes(), value.as_bytes());
                }
                StorageOperation::UpdateMarket(market) => {
                    let key = format!("market:{}", market.tick);
                    let value = serde_json::to_string(market)?;
                    batch.put(key.as_bytes(), value.as_bytes());
                }
                StorageOperation::UpdateRuntimeState(state) => {
                    let key = "runtime:state";
                    let value = serde_json::to_string(state)?;
                    batch.put(key.as_bytes(), value.as_bytes());
                }
                StorageOperation::InsertBlacklist(entry) => {
                    let key = format!("blacklist:{}", entry.tick);
                    let value = serde_json::to_string(entry)?;
                    batch.put(key.as_bytes(), value.as_bytes());
                }
                StorageOperation::InsertReservedToken(reserved) => {
                    let key = format!("reserved:{}", reserved.tick);
                    let value = serde_json::to_string(reserved)?;
                    batch.put(key.as_bytes(), value.as_bytes());
                }
                StorageOperation::DeleteVSPC(block_hash) => {
                    let key = format!("vspc:{}", block_hash);
                    batch.delete(key.as_bytes());
                }
                StorageOperation::DeleteOperation(tx_hash) => {
                    let key = format!("operation:{}", tx_hash);
                    batch.delete(key.as_bytes());
                }
                _ => {
                    debug!("Skipping non-RocksDB operation: {:?}", operation);
                }
            }
        }

        self.db.write(batch)?;
        debug!("Batch write completed with {} operations", operations.len());
        Ok(())
    }

    /// Execute WriteBatch operation
    pub fn write_batch(&self, batch: WriteBatch) -> Result<()> {
        self.db.write(batch)?;
        Ok(())
    }

    // Utility methods
    pub fn is_token_blacklisted(&self, tick: &str) -> Result<bool> {
        Ok(self.get_blacklist(tick)?.is_some())
    }

    pub fn is_token_reserved(&self, tick: &str) -> Result<bool> {
        Ok(self.get_reserved_token(tick)?.is_some())
    }
}
