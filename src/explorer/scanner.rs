use crate::config::types::StartupConfig;
use crate::explorer::RollbackManager;
use crate::explorer::ScanStats;
use crate::operations::handler::OperationManager;
use crate::storage::StorageManager;
use crate::storage::types::*;
use anyhow::Result;
use reqwest::Client;
use serde_json::Value;
use std::collections::HashMap;
use std::sync::Arc;
use std::time::{Duration, SystemTime, UNIX_EPOCH};
use tokio::time::sleep;
use tracing::{debug, error, info};

// Constant definitions, consistent with Go version
const LEN_VSPC_LIST_MAX: usize = 1200;
const LEN_VSPC_LIST_RUNTIME_MAX: usize = 3600;
const LEN_VSPC_CHECK: usize = 200;
const LEN_ROLLBACK_LIST_RUNTIME_MAX: usize = 3600;

// DAA Score range, consistent with Go version
const DAA_SCORE_RANGE: [[u64; 2]; 2] = [[83441551, 83525600], [90090600, 18446744073709551615]];

pub struct VSPCScanner {
    last_scan_time: u64,
    total_vspc_processed: u64,
    total_operations_found: u64,
    storage: Arc<StorageManager>,
    config: StartupConfig,
    is_scanning: bool,
    scan_start_time: u64,
    http_client: Client,
    // Runtime state from Go version
    vspc_list: Vec<DataVspcType>,
    rollback_list: Vec<DataRollbackType>,
    synced: bool,
    op_score_last: u64,
    testnet: bool,
    // Add operation manager
    operation_manager: Option<OperationManager>,
    // Add rollback manager
    rollback_manager: RollbackManager,
}

impl VSPCScanner {
    pub fn new(storage: Arc<StorageManager>, config: StartupConfig, testnet: bool) -> Result<Self> {
        let rollback_manager = RollbackManager::new(storage.clone())?;

        Ok(Self {
            storage,
            config,
            is_scanning: false,
            last_scan_time: 0,
            total_vspc_processed: 0,
            total_operations_found: 0,
            scan_start_time: 0,
            http_client: Client::new(),
            vspc_list: Vec::new(),
            rollback_list: Vec::new(),
            synced: false,
            op_score_last: 0,
            testnet,
            operation_manager: None,
            rollback_manager,
        })
    }

    pub async fn init(&mut self) -> Result<()> {
        info!("explorer.Init start.");

        // Set sync state to false, consistent with Go version
        self.synced = false;

        // Handle hysteresis configuration, consistent with Go version
        if self.config.hysteresis < 0 {
            // Need to modify config structure to support hysteresis field
        } else if self.config.hysteresis > 1000 {
            // Need to modify config structure to support hysteresis field
        }

        // Handle DAA Score range, consistent with Go version
        if !self.testnet || self.config.daa_score_range.is_empty() {
            // Use default DAA_SCORE_RANGE
        }

        // Handle reserved ticks, consistent with Go version
        if self.testnet && !self.config.tick_reserved.is_empty() {
            // Apply reserved ticks
        }

        // Get rollback list, consistent with Go version
        self.rollback_list = self.storage.runtime.get_runtime_rollback_last().await?;

        // Get VSPC list, consistent with Go version
        self.vspc_list = self.storage.runtime.get_runtime_vspc_last().await?;

        // Set op_score_last, consistent with Go version
        if let Some(last_rollback) = self.rollback_list.last() {
            self.op_score_last = last_rollback.op_score_last;
        }

        // Set sync state, consistent with Go version
        if !self.vspc_list.is_empty() {
            let vspc_last = self.vspc_list.last().unwrap();
            info!(
                "explorer.Init lastVspcDaaScore={} lastVspcBlockHash={}",
                vspc_last.daa_score, vspc_last.hash
            );
            self.storage
                .runtime
                .set_runtime_synced(false, self.op_score_last, vspc_last.daa_score)
                .await?;
        } else {
            info!(
                "explorer.Init lastVspcDaaScore={} lastVspcBlockHash=",
                DAA_SCORE_RANGE[0][0]
            );
            self.storage
                .runtime
                .set_runtime_synced(false, self.op_score_last, DAA_SCORE_RANGE[0][0])
                .await?;
        }

        // Initialize operation manager
        self.operation_manager = Some(OperationManager::new(self.storage.clone()));

        // Initialize rollback manager
        self.rollback_manager.init()?;

        info!("explorer ready.");

        Ok(())
    }

    pub async fn start_scanning(&mut self) -> Result<()> {
        self.is_scanning = true;
        self.scan_start_time = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap()
            .as_secs();

        info!("Starting VSPC scanning...");

        while self.is_scanning {
            if let Err(e) = self.scan_vspc_batch().await {
                error!("Error scanning VSPC batch: {}", e);
                sleep(Duration::from_secs(3)).await;
                continue;
            }

            // Sleep between scans
            sleep(Duration::from_secs(1)).await;
        }

        Ok(())
    }

    pub async fn stop_scanning(&mut self) -> Result<()> {
        self.is_scanning = false;
        info!("VSPC scanning stopped");
        Ok(())
    }

    async fn scan_vspc_batch(&mut self) -> Result<()> {
        let start_time = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap()
            .as_millis() as u64;

        // Get next VSPC data list, consistent with Go version
        let mut vspc_last = DataVspcType {
            daa_score: DAA_SCORE_RANGE[0][0],
            hash: String::new(),
            tx_id_list: Vec::new(),
        };
        let mut daa_score_start = vspc_last.daa_score;

        // Use last VSPC (if list is not empty), consistent with Go version
        let len_vspc_runtime = self.vspc_list.len();
        if len_vspc_runtime > 0 {
            vspc_last = self.vspc_list[len_vspc_runtime - 1].clone();
            daa_score_start = vspc_last.daa_score - LEN_VSPC_CHECK as u64;
            if daa_score_start < self.vspc_list[0].daa_score {
                daa_score_start = self.vspc_list[0].daa_score;
            }
        }

        // Get next VSPC data list from cluster database - call StateManager method
        let vspc_list_next = self
            .storage
            .state
            .get_node_vspc_list(daa_score_start, LEN_VSPC_LIST_MAX + 5)
            .await?;

        if vspc_list_next.is_empty() {
            debug!(
                "storage.GetNodeVspcList empty. daaScore: {}",
                daa_score_start
            );
            sleep(Duration::from_millis(1550)).await;
            return Ok(());
        }

        let mut len_vspc_next = vspc_list_next.len();

        // If synced, ignore last reserved VSPC data to reduce VSPC reorganization probability
        if self.synced {
            len_vspc_next -= self.config.hysteresis as usize;
        }

        if len_vspc_next <= 0 {
            debug!(
                "storage.GetNodeVspcList empty. daaScore: {}",
                daa_score_start
            );
            sleep(Duration::from_millis(1550)).await;
            return Ok(());
        }

        let vspc_list_next = vspc_list_next[..len_vspc_next].to_vec();

        info!(
            "storage.GetNodeVspcList daaScore: {}, lenBlock/mSecond: {}/{} lenVspcListMax: {} synced: {}",
            daa_score_start, len_vspc_next, start_time, LEN_VSPC_LIST_MAX, self.synced
        );

        // Check rollback, corresponding to Go version's checkRollbackNext
        let (rollback_daa_score, vspc_list_filtered) =
            self.check_rollback_next(&self.vspc_list, &vspc_list_next, daa_score_start);

        if rollback_daa_score > 0 {
            // Need rollback, corresponding to Go version's rollback logic
            let mut daa_score_last = 0u64;
            let mut mts_rollback = 0i64;

            // Rollback to last state data batch
            let len_rollback = self.rollback_list.len().saturating_sub(1);
            if len_rollback > 0
                && self.rollback_list[len_rollback].daa_score_end >= rollback_daa_score
            {
                daa_score_last = self.rollback_list[len_rollback].daa_score_start;
                let rollback_data = &self.rollback_list[len_rollback];
                mts_rollback = self
                    .rollback_manager
                    .rollback_op_state_batch(rollback_data)
                    .await?;

                // Remove rolled back VSPC data
                while !self.vspc_list.is_empty() {
                    let len_vspc_runtime = self.vspc_list.len();
                    if len_vspc_runtime == 0 {
                        break;
                    }

                    let last_vspc = &self.vspc_list[len_vspc_runtime - 1];
                    if last_vspc.daa_score >= daa_score_last {
                        if len_vspc_runtime == 1 {
                            self.vspc_list.clear();
                            break;
                        }
                        self.vspc_list.truncate(len_vspc_runtime - 1);
                        continue;
                    }
                    break;
                }

                // Remove last rollback data
                self.rollback_list.truncate(len_rollback);
                self.storage
                    .runtime
                    .set_runtime_rollback_last(&self.rollback_list)
                    .await?;
            } else {
                self.vspc_list = vspc_list_filtered.clone();
            }

            self.storage
                .runtime
                .set_runtime_vspc_last(&self.vspc_list)
                .await?;
            info!(
                "explorer.checkRollbackNext start/rollback/last: {}/{}/{} mSecond: {}",
                daa_score_start, rollback_daa_score, daa_score_last, mts_rollback
            );
            return Ok(());
        }

        // Get transaction data list - call StateManager method
        let tx_data_list = self
            .storage
            .state
            .get_node_transaction_data_list(&vspc_list_filtered)
            .await?;
        let len_tx_data = tx_data_list.len();

        if len_tx_data == 0 {
            debug!("storage.GetNodeTransactionDataList empty");
            sleep(Duration::from_millis(1550)).await;
            return Ok(());
        }

        info!(
            "storage.GetNodeTransactionDataList lenTransaction: {}",
            len_tx_data
        );

        // Parse operation data list - call OperationManager method
        let mut op_data_list = self.parse_op_data_list(tx_data_list).await?;
        let len_op_data = op_data_list.len();

        info!("explorer.ParseOpDataList lenOperation: {}", len_op_data);

        // Prepare state batch - call OperationManager method
        let (state_map, _) = self.prepare_state_batch(&op_data_list).await?;

        debug!(
            "operation.PrepareStateBatch lenToken: {}, lenBalance: {}",
            state_map.state_token_map.len(),
            state_map.state_balance_map.len()
        );

        // Execute batch - call OperationManager method
        let checkpoint_last = if !self.rollback_list.is_empty() {
            self.rollback_list.last().unwrap().checkpoint_after.clone()
        } else {
            String::new()
        };

        let (rollback, _) = self
            .execute_batch(&mut op_data_list, state_map, &checkpoint_last)
            .await?;

        // Update runtime state
        self.update_runtime_state(&vspc_list_filtered, &rollback)
            .await?;

        // Update progress, call update_progress method
        if let Some(vspc_data) = self.fetch_vspc_data(daa_score_start, 1).await?.first() {
            self.update_progress(vspc_data).await?;
        }

        // Record scan statistics
        let current_time = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap()
            .as_millis() as u64;
        let scan_duration = current_time - start_time;
        info!(
            "explorer.scan lenRuntimeVspc: {}, lenRuntimeRollback: {}, lenOperation: {}, mSecondLoop: {}",
            self.vspc_list.len(),
            self.rollback_list.len(),
            len_op_data,
            scan_duration
        );

        // If synced, add extra delay
        if self.synced {
            let delay = 850 - scan_duration as i64;
            if delay > 0 {
                sleep(Duration::from_millis(delay as u64)).await;
            }
        }

        Ok(())
    }

    /// Check rollback, corresponding to Go version's checkRollbackNext
    fn check_rollback_next(
        &self,
        vspc_list_prev: &[DataVspcType],
        vspc_list_next: &[DataVspcType],
        daa_score_start: u64,
    ) -> (u64, Vec<DataVspcType>) {
        if vspc_list_prev.is_empty() {
            return (0, vspc_list_next.to_vec());
        }

        let mut vspc_list1 = Vec::new();
        let mut vspc_list2 = Vec::new();

        for vspc in vspc_list_prev {
            if vspc.daa_score < daa_score_start {
                continue;
            }
            vspc_list1.push(vspc.clone());
        }

        let len_check = vspc_list1.len();
        if len_check > 0 {
            if vspc_list_next.len() <= len_check {
                return (0, Vec::new());
            } else {
                vspc_list2 = vspc_list_next[..len_check].to_vec();
            }
        } else {
            vspc_list2 = vspc_list_next.to_vec();
        }

        // Check rollback
        let mut rollback_daa_score = 0;
        for (i, vspc1) in vspc_list1.iter().enumerate() {
            if i >= vspc_list2.len() {
                break;
            }
            let vspc2 = &vspc_list2[i];
            if vspc1.hash != vspc2.hash {
                rollback_daa_score = vspc1.daa_score;
                break;
            }
        }

        (rollback_daa_score, vspc_list2)
    }

    /// Update runtime state
    async fn update_runtime_state(
        &mut self,
        vspc_list: &[DataVspcType],
        rollback: &DataRollbackType,
    ) -> Result<()> {
        // Update sync state
        self.synced = vspc_list.len() < 99;

        // Update VSPC list
        self.vspc_list.extend(vspc_list.to_vec());

        // Limit VSPC list size
        if self.vspc_list.len() > LEN_VSPC_LIST_RUNTIME_MAX {
            let len_start = self.vspc_list.len() - LEN_VSPC_LIST_RUNTIME_MAX;
            self.vspc_list = self.vspc_list[len_start..].to_vec();
        }

        // Update rollback list
        self.rollback_list.push(rollback.clone());

        // Limit rollback list size
        let len_rollback = self.rollback_list.len();
        let mut len_start = 0;
        for i in (0..len_rollback).rev() {
            if self.rollback_list[len_rollback - 1].daa_score_end
                - self.rollback_list[i].daa_score_start
                >= LEN_ROLLBACK_LIST_RUNTIME_MAX as u64
            {
                len_start = i;
                break;
            }
        }

        if len_start > 0 {
            self.rollback_list = self.rollback_list[len_start..].to_vec();
        }

        // Update operation score
        if rollback.op_score_last > 0 {
            self.op_score_last = rollback.op_score_last;
        }

        Ok(())
    }

    /// Get VSPC data, corresponding to Go version functionality
    async fn fetch_vspc_data(&self, from_daa_score: u64, limit: usize) -> Result<Vec<VSPCData>> {
        debug!(
            "Fetching VSPC data from DAA score: {} with limit: {}",
            from_daa_score, limit
        );

        // Implement fetching VSPC data from Kaspa node
        let kaspa_url = &self.config.kaspa_node_url;

        // Build RPC request
        let request_body = serde_json::json!({
            "jsonrpc": "2.0",
            "id": 1,
            "method": "getVirtualSelectedParentChainFromBlock",
            "params": {
                "startHash": "0000000000000000000000000000000000000000000000000000000000000000",
                "includeAcceptedTransactionIds": true
            }
        });

        let mut vspc_data_list = Vec::new();

        // Send HTTP request to Kaspa node
        let response = self
            .http_client
            .post(kaspa_url)
            .header("Content-Type", "application/json")
            .json(&request_body)
            .send()
            .await?;

        if response.status().is_success() {
            let response_json: Value = response.json().await?;

            if let Some(result) = response_json.get("result") {
                if let Some(blocks) = result.get("blocks") {
                    if let Some(blocks_array) = blocks.as_array() {
                        for block in blocks_array {
                            if let Some(daa_score) = block.get("daaScore").and_then(|v| v.as_u64())
                            {
                                if daa_score > from_daa_score && vspc_data_list.len() < limit {
                                    // Parse VSPC data
                                    let vspc_data = self.parse_vspc_block(block).await?;
                                    vspc_data_list.push(vspc_data);
                                }
                            }
                        }
                    }
                }
            }
        }

        info!("Fetched {} VSPC data records", vspc_data_list.len());
        Ok(vspc_data_list)
    }

    /// Parse VSPC block, corresponding to Go version functionality
    async fn parse_vspc_block(&self, block: &Value) -> Result<VSPCData> {
        let block_hash = block
            .get("hash")
            .and_then(|v| v.as_str())
            .ok_or_else(|| anyhow::anyhow!("Missing block hash"))?;

        let daa_score = block
            .get("daaScore")
            .and_then(|v| v.as_u64())
            .ok_or_else(|| anyhow::anyhow!("Missing DAA score"))?;

        let timestamp = block
            .get("timestamp")
            .and_then(|v| v.as_u64())
            .ok_or_else(|| anyhow::anyhow!("Missing timestamp"))?;

        let blue_score = block.get("blueScore").and_then(|v| v.as_u64()).unwrap_or(0);

        let parent_hashes = block
            .get("parents")
            .and_then(|v| v.as_array())
            .map(|arr| {
                arr.iter()
                    .filter_map(|v| v.as_str().map(|s| s.to_string()))
                    .collect()
            })
            .unwrap_or_default();

        let _accepted_tx_ids: Vec<String> = block
            .get("acceptedTransactionIds")
            .and_then(|v| v.as_array())
            .map(|arr| {
                arr.iter()
                    .filter_map(|v| v.as_str())
                    .map(|s| s.to_string())
                    .collect()
            })
            .unwrap_or_default();

        Ok(VSPCData {
            block_hash: block_hash.to_string(),
            parent_hashes,
            daa_score,
            timestamp,
            blue_score,
            blue_work: "0".to_string(),
            pruning_point: "0".to_string(),
            difficulty: 0.0,
            is_header_only: false,
            block_level: 0,
            block_status: 0,
            merge_set_blues: Vec::new(),
            merge_set_reds: Vec::new(),
            selected_parent: "0".to_string(),
            selected_tip: "0".to_string(),
            block_ghostdag_data: GhostDagData::default(),
            block_relations: BlockRelations::default(),
            block_acceptance_data: BlockAcceptanceData::default(),
        })
    }

    /// Process VSPC data, corresponding to Go version's process_vspc_data function
    async fn process_vspc_data(&mut self, vspc_data: &VSPCData) -> Result<()> {
        // Convert VSPCData to DataTransactionType
        let tx_data = DataTransactionType {
            tx_id: vspc_data.block_hash.clone(), // Use block_hash as tx_id
            daa_score: vspc_data.daa_score,
            block_accept: vspc_data.block_hash.clone(),
            data: None, // Temporarily set to None
        };

        // Get transaction data list
        let tx_data_list = self.get_node_transaction_data_list(vec![tx_data]).await?;
        let len_tx_data = tx_data_list.len();

        if len_tx_data == 0 {
            debug!("storage.GetNodeTransactionDataList empty");
            return Ok(());
        }

        info!(
            "storage.GetNodeTransactionDataList lenTransaction: {}",
            len_tx_data
        );

        // Parse operation data list - corresponding to Go version's ParseOpDataList
        let op_data_list = self.parse_op_data_list(tx_data_list).await?;
        let len_op_data = op_data_list.len();

        info!("explorer.ParseOpDataList lenOperation: {}", len_op_data);

        if len_op_data == 0 {
            return Ok(());
        }

        // Prepare state batch - corresponding to Go version's PrepareStateBatch
        let state_map = DataStateMapType::new();
        // TODO: Implement batch state preparation

        debug!(
            "operation.PrepareStateBatch lenToken: {}, lenBalance: {}",
            state_map.state_token_map.len(),
            state_map.state_balance_map.len()
        );

        // Execute batch - corresponding to Go version's ExecuteBatch
        let _checkpoint_last = if !self.rollback_list.is_empty() {
            self.rollback_list.last().unwrap().checkpoint_after.clone()
        } else {
            String::new()
        };

        // TODO: Implement batch execution
        let rollback = DataRollbackType::new(
            DataStateMapType::new(),
            DataStateMapType::new(),
            Vec::new(),
            Vec::new(),
            0,
            0,
            "".to_string(),
            "".to_string(),
            0,
        );

        // Save operation state batch
        self.save_op_state_batch(&op_data_list).await?;

        // Update rollback list
        self.rollback_list.push(rollback);

        info!(
            "Successfully processed {} operations from VSPC",
            len_op_data
        );
        Ok(())
    }

    /// Extract operations, corresponding to Go version's functionality
    async fn extract_operations(&self, vspc_data: &VSPCData) -> Result<Vec<OperationData>> {
        debug!("Extracting operations from VSPC data");

        // Implement operation extraction logic
        let mut operations = Vec::new();

        // Extract transaction ID list from VSPC data
        let tx_ids = self.get_transaction_ids_from_vspc(vspc_data).await?;

        for tx_id in tx_ids {
            // Get transaction details
            if let Some(tx_data) = self.fetch_transaction_data(&tx_id).await? {
                // Parse operations in transaction
                let tx_operations = self
                    .parse_transaction_operations(&tx_data, vspc_data)
                    .await?;
                operations.extend(tx_operations);
            }
        }

        info!("Extracted {} operations from VSPC data", operations.len());
        Ok(operations)
    }

    /// Get transaction ID list from VSPC data, corresponding to Go version's functionality
    async fn get_transaction_ids_from_vspc(&self, _vspc_data: &VSPCData) -> Result<Vec<String>> {
        // TODO: Implement extracting transaction IDs from VSPC data
        Ok(Vec::new())
    }

    /// Get transaction data, corresponding to Go version's functionality
    async fn fetch_transaction_data(&self, tx_id: &str) -> Result<Option<Value>> {
        debug!("Fetching transaction data for tx_id: {}", tx_id);

        // Get transaction details from Kaspa node
        let kaspa_url = &self.config.kaspa_node_url;

        let request_body = serde_json::json!({
            "jsonrpc": "2.0",
            "id": 1,
            "method": "getTransaction",
            "params": {
                "transactionId": tx_id,
                "includeTransactionVerboseData": true
            }
        });

        let response = self
            .http_client
            .post(kaspa_url)
            .header("Content-Type", "application/json")
            .json(&request_body)
            .send()
            .await?;

        if response.status().is_success() {
            let response_json: Value = response.json().await?;
            if let Some(result) = response_json.get("result") {
                return Ok(Some(result.clone()));
            }
        }

        Ok(None)
    }

    /// Parse transaction operations, corresponding to Go version's functionality
    async fn parse_transaction_operations(
        &self,
        tx_data: &Value,
        vspc_data: &VSPCData,
    ) -> Result<Vec<OperationData>> {
        debug!("Parsing transaction operations from tx_data");

        let mut operations = Vec::new();

        // Parse operation scripts in transaction outputs
        if let Some(outputs) = tx_data.get("outputs").and_then(|v| v.as_array()) {
            for (index, output) in outputs.iter().enumerate() {
                if let Some(script_public_key) = output.get("scriptPublicKey") {
                    if let Some(script) = script_public_key.as_str() {
                        // Parse operations in script
                        if let Some(operation) = self
                            .parse_operation_script(script, tx_data, index, vspc_data)
                            .await?
                        {
                            operations.push(operation);
                        }
                    }
                }
            }
        }

        info!("Parsed {} operations from transaction", operations.len());
        Ok(operations)
    }

    async fn parse_operation_script(
        &self,
        script: &str,
        tx_data: &Value,
        output_index: usize,
        vspc_data: &VSPCData,
    ) -> Result<Option<OperationData>> {
        // Parse KRC-20 operation script
        // Check if it is a KRC-20 operation
        if !script.contains("KRC-20") {
            return Ok(None);
        }

        // Parse operation type and parameters
        let operation_type = self.extract_operation_type(script)?;
        let script_data = self.extract_script_data(script, tx_data, output_index)?;

        let operation = OperationData {
            ca: None,
            operation_type: operation_type.clone(),
            tick: script_data.tick.clone().unwrap_or_default(),
            from_address: script_data.from.clone(),
            to_address: script_data.to.clone(),
            amount: script_data
                .amount
                .as_ref()
                .and_then(|s| s.parse::<u64>().ok()),
            tx_hash: tx_data
                .get("transactionId")
                .and_then(|v| v.as_str())
                .unwrap_or("")
                .to_string(),
            block_hash: vspc_data.block_hash.clone(),
            timestamp: vspc_data.timestamp,
            block_daa_score: vspc_data.daa_score,
            script: Some(script_data.clone()),
            is_testnet: self.config.is_testnet,
            daa_score: vspc_data.daa_score,
            tx_id: script_data.tx_hash.clone().unwrap_or_default(),
        };

        Ok(Some(operation))
    }

    fn extract_operation_type(&self, script: &str) -> Result<String> {
        // Extract operation type from script
        if script.contains("send") {
            Ok("send".to_string())
        } else if script.contains("transfer") {
            Ok("transfer".to_string())
        } else if script.contains("issue") {
            Ok("issue".to_string())
        } else if script.contains("burn") {
            Ok("burn".to_string())
        } else if script.contains("list") {
            Ok("list".to_string())
        } else if script.contains("chown") {
            Ok("chown".to_string())
        } else if script.contains("blacklist") {
            Ok("blacklist".to_string())
        } else {
            Ok("unknown".to_string())
        }
    }

    fn extract_script_data(
        &self,
        script: &str,
        tx_data: &Value,
        output_index: usize,
    ) -> Result<crate::storage::types::ScriptData> {
        // Extract data from script
        Ok(crate::storage::types::ScriptData {
            p: "KRC-20".to_string(),
            operation: self.extract_operation_type(script)?,
            from: self.extract_from_address(tx_data, output_index)?,
            to: self.extract_to_address(tx_data, output_index)?,
            tick: self.extract_tick(script)?,
            amount: self.extract_amount(script)?,
            utxo: self.extract_utxo(tx_data, output_index)?,
            ca: self.extract_ca(script)?,
            mode: self.extract_mode(script)?,
            tx_hash: None, // Get from transaction data
        })
    }

    fn extract_from_address(
        &self,
        tx_data: &Value,
        _output_index: usize,
    ) -> Result<Option<String>> {
        // Extract sender address from transaction data
        if let Some(inputs) = tx_data.get("inputs").and_then(|v| v.as_array()) {
            if let Some(input) = inputs.get(0) {
                if let Some(address) = input.get("address").and_then(|v| v.as_str()) {
                    return Ok(Some(address.to_string()));
                }
            }
        }
        Ok(None)
    }

    fn extract_to_address(&self, tx_data: &Value, output_index: usize) -> Result<Option<String>> {
        // Extract receiver address from transaction data
        if let Some(outputs) = tx_data.get("outputs").and_then(|v| v.as_array()) {
            if let Some(output) = outputs.get(output_index) {
                if let Some(address) = output.get("address").and_then(|v| v.as_str()) {
                    return Ok(Some(address.to_string()));
                }
            }
        }
        Ok(None)
    }

    fn extract_tick(&self, _script: &str) -> Result<Option<String>> {
        // Extract tick from script
        // TODO: Implement specific parsing logic here
        Ok(None)
    }

    fn extract_amount(&self, _script: &str) -> Result<Option<String>> {
        // Extract amount from script
        // TODO: Implement specific parsing logic here
        Ok(None)
    }

    fn extract_utxo(&self, tx_data: &Value, output_index: usize) -> Result<Option<String>> {
        // Extract UTXO from transaction data
        if let Some(outputs) = tx_data.get("outputs").and_then(|v| v.as_array()) {
            if let Some(output) = outputs.get(output_index) {
                if let Some(amount) = output.get("amount").and_then(|v| v.as_u64()) {
                    let tx_id = tx_data
                        .get("transactionId")
                        .and_then(|v| v.as_str())
                        .unwrap_or("");
                    return Ok(Some(format!("{}_{}", tx_id, amount)));
                }
            }
        }
        Ok(None)
    }

    fn extract_ca(&self, _script: &str) -> Result<Option<String>> {
        // Extract CA from script
        // TODO: Implement specific parsing logic here
        Ok(None)
    }

    fn extract_mode(&self, _script: &str) -> Result<Option<String>> {
        // Extract mode from script
        // TODO: Implement specific parsing logic here
        Ok(None)
    }

    /// Process operation, corresponding to Go version's process_operation function
    async fn process_operation(&self, operation: OperationData) -> Result<()> {
        debug!(
            "Processing operation: {} for tick: {}",
            operation.operation_type, operation.tick
        );

        // TODO: Use operation manager to process operations
        // let operation_manager = self.storage.get_operation_manager()?;
        // Should call specific operation methods here, not generic validate_operation and execute_operation

        // Save operation to distributed storage
        if let Some(_distributed) = &self.storage.distributed {
            // TODO: Implement operation insertion in distributed storage
            debug!("Operation would be saved to distributed storage");
        }

        info!("Successfully processed operation: {}", operation.tx_id);
        Ok(())
    }

    /// Update progress, corresponding to Go version's functionality
    async fn update_progress(&self, vspc_data: &VSPCData) -> Result<()> {
        // Update runtime state with progress
        self.storage
            .runtime
            .update_progress(&vspc_data.block_hash, vspc_data.daa_score)?;

        // Save checkpoint
        self.storage
            .runtime
            .save_checkpoint(&vspc_data.block_hash, vspc_data.daa_score)?;

        Ok(())
    }

    /// Get statistics, corresponding to Go version's functionality
    pub fn get_stats(&self) -> Result<ScanStats> {
        let current_time = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap()
            .as_secs();

        let scan_duration = if self.is_scanning && self.scan_start_time > 0 {
            current_time - self.scan_start_time
        } else {
            0
        };

        let vspc_per_second = if scan_duration > 0 {
            self.total_vspc_processed as f64 / scan_duration as f64
        } else {
            0.0
        };

        Ok(ScanStats {
            total_vspc_processed: self.total_vspc_processed,
            total_operations_found: self.total_operations_found,
            last_scan_time: self.last_scan_time,
            scan_start_time: self.scan_start_time,
            is_scanning: self.is_scanning,
            scan_duration,
            vspc_per_second,
        })
    }

    /// Validate configuration, corresponding to Go version's functionality
    pub fn validate_config(&self) -> Result<()> {
        // Validate DAA score range
        if self.config.daa_score_range.is_empty() {
            return Err(anyhow::anyhow!("DAA score range is empty"));
        }

        // Validate Kaspa node URL
        if self.config.kaspa_node_url.is_empty() {
            return Err(anyhow::anyhow!("Kaspa node URL is empty"));
        }

        Ok(())
    }

    /// Check if DAA score is valid, corresponding to Go version's functionality
    pub fn is_daa_score_valid(&self, daa_score: u64) -> bool {
        // Check if DAA score is within valid range
        for range in &self.config.daa_score_range {
            if daa_score >= range[0] && daa_score <= range[1] {
                return true;
            }
        }
        false
    }

    /// Check if it is a reserved token, corresponding to Go version's functionality
    fn is_tick_reserved(&self, tick: &str) -> bool {
        crate::operations::is_tick_reserved(tick)
    }

    /// Check DAA score range, corresponding to Go version's checkDaaScoreRange
    fn check_daa_score_range(&self, daa_score: u64) -> (bool, u64) {
        for range in &self.config.daa_score_range {
            if daa_score < range[0] {
                return (false, range[0]);
            } else if daa_score <= range[1] {
                return (true, daa_score);
            }
        }
        (false, daa_score)
    }

    /// Get node VSPC list, corresponding to Go version's GetNodeVspcList
    async fn get_node_vspc_list(
        &self,
        _daa_score_start: u64,
        _limit: usize,
    ) -> Result<Vec<DataVspcType>> {
        // TODO: Implement getting VSPC list from Kaspa node
        // Should call Kaspa RPC API here
        Ok(Vec::new())
    }

    /// Get node transaction data list, corresponding to Go version's GetNodeTransactionDataList
    async fn get_node_transaction_data_list(
        &self,
        tx_data_list: Vec<DataTransactionType>,
    ) -> Result<Vec<DataTransactionType>> {
        // TODO: Implement getting transaction data from Kaspa node
        Ok(tx_data_list)
    }

    /// Parse operation data list, corresponding to Go version's ParseOpDataList
    async fn parse_op_data_list(
        &self,
        tx_data_list: Vec<DataTransactionType>,
    ) -> Result<Vec<DataOperationType>> {
        // TODO: Implement transaction parsing logic
        let mut op_data_list = Vec::new();

        for _tx_data in tx_data_list {
            // Parse operations in transaction
            // TODO: Need to provide VSPC data parameter
            let operations = self
                .parse_transaction_operations(&serde_json::Value::Null, &VSPCData::default())
                .await?;
            // Convert OperationData to DataOperationType
            for op in operations {
                let data_op = DataOperationType {
                    tx_id: op.tx_id,
                    daa_score: op.daa_score,
                    block_accept: op.block_hash,
                    fee: 0,
                    fee_least: 0,
                    mts_add: op.timestamp as i64,
                    op_score: op.block_daa_score,
                    op_accept: 0,
                    op_error: String::new(),
                    op_script: Vec::new(),
                    script_sig: String::new(),
                    st_before: Vec::new(),
                    st_after: Vec::new(),
                    checkpoint: String::new(),
                    ss_info: None,
                };
                op_data_list.push(data_op);
            }
        }

        Ok(op_data_list)
    }

    /// Prepare state batch, corresponding to Go version's PrepareStateBatch
    async fn prepare_state_batch(
        &self,
        _op_data_list: &[DataOperationType],
    ) -> Result<(DataStateMapType, i64)> {
        // TODO: Implement state preparation logic
        let state_map = DataStateMapType {
            state_token_map: HashMap::new(),
            state_balance_map: HashMap::new(),
            state_market_map: HashMap::new(),
            state_blacklist_map: HashMap::new(),
        };
        Ok((state_map, 0))
    }

    /// Execute batch, corresponding to Go version's ExecuteBatch
    async fn execute_batch(
        &self,
        _op_data_list: &mut [DataOperationType],
        _state_map: DataStateMapType,
        checkpoint_last: &str,
    ) -> Result<(DataRollbackType, i64)> {
        // TODO: Implement batch execution logic
        let rollback = DataRollbackType {
            checkpoint_before: checkpoint_last.to_string(),
            checkpoint_after: checkpoint_last.to_string(),
            daa_score_start: 0,
            daa_score_end: 0,
            state_map_before: _state_map,
            state_map_after: DataStateMapType {
                state_token_map: HashMap::new(),
                state_balance_map: HashMap::new(),
                state_market_map: HashMap::new(),
                state_blacklist_map: HashMap::new(),
            },
            op_score_list: Vec::new(),
            tx_id_list: Vec::new(),
            op_score_last: 0,
        };
        Ok((rollback, 0))
    }

    /// Save operation state batch, corresponding to Go version's SaveOpStateBatch
    async fn save_op_state_batch(&self, _op_data_list: &[DataOperationType]) -> Result<Vec<i64>> {
        // TODO: Implement state saving logic
        Ok(Vec::new())
    }

    /// Rollback operation state batch, corresponding to Go version's RollbackOpStateBatch
    async fn rollback_op_state_batch(&self, _rollback: &DataRollbackType) -> Result<i64> {
        // TODO: Implement rollback logic
        Ok(0)
    }
}
