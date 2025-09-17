use crate::storage::types::*;
use anyhow::Result;
use reqwest::Client;
use serde_json::{Value, json};
use std::collections::HashMap;
use tracing::{error, info};

/// VSPC client for fetching data from Kaspa node
pub struct VSPCClient {
    client: Client,
    node_url: String,
    timeout: std::time::Duration,
}

impl VSPCClient {
    pub fn new(node_url: String) -> Self {
        Self {
            client: Client::new(),
            node_url,
            timeout: std::time::Duration::from_secs(30),
        }
    }

    /// Get VSPC data list, corresponding to Go version's GetNodeVspcList
    pub async fn get_vspc_list(
        &self,
        daa_score_start: u64,
        limit: usize,
    ) -> Result<Vec<DataVspcType>> {
        info!(
            "Fetching VSPC data from DAA score: {} with limit: {}",
            daa_score_start, limit
        );

        let mut vspc_list = Vec::new();

        // Build RPC request to get block information
        let request_body = json!({
            "jsonrpc": "2.0",
            "id": 1,
            "method": "getVirtualSelectedParentChainFromBlock",
            "params": {
                "startHash": "0000000000000000000000000000000000000000000000000000000000000000",
                "includeAcceptedTransactionIds": true
            }
        });

        let response = self
            .client
            .post(&self.node_url)
            .header("Content-Type", "application/json")
            .timeout(self.timeout)
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
                                if daa_score >= daa_score_start && vspc_list.len() < limit {
                                    // Parse VSPC data
                                    if let Ok(vspc_data) = self.parse_vspc_block(block).await {
                                        vspc_list.push(vspc_data);
                                    }
                                }
                            }
                        }
                    }
                }
            }
        } else {
            error!("Failed to fetch VSPC data: {}", response.status());
        }

        info!("Fetched {} VSPC data records", vspc_list.len());
        Ok(vspc_list)
    }

    /// Get transaction data map, corresponding to Go version's GetNodeTransactionDataMap
    pub async fn get_transaction_data_map(
        &self,
        tx_data_list: &[DataTransactionType],
    ) -> Result<HashMap<String, Value>> {
        info!(
            "Fetching transaction data for {} transactions",
            tx_data_list.len()
        );

        let mut tx_data_map = HashMap::new();

        // Batch fetch transaction data
        for tx_data in tx_data_list {
            let request_body = json!({
                "jsonrpc": "2.0",
                "id": 1,
                "method": "getTransaction",
                "params": {
                    "transactionId": tx_data.tx_id,
                    "includeTransactionVerboseData": true
                }
            });

            let response = self
                .client
                .post(&self.node_url)
                .header("Content-Type", "application/json")
                .timeout(self.timeout)
                .json(&request_body)
                .send()
                .await?;

            if response.status().is_success() {
                let response_json: Value = response.json().await?;

                if let Some(result) = response_json.get("result") {
                    tx_data_map.insert(tx_data.tx_id.clone(), result.clone());
                }
            }
        }

        info!("Fetched {} transaction data records", tx_data_map.len());
        Ok(tx_data_map)
    }

    /// Get transaction data list, corresponding to Go version's GetNodeTransactionDataList
    pub async fn get_transaction_data_list(
        &self,
        tx_data_list: &[DataTransactionType],
    ) -> Result<Vec<DataTransactionType>> {
        let tx_data_map = self.get_transaction_data_map(tx_data_list).await?;

        let mut updated_tx_list = Vec::new();

        for tx_data in tx_data_list {
            let mut updated_tx = tx_data.clone();
            if let Some(tx_info) = tx_data_map.get(&tx_data.tx_id) {
                updated_tx.data = Some(tx_info.clone());
            }
            updated_tx_list.push(updated_tx);
        }

        Ok(updated_tx_list)
    }

    /// Parse VSPC block, corresponding to Go version functionality
    async fn parse_vspc_block(&self, block: &Value) -> Result<DataVspcType> {
        let block_hash = block
            .get("hash")
            .and_then(|v| v.as_str())
            .ok_or_else(|| anyhow::anyhow!("Missing block hash"))?;

        let daa_score = block
            .get("daaScore")
            .and_then(|v| v.as_u64())
            .ok_or_else(|| anyhow::anyhow!("Missing DAA score"))?;

        let accepted_tx_ids: Vec<String> = block
            .get("acceptedTransactionIds")
            .and_then(|v| v.as_array())
            .map(|arr| {
                arr.iter()
                    .filter_map(|v| v.as_str())
                    .map(|s| s.to_string())
                    .collect()
            })
            .unwrap_or_default();

        Ok(DataVspcType {
            daa_score,
            hash: block_hash.to_string(),
            tx_id_list: accepted_tx_ids,
        })
    }

    /// Get block information
    pub async fn get_block_info(&self, block_hash: &str) -> Result<Value> {
        let request_body = json!({
            "jsonrpc": "2.0",
            "id": 1,
            "method": "getBlock",
            "params": {
                "blockHash": block_hash,
                "includeTransactionVerboseData": true
            }
        });

        let response = self
            .client
            .post(&self.node_url)
            .header("Content-Type", "application/json")
            .timeout(self.timeout)
            .json(&request_body)
            .send()
            .await?;

        if response.status().is_success() {
            let response_json: Value = response.json().await?;
            if let Some(result) = response_json.get("result") {
                Ok(result.clone())
            } else {
                Err(anyhow::anyhow!("No result in response"))
            }
        } else {
            Err(anyhow::anyhow!(
                "Failed to get block info: {}",
                response.status()
            ))
        }
    }

    /// Get transaction information
    pub async fn get_transaction_info(&self, tx_id: &str) -> Result<Value> {
        let request_body = json!({
            "jsonrpc": "2.0",
            "id": 1,
            "method": "getTransaction",
            "params": {
                "transactionId": tx_id,
                "includeTransactionVerboseData": true
            }
        });

        let response = self
            .client
            .post(&self.node_url)
            .header("Content-Type", "application/json")
            .timeout(self.timeout)
            .json(&request_body)
            .send()
            .await?;

        if response.status().is_success() {
            let response_json: Value = response.json().await?;
            if let Some(result) = response_json.get("result") {
                Ok(result.clone())
            } else {
                Err(anyhow::anyhow!("No result in response"))
            }
        } else {
            Err(anyhow::anyhow!(
                "Failed to get transaction info: {}",
                response.status()
            ))
        }
    }

    /// Health check
    pub async fn health_check(&self) -> Result<bool> {
        let request_body = json!({
            "jsonrpc": "2.0",
            "id": 1,
            "method": "getInfo"
        });

        let response = self
            .client
            .post(&self.node_url)
            .header("Content-Type", "application/json")
            .timeout(self.timeout)
            .json(&request_body)
            .send()
            .await?;

        Ok(response.status().is_success())
    }
}
