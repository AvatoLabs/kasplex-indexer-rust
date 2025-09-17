use crate::storage::rocksdb::RocksDBClient;
use crate::storage::types::*;
use anyhow::Result;
use std::sync::Arc;
use std::time::{SystemTime, UNIX_EPOCH};
use tracing::info;

#[derive(Debug, Clone)]
pub struct RuntimeManager {
    rocksdb: Arc<RocksDBClient>,
}

impl RuntimeManager {
    pub fn new(rocksdb: Arc<RocksDBClient>) -> Result<Self> {
        Ok(Self { rocksdb })
    }

    pub fn init(&self) -> Result<()> {
        if self.rocksdb.get_runtime_state()?.is_none() {
            let initial_state = RuntimeState {
                last_processed_block: String::new(),
                last_processed_daa_score: 0,
                is_syncing: false,
                sync_start_time: 0,
                total_blocks_processed: 0,
                total_operations_processed: 0,
            };
            self.rocksdb.set_runtime_state(&initial_state)?;
        }
        Ok(())
    }

    pub fn get_runtime_state(&self) -> Result<RuntimeState> {
        if let Some(state) = self.rocksdb.get_runtime_state()? {
            Ok(state)
        } else {
            Err(anyhow::anyhow!("Runtime state not found"))
        }
    }

    pub fn set_runtime_state(&self, state: &RuntimeState) -> Result<()> {
        self.rocksdb.set_runtime_state(state)?;
        Ok(())
    }

    pub fn update_runtime_state(&self, state: RuntimeState) -> Result<()> {
        self.rocksdb.set_runtime_state(&state)?;
        Ok(())
    }

    // Sync state management
    pub fn start_sync(&self) -> Result<()> {
        let mut state = self.get_runtime_state()?;
        state.is_syncing = true;
        state.sync_start_time = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap()
            .as_secs();
        self.update_runtime_state(state)?;
        info!("Sync started");
        Ok(())
    }

    pub fn stop_sync(&self) -> Result<()> {
        let mut state = self.get_runtime_state()?;
        state.is_syncing = false;
        self.update_runtime_state(state)?;
        info!("Sync stopped");
        Ok(())
    }

    pub fn is_syncing(&self) -> Result<bool> {
        let state = self.get_runtime_state()?;
        Ok(state.is_syncing)
    }

    // Progress tracking
    pub fn update_progress(&self, block_hash: &str, daa_score: u64) -> Result<()> {
        let mut state = self.get_runtime_state()?;
        state.last_processed_block = block_hash.to_string();
        state.last_processed_daa_score = daa_score;
        state.total_blocks_processed += 1;
        self.update_runtime_state(state)?;
        Ok(())
    }

    pub fn increment_operations_processed(&self, count: u64) -> Result<()> {
        let mut state = self.get_runtime_state()?;
        state.total_operations_processed += count;
        self.update_runtime_state(state)?;
        Ok(())
    }

    // Sync statistics
    pub fn get_sync_stats(&self) -> Result<SyncStats> {
        let state = self.get_runtime_state()?;
        let current_time = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap()
            .as_secs();

        let sync_duration = if state.is_syncing {
            current_time - state.sync_start_time
        } else {
            0
        };

        let blocks_per_second = if sync_duration > 0 {
            state.total_blocks_processed as f64 / sync_duration as f64
        } else {
            0.0
        };

        let operations_per_second = if sync_duration > 0 {
            state.total_operations_processed as f64 / sync_duration as f64
        } else {
            0.0
        };

        Ok(SyncStats {
            total_blocks_processed: state.total_blocks_processed,
            total_operations_processed: state.total_operations_processed,
            sync_start_time: state.sync_start_time,
            last_processed_block: state.last_processed_block,
            is_syncing: state.is_syncing,
            sync_duration,
            last_processed_daa_score: state.last_processed_daa_score,
            blocks_per_second,
            operations_per_second,
        })
    }

    // Runtime data management - corresponding to Go version's SetRuntimeRollbackLast
    pub async fn set_runtime_rollback_last(
        &self,
        rollback_list: &[DataRollbackType],
    ) -> Result<()> {
        let value_json = serde_json::to_string(rollback_list)?;
        self.rocksdb.set_runtime_data("ROLLBACKLAST", &value_json)?;
        Ok(())
    }

    pub async fn get_runtime_rollback_last(&self) -> Result<Vec<DataRollbackType>> {
        if let Some(value_json) = self.rocksdb.get_runtime_data("ROLLBACKLAST")? {
            let rollback_list: Vec<DataRollbackType> = serde_json::from_str(&value_json)?;
            Ok(rollback_list)
        } else {
            Ok(Vec::new())
        }
    }

    pub async fn set_runtime_vspc_last(&self, vspc_list: &[DataVspcType]) -> Result<()> {
        let value_json = serde_json::to_string(vspc_list)?;
        self.rocksdb.set_runtime_data("VSPCLAST", &value_json)?;
        Ok(())
    }

    pub async fn get_runtime_vspc_last(&self) -> Result<Vec<DataVspcType>> {
        if let Some(value_json) = self.rocksdb.get_runtime_data("VSPCLAST")? {
            let vspc_list: Vec<DataVspcType> = serde_json::from_str(&value_json)?;
            Ok(vspc_list)
        } else {
            Ok(Vec::new())
        }
    }

    pub async fn set_runtime_synced(
        &self,
        synced: bool,
        op_score_last: u64,
        daa_score: u64,
    ) -> Result<()> {
        let sync_data = serde_json::json!({
            "synced": synced,
            "op_score_last": op_score_last,
            "daa_score": daa_score
        });
        let value_json = serde_json::to_string(&sync_data)?;
        self.rocksdb.set_runtime_data("SYNCED", &value_json)?;
        Ok(())
    }

    // Checkpoint management
    pub fn save_checkpoint(&self, block_hash: &str, daa_score: u64) -> Result<()> {
        let mut state = self.get_runtime_state()?;
        state.last_processed_block = block_hash.to_string();
        state.last_processed_daa_score = daa_score;
        self.update_runtime_state(state)?;
        info!(
            "Checkpoint saved: block={}, daa_score={}",
            block_hash, daa_score
        );
        Ok(())
    }

    pub fn get_checkpoint(&self) -> Result<(String, u64)> {
        let state = self.get_runtime_state()?;
        Ok((state.last_processed_block, state.last_processed_daa_score))
    }

    // Recovery management
    pub fn can_resume_from_checkpoint(&self) -> Result<bool> {
        let state = self.get_runtime_state()?;
        Ok(!state.last_processed_block.is_empty() && state.last_processed_daa_score > 0)
    }

    pub fn reset_sync_state(&self) -> Result<()> {
        let reset_state = RuntimeState {
            last_processed_block: "".to_string(),
            last_processed_daa_score: 0,
            is_syncing: false,
            sync_start_time: 0,
            total_blocks_processed: 0,
            total_operations_processed: 0,
        };
        self.update_runtime_state(reset_state)?;
        info!("Sync state reset");
        Ok(())
    }

    // Health checks
    pub fn health_check(&self) -> Result<HealthStatus> {
        let state = self.get_runtime_state()?;
        let current_time = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap()
            .as_secs();

        let is_healthy = if state.is_syncing {
            // Check if sync has been running for too long without progress
            let sync_duration = current_time - state.sync_start_time;
            sync_duration < 3600 // 1 hour timeout
        } else {
            true
        };

        Ok(HealthStatus {
            is_healthy,
            is_syncing: state.is_syncing,
            last_processed_block: state.last_processed_block,
            total_blocks_processed: state.total_blocks_processed,
            total_operations_processed: state.total_operations_processed,
        })
    }
}

#[derive(Debug, Clone)]
pub struct HealthStatus {
    pub is_healthy: bool,
    pub is_syncing: bool,
    pub last_processed_block: String,
    pub total_blocks_processed: u64,
    pub total_operations_processed: u64,
}

#[derive(Debug, Clone)]
pub struct SyncStats {
    pub total_blocks_processed: u64,
    pub total_operations_processed: u64,
    pub sync_start_time: u64,
    pub last_processed_block: String,
    pub is_syncing: bool,
    pub sync_duration: u64,
    pub last_processed_daa_score: u64,
    pub blocks_per_second: f64,
    pub operations_per_second: f64,
}
