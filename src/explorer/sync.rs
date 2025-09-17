use crate::storage::StorageManager;
use anyhow::Result;
use std::sync::Arc;
use tracing::info;

pub struct SyncManager {
    storage: Arc<StorageManager>,
}

impl SyncManager {
    pub fn new(storage: Arc<StorageManager>) -> Result<Self> {
        Ok(Self { storage })
    }

    pub fn init(&self) -> Result<()> {
        info!("Sync manager initialized");
        Ok(())
    }

    pub fn start_sync(&self) -> Result<()> {
        self.storage.runtime.start_sync()?;
        info!("Sync started");
        Ok(())
    }

    pub fn stop_sync(&self) -> Result<()> {
        self.storage.runtime.stop_sync()?;
        info!("Sync stopped");
        Ok(())
    }

    pub fn is_syncing(&self) -> Result<bool> {
        self.storage.runtime.is_syncing()
    }

    pub fn get_status(&self) -> Result<SyncStatus> {
        let stats = self.storage.runtime.get_sync_stats()?;

        Ok(SyncStatus {
            is_syncing: stats.is_syncing,
            last_processed_block: stats.last_processed_block,
            total_blocks_processed: stats.total_blocks_processed,
            total_operations_processed: stats.total_operations_processed,
            sync_start_time: stats.sync_start_time,
            last_processed_daa_score: stats.last_processed_daa_score,
            sync_duration: stats.sync_duration,
            blocks_per_second: stats.blocks_per_second,
        })
    }

    pub fn can_resume(&self) -> Result<bool> {
        self.storage.runtime.can_resume_from_checkpoint()
    }

    pub fn reset_sync(&self) -> Result<()> {
        self.storage.runtime.reset_sync_state()?;
        info!("Sync state reset");
        Ok(())
    }

    pub fn get_checkpoint(&self) -> Result<(String, u64)> {
        self.storage.runtime.get_checkpoint()
    }

    pub fn save_checkpoint(&self, block_hash: &str, daa_score: u64) -> Result<()> {
        self.storage.runtime.save_checkpoint(block_hash, daa_score)
    }
}

#[derive(Debug, Clone)]
pub struct SyncStatus {
    pub is_syncing: bool,
    pub last_processed_block: String,
    pub total_blocks_processed: u64,
    pub total_operations_processed: u64,
    pub sync_start_time: u64,
    pub last_processed_daa_score: u64,
    pub sync_duration: u64,
    pub blocks_per_second: f64,
}
