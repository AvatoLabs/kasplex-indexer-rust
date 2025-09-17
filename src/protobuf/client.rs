use crate::protobuf::ProtobufHandler;
use crate::protobuf::protowire::{RpcNotifyCommand, *};
use anyhow::Result;

#[derive(Debug, Clone)]
pub struct KaspaRpcClient {
    pub handler: ProtobufHandler,
}

/// Advanced Kaspa RPC client
impl KaspaRpcClient {
    pub fn new() -> Self {
        Self {
            handler: ProtobufHandler::new(),
        }
    }

    pub async fn connect(&mut self, endpoint: String) -> Result<()> {
        self.handler.connect(endpoint).await
    }

    pub fn is_connected(&self) -> bool {
        self.handler.is_connected()
    }

    pub async fn disconnect(&mut self) -> Result<()> {
        self.handler.disconnect().await
    }

    // Block-related methods
    pub async fn get_block(&mut self, hash: String) -> Result<Option<RpcBlock>> {
        let request = KaspadRequest {
            id: 1,
            payload: Some(kaspad_request::Payload::GetBlockRequest(
                GetBlockRequestMessage {
                    hash,
                    include_transactions: true,
                },
            )),
        };

        let response = self.handler.send_request(request).await?;
        match response.payload {
            Some(kaspad_response::Payload::GetBlockResponse(resp)) => Ok(resp.block),
            _ => Err(anyhow::anyhow!("Unexpected response type")),
        }
    }

    pub async fn get_blocks(
        &mut self,
        low_hash: String,
        _high_hash: String,
    ) -> Result<Vec<RpcBlock>> {
        let request = KaspadRequest {
            id: 2,
            payload: Some(kaspad_request::Payload::GetBlocksRequest(
                GetBlocksRequestMessage {
                    include_transactions: true,
                    include_blocks: true,
                    low_hash,
                },
            )),
        };

        let response = self.handler.send_request(request).await?;
        match response.payload {
            Some(kaspad_response::Payload::GetBlocksResponse(resp)) => Ok(resp.blocks),
            _ => Err(anyhow::anyhow!("Unexpected response type")),
        }
    }

    pub async fn get_block_count(&mut self) -> Result<u64> {
        let request = KaspadRequest {
            id: 3,
            payload: Some(kaspad_request::Payload::GetBlockCountRequest(
                GetBlockCountRequestMessage {},
            )),
        };

        let response = self.handler.send_request(request).await?;
        match response.payload {
            Some(kaspad_response::Payload::GetBlockCountResponse(resp)) => Ok(resp.block_count),
            _ => Err(anyhow::anyhow!("Unexpected response type")),
        }
    }

    pub async fn get_block_template(&mut self) -> Result<Option<RpcBlock>> {
        let request = KaspadRequest {
            id: 4,
            payload: Some(kaspad_request::Payload::GetBlockTemplateRequest(
                GetBlockTemplateRequestMessage {
                    pay_address: "".to_string(),
                    extra_data: "".to_string(),
                },
            )),
        };

        let response = self.handler.send_request(request).await?;
        match response.payload {
            Some(kaspad_response::Payload::GetBlockTemplateResponse(resp)) => Ok(resp.block),
            _ => Err(anyhow::anyhow!("Unexpected response type")),
        }
    }

    // Transaction-related methods
    pub async fn submit_transaction(&mut self, transaction: RpcTransaction) -> Result<String> {
        let request = KaspadRequest {
            id: 5,
            payload: Some(kaspad_request::Payload::SubmitTransactionRequest(
                SubmitTransactionRequestMessage {
                    transaction: Some(transaction),
                    allow_orphan: false,
                },
            )),
        };

        let response = self.handler.send_request(request).await?;
        match response.payload {
            Some(kaspad_response::Payload::SubmitTransactionResponse(resp)) => {
                Ok(resp.transaction_id)
            }
            _ => Err(anyhow::anyhow!("Unexpected response type")),
        }
    }

    pub async fn get_mempool_entries(&mut self) -> Result<Vec<RpcMempoolEntry>> {
        let request = KaspadRequest {
            id: 6,
            payload: Some(kaspad_request::Payload::GetMempoolEntriesRequest(
                GetMempoolEntriesRequestMessage {
                    filter_transaction_pool: false,
                    include_orphan_pool: false,
                },
            )),
        };

        let response = self.handler.send_request(request).await?;
        match response.payload {
            Some(kaspad_response::Payload::GetMempoolEntriesResponse(resp)) => Ok(resp.entries),
            _ => Err(anyhow::anyhow!("Unexpected response type")),
        }
    }

    pub async fn get_mempool_entry(&mut self, tx_id: String) -> Result<Option<RpcMempoolEntry>> {
        let request = KaspadRequest {
            id: 7,
            payload: Some(kaspad_request::Payload::GetMempoolEntryRequest(
                GetMempoolEntryRequestMessage {
                    tx_id,
                    filter_transaction_pool: false,
                    include_orphan_pool: false,
                },
            )),
        };

        let response = self.handler.send_request(request).await?;
        match response.payload {
            Some(kaspad_response::Payload::GetMempoolEntryResponse(resp)) => Ok(resp.entry),
            _ => Err(anyhow::anyhow!("Unexpected response type")),
        }
    }

    // UTXO-related methods
    pub async fn get_utxos_by_addresses(
        &mut self,
        addresses: Vec<String>,
    ) -> Result<Vec<RpcUtxosByAddressesEntry>> {
        let request = KaspadRequest {
            id: 8,
            payload: Some(kaspad_request::Payload::GetUtxosByAddressesRequest(
                GetUtxosByAddressesRequestMessage { addresses },
            )),
        };

        let response = self.handler.send_request(request).await?;
        match response.payload {
            Some(kaspad_response::Payload::GetUtxosByAddressesResponse(resp)) => Ok(resp.entries),
            _ => Err(anyhow::anyhow!("Unexpected response type")),
        }
    }

    // Balance related methods
    pub async fn get_balance_by_address(&mut self, address: String) -> Result<u64> {
        let request = KaspadRequest {
            id: 9,
            payload: Some(kaspad_request::Payload::GetBalanceByAddressRequest(
                GetBalanceByAddressRequestMessage { address },
            )),
        };

        let response = self.handler.send_request(request).await?;
        match response.payload {
            Some(kaspad_response::Payload::GetBalanceByAddressResponse(resp)) => Ok(resp.balance),
            _ => Err(anyhow::anyhow!("Unexpected response type")),
        }
    }

    pub async fn get_balances_by_addresses(
        &mut self,
        addresses: Vec<String>,
    ) -> Result<Vec<RpcBalancesByAddressesEntry>> {
        let request = KaspadRequest {
            id: 10,
            payload: Some(kaspad_request::Payload::GetBalancesByAddressesRequest(
                GetBalancesByAddressesRequestMessage { addresses },
            )),
        };

        let response = self.handler.send_request(request).await?;
        match response.payload {
            Some(kaspad_response::Payload::GetBalancesByAddressesResponse(resp)) => {
                Ok(resp.entries)
            }
            _ => Err(anyhow::anyhow!("Unexpected response type")),
        }
    }

    // Network information methods
    pub async fn get_current_network(&mut self) -> Result<String> {
        let request = KaspadRequest {
            id: 11,
            payload: Some(kaspad_request::Payload::GetCurrentNetworkRequest(
                GetCurrentNetworkRequestMessage {},
            )),
        };

        let response = self.handler.send_request(request).await?;
        match response.payload {
            Some(kaspad_response::Payload::GetCurrentNetworkResponse(resp)) => {
                Ok(resp.current_network)
            }
            _ => Err(anyhow::anyhow!("Unexpected response type")),
        }
    }

    pub async fn get_block_dag_info(&mut self) -> Result<RpcBlockDagInfo> {
        let request = KaspadRequest {
            id: 12,
            payload: Some(kaspad_request::Payload::GetBlockDagInfoRequest(
                GetBlockDagInfoRequestMessage {},
            )),
        };

        let response = self.handler.send_request(request).await?;
        match response.payload {
            Some(kaspad_response::Payload::GetBlockDagInfoResponse(resp)) => Ok(RpcBlockDagInfo {
                network_name: resp.network_name,
                block_count: resp.block_count,
                header_count: resp.header_count,
                tip_hashes: resp.tip_hashes,
                difficulty: resp.difficulty,
                past_median_time: resp.past_median_time as u64,
                virtual_parent_hashes: resp.virtual_parent_hashes,
                pruning_point_hash: resp.pruning_point_hash,
                virtual_daa_score: resp.virtual_daa_score,
                sink: resp.sink,
            }),
            _ => Err(anyhow::anyhow!("Unexpected response type")),
        }
    }

    // Notification methods
    pub async fn notify_block_added(&mut self) -> Result<()> {
        let request = KaspadRequest {
            id: 13,
            payload: Some(kaspad_request::Payload::NotifyBlockAddedRequest(
                NotifyBlockAddedRequestMessage {
                    command: RpcNotifyCommand::NotifyStart as i32,
                },
            )),
        };

        let response = self.handler.send_request(request).await?;
        match response.payload {
            Some(kaspad_response::Payload::NotifyBlockAddedResponse(_)) => Ok(()),
            _ => Err(anyhow::anyhow!("Unexpected response type")),
        }
    }

    pub async fn notify_virtual_chain_changed(&mut self) -> Result<()> {
        let request = KaspadRequest {
            id: 14,
            payload: Some(kaspad_request::Payload::NotifyVirtualChainChangedRequest(
                NotifyVirtualChainChangedRequestMessage {
                    command: RpcNotifyCommand::NotifyStart as i32,
                    include_accepted_transaction_ids: false,
                },
            )),
        };

        let response = self.handler.send_request(request).await?;
        match response.payload {
            Some(kaspad_response::Payload::NotifyVirtualChainChangedResponse(_)) => Ok(()),
            _ => Err(anyhow::anyhow!("Unexpected response type")),
        }
    }

    pub async fn notify_utxos_changed(&mut self, addresses: Vec<String>) -> Result<()> {
        let request = KaspadRequest {
            id: 15,
            payload: Some(kaspad_request::Payload::NotifyUtxosChangedRequest(
                NotifyUtxosChangedRequestMessage {
                    addresses,
                    command: RpcNotifyCommand::NotifyStart as i32,
                },
            )),
        };

        let response = self.handler.send_request(request).await?;
        match response.payload {
            Some(kaspad_response::Payload::NotifyUtxosChangedResponse(_)) => Ok(()),
            _ => Err(anyhow::anyhow!("Unexpected response type")),
        }
    }

    // System information methods
    pub async fn get_info(&mut self) -> Result<RpcInfo> {
        let request = KaspadRequest {
            id: 16,
            payload: Some(kaspad_request::Payload::GetInfoRequest(
                GetInfoRequestMessage {},
            )),
        };

        let response = self.handler.send_request(request).await?;
        match response.payload {
            Some(kaspad_response::Payload::GetInfoResponse(resp)) => Ok(RpcInfo {
                p2p_id: resp.p2p_id,
                mempool_size: resp.mempool_size,
                server_version: resp.server_version,
                is_utxo_indexed: resp.is_utxo_indexed,
                is_synced: resp.is_synced,
            }),
            _ => Err(anyhow::anyhow!("Unexpected response type")),
        }
    }

    pub async fn get_sync_status(&mut self) -> Result<RpcSyncStatus> {
        let request = KaspadRequest {
            id: 17,
            payload: Some(kaspad_request::Payload::GetSyncStatusRequest(
                GetSyncStatusRequestMessage {},
            )),
        };

        let response = self.handler.send_request(request).await?;
        match response.payload {
            Some(kaspad_response::Payload::GetSyncStatusResponse(resp)) => Ok(RpcSyncStatus {
                is_synced: resp.is_synced,
                sync_duration: 0,
            }),
            _ => Err(anyhow::anyhow!("Unexpected response type")),
        }
    }

    pub async fn ping(&mut self) -> Result<()> {
        let request = KaspadRequest {
            id: 18,
            payload: Some(kaspad_request::Payload::PingRequest(PingRequestMessage {})),
        };

        let response = self.handler.send_request(request).await?;
        match response.payload {
            Some(kaspad_response::Payload::PingResponse(_)) => Ok(()),
            _ => Err(anyhow::anyhow!("Unexpected response type")),
        }
    }

    // Utility methods
    pub async fn get_virtual_chain_from_block(
        &mut self,
        start_hash: String,
    ) -> Result<VirtualChainFromBlockResponse> {
        let request = KaspadRequest {
            id: 19,
            payload: Some(kaspad_request::Payload::GetVirtualChainFromBlockRequest(
                GetVirtualChainFromBlockRequestMessage {
                    start_hash,
                    include_accepted_transaction_ids: false,
                },
            )),
        };

        let response = self.handler.send_request(request).await?;
        match response.payload {
            Some(kaspad_response::Payload::GetVirtualChainFromBlockResponse(resp)) => {
                Ok(VirtualChainFromBlockResponse {
                    added_chain_block_hashes: resp.added_chain_block_hashes,
                    accepted_transaction_ids: resp
                        .accepted_transaction_ids
                        .iter()
                        .flat_map(|id| id.accepted_transaction_ids.clone())
                        .collect(),
                    removed_chain_block_hashes: resp.removed_chain_block_hashes,
                })
            }
            _ => Err(anyhow::anyhow!("Unexpected response type")),
        }
    }
}

// Response data structures
#[derive(Debug, Clone)]
pub struct RpcBlockDagInfo {
    pub network_name: String,
    pub block_count: u64,
    pub header_count: u64,
    pub tip_hashes: Vec<String>,
    pub difficulty: f64,
    pub past_median_time: u64,
    pub virtual_parent_hashes: Vec<String>,
    pub pruning_point_hash: String,
    pub virtual_daa_score: u64,
    pub sink: String,
}

#[derive(Debug, Clone)]
pub struct RpcInfo {
    pub p2p_id: String,
    pub mempool_size: u64,
    pub server_version: String,
    pub is_utxo_indexed: bool,
    pub is_synced: bool,
}

#[derive(Debug, Clone)]
pub struct RpcSyncStatus {
    pub is_synced: bool,
    pub sync_duration: u64,
}

#[derive(Debug, Clone)]
pub struct VirtualChainFromBlockResponse {
    pub added_chain_block_hashes: Vec<String>,
    pub accepted_transaction_ids: Vec<String>,
    pub removed_chain_block_hashes: Vec<String>,
}
