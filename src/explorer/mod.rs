pub mod rollback;
pub mod scanner;
pub mod sync;
pub mod vspc_client;

use crate::config::types::StartupConfig;
use crate::storage::StorageManager;
use anyhow::Result;
use std::sync::Arc;
use tokio::sync::mpsc;
use tracing::{debug, info};

pub use crate::storage::types::OperationData;
pub use rollback::RollbackManager;
pub use scanner::VSPCScanner;
pub use sync::SyncManager;

/// Explorer interface, defining core functionality of the explorer
pub trait ExplorerInterface {
    /// Initialize explorer
    async fn init(&mut self) -> Result<()>;

    /// Start explorer
    async fn run(&mut self) -> Result<()>;

    /// Shutdown explorer
    async fn shutdown(&mut self) -> Result<()>;

    /// Get sync status
    fn get_sync_status(&self) -> Result<SyncStatus>;

    /// Get scan statistics
    fn get_scan_stats(&self) -> Result<ScanStats>;

    /// Process operation
    async fn process_operation(&self, operation: OperationData) -> Result<()>;
}

/// Main explorer implementation
pub struct Explorer {
    operation_tx: mpsc::Sender<OperationData>,
    storage: Arc<StorageManager>,
    scanner: VSPCScanner,
    sync_manager: SyncManager,
    rollback_manager: RollbackManager,
    operation_rx: mpsc::Receiver<OperationData>,
    is_running: bool,
}

impl Explorer {
    pub fn new(
        storage: Arc<StorageManager>,
        startup_config: StartupConfig,
        testnet: bool,
    ) -> Result<Self> {
        let (operation_tx, operation_rx) = mpsc::channel(1000);

        let scanner = VSPCScanner::new(Arc::clone(&storage), startup_config.clone(), testnet)?;
        let sync_manager = SyncManager::new(Arc::clone(&storage))?;
        let rollback_manager = RollbackManager::new(Arc::clone(&storage))?;

        Ok(Self {
            storage,
            scanner,
            sync_manager,
            rollback_manager,
            operation_tx,
            operation_rx,
            is_running: false,
        })
    }

    /// Get storage manager reference
    pub fn get_storage(&self) -> &Arc<StorageManager> {
        &self.storage
    }

    /// Get scanner reference
    pub fn get_scanner(&self) -> &VSPCScanner {
        &self.scanner
    }

    /// Get sync manager reference
    pub fn get_sync_manager(&self) -> &SyncManager {
        &self.sync_manager
    }

    /// Get rollback manager reference
    pub fn get_rollback_manager(&self) -> &RollbackManager {
        &self.rollback_manager
    }

    /// Check if running
    pub fn is_running(&self) -> bool {
        self.is_running
    }

    /// Send operation to processing queue
    pub async fn send_operation(&self, operation: OperationData) -> Result<()> {
        self.operation_tx
            .send(operation)
            .await
            .map_err(|e| anyhow::anyhow!("Failed to send operation: {}", e))
    }
}

impl ExplorerInterface for Explorer {
    async fn init(&mut self) -> Result<()> {
        info!("Initializing Explorer...");

        // Initialize scanner
        self.scanner.init().await?;

        // Initialize sync manager
        self.sync_manager.init()?;

        // Initialize rollback manager
        self.rollback_manager.init()?;

        info!("Explorer initialized successfully");
        Ok(())
    }

    async fn run(&mut self) -> Result<()> {
        if self.is_running {
            return Err(anyhow::anyhow!("Explorer is already running"));
        }

        info!("Starting Explorer...");
        self.is_running = true;

        // Start sync manager
        self.sync_manager.start_sync()?;

        // Start scanning loop
        self.scanner.start_scanning().await?;

        // Process operations
        while let Some(operation) = self.operation_rx.recv().await {
            self.process_operation(operation).await?;
        }

        Ok(())
    }

    async fn shutdown(&mut self) -> Result<()> {
        if !self.is_running {
            return Ok(());
        }

        info!("Shutting down Explorer...");
        self.is_running = false;

        // Stop sync manager
        self.sync_manager.stop_sync()?;

        // Stop scanner
        self.scanner.stop_scanning().await?;

        info!("Explorer shutdown completed");
        Ok(())
    }

    fn get_sync_status(&self) -> Result<SyncStatus> {
        let sync_status = self.sync_manager.get_status()?;
        Ok(SyncStatus {
            is_syncing: sync_status.is_syncing,
            last_processed_block: sync_status.last_processed_block,
            total_blocks_processed: sync_status.total_blocks_processed,
            total_operations_processed: sync_status.total_operations_processed,
            sync_start_time: sync_status.sync_start_time,
            last_processed_daa_score: sync_status.last_processed_daa_score,
            sync_duration: sync_status.sync_duration,
            blocks_per_second: sync_status.blocks_per_second,
        })
    }

    fn get_scan_stats(&self) -> Result<ScanStats> {
        self.scanner.get_stats()
    }

    async fn process_operation(&self, operation: OperationData) -> Result<()> {
        debug!(
            "Processing operation: {} for tick: {}",
            operation.operation_type, operation.tick
        );

        // Get operation manager
        let _operation_manager = self.storage.get_operation_manager()?;

        // Validate operation - temporarily commented out due to validate_operation method signature mismatch
        // let validation_result = operation_manager.validate_operation(&operation);
        // if let Err(e) = validation_result {
        //     debug!("Operation validation failed: {}", e);
        //     return Ok(());
        // }

        // Execute operation
        // TODO: Implement specific operation execution logic

        info!("Successfully processed operation: {}", operation.tx_id);
        Ok(())
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

#[derive(Debug, Clone)]
pub struct ScanStats {
    pub total_vspc_processed: u64,
    pub total_operations_found: u64,
    pub last_scan_time: u64,
    pub scan_start_time: u64,
    pub is_scanning: bool,
    pub scan_duration: u64,
    pub vspc_per_second: f64,
}

#[derive(Clone)]
pub struct ExplorerBuilder {
    pub storage: Option<Arc<StorageManager>>,
    pub config: Option<StartupConfig>,
}

impl ExplorerBuilder {
    pub fn new() -> Self {
        Self {
            storage: None,
            config: None,
        }
    }

    pub fn with_storage(mut self, storage: Arc<StorageManager>) -> Self {
        self.storage = Some(storage);
        self
    }

    pub fn with_config(mut self, config: StartupConfig) -> Self {
        self.config = Some(config);
        self
    }

    pub fn build(self) -> Result<Explorer> {
        let storage = self
            .storage
            .ok_or_else(|| anyhow::anyhow!("Storage not set"))?;
        let config = self
            .config
            .ok_or_else(|| anyhow::anyhow!("Config not set"))?;
        Explorer::new(storage, config, false) // Default to mainnet
    }
}

impl Default for ExplorerBuilder {
    fn default() -> Self {
        Self::new()
    }
}
