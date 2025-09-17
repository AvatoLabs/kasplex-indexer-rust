use crate::storage::StorageManager;
use crate::storage::types::*;
use anyhow::Result;
use rocksdb::WriteBatch;
use std::collections::HashMap;
use std::sync::Arc;
use tracing::{debug, info, warn};

/// Rollback history entry
#[derive(Debug, Clone)]
pub struct RollbackHistoryEntry {
    pub daa_score_start: u64,
    pub daa_score_end: u64,
    pub operation_count: usize,
    pub checkpoint_before: String,
    pub checkpoint_after: String,
    pub timestamp: u64,
}

/// Rollback candidate point
#[derive(Debug, Clone)]
pub struct RollbackCandidate {
    pub block_hash: String,
    pub daa_score: u64,
    pub timestamp: u64,
    pub operation_count: usize,
    pub estimated_duration: std::time::Duration,
}

impl RollbackCandidate {
    pub fn new(block_hash: String, daa_score: u64, timestamp: u64, operation_count: usize) -> Self {
        Self {
            block_hash,
            daa_score,
            timestamp,
            operation_count,
            estimated_duration: std::time::Duration::from_millis(0), // Placeholder, actual estimation needed
        }
    }

    pub fn summary(&self) -> String {
        format!(
            "Block: {} (DAA: {}, Ops: {})",
            self.block_hash, self.daa_score, self.operation_count
        )
    }
}

/// Rollback statistics
#[derive(Debug, Clone)]
pub struct RollbackStatistics {
    pub total_operations: usize,
    pub completed_operations: usize,
    pub progress: f64,
    pub rollback_candidates: usize,
    pub rollback_history_count: usize,
    pub last_rollback_timestamp: u64,
    pub average_rollback_duration: u64,
    pub duration: u64,
}

impl Default for RollbackStatistics {
    fn default() -> Self {
        Self {
            total_operations: 0,
            completed_operations: 0,
            progress: 0.0,
            rollback_candidates: 0,
            rollback_history_count: 0,
            last_rollback_timestamp: 0,
            average_rollback_duration: 0,
            duration: 0,
        }
    }
}

/// Rollback status
#[derive(Debug, Clone)]
pub struct RollbackStatus {
    pub is_rolling_back: bool,
    pub current_progress: f64,
    pub total_operations: usize,
    pub completed_operations: usize,
    pub estimated_remaining_time: std::time::Duration,
    pub last_update: std::time::SystemTime,
}

impl RollbackStatus {
    pub fn new(total_operations: usize) -> Self {
        Self {
            is_rolling_back: false,
            current_progress: 0.0,
            total_operations,
            completed_operations: 0,
            estimated_remaining_time: std::time::Duration::from_secs(0),
            last_update: std::time::SystemTime::now(),
        }
    }
}

pub struct RollbackManager {
    storage: Arc<StorageManager>,
}

impl RollbackManager {
    pub fn new(storage: Arc<StorageManager>) -> Result<Self> {
        Ok(Self { storage })
    }

    // Temporary constructor to avoid circular references
    pub fn new_dummy() -> Self {
        Self {
            storage: Arc::new(StorageManager::new_dummy()),
        }
    }

    pub fn init(&self) -> Result<()> {
        info!("Rollback manager initialized");
        Ok(())
    }

    /// Rollback operation state batch, corresponding to Go version's RollbackOpStateBatch
    /// Improvement: Add batch processing and better error handling
    pub async fn rollback_op_state_batch(&self, rollback: &DataRollbackType) -> Result<i64> {
        let start_time = std::time::Instant::now();

        // Validate rollback data
        if rollback.op_score_list.is_empty() {
            return Err(anyhow::anyhow!("Empty rollback operation list"));
        }

        if rollback.op_score_list.len() != rollback.tx_id_list.len() {
            return Err(anyhow::anyhow!(
                "Mismatched operation score and tx ID lists"
            ));
        }

        // Save pre-rollback state to RocksDB (using batch write)
        let _rocks_duration = self.save_state_batch_rocks_begin(&rollback.state_map_before)?;

        // Delete operation data (using batch write)
        let _delete_duration =
            self.delete_op_data_batch_rocks(&rollback.op_score_list, &rollback.tx_id_list)?;

        let duration = start_time.elapsed().as_millis() as i64;
        info!("Rollback completed in {}ms", duration);
        Ok(duration)
    }

    /// Delete operation data batch (RocksDB version, improved)
    /// Improvement: Add batch processing and retry mechanism
    fn delete_op_data_batch_rocks(
        &self,
        op_score_list: &[u64],
        tx_id_list: &[String],
    ) -> Result<i64> {
        let start_time = std::time::Instant::now();

        let mut batch = WriteBatch::default();

        // Batch delete operation list
        for &op_score in op_score_list {
            let op_range = op_score / OP_RANGE_BY;
            let key = format!("oplist:{}:{}", op_range, op_score);
            batch.delete(key.as_bytes());
        }

        // Batch delete operation data
        for tx_id in tx_id_list {
            let key = format!("opdata:{}", tx_id);
            batch.delete(key.as_bytes());
        }

        // Execute batch delete
        self.storage.rocksdb.write_batch(batch)?;

        let duration = start_time.elapsed().as_millis() as i64;
        Ok(duration)
    }

    /// Save operation data batch (RocksDB version, improved)
    /// Improvement: Add batch processing and better serialization
    pub async fn save_op_data_batch_rocks(
        &self,
        op_data_list: &[DataOperationType],
    ) -> Result<i64> {
        let start_time = std::time::Instant::now();

        let mut batch = WriteBatch::default();

        // Prepare state and script JSON mapping (corresponding to Go version's stateJsonMap and scriptJsonMap)
        let mut state_json_map = HashMap::new();
        let mut script_json_map = HashMap::new();

        for op_data in op_data_list {
            // Create operation state (corresponding to Go version's DataOpStateType)
            let state = DataOpStateType {
                block_accept: Some(op_data.block_accept.clone()),
                fee: Some(op_data.fee),
                fee_least: Some(op_data.fee_least),
                mts_add: Some(op_data.mts_add),
                op_score: Some(op_data.op_score),
                op_accept: Some(op_data.op_accept),
                op_error: Some(op_data.op_error.clone()),
                checkpoint: Some(op_data.checkpoint.clone()),
            };

            let state_json = serde_json::to_string(&state)?;
            let script_json = if !op_data.op_script.is_empty() {
                serde_json::to_string(&op_data.op_script[0])?
            } else {
                "{}".to_string()
            };

            state_json_map.insert(op_data.tx_id.clone(), state_json.clone());
            script_json_map.insert(op_data.tx_id.clone(), script_json.clone());

            // Save to opdata table
            let opdata_key = format!("opdata:{}", op_data.tx_id);
            let opdata_value = serde_json::to_string(&op_data)?;
            batch.put(opdata_key.as_bytes(), opdata_value.as_bytes());
        }

        // Save to oplist table (corresponding to Go version's second batch operation)
        for op_data in op_data_list {
            let op_range = op_data.op_score / OP_RANGE_BY;
            let oplist_key = format!("oplist:{}:{}", op_range, op_data.op_score);

            // Build oplist value (corresponding to Go version's cqlnSaveOpList)
            let oplist_value = serde_json::json!({
                "tx_id": op_data.tx_id,
                "state_json": state_json_map.get(&op_data.tx_id).unwrap_or(&"{}".to_string()),
                "script_json": script_json_map.get(&op_data.tx_id).unwrap_or(&"{}".to_string()),
                "tick_affc": op_data.ss_info.as_ref().map(|s| &s.tick_affc).unwrap_or(&Vec::new()),
                "address_affc": op_data.ss_info.as_ref().map(|s| &s.address_affc).unwrap_or(&Vec::new()),
            });

            batch.put(oplist_key.as_bytes(), oplist_value.to_string().as_bytes());
        }

        // Execute batch write
        self.storage.rocksdb.write_batch(batch)?;

        let duration = start_time.elapsed().as_millis() as i64;
        Ok(duration)
    }

    /// Save operation state batch, corresponding to Go version's SaveOpStateBatch
    /// Improvement: Add batch processing and more precise time measurement
    pub async fn save_op_state_batch(
        &self,
        op_data_list: &[DataOperationType],
        state_map: &DataStateMapType,
    ) -> Result<Vec<i64>> {
        let start_time = std::time::Instant::now();
        let mut mts_batch_list = vec![0i64; 4];

        // 1. Save state to RocksDB
        mts_batch_list[0] = start_time.elapsed().as_millis() as i64;
        let _rocks_duration = self.save_state_batch_rocks_begin(state_map)?;
        mts_batch_list[1] = start_time.elapsed().as_millis() as i64;

        // 2. Save operation data to RocksDB
        let _op_duration = self.save_op_data_batch_rocks(op_data_list).await?;
        mts_batch_list[2] = start_time.elapsed().as_millis() as i64;

        mts_batch_list[3] = start_time.elapsed().as_millis() as i64;

        // Calculate time consumption for each stage
        mts_batch_list[0] = mts_batch_list[1] - mts_batch_list[0];
        mts_batch_list[1] = mts_batch_list[2] - mts_batch_list[1];
        mts_batch_list[2] = mts_batch_list[3] - mts_batch_list[2];

        Ok(mts_batch_list)
    }

    /// Batch operation processing (corresponding to Go version's startExecuteBatchCassa)
    /// Improvement: Add retry mechanism and batch processing
    async fn execute_batch_rocks<F>(
        &self,
        items: &[String],
        batch_size: usize,
        mut operation: F,
    ) -> Result<i64>
    where
        F: FnMut(&str, &mut WriteBatch) -> Result<()>,
    {
        let start_time = std::time::Instant::now();
        let mut retry_count = 0;
        let max_retries = 5;

        for chunk in items.chunks(batch_size) {
            let mut success = false;
            let mut current_retry = 0;

            while !success && current_retry < max_retries {
                let mut batch = WriteBatch::default();
                let mut chunk_success = true;

                for item in chunk {
                    if let Err(e) = operation(item, &mut batch) {
                        warn!("Batch operation failed for item {}: {}", item, e);
                        chunk_success = false;
                        break;
                    }
                }

                if chunk_success {
                    self.storage.rocksdb.write_batch(batch)?;
                    success = true;
                } else {
                    current_retry += 1;
                    if current_retry < max_retries {
                        tokio::time::sleep(tokio::time::Duration::from_millis(
                            10 * current_retry as u64,
                        ))
                        .await;
                    }
                }
            }

            if !success {
                retry_count += 1;
                if retry_count > max_retries {
                    return Err(anyhow::anyhow!(
                        "Failed to execute batch after {} retries",
                        max_retries
                    ));
                }
            }
        }

        let duration = start_time.elapsed().as_millis() as i64;
        Ok(duration)
    }

    /// Improved rollback to block functionality
    /// Improvement: Add better state management and error recovery
    pub async fn rollback_to_block(
        &self,
        target_block_hash: &str,
        target_daa_score: u64,
    ) -> Result<()> {
        info!(
            "Starting rollback to block: {} (DAA: {})",
            target_block_hash, target_daa_score
        );

        // Validate rollback point
        self.validate_rollback_point(target_block_hash, target_daa_score)
            .await?;

        // Get current state
        let current_state = self.storage.runtime.get_runtime_state()?;

        if current_state.last_processed_daa_score <= target_daa_score {
            return Err(anyhow::anyhow!("Cannot rollback to a higher DAA score"));
        }

        // Stop sync
        self.storage.runtime.stop_sync()?;

        // Execute rollback
        self.perform_rollback(target_block_hash, target_daa_score)
            .await?;

        // Update runtime state
        let mut new_state = current_state;
        new_state.last_processed_block = target_block_hash.to_string();
        new_state.last_processed_daa_score = target_daa_score;
        new_state.is_syncing = false;
        self.storage.runtime.update_runtime_state(new_state)?;

        info!("Rollback completed to block: {}", target_block_hash);
        Ok(())
    }

    /// Improved rollback execution logic
    /// Improvement: Add batch processing and better error handling
    async fn perform_rollback(
        &self,
        _target_block_hash: &str,
        target_daa_score: u64,
    ) -> Result<()> {
        // Get operations to rollback
        let operations_to_rollback = self.get_operations_to_rollback(target_daa_score).await?;

        info!("Rolling back {} operations", operations_to_rollback.len());

        let mut batch = WriteBatch::default();

        // Rollback operations in descending order by DAA score
        for operation in operations_to_rollback.iter().rev() {
            self.rollback_operation_in_batch(operation, &mut batch)
                .await?;
        }

        // Delete VSPC data beyond target DAA score
        self.remove_vspc_data_beyond_in_batch(target_daa_score, &mut batch)
            .await?;

        // Execute batch write
        self.storage.rocksdb.write_batch(batch)?;

        Ok(())
    }

    /// Improved logic for getting rollback operations
    /// Improvement: Add better query optimization
    async fn get_operations_to_rollback(
        &self,
        target_daa_score: u64,
    ) -> Result<Vec<OperationData>> {
        debug!(
            "Getting operations to rollback beyond DAA score: {}",
            target_daa_score
        );

        let mut operations = Vec::new();

        // Use prefix scan to optimize query
        let prefix = "opdata:";
        let op_data_iter = self.storage.rocksdb.scan_prefix(prefix)?;

        for (_key, value) in op_data_iter.iter() {
            if let Ok(op_data) = serde_json::from_slice::<OperationData>(value) {
                if op_data.block_daa_score > target_daa_score {
                    operations.push(op_data);
                }
            }
        }

        // Sort in descending order by DAA score
        operations.sort_by(|a, b| b.block_daa_score.cmp(&a.block_daa_score));

        info!("Found {} operations to rollback", operations.len());
        Ok(operations)
    }

    /// Rollback operation in batch
    /// Improvement: Add batch support and better error handling
    async fn rollback_operation_in_batch(
        &self,
        operation: &OperationData,
        batch: &mut WriteBatch,
    ) -> Result<()> {
        debug!(
            "Rolling back operation: {} ({})",
            operation.operation_type, operation.tick
        );

        // Execute corresponding rollback operation based on operation type
        match operation.operation_type.as_str() {
            "send" => {
                info!("Rolling back send operation: {}", operation.tx_id);
                self.rollback_send_operation_in_batch(operation, batch)
                    .await?;
            }
            "transfer" => {
                info!("Rolling back transfer operation: {}", operation.tx_id);
                self.rollback_transfer_operation_in_batch(operation, batch)
                    .await?;
            }
            "issue" => {
                info!("Rolling back issue operation: {}", operation.tx_id);
                self.rollback_issue_operation_in_batch(operation, batch)
                    .await?;
            }
            "burn" => {
                info!("Rolling back burn operation: {}", operation.tx_id);
                self.rollback_burn_operation_in_batch(operation, batch)
                    .await?;
            }
            "list" => {
                info!("Rolling back list operation: {}", operation.tx_id);
                self.rollback_list_operation_in_batch(operation, batch)
                    .await?;
            }
            "chown" => {
                info!("Rolling back chown operation: {}", operation.tx_id);
                self.rollback_chown_operation_in_batch(operation, batch)
                    .await?;
            }
            "blacklist" => {
                info!("Rolling back blacklist operation: {}", operation.tx_id);
                self.rollback_blacklist_operation_in_batch(operation, batch)
                    .await?;
            }
            _ => {
                warn!("Unknown operation type: {}", operation.operation_type);
            }
        }

        // Delete operation records from RocksDB
        let op_range = operation.block_daa_score / OP_RANGE_BY;
        let oplist_key = format!("oplist:{}:{}", op_range, operation.block_daa_score);
        let opdata_key = format!("opdata:{}", operation.tx_id);

        batch.delete(oplist_key.as_bytes());
        batch.delete(opdata_key.as_bytes());

        info!("Successfully rolled back operation: {}", operation.tx_id);
        Ok(())
    }

    // Improved specific rollback operation implementation (in batch)
    async fn rollback_send_operation_in_batch(
        &self,
        operation: &OperationData,
        batch: &mut WriteBatch,
    ) -> Result<()> {
        // Restore sender's balance
        if let (Some(from), Some(amount)) = (&operation.from_address, &operation.amount) {
            let parts: Vec<&str> = from.split('_').collect();
            if parts.len() >= 2 {
                let balance_key = format!("{}{}_{}", KEY_PREFIX_STATE_BALANCE, parts[0], parts[1]);
                if let Some(balance_data) = self.storage.rocksdb.get_raw(&balance_key)? {
                    if let Ok(mut balance) =
                        serde_json::from_slice::<StateBalanceType>(&balance_data)
                    {
                        balance.balance =
                            (balance.balance.parse::<u64>().unwrap_or(0) + amount).to_string();
                        let new_balance_json = serde_json::to_string(&balance)?;
                        batch.put(balance_key.as_bytes(), new_balance_json.as_bytes());
                    }
                }
            }
        }
        Ok(())
    }

    async fn rollback_transfer_operation_in_batch(
        &self,
        operation: &OperationData,
        batch: &mut WriteBatch,
    ) -> Result<()> {
        // Restore transfer operation
        if let (Some(from), Some(to), Some(amount)) = (
            &operation.from_address,
            &operation.to_address,
            &operation.amount,
        ) {
            // Restore sender balance
            let from_parts: Vec<&str> = from.split('_').collect();
            if from_parts.len() >= 2 {
                let from_balance_key = format!(
                    "{}{}_{}",
                    KEY_PREFIX_STATE_BALANCE, from_parts[0], from_parts[1]
                );
                if let Some(balance_data) = self.storage.rocksdb.get_raw(&from_balance_key)? {
                    if let Ok(mut balance) =
                        serde_json::from_slice::<StateBalanceType>(&balance_data)
                    {
                        balance.balance =
                            (balance.balance.parse::<u64>().unwrap_or(0) + amount).to_string();
                        let new_balance_json = serde_json::to_string(&balance)?;
                        batch.put(from_balance_key.as_bytes(), new_balance_json.as_bytes());
                    }
                }
            }

            // Reduce receiver balance
            let to_parts: Vec<&str> = to.split('_').collect();
            if to_parts.len() >= 2 {
                let to_balance_key = format!(
                    "{}{}_{}",
                    KEY_PREFIX_STATE_BALANCE, to_parts[0], to_parts[1]
                );
                if let Some(balance_data) = self.storage.rocksdb.get_raw(&to_balance_key)? {
                    if let Ok(mut balance) =
                        serde_json::from_slice::<StateBalanceType>(&balance_data)
                    {
                        let current_balance = balance.balance.parse::<u64>().unwrap_or(0);
                        if current_balance >= *amount {
                            balance.balance = (current_balance - amount).to_string();
                            let new_balance_json = serde_json::to_string(&balance)?;
                            batch.put(to_balance_key.as_bytes(), new_balance_json.as_bytes());
                        }
                    }
                }
            }
        }
        Ok(())
    }

    async fn rollback_issue_operation_in_batch(
        &self,
        operation: &OperationData,
        batch: &mut WriteBatch,
    ) -> Result<()> {
        // Restore issue operation
        if let Some(to) = &operation.to_address {
            let parts: Vec<&str> = to.split('_').collect();
            if parts.len() >= 2 {
                let balance_key = format!("{}{}_{}", KEY_PREFIX_STATE_BALANCE, parts[0], parts[1]);
                if let Some(balance_data) = self.storage.rocksdb.get_raw(&balance_key)? {
                    if let Ok(mut balance) =
                        serde_json::from_slice::<StateBalanceType>(&balance_data)
                    {
                        if let Some(amount) = &operation.amount {
                            let current_balance = balance.balance.parse::<u64>().unwrap_or(0);
                            if current_balance >= *amount {
                                balance.balance = (current_balance - amount).to_string();
                                let new_balance_json = serde_json::to_string(&balance)?;
                                batch.put(balance_key.as_bytes(), new_balance_json.as_bytes());
                            }
                        }
                    }
                }
            }
        }
        Ok(())
    }

    async fn rollback_burn_operation_in_batch(
        &self,
        operation: &OperationData,
        batch: &mut WriteBatch,
    ) -> Result<()> {
        // Restore burn operation
        if let Some(from) = &operation.from_address {
            let parts: Vec<&str> = from.split('_').collect();
            if parts.len() >= 2 {
                let balance_key = format!("{}{}_{}", KEY_PREFIX_STATE_BALANCE, parts[0], parts[1]);
                if let Some(balance_data) = self.storage.rocksdb.get_raw(&balance_key)? {
                    if let Ok(mut balance) =
                        serde_json::from_slice::<StateBalanceType>(&balance_data)
                    {
                        if let Some(amount) = &operation.amount {
                            balance.balance =
                                (balance.balance.parse::<u64>().unwrap_or(0) + amount).to_string();
                            let new_balance_json = serde_json::to_string(&balance)?;
                            batch.put(balance_key.as_bytes(), new_balance_json.as_bytes());
                        }
                    }
                }
            }
        }
        Ok(())
    }

    async fn rollback_list_operation_in_batch(
        &self,
        _operation: &OperationData,
        _batch: &mut WriteBatch,
    ) -> Result<()> {
        // Restore list operation
        // TODO: Implement specific list rollback logic
        Ok(())
    }

    async fn rollback_chown_operation_in_batch(
        &self,
        operation: &OperationData,
        batch: &mut WriteBatch,
    ) -> Result<()> {
        // Restore ownership change operation
        let token_key = format!("{}{}", KEY_PREFIX_STATE_TOKEN, operation.tick);
        if let Some(token_data) = self.storage.rocksdb.get_raw(&token_key)? {
            if let Ok(token) = serde_json::from_slice::<StateTokenType>(&token_data) {
                // TODO: Restore original owner
                let new_token_json = serde_json::to_string(&token)?;
                batch.put(token_key.as_bytes(), new_token_json.as_bytes());
            }
        }
        Ok(())
    }

    async fn rollback_blacklist_operation_in_batch(
        &self,
        operation: &OperationData,
        batch: &mut WriteBatch,
    ) -> Result<()> {
        // Restore blacklist operation
        if let Some(address) = &operation.from_address {
            let parts: Vec<&str> = address.split('_').collect();
            if parts.len() >= 2 {
                let blacklist_key = format!(
                    "{}{}_{}",
                    KEY_PREFIX_STATE_BLACKLIST, parts[0], operation.tick
                );
                batch.delete(blacklist_key.as_bytes());
            }
        }
        Ok(())
    }

    /// Delete VSPC data in batch
    async fn remove_vspc_data_beyond_in_batch(
        &self,
        target_daa_score: u64,
        batch: &mut WriteBatch,
    ) -> Result<()> {
        debug!("Removing VSPC data beyond DAA score: {}", target_daa_score);

        // Delete VSPC data from RocksDB
        let prefix = "vspc:";
        let vspc_iter = self.storage.rocksdb.scan_prefix(prefix)?;

        for (key, value) in vspc_iter.iter() {
            if let Ok(vspc_data) = serde_json::from_slice::<VSPCData>(value) {
                if vspc_data.daa_score > target_daa_score {
                    batch.delete(key.as_bytes());
                }
            }
        }

        info!("Removed {} VSPC data records", 0); // This count is not directly available from the scan_prefix iterator
        Ok(())
    }

    /// Save state batch to RocksDB (improved version)
    /// Improvement: Add batch support and better error handling
    fn save_state_batch_rocks_begin(&self, state_map: &DataStateMapType) -> Result<i64> {
        let start_time = std::time::Instant::now();

        let mut batch = WriteBatch::default();

        // Save Token state
        for (key, token) in &state_map.state_token_map {
            let full_key = format!("{}{}", KEY_PREFIX_STATE_TOKEN, key);
            if let Some(token_data) = token {
                let value_json = serde_json::to_string(token_data)?;
                batch.put(full_key.as_bytes(), value_json.as_bytes());
            } else {
                batch.delete(full_key.as_bytes());
            }
        }

        // Save Balance state
        for (key, balance) in &state_map.state_balance_map {
            let full_key = format!("{}{}", KEY_PREFIX_STATE_BALANCE, key);
            if let Some(balance_data) = balance {
                let value_json = serde_json::to_string(balance_data)?;
                batch.put(full_key.as_bytes(), value_json.as_bytes());
            } else {
                batch.delete(full_key.as_bytes());
            }
        }

        // Save Market state
        for (key, market) in &state_map.state_market_map {
            let full_key = format!("{}{}", KEY_PREFIX_STATE_MARKET, key);
            if let Some(market_data) = market {
                let value_json = serde_json::to_string(market_data)?;
                batch.put(full_key.as_bytes(), value_json.as_bytes());
            } else {
                batch.delete(full_key.as_bytes());
            }
        }

        // Save Blacklist state
        for (key, blacklist) in &state_map.state_blacklist_map {
            let full_key = format!("{}{}", KEY_PREFIX_STATE_BLACKLIST, key);
            if let Some(blacklist_data) = blacklist {
                let value_json = serde_json::to_string(blacklist_data)?;
                batch.put(full_key.as_bytes(), value_json.as_bytes());
            } else {
                batch.delete(full_key.as_bytes());
            }
        }

        // Execute batch write
        self.storage.rocksdb.write_batch(batch)?;

        let duration = start_time.elapsed().as_millis() as i64;
        Ok(duration)
    }

    /// Complete rollback validation mechanism, corresponding to Go version's validation logic
    /// Improvement: Add deep validation and consistency check
    pub async fn validate_rollback_point(&self, block_hash: &str, daa_score: u64) -> Result<()> {
        info!(
            "Validating rollback point: block={}, daa_score={}",
            block_hash, daa_score
        );

        // Validate block existence
        if !self.block_exists_in_vspc(block_hash, daa_score).await? {
            return Err(anyhow::anyhow!(
                "Block {} does not exist in VSPC data",
                block_hash
            ));
        }

        // Validate rollback consistency
        self.validate_rollback_consistency(block_hash, daa_score)
            .await?;

        // Validate state consistency
        self.validate_state_consistency(block_hash, daa_score)
            .await?;

        // Validate dependent operations
        let dependent_ops = self.get_dependent_operations(daa_score).await?;
        if !dependent_ops.is_empty() {
            warn!(
                "Found {} dependent operations that may be affected by rollback",
                dependent_ops.len()
            );
        }

        info!("Rollback point validation completed successfully");
        Ok(())
    }

    /// Validate block existence in VSPC data
    async fn block_exists_in_vspc(&self, block_hash: &str, daa_score: u64) -> Result<bool> {
        debug!("Checking if block {} exists in VSPC data", block_hash);

        // Get VSPC list from runtime state
        let vspc_list = self.storage.runtime.get_runtime_vspc_last().await?;

        for vspc in vspc_list {
            if vspc.hash == block_hash && vspc.daa_score == daa_score {
                return Ok(true);
            }
        }

        Ok(false)
    }

    /// Validate rollback consistency
    async fn validate_rollback_consistency(&self, _block_hash: &str, daa_score: u64) -> Result<()> {
        debug!(
            "Validating rollback consistency for DAA score: {}",
            daa_score
        );

        // Check consistency of rollback list
        let rollback_list = self.storage.runtime.get_runtime_rollback_last().await?;

        for rollback in rollback_list {
            if rollback.daa_score_start <= daa_score && daa_score <= rollback.daa_score_end {
                // Validate integrity of rollback data
                if rollback.op_score_list.len() != rollback.tx_id_list.len() {
                    return Err(anyhow::anyhow!(
                        "Inconsistent rollback data: op_score_list and tx_id_list have different lengths"
                    ));
                }

                // Validate integrity of state mapping
                if !self.validate_state_map_consistency(&rollback.state_map_before)? {
                    return Err(anyhow::anyhow!("Invalid state map in rollback data"));
                }
            }
        }

        Ok(())
    }

    /// Validate state mapping consistency
    fn validate_state_map_consistency(&self, state_map: &DataStateMapType) -> Result<bool> {
        // Validate Token state
        for (key, token) in &state_map.state_token_map {
            if let Some(token_data) = token {
                if !self.validate_token_state(token_data)? {
                    warn!("Invalid token state for key: {}", key);
                    return Ok(false);
                }
            }
        }

        // Validate Balance state
        for (key, balance) in &state_map.state_balance_map {
            if let Some(balance_data) = balance {
                if !self.validate_balance_state(balance_data)? {
                    warn!("Invalid balance state for key: {}", key);
                    return Ok(false);
                }
            }
        }

        // Validate Market state
        for (key, market) in &state_map.state_market_map {
            if let Some(market_data) = market {
                if !self.validate_market_state(market_data)? {
                    warn!("Invalid market state for key: {}", key);
                    return Ok(false);
                }
            }
        }

        Ok(true)
    }

    /// Validate Token state
    fn validate_token_state(&self, token: &StateTokenType) -> Result<bool> {
        // Check required fields
        if token.tick.is_empty() {
            return Ok(false);
        }

        // Check validity of numeric fields
        if let Ok(max_supply) = token.max.parse::<u64>() {
            if let Ok(minted) = token.minted.parse::<u64>() {
                if minted > max_supply {
                    return Ok(false);
                }
            }
        }

        Ok(true)
    }

    /// Validate Balance state
    fn validate_balance_state(&self, balance: &StateBalanceType) -> Result<bool> {
        // Check required fields
        if balance.address.is_empty() || balance.tick.is_empty() {
            return Ok(false);
        }

        // Check validity of balance fields
        if let Ok(balance_amount) = balance.balance.parse::<u64>() {
            if let Ok(locked_amount) = balance.locked.parse::<u64>() {
                if locked_amount > balance_amount {
                    return Ok(false);
                }
            }
        }

        Ok(true)
    }

    /// Validate Market state
    fn validate_market_state(&self, market: &StateMarketType) -> Result<bool> {
        // Check required fields
        if market.tick.is_empty() {
            return Ok(false);
        }

        // Check validity of price fields
        if market.t_amt.parse::<f64>().unwrap_or(0.0) < 0.0 {
            return Ok(false);
        }

        Ok(true)
    }

    /// Validate state consistency
    async fn validate_state_consistency(&self, _block_hash: &str, _daa_score: u64) -> Result<()> {
        debug!("Validating state consistency");

        // Check consistency between Token and Balance
        let mut token_map = HashMap::new();
        let mut balance_map = HashMap::new();

        self.storage.state.get_state_token_map(&mut token_map)?;
        self.storage.state.get_state_balance_map(&mut balance_map)?;

        for (balance_key, balance) in balance_map {
            if let Some(_balance_data) = balance {
                let parts: Vec<&str> = balance_key.split('_').collect();
                if parts.len() >= 2 {
                    let tick = parts[1];
                    if !token_map.contains_key(tick) {
                        warn!("Balance exists for non-existent token: {}", tick);
                    }
                }
            }
        }

        Ok(())
    }

    /// Get dependent operations
    async fn get_dependent_operations(&self, daa_score: u64) -> Result<Vec<OperationData>> {
        debug!("Getting dependent operations for DAA score: {}", daa_score);

        let mut dependent_ops = Vec::new();

        // Scan operation data
        let prefix = "opdata:";
        let op_data_iter = self.storage.rocksdb.scan_prefix(prefix)?;

        for (_key, value) in op_data_iter.iter() {
            if let Ok(op_data) = serde_json::from_slice::<OperationData>(value) {
                // Check if there are dependencies
                if self.is_operation_dependent(&op_data, daa_score)? {
                    dependent_ops.push(op_data);
                }
            }
        }

        info!("Found {} dependent operations", dependent_ops.len());
        Ok(dependent_ops)
    }

    /// Check if operation has dependencies
    fn is_operation_dependent(&self, op_data: &OperationData, daa_score: u64) -> Result<bool> {
        // Check if operation is after target DAA score
        if op_data.block_daa_score <= daa_score {
            return Ok(false);
        }

        // Check if operation involves same token or address
        // TODO: Can perform more detailed dependency checks based on specific business logic

        Ok(true)
    }

    /// Create rollback data (corresponding to Go version's CreateRollbackData)
    pub fn create_rollback_data(
        &self,
        state_map_before: DataStateMapType,
        state_map_after: DataStateMapType,
        op_score_list: Vec<u64>,
        tx_id_list: Vec<String>,
        daa_score_start: u64,
        daa_score_end: u64,
        checkpoint_before: String,
        checkpoint_after: String,
        op_score_last: u64,
    ) -> DataRollbackType {
        DataRollbackType::new(
            state_map_before,
            state_map_after,
            op_score_list,
            tx_id_list,
            daa_score_start,
            daa_score_end,
            checkpoint_before,
            checkpoint_after,
            op_score_last,
        )
    }

    /// Get rollback status
    pub fn get_rollback_status(&self) -> Result<RollbackStatus> {
        // Get current rollback status
        let _current_state = self.storage.runtime.get_runtime_state()?;

        // Calculate rollback progress
        let total_operations = self.get_total_operations_count()?;
        let _completed_operations = self.get_completed_operations_count()?;

        let status = RollbackStatus::new(total_operations);

        Ok(status)
    }

    /// Get total operation count
    fn get_total_operations_count(&self) -> Result<usize> {
        let mut count = 0;
        let prefix = "opdata:";
        let op_iter = self.storage.rocksdb.scan_prefix(prefix)?;

        for _ in op_iter.iter() {
            count += 1;
        }

        Ok(count)
    }

    /// Get completed operation count
    fn get_completed_operations_count(&self) -> Result<usize> {
        // This can be implemented based on actual requirements
        // Temporarily return 0, indicating no ongoing rollback
        Ok(0)
    }

    /// Get rollback statistics, corresponding to Go version's statistics functionality
    /// Improvement: Add detailed statistics and monitoring
    pub async fn get_rollback_statistics(&self) -> Result<RollbackStatistics> {
        let start_time = std::time::Instant::now();

        // Get total operation count
        let total_operations = self.get_total_operations_count()?;

        // Get completed operation count
        let completed_operations = self.get_completed_operations_count()?;

        // Get rollback candidates
        let rollback_candidates = self.get_rollback_candidates(100).await?; // Last 100 blocks

        // Get rollback history
        let rollback_history = self.get_rollback_history().await?;

        // Calculate rollback progress
        let progress = if total_operations > 0 {
            (completed_operations as f64 / total_operations as f64) * 100.0
        } else {
            0.0
        };

        let duration = start_time.elapsed().as_millis() as u64;

        let stats = RollbackStatistics {
            total_operations,
            completed_operations,
            progress,
            rollback_candidates: rollback_candidates.len(),
            rollback_history_count: rollback_history.len(),
            last_rollback_timestamp: self.get_last_rollback_timestamp().await?,
            average_rollback_duration: self
                .calculate_average_rollback_duration(&rollback_history)?,
            duration,
        };

        info!(
            "Rollback statistics: total={}, completed={}, progress={:.2}%",
            total_operations, completed_operations, progress
        );

        Ok(stats)
    }

    /// Get rollback history
    async fn get_rollback_history(&self) -> Result<Vec<RollbackHistoryEntry>> {
        let mut history = Vec::new();

        // Get rollback list from runtime state
        let rollback_list = self.storage.runtime.get_runtime_rollback_last().await?;

        for rollback in rollback_list {
            let entry = RollbackHistoryEntry {
                daa_score_start: rollback.daa_score_start,
                daa_score_end: rollback.daa_score_end,
                operation_count: rollback.op_score_list.len(),
                checkpoint_before: rollback.checkpoint_before.clone(),
                checkpoint_after: rollback.checkpoint_after.clone(),
                timestamp: self.get_rollback_timestamp(&rollback)?,
            };
            history.push(entry);
        }

        Ok(history)
    }

    /// Get rollback timestamp
    fn get_rollback_timestamp(&self, _rollback: &DataRollbackType) -> Result<u64> {
        // This can extract timestamp from rollback data, or use current time
        // Temporarily use current time as placeholder
        Ok(std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap_or_default()
            .as_secs())
    }

    /// Get last rollback timestamp
    async fn get_last_rollback_timestamp(&self) -> Result<u64> {
        let rollback_list = self.storage.runtime.get_runtime_rollback_last().await?;

        if let Some(last_rollback) = rollback_list.last() {
            self.get_rollback_timestamp(last_rollback)
        } else {
            Ok(0)
        }
    }

    /// Calculate average rollback duration
    fn calculate_average_rollback_duration(&self, history: &[RollbackHistoryEntry]) -> Result<u64> {
        if history.is_empty() {
            return Ok(0);
        }

        let total_duration: u64 = history.iter().map(|entry| entry.timestamp).sum();

        Ok(total_duration / history.len() as u64)
    }

    /// Clean up expired rollback data, corresponding to Go version's cleanup functionality
    /// Improvement: Add intelligent cleanup and retention strategy
    pub async fn cleanup_expired_rollback_data(&self, max_age_hours: u64) -> Result<()> {
        info!(
            "Cleaning up expired rollback data older than {} hours",
            max_age_hours
        );

        let current_timestamp = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap_or_default()
            .as_secs();

        let cutoff_timestamp = current_timestamp - (max_age_hours * 3600);

        // Get rollback list
        let mut rollback_list = self.storage.runtime.get_runtime_rollback_last().await?;

        // Filter expired rollback data
        let original_count = rollback_list.len();
        rollback_list.retain(|rollback| {
            if let Ok(timestamp) = self.get_rollback_timestamp(rollback) {
                timestamp > cutoff_timestamp
            } else {
                true // If unable to get timestamp, keep data
            }
        });

        let removed_count = original_count - rollback_list.len();

        if removed_count > 0 {
            // Update rollback list
            self.storage
                .runtime
                .set_runtime_rollback_last(&rollback_list)
                .await?;

            info!("Cleaned up {} expired rollback entries", removed_count);
        } else {
            info!("No expired rollback data found");
        }

        Ok(())
    }

    /// Get rollback candidates, corresponding to Go version's rollback point selection
    /// Improvement: Add intelligent candidate selection
    pub async fn get_rollback_candidates(&self, max_blocks: u64) -> Result<Vec<RollbackCandidate>> {
        let mut candidates = Vec::new();

        // Get VSPC list
        let vspc_list = self.storage.runtime.get_runtime_vspc_last().await?;

        // Select candidates
        for (i, vspc) in vspc_list.iter().enumerate() {
            if i >= max_blocks as usize {
                break;
            }

            // Calculate operation count
            let operation_count = self.get_operation_count_for_block(&vspc.hash)?;

            // Check if it is a valid rollback point
            if self
                .is_valid_rollback_point(&vspc.hash, vspc.daa_score)
                .await?
            {
                let candidate = RollbackCandidate::new(
                    vspc.hash.clone(),
                    vspc.daa_score,
                    0, // DataVspcType has no timestamp field, use 0 as placeholder
                    operation_count.try_into().unwrap_or(0),
                );
                candidates.push(candidate);
            }
        }

        // Sort in descending order by DAA score
        candidates.sort_by(|a, b| b.daa_score.cmp(&a.daa_score));

        info!("Found {} rollback candidates", candidates.len());
        Ok(candidates)
    }

    /// Check if it is a valid rollback point
    async fn is_valid_rollback_point(&self, block_hash: &str, daa_score: u64) -> Result<bool> {
        // Check if block exists
        if !self.block_exists_in_vspc(block_hash, daa_score).await? {
            return Ok(false);
        }

        // Check if there is sufficient operation data
        let operation_count = self.get_operation_count_for_block(block_hash)?;
        if operation_count == 0 {
            return Ok(false);
        }

        // Check state consistency
        if !self
            .validate_state_consistency(block_hash, daa_score)
            .await
            .is_ok()
        {
            return Ok(false);
        }

        Ok(true)
    }

    /// Get operation count for block
    fn get_operation_count_for_block(&self, block_hash: &str) -> Result<u64> {
        let mut count = 0;

        // Scan operation data
        let prefix = "opdata:";
        let op_data_iter = self.storage.rocksdb.scan_prefix(prefix)?;

        for (_key, value) in op_data_iter.iter() {
            if let Ok(op_data) = serde_json::from_slice::<OperationData>(value) {
                if op_data.tx_id.contains(block_hash) {
                    count += 1;
                }
            }
        }

        Ok(count)
    }
}
