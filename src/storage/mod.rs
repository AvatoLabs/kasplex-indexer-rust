pub mod distributed;
pub mod rocksdb;
pub mod runtime;
pub mod state;
pub mod types;

use crate::config::types::{DistributedConfig, RocksConfig};
use crate::operations::handler::OperationManager;
use crate::storage::distributed::DistributedStorage;
use crate::storage::rocksdb::RocksDBClient;
use crate::storage::runtime::RuntimeManager;
use crate::storage::state::StateManager;
use anyhow::Result;
use std::collections::HashMap;
use std::sync::{Arc, Mutex};
use tracing::{error, info};

#[derive(Clone)]
pub struct StorageManager {
    pub rocksdb: RocksDBClient,
    pub state: StateManager,
    pub runtime: RuntimeManager,
    pub distributed: Option<Arc<DistributedStorage>>,
    operation_manager: Arc<Mutex<Option<Arc<OperationManager>>>>,
}

impl StorageManager {
    /// Initialize storage manager, corresponding to Go version's storage.Init function
    pub async fn new(
        rocks_config: RocksConfig,
        distributed_config: Option<DistributedConfig>,
    ) -> Result<Self> {
        info!("storage.Init start.");

        // Initialize distributed storage
        let distributed = if let Some(config) = distributed_config {
            if config.node.enabled {
                match DistributedStorage::new(config).await {
                    Ok(dist) => {
                        info!("Distributed storage initialized");
                        Some(Arc::new(dist))
                    }
                    Err(e) => {
                        error!("Failed to initialize distributed storage: {}", e);
                        None
                    }
                }
            } else {
                None
            }
        } else {
            None
        };

        // Initialize RocksDB client (corresponding to Go version's RocksDB initialization)
        let rocksdb = match rocksdb::RocksDBClient::new(rocks_config) {
            Ok(client) => {
                info!("RocksDB client initialized");
                Arc::new(client)
            }
            Err(e) => {
                error!("storage.Init fatal: {}", e);
                return Err(e);
            }
        };

        // Initialize state manager
        let state = match state::StateManager::new(Arc::clone(&rocksdb)) {
            Ok(state_mgr) => {
                info!("State manager initialized");
                Arc::new(state_mgr)
            }
            Err(e) => {
                error!("Failed to initialize state manager: {}", e);
                return Err(e);
            }
        };

        // Initialize runtime manager
        let runtime = match runtime::RuntimeManager::new(Arc::clone(&rocksdb)) {
            Ok(runtime_mgr) => {
                info!("Runtime manager initialized");
                runtime_mgr
            }
            Err(e) => {
                error!("Failed to initialize runtime manager: {}", e);
                return Err(e);
            }
        };

        info!("storage ready.");

        Ok(Self {
            distributed,
            rocksdb: rocksdb.as_ref().clone(),
            state: state.as_ref().clone(),
            runtime,
            operation_manager: Arc::new(Mutex::new(None)),
        })
    }

    /// Create single-node RocksDB storage manager
    pub async fn new_single_node(rocks_config: RocksConfig) -> Result<Self> {
        let distributed = None;

        let rocksdb = Arc::new(rocksdb::RocksDBClient::new(rocks_config)?);
        let state = Arc::new(state::StateManager::new(Arc::clone(&rocksdb))?);
        let runtime = Arc::new(runtime::RuntimeManager::new(Arc::clone(&rocksdb))?);

        Ok(Self {
            distributed,
            rocksdb: rocksdb.as_ref().clone(),
            state: state.as_ref().clone(),
            runtime: runtime.as_ref().clone(),
            operation_manager: Arc::new(Mutex::new(None)),
        })
    }

    /// Create distributed storage manager
    pub async fn new_distributed(distributed_config: DistributedConfig) -> Result<Self> {
        let distributed = if distributed_config.node.enabled {
            Some(Arc::new(DistributedStorage::new(distributed_config).await?))
        } else {
            return Err(anyhow::anyhow!("Distributed mode is disabled in config"));
        };

        // Create default RocksDB configuration for distributed mode
        let rocks_config = RocksConfig {
            path: "./data/standalone".to_string(),
        };
        let rocksdb = Arc::new(rocksdb::RocksDBClient::new(rocks_config)?);
        let state = Arc::new(state::StateManager::new(Arc::clone(&rocksdb))?);
        let runtime = Arc::new(runtime::RuntimeManager::new(Arc::clone(&rocksdb))?);

        Ok(Self {
            distributed,
            rocksdb: rocksdb.as_ref().clone(),
            state: state.as_ref().clone(),
            runtime: runtime.as_ref().clone(),
            operation_manager: Arc::new(Mutex::new(None)),
        })
    }

    /// Create temporary instance to avoid circular references
    pub fn new_dummy() -> Self {
        let rocks_config = RocksConfig {
            path: "./data/dummy".to_string(),
        };
        let rocksdb = rocksdb::RocksDBClient::new(rocks_config).unwrap();
        let state = state::StateManager::new(Arc::new(rocksdb.clone())).unwrap();
        let runtime = runtime::RuntimeManager::new(Arc::new(rocksdb.clone())).unwrap();

        Self {
            distributed: None,
            rocksdb,
            state,
            runtime,
            operation_manager: Arc::new(Mutex::new(None)),
        }
    }

    /// Initialize storage, corresponding to Go version's initialization logic
    pub async fn init(&mut self) -> Result<()> {
        // Initialize RocksDB
        self.rocksdb.init()?;

        // Initialize state manager
        self.state.init()?;

        // Initialize runtime manager
        self.runtime.init()?;

        info!("Storage initialization completed");
        Ok(())
    }

    pub fn set_operation_manager(&mut self, operation_manager: OperationManager) {
        self.operation_manager
            .lock()
            .unwrap()
            .replace(Arc::new(operation_manager));
    }

    pub fn set_operation_manager_arc(&self, operation_manager: Arc<OperationManager>) {
        // Use interior mutability to set operation_manager
        self.operation_manager
            .lock()
            .unwrap()
            .replace(operation_manager);
    }

    pub fn get_operation_manager(&self) -> Result<Arc<OperationManager>> {
        self.operation_manager
            .lock()
            .unwrap()
            .as_ref()
            .cloned()
            .ok_or_else(|| anyhow::anyhow!("Operation manager not initialized"))
    }

    /// Shutdown storage, corresponding to Go version's shutdown logic
    pub async fn shutdown(&self) -> Result<()> {
        info!("Shutting down storage...");

        // Shutdown RocksDB
        self.rocksdb.shutdown()?;

        info!("Storage shutdown completed");
        Ok(())
    }

    /// Get distributed storage instance
    pub fn get_distributed(&self) -> Result<Arc<DistributedStorage>> {
        self.distributed
            .as_ref()
            .cloned()
            .ok_or_else(|| anyhow::anyhow!("Distributed storage not initialized"))
    }

    /// Distributed insert operation
    pub async fn distributed_insert(&self, key: &str, value: &[u8]) -> Result<()> {
        if let Some(distributed) = &self.distributed {
            distributed.distributed_insert(key, value).await
        } else {
            // Fallback to single-node RocksDB
            self.rocksdb.put_raw(key, value)
        }
    }

    /// Distributed get operation
    pub async fn distributed_get(&self, key: &str) -> Result<Option<Vec<u8>>> {
        if let Some(distributed) = &self.distributed {
            distributed.distributed_get(key).await
        } else {
            // Fallback to single-node RocksDB
            self.rocksdb.get_raw(key)
        }
    }

    /// Distributed batch operation
    pub async fn distributed_batch_write(
        &self,
        operations: &[crate::storage::types::StorageOperation],
    ) -> Result<()> {
        if let Some(distributed) = &self.distributed {
            distributed.distributed_batch_write(operations).await
        } else {
            // Fallback to single-node RocksDB
            self.rocksdb.batch_write(operations)
        }
    }

    /// Health check
    pub async fn health_check(&self) -> Result<crate::storage::distributed::HealthStatus> {
        if let Some(distributed) = &self.distributed {
            distributed.health_check().await
        } else {
            // Single-node mode returns simple health status
            Ok(crate::storage::distributed::HealthStatus {
                is_healthy: true,
                healthy_shards: 1,
                total_shards: 1,
                health_ratio: 1.0,
                last_check: std::time::Instant::now(),
                shard_details: std::collections::HashMap::new(),
            })
        }
    }

    /// Get distributed storage metrics
    pub async fn get_distributed_metrics(
        &self,
    ) -> Option<crate::storage::distributed::DistributedMetrics> {
        if let Some(distributed) = &self.distributed {
            Some(distributed.get_metrics().await)
        } else {
            None
        }
    }

    /// Get shard information
    pub fn get_shard_info(&self) -> Vec<crate::config::types::ShardConfig> {
        if let Some(distributed) = &self.distributed {
            // TODO: Implement shard information retrieval
            Vec::new()
        } else {
            Vec::new()
        }
    }

    /// Get hash ring information
    pub fn get_hash_ring_info(&self) -> HashMap<String, u32> {
        if let Some(distributed) = &self.distributed {
            // TODO: Implement hash ring information retrieval
            HashMap::new()
        } else {
            HashMap::new()
        }
    }

    /// Check if distributed mode is enabled
    pub fn is_distributed_enabled(&self) -> bool {
        self.distributed.is_some()
    }

    /// Get distributed configuration
    pub fn get_distributed_config(&self) -> Option<&DistributedConfig> {
        if let Some(distributed) = &self.distributed {
            Some(distributed.get_config())
        } else {
            None
        }
    }
}
