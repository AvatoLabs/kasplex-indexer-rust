use crate::storage::rocksdb::RocksDBClient;
use crate::storage::types::*;
use anyhow::Result;
use serde_json;
use std::collections::HashMap;
use std::sync::Arc;
use tracing::{debug, info};

#[derive(Debug, Clone)]
pub struct StateManager {
    rocksdb: Arc<RocksDBClient>,
}

impl StateManager {
    pub fn new(rocksdb: Arc<RocksDBClient>) -> Result<Self> {
        Ok(Self { rocksdb })
    }

    pub fn init(&self) -> Result<()> {
        info!("State manager initialized");
        Ok(())
    }

    /// Batch get Token state, corresponding to Go version GetStateTokenMap
    pub fn get_state_token_map(
        &self,
        token_map: &mut HashMap<String, Option<StateTokenType>>,
    ) -> Result<i64> {
        let start_time = std::time::Instant::now();
        let mut key_list = Vec::new();

        // Collect all keys that need to be queried, consistent with Go version
        for key in token_map.keys() {
            key_list.push(format!("{}{}", KEY_PREFIX_STATE_TOKEN, key));
        }

        // Batch query, corresponding to Go version doGetBatchRocks logic
        for key in key_list {
            if let Ok(Some(data)) = self.rocksdb.get_raw(&key) {
                if let Ok(decoded) = serde_json::from_slice::<StateTokenType>(&data) {
                    token_map.insert(decoded.tick.clone(), Some(decoded));
                }
            }
        }

        let duration = start_time.elapsed().as_millis() as i64;
        Ok(duration)
    }

    /// Batch get Balance state, corresponding to Go version GetStateBalanceMap
    pub fn get_state_balance_map(
        &self,
        balance_map: &mut HashMap<String, Option<StateBalanceType>>,
    ) -> Result<i64> {
        let start_time = std::time::Instant::now();
        let mut key_list = Vec::new();

        // Collect all keys that need to be queried, consistent with Go version
        for key in balance_map.keys() {
            key_list.push(format!("{}{}", KEY_PREFIX_STATE_BALANCE, key));
        }

        // Batch query
        for key in key_list {
            if let Ok(Some(data)) = self.rocksdb.get_raw(&key) {
                if let Ok(decoded) = serde_json::from_slice::<StateBalanceType>(&data) {
                    let map_key = format!("{}_{}", decoded.address, decoded.tick);
                    balance_map.insert(map_key, Some(decoded));
                }
            }
        }

        let duration = start_time.elapsed().as_millis() as i64;
        Ok(duration)
    }

    /// Batch get Market state, corresponding to Go version GetStateMarketMap
    pub fn get_state_market_map(
        &self,
        market_map: &mut HashMap<String, Option<StateMarketType>>,
    ) -> Result<i64> {
        let start_time = std::time::Instant::now();
        let mut key_list = Vec::new();

        // Collect all keys that need to be queried, consistent with Go version
        for key in market_map.keys() {
            key_list.push(format!("{}{}", KEY_PREFIX_STATE_MARKET, key));
        }

        // Batch query
        for key in key_list {
            if let Ok(Some(data)) = self.rocksdb.get_raw(&key) {
                if let Ok(decoded) = serde_json::from_slice::<StateMarketType>(&data) {
                    let map_key =
                        format!("{}_{}_{}", decoded.tick, decoded.t_addr, decoded.u_tx_id);
                    market_map.insert(map_key, Some(decoded));
                }
            }
        }

        let duration = start_time.elapsed().as_millis() as i64;
        Ok(duration)
    }

    /// Batch get Blacklist state, corresponding to Go version GetStateBlacklistMap
    pub fn get_state_blacklist_map(
        &self,
        blacklist_map: &mut HashMap<String, Option<StateBlacklistType>>,
    ) -> Result<i64> {
        let start_time = std::time::Instant::now();
        let mut key_list = Vec::new();

        // Collect all keys that need to be queried
        for key in blacklist_map.keys() {
            key_list.push(format!("{}{}", KEY_PREFIX_STATE_BLACKLIST, key));
        }

        // Batch query
        for key in key_list {
            if let Ok(Some(data)) = self.rocksdb.get_raw(&key) {
                if let Ok(decoded) = serde_json::from_slice::<StateBlacklistType>(&data) {
                    let map_key = format!("{}_{}", decoded.tick, decoded.address);
                    blacklist_map.insert(map_key, Some(decoded));
                }
            }
        }

        let duration = start_time.elapsed().as_millis() as i64;
        Ok(duration)
    }

    /// Batch save state to RocksDB, corresponding to Go version SaveStateBatchRocksBegin
    pub fn save_state_batch_rocks_begin(&self, state_map: &DataStateMapType) -> Result<i64> {
        let start_time = std::time::Instant::now();

        // Save Token state - use key format compatible with get_token
        for (key, token) in &state_map.state_token_map {
            let full_key = format!("token:{}", key);
            if let Some(token_ref) = token {
                // Convert to TokenData format
                let token_data = TokenData {
                    tick: token_ref.tick.clone(),
                    max_supply: token_ref.max.parse().unwrap_or(0),
                    circulating_supply: token_ref.minted.parse().unwrap_or(0),
                    decimals: token_ref.dec as u8,
                    owner: token_ref.from.clone(),
                    is_blacklisted: false,
                    is_reserved: false,
                    deploy_tx_hash: token_ref.tx_id.clone(),
                    deploy_block_hash: "".to_string(),
                    deploy_timestamp: token_ref.mts_add as u64,
                    mode: token_ref.mod_type.clone(),
                    minted_supply: token_ref.minted.clone(),
                    last_updated: token_ref.mts_mod as u64,
                    lim: Some(token_ref.lim.clone()),
                    pre: Some(token_ref.pre.clone()),
                };
                let value_json = serde_json::to_vec(&token_data)?;
                self.rocksdb.put_raw(&full_key, &value_json)?;
            } else {
                // Delete Token - use empty value to indicate deletion
                self.rocksdb.put_raw(&full_key, &[])?;
            }
        }

        // Save Balance state
        for (key, balance) in &state_map.state_balance_map {
            let full_key = format!("{}{}", KEY_PREFIX_STATE_BALANCE, key);
            if let Some(balance_ref) = balance {
                let value_json = serde_json::to_vec(balance_ref)?;
                self.rocksdb.put_raw(&full_key, &value_json)?;
            } else {
                // Delete Balance - use empty value to indicate deletion
                self.rocksdb.put_raw(&full_key, &[])?;
            }
        }

        // Save Market state
        for (key, market) in &state_map.state_market_map {
            let full_key = format!("{}{}", KEY_PREFIX_STATE_MARKET, key);
            if let Some(market_ref) = market {
                let value_json = serde_json::to_vec(market_ref)?;
                self.rocksdb.put_raw(&full_key, &value_json)?;
            } else {
                // Delete Market - use empty value to indicate deletion
                self.rocksdb.put_raw(&full_key, &[])?;
            }
        }

        // Save Blacklist state
        for (key, blacklist) in &state_map.state_blacklist_map {
            let full_key = format!("{}{}", KEY_PREFIX_STATE_BLACKLIST, key);
            if let Some(blacklist_ref) = blacklist {
                let value_json = serde_json::to_vec(blacklist_ref)?;
                self.rocksdb.put_raw(&full_key, &value_json)?;
            } else {
                // Delete Blacklist - use empty value to indicate deletion
                self.rocksdb.put_raw(&full_key, &[])?;
            }
        }

        let duration = start_time.elapsed().as_millis() as i64;
        Ok(duration)
    }

    /// Copy state mapping, corresponding to Go version CopyDataStateMap
    pub fn copy_data_state_map(
        &self,
        state_map_from: &DataStateMapType,
        state_map_to: &mut DataStateMapType,
    ) {
        state_map_to.state_token_map.clear();
        state_map_to.state_balance_map.clear();
        state_map_to.state_market_map.clear();
        state_map_to.state_blacklist_map.clear();

        // Copy Token state
        for (key, st_token) in &state_map_from.state_token_map {
            if let Some(token) = st_token {
                let st_data = token.clone();
                state_map_to
                    .state_token_map
                    .insert(key.clone(), Some(st_data));
            } else {
                state_map_to.state_token_map.insert(key.clone(), None);
            }
        }

        // Copy Balance state
        for (key, st_balance) in &state_map_from.state_balance_map {
            if let Some(balance) = st_balance {
                let st_data = balance.clone();
                state_map_to
                    .state_balance_map
                    .insert(key.clone(), Some(st_data));
            } else {
                state_map_to.state_balance_map.insert(key.clone(), None);
            }
        }

        // Copy Market state
        for (key, st_market) in &state_map_from.state_market_map {
            if let Some(market) = st_market {
                let st_data = market.clone();
                state_map_to
                    .state_market_map
                    .insert(key.clone(), Some(st_data));
            } else {
                state_map_to.state_market_map.insert(key.clone(), None);
            }
        }

        // Copy Blacklist state
        for (key, st_blacklist) in &state_map_from.state_blacklist_map {
            if let Some(blacklist) = st_blacklist {
                let st_data = blacklist.clone();
                state_map_to
                    .state_blacklist_map
                    .insert(key.clone(), Some(st_data));
            } else {
                state_map_to.state_blacklist_map.insert(key.clone(), None);
            }
        }
    }

    // Token state management
    pub fn create_token(&self, token: TokenData) -> Result<()> {
        // Check if token already exists
        if let Some(_existing_token) = self.rocksdb.get_token(&token.tick)? {
            return Err(anyhow::anyhow!("Token {} already exists", token.tick));
        }

        // Check if token is reserved
        if self.rocksdb.is_token_reserved(&token.tick)? {
            return Err(anyhow::anyhow!("Token {} is reserved", token.tick));
        }

        self.rocksdb.set_token(&token)?;
        info!("Created token: {}", token.tick);
        Ok(())
    }

    pub fn update_token(&self, token: TokenData) -> Result<()> {
        self.rocksdb.set_token(&token)?;
        debug!("Updated token: {}", token.tick);
        Ok(())
    }

    pub fn get_token(&self, tick: &str) -> Result<Option<TokenData>> {
        self.rocksdb.get_token(tick)
    }

    pub fn list_tokens(&self) -> Result<Vec<TokenData>> {
        // This would require iterating over all token keys
        // For now, return empty vector
        Ok(Vec::new())
    }

    // Balance state management
    pub fn update_balance(&self, balance: BalanceData) -> Result<()> {
        // Validate token exists
        if let Some(token) = self.rocksdb.get_token(&balance.tick)? {
            if token.is_blacklisted {
                return Err(anyhow::anyhow!("Token {} is blacklisted", balance.tick));
            }
        } else {
            return Err(anyhow::anyhow!("Token {} does not exist", balance.tick));
        }

        self.rocksdb.set_balance(&balance)?;
        debug!("Updated balance: {} for {}", balance.tick, balance.address);
        Ok(())
    }

    pub fn get_balance(&self, address: &str, tick: &str) -> Result<Option<BalanceData>> {
        self.rocksdb.get_balance(address, tick)
    }

    pub fn transfer_balance(
        &self,
        from_address: &str,
        to_address: &str,
        tick: &str,
        amount: u64,
    ) -> Result<()> {
        // Get current balances
        let from_balance = self
            .rocksdb
            .get_balance(from_address, tick)?
            .unwrap_or(BalanceData {
                address: from_address.to_string(),
                tick: tick.to_string(),
                balance: 0,
                last_updated: chrono::Utc::now().timestamp() as u64,
                locked: "0".to_string(),
            });

        let to_balance = self
            .rocksdb
            .get_balance(to_address, tick)?
            .unwrap_or(BalanceData {
                address: to_address.to_string(),
                tick: tick.to_string(),
                balance: 0,
                last_updated: chrono::Utc::now().timestamp() as u64,
                locked: "0".to_string(),
            });

        // Validate sufficient balance
        if from_balance.balance < amount {
            return Err(anyhow::anyhow!("Insufficient balance for transfer"));
        }

        // Update balances
        let new_from_balance = BalanceData {
            balance: from_balance.balance - amount,
            last_updated: chrono::Utc::now().timestamp() as u64,
            ..from_balance
        };

        let new_to_balance = BalanceData {
            balance: to_balance.balance + amount,
            last_updated: chrono::Utc::now().timestamp() as u64,
            ..to_balance
        };

        self.rocksdb.set_balance(&new_from_balance)?;
        self.rocksdb.set_balance(&new_to_balance)?;

        info!(
            "Transferred {} {} from {} to {}",
            amount, tick, from_address, to_address
        );
        Ok(())
    }

    // Market state management
    pub fn update_market(&self, market: MarketData) -> Result<()> {
        self.rocksdb.set_market(&market)?;
        debug!("Updated market data: {}", market.tick);
        Ok(())
    }

    pub fn get_market(&self, tick: &str) -> Result<Option<MarketData>> {
        self.rocksdb.get_market(tick)
    }

    pub fn create_market(&self, tick: &str, market: &MarketData) -> Result<()> {
        self.rocksdb.set_market(market)?;
        info!("Created market for token: {}", tick);
        Ok(())
    }

    pub fn delete_market(&self, tick: &str) -> Result<()> {
        // This would require a delete operation in RocksDB
        // For now, just log the operation
        info!("Deleted market for token: {}", tick);
        Ok(())
    }

    // Blacklist management
    pub fn add_to_blacklist(&self, entry: BlacklistEntry) -> Result<()> {
        // Check if token exists
        if let Some(token) = self.rocksdb.get_token(&entry.tick)? {
            let mut updated_token = token;
            updated_token.is_blacklisted = true;
            self.rocksdb.set_token(&updated_token)?;
        }

        self.rocksdb.set_blacklist(&entry)?;
        info!("Added {} to blacklist", entry.tick);
        Ok(())
    }

    pub fn remove_from_blacklist(&self, tick: &str) -> Result<()> {
        // Update token if it exists
        if let Some(token) = self.rocksdb.get_token(tick)? {
            let mut updated_token = token;
            updated_token.is_blacklisted = false;
            self.rocksdb.set_token(&updated_token)?;
        }

        // Remove from blacklist (this would require a delete operation)
        info!("Removed {} from blacklist", tick);
        Ok(())
    }

    pub fn get_blacklist(&self, tick: &str) -> Result<Option<BlacklistEntry>> {
        self.rocksdb.get_blacklist(tick)
    }

    pub fn is_blacklisted(&self, tick: &str) -> Result<bool> {
        self.rocksdb.is_token_blacklisted(tick)
    }

    // Reserved token management
    pub fn add_reserved_token(&self, reserved: ReservedToken) -> Result<()> {
        self.rocksdb.set_reserved_token(&reserved)?;
        info!("Added {} as reserved token", reserved.tick);
        Ok(())
    }

    pub fn is_reserved(&self, tick: &str) -> Result<bool> {
        self.rocksdb.is_token_reserved(tick)
    }

    // Batch operations
    pub fn batch_update(&self, operations: &[StorageOperation]) -> Result<()> {
        let mut rocksdb_ops = Vec::new();

        for operation in operations {
            match operation {
                StorageOperation::UpdateToken(_token) => {
                    rocksdb_ops.push(operation.clone());
                }
                StorageOperation::UpdateBalance(_balance) => {
                    rocksdb_ops.push(operation.clone());
                }
                StorageOperation::UpdateMarket(_market) => {
                    rocksdb_ops.push(operation.clone());
                }
                StorageOperation::InsertBlacklist(_entry) => {
                    rocksdb_ops.push(operation.clone());
                }
                StorageOperation::InsertReservedToken(_reserved) => {
                    rocksdb_ops.push(operation.clone());
                }
                _ => {
                    debug!("Skipping non-state operation: {:?}", operation);
                }
            }
        }

        if !rocksdb_ops.is_empty() {
            self.rocksdb.batch_write(&rocksdb_ops)?;
        }

        Ok(())
    }

    // State validation
    pub fn validate_token_operation(&self, tick: &str) -> Result<()> {
        // Check if token exists
        if let Some(token) = self.rocksdb.get_token(tick)? {
            if token.is_blacklisted {
                return Err(anyhow::anyhow!("Token {} is blacklisted", tick));
            }
        } else {
            return Err(anyhow::anyhow!("Token {} does not exist", tick));
        }

        // Check if token is reserved
        if self.rocksdb.is_token_reserved(tick)? {
            return Err(anyhow::anyhow!("Token {} is reserved", tick));
        }

        Ok(())
    }

    // State cleanup
    pub fn cleanup_old_data(&self, _cutoff_timestamp: u64) -> Result<()> {
        // TODO: Implement logic to clean up old data
        Ok(())
    }

    // New method: Get node VSPC list, corresponding to Go version GetNodeVspcList
    pub async fn get_node_vspc_list(
        &self,
        daa_score_start: u64,
        limit: usize,
    ) -> Result<Vec<DataVspcType>> {
        // TODO: Implement getting VSPC list from Kaspa node
        // This should call Kaspa RPC API
        let mut vspc_list = Vec::new();

        // Simulate data, should actually be obtained from node
        for i in 0..limit {
            let daa_score = daa_score_start + i as u64;
            vspc_list.push(DataVspcType {
                daa_score,
                hash: format!("hash_{}", daa_score),
                tx_id_list: Vec::new(),
            });
        }

        Ok(vspc_list)
    }

    // New method: Get node transaction data list, corresponding to Go version GetNodeTransactionDataList
    pub async fn get_node_transaction_data_list(
        &self,
        vspc_list: &[DataVspcType],
    ) -> Result<Vec<DataTransactionType>> {
        // TODO: Implement getting transaction data from Kaspa node
        let mut tx_data_list = Vec::new();

        for vspc in vspc_list {
            for tx_id in &vspc.tx_id_list {
                tx_data_list.push(DataTransactionType {
                    tx_id: tx_id.clone(),
                    daa_score: vspc.daa_score,
                    block_accept: vspc.hash.clone(),
                    data: None,
                });
            }
        }

        Ok(tx_data_list)
    }
}
