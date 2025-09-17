use serde::{Deserialize, Serialize};

// Constant definitions from Go version
pub const OP_RANGE_BY: u64 = 100000;

// State key prefixes, consistent with Go version
pub const KEY_PREFIX_STATE_TOKEN: &str = "sttoken_";
pub const KEY_PREFIX_STATE_BALANCE: &str = "stbalance_";
pub const KEY_PREFIX_STATE_MARKET: &str = "stmarket_";
pub const KEY_PREFIX_STATE_BLACKLIST: &str = "stblacklist_";

// VSPC list related constants
pub const LEN_VSPC_LIST_MAX: usize = 1200;
pub const LEN_VSPC_LIST_RUNTIME_MAX: usize = 3600;
pub const LEN_VSPC_CHECK: usize = 200;
pub const LEN_ROLLBACK_LIST_RUNTIME_MAX: usize = 3600;

// Implement From trait for type conversion
impl From<TokenData> for StateTokenType {
    fn from(token: TokenData) -> Self {
        Self {
            tick: token.tick,
            max: token.max_supply.to_string(),
            lim: token.lim.unwrap_or("0".to_string()), // Read lim field from TokenData
            pre: token.pre.unwrap_or("0".to_string()), // Read pre field from TokenData
            dec: token.decimals as i32,
            mod_type: token.mode.clone(), // Read mode field from TokenData
            from: token.owner.clone(),
            to: token.owner,
            minted: token.minted_supply,
            burned: "0".to_string(), // Default value
            name: "".to_string(),    // Default value
            tx_id: token.deploy_tx_hash,
            op_add: 0, // Default value
            op_mod: 0, // Default value
            mts_add: token.deploy_timestamp as i64,
            mts_mod: 0, // Default value
        }
    }
}

impl From<BalanceData> for StateBalanceType {
    fn from(balance: BalanceData) -> Self {
        Self {
            address: balance.address,
            tick: balance.tick,
            dec: 0, // Default value
            balance: balance.balance.to_string(),
            locked: balance.locked,
            op_mod: 0, // Default value
        }
    }
}

impl From<MarketData> for StateMarketType {
    fn from(market: MarketData) -> Self {
        Self {
            tick: market.tick,
            t_addr: "".to_string(),   // Default value
            u_tx_id: "".to_string(),  // Default value
            u_addr: "".to_string(),   // Default value
            u_amt: "0".to_string(),   // Default value
            u_script: "".to_string(), // Default value
            t_amt: "0".to_string(),   // Default value
            op_add: 0,                // Default value
        }
    }
}

impl From<BlacklistEntry> for StateBlacklistType {
    fn from(entry: BlacklistEntry) -> Self {
        Self {
            tick: entry.tick,
            address: entry.address,
            op_add: 0, // Default value
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct VSPCData {
    pub block_hash: String,
    pub parent_hashes: Vec<String>,
    pub daa_score: u64,
    pub timestamp: u64,
    pub blue_score: u64,
    pub blue_work: String,
    pub pruning_point: String,
    pub difficulty: f64,
    pub is_header_only: bool,
    pub block_level: u32,
    pub block_status: u32,
    pub merge_set_blues: Vec<String>,
    pub merge_set_reds: Vec<String>,
    pub selected_parent: String,
    pub selected_tip: String,
    pub block_ghostdag_data: GhostDagData,
    pub block_relations: BlockRelations,
    pub block_acceptance_data: BlockAcceptanceData,
}

impl Default for VSPCData {
    fn default() -> Self {
        Self {
            block_hash: String::new(),
            parent_hashes: Vec::new(),
            daa_score: 0,
            timestamp: 0,
            blue_score: 0,
            blue_work: String::new(),
            pruning_point: String::new(),
            difficulty: 0.0,
            is_header_only: false,
            block_level: 0,
            block_status: 0,
            merge_set_blues: Vec::new(),
            merge_set_reds: Vec::new(),
            selected_parent: String::new(),
            selected_tip: String::new(),
            block_ghostdag_data: GhostDagData::default(),
            block_relations: BlockRelations::default(),
            block_acceptance_data: BlockAcceptanceData::default(),
        }
    }
}

// VSPC data structure corresponding to Go version
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DataVspcType {
    pub daa_score: u64,
    pub hash: String,
    pub tx_id_list: Vec<String>,
}

// Transaction data structure corresponding to Go version
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DataTransactionType {
    pub tx_id: String,
    pub daa_score: u64,
    pub block_accept: String,
    pub data: Option<serde_json::Value>, // Corresponding to Go version's *protowire.RpcTransaction
}

// Script data structure corresponding to Go version
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DataScriptType {
    pub p: String, // Corresponding to Go version's p field
    pub op: String,
    pub from: Option<String>,  // Keep Option type
    pub to: Option<String>,    // Keep Option type
    pub tick: Option<String>,  // Keep Option type
    pub max: Option<String>,   // Keep Option type
    pub lim: Option<String>,   // Keep Option type
    pub pre: Option<String>,   // Keep Option type
    pub dec: Option<String>,   // Keep Option type
    pub amt: Option<String>,   // Keep Option type
    pub utxo: Option<String>,  // Keep Option type
    pub price: Option<String>, // Keep Option type
    pub mod_type: String,      // Corresponding to Go version's mod field
    pub name: Option<String>,  // Keep Option type
    pub ca: Option<String>,    // Keep Option type
}

// Operation state data structure corresponding to Go version
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DataOpStateType {
    pub block_accept: Option<String>,
    pub fee: Option<u64>,
    pub fee_least: Option<u64>,
    pub mts_add: Option<i64>,
    pub op_score: Option<u64>,
    pub op_accept: Option<i8>,
    pub op_error: Option<String>,
    pub checkpoint: Option<String>,
}

// Statistics data structure corresponding to Go version
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DataStatsType {
    pub tick_affc: Vec<String>,
    pub address_affc: Vec<String>,
}

// Operation data structure corresponding to Go version
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DataOperationType {
    pub tx_id: String,
    pub daa_score: u64,
    pub block_accept: String,
    pub fee: u64,
    pub fee_least: u64,
    pub mts_add: i64,
    pub op_score: u64,
    pub op_accept: i8,
    pub op_error: String,
    pub op_script: Vec<DataScriptType>,
    pub script_sig: String,
    pub st_before: Vec<String>,
    pub st_after: Vec<String>,
    pub checkpoint: String,
    pub ss_info: Option<DataStatsType>, // Keep Option type
}

// Script data structure
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ScriptData {
    pub p: String,
    pub operation: String,
    pub from: Option<String>,
    pub to: Option<String>,
    pub tick: Option<String>,
    pub amount: Option<String>,
    pub utxo: Option<String>,
    pub ca: Option<String>,
    pub mode: Option<String>,
    pub tx_hash: Option<String>,
}

// State Token type corresponding to Go version
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct StateTokenType {
    pub tick: String,
    pub max: String,
    pub lim: String,
    pub pre: String,
    pub dec: i32,
    pub mod_type: String, // Corresponding to Go version's Mod field
    pub from: String,
    pub to: String,
    pub minted: String,
    pub burned: String,
    pub name: String,
    pub tx_id: String,
    pub op_add: u64,
    pub op_mod: u64,
    pub mts_add: i64,
    pub mts_mod: i64,
}

impl Default for StateTokenType {
    fn default() -> Self {
        Self {
            tick: String::new(),
            max: String::new(),
            lim: String::new(),
            pre: String::new(),
            dec: 0,
            mod_type: String::new(),
            from: String::new(),
            to: String::new(),
            minted: "0".to_string(),
            burned: "0".to_string(),
            name: String::new(),
            tx_id: String::new(),
            op_add: 0,
            op_mod: 0,
            mts_add: 0,
            mts_mod: 0,
        }
    }
}

// State Balance type corresponding to Go version
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct StateBalanceType {
    pub address: String,
    pub tick: String,
    pub dec: i32,        // Corresponding to Go version's Dec field
    pub balance: String, // Corresponding to Go version's Balance field
    pub locked: String,  // Corresponding to Go version's Locked field
    pub op_mod: u64,     // Corresponding to Go version's OpMod field
}

impl Default for StateBalanceType {
    fn default() -> Self {
        Self {
            address: String::new(),
            tick: String::new(),
            dec: 0,
            balance: "0".to_string(),
            locked: "0".to_string(),
            op_mod: 0,
        }
    }
}

// State Market type corresponding to Go version
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct StateMarketType {
    pub tick: String,
    pub t_addr: String,   // Corresponding to Go version's TAddr field
    pub u_tx_id: String,  // Corresponding to Go version's UTxId field
    pub u_addr: String,   // Corresponding to Go version's UAddr field
    pub u_amt: String,    // Corresponding to Go version's UAmt field
    pub u_script: String, // Corresponding to Go version's UScript field
    pub t_amt: String,    // Corresponding to Go version's TAmt field
    pub op_add: u64,      // Corresponding to Go version's OpAdd field
}

impl Default for StateMarketType {
    fn default() -> Self {
        Self {
            tick: String::new(),
            t_addr: String::new(),
            u_tx_id: String::new(),
            u_addr: String::new(),
            u_amt: "0".to_string(),
            u_script: String::new(),
            t_amt: "0".to_string(),
            op_add: 0,
        }
    }
}

// State blacklist type corresponding to Go version
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct StateBlacklistType {
    pub tick: String,
    pub address: String, // Corresponding to Go version's Address field
    pub op_add: u64,     // Corresponding to Go version's OpAdd field
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GhostDagData {
    pub blue_score: u64,
    pub blue_work: String,
    pub selected_parent: String,
    pub merge_set_blues: Vec<String>,
    pub merge_set_reds: Vec<String>,
}

impl Default for GhostDagData {
    fn default() -> Self {
        Self {
            blue_score: 0,
            blue_work: String::new(),
            selected_parent: String::new(),
            merge_set_blues: Vec::new(),
            merge_set_reds: Vec::new(),
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BlockRelations {
    pub parents: Vec<String>,
    pub children: Vec<String>,
}

impl Default for BlockRelations {
    fn default() -> Self {
        Self {
            parents: Vec::new(),
            children: Vec::new(),
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BlockAcceptanceData {
    pub block_hash: String,
    pub accepting_block_daa_score: u64,
    pub accepting_block_hash: String,
    pub block_acceptance_data: Vec<TransactionAcceptanceData>,
}

impl Default for BlockAcceptanceData {
    fn default() -> Self {
        Self {
            block_hash: String::new(),
            accepting_block_daa_score: 0,
            accepting_block_hash: String::new(),
            block_acceptance_data: Vec::new(),
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TransactionAcceptanceData {
    pub transaction_hash: String,
    pub accepting_block_hash: String,
    pub block_daa_score: u64,
    pub is_accepted: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TokenData {
    pub tick: String,
    pub max_supply: u64,
    pub circulating_supply: u64,
    pub decimals: u8,
    pub owner: String,
    pub is_blacklisted: bool,
    pub is_reserved: bool,
    pub deploy_tx_hash: String,
    pub deploy_block_hash: String,
    pub deploy_timestamp: u64,
    // Add missing fields
    pub mode: String,
    pub minted_supply: String,
    pub last_updated: u64,
    // Add lim and pre fields
    pub lim: Option<String>,
    pub pre: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BalanceData {
    pub address: String,
    pub tick: String,
    pub balance: u64,
    pub last_updated: u64,
    // Add missing fields
    pub locked: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MarketData {
    pub tick: String,
    pub price: f64,
    pub volume_24h: f64,
    pub market_cap: f64,
    pub last_updated: u64,
    // Add missing fields
    pub seller: String,
    pub tx_id: String,
    pub amount: String,
    pub status: String,
    pub created_at: u64,
    pub updated_at: u64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct OperationData {
    pub operation_type: String,
    pub tick: String,
    pub from_address: Option<String>,
    pub to_address: Option<String>,
    pub amount: Option<u64>,
    pub tx_hash: String,
    pub block_hash: String,
    pub timestamp: u64,
    pub block_daa_score: u64,
    // Add missing fields
    pub script: Option<ScriptData>,
    pub is_testnet: bool,
    pub daa_score: u64,
    pub tx_id: String,
    pub ca: Option<String>, // Add ca field
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RuntimeState {
    pub last_processed_block: String,
    pub last_processed_daa_score: u64,
    pub is_syncing: bool,
    pub sync_start_time: u64,
    pub total_blocks_processed: u64,
    pub total_operations_processed: u64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BlacklistEntry {
    pub tick: String,
    pub reason: String,
    pub added_at: u64,
    pub added_by: String,
    pub address: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ReservedToken {
    pub tick: String,
    pub reason: String,
    pub reserved_at: u64,
}

/// Storage operation enum, corresponding to Go version's storage operation types
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum StorageOperation {
    UpdateToken(TokenData),
    UpdateBalance(BalanceData),
    UpdateMarket(MarketData),
    UpdateRuntimeState(RuntimeState),
    InsertBlacklist(BlacklistEntry),
    InsertReservedToken(ReservedToken),
    InsertVSPC(VSPCData),
    InsertOperation(OperationData),
    DeleteVSPC(String),      // block_hash
    DeleteOperation(String), // tx_hash
    BatchRocksDB(Vec<StorageOperation>),
}

/// Rollback data structure, corresponding to Go version's DataRollbackType
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DataRollbackType {
    pub state_map_before: DataStateMapType,
    pub state_map_after: DataStateMapType,
    pub op_score_list: Vec<u64>,
    pub tx_id_list: Vec<String>,
    pub daa_score_start: u64,
    pub daa_score_end: u64,
    // Add missing fields from Go version
    pub checkpoint_before: String,
    pub checkpoint_after: String,
    pub op_score_last: u64,
}

impl DataRollbackType {
    pub fn new(
        state_map_before: DataStateMapType,
        state_map_after: DataStateMapType,
        op_score_list: Vec<u64>,
        tx_id_list: Vec<String>,
        daa_score_start: u64,
        daa_score_end: u64,
        checkpoint_before: String,
        checkpoint_after: String,
        op_score_last: u64,
    ) -> Self {
        Self {
            state_map_before,
            state_map_after,
            op_score_list,
            tx_id_list,
            daa_score_start,
            daa_score_end,
            checkpoint_before,
            checkpoint_after,
            op_score_last,
        }
    }
}

/// Rollback status
#[derive(Debug, Clone)]
pub struct RollbackStatus {
    pub is_rolling_back: bool,
    pub current_operation: Option<String>,
    pub total_operations: usize,
    pub completed_operations: usize,
    pub start_time: std::time::Instant,
    pub estimated_completion: Option<std::time::Instant>,
}

impl RollbackStatus {
    pub fn new(total_operations: usize) -> Self {
        Self {
            is_rolling_back: true,
            current_operation: None,
            total_operations,
            completed_operations: 0,
            start_time: std::time::Instant::now(),
            estimated_completion: None,
        }
    }

    pub fn progress(&self) -> f64 {
        if self.total_operations == 0 {
            0.0
        } else {
            self.completed_operations as f64 / self.total_operations as f64
        }
    }

    pub fn elapsed(&self) -> std::time::Duration {
        self.start_time.elapsed()
    }

    pub fn estimated_remaining(&self) -> Option<std::time::Duration> {
        if self.completed_operations == 0 {
            return None;
        }

        let elapsed = self.elapsed();
        let progress = self.progress();

        if progress > 0.0 {
            let total_estimated = elapsed.mul_f64(1.0 / progress);
            Some(total_estimated - elapsed)
        } else {
            None
        }
    }

    pub fn update_progress(&mut self, completed: usize, current_op: Option<String>) {
        self.completed_operations = completed;
        self.current_operation = current_op;

        // Update estimated completion time
        if self.completed_operations > 0 {
            let progress = self.progress();
            if progress > 0.0 {
                let elapsed = self.elapsed();
                let total_estimated = elapsed.mul_f64(1.0 / progress);
                self.estimated_completion = Some(self.start_time + total_estimated);
            }
        }
    }

    pub fn complete(&mut self) {
        self.completed_operations = self.total_operations;
        self.current_operation = None;
        self.is_rolling_back = false;
    }
}

impl Default for TokenData {
    fn default() -> Self {
        Self {
            tick: "".to_string(),
            max_supply: 0,
            circulating_supply: 0,
            decimals: 0,
            owner: "".to_string(),
            is_blacklisted: false,
            is_reserved: false,
            deploy_tx_hash: "".to_string(),
            deploy_block_hash: "".to_string(),
            deploy_timestamp: 0,
            mode: "".to_string(),
            minted_supply: "0".to_string(),
            last_updated: 0,
            lim: None,
            pre: None,
        }
    }
}

impl Default for BalanceData {
    fn default() -> Self {
        Self {
            address: "".to_string(),
            tick: "".to_string(),
            balance: 0,
            last_updated: 0,
            locked: "0".to_string(),
        }
    }
}

impl Default for MarketData {
    fn default() -> Self {
        Self {
            tick: "".to_string(),
            price: 0.0,
            volume_24h: 0.0,
            market_cap: 0.0,
            last_updated: 0,
            seller: "".to_string(),
            tx_id: "".to_string(),
            amount: "0".to_string(),
            status: "".to_string(),
            created_at: 0,
            updated_at: 0,
        }
    }
}

impl Default for OperationData {
    fn default() -> Self {
        Self {
            operation_type: "".to_string(),
            tick: "".to_string(),
            from_address: None,
            to_address: None,
            amount: None,
            tx_hash: "".to_string(),
            block_hash: "".to_string(),
            timestamp: 0,
            block_daa_score: 0,
            script: None,
            is_testnet: false,
            daa_score: 0,
            tx_id: "".to_string(),
            ca: None,
        }
    }
}

impl Default for BlacklistEntry {
    fn default() -> Self {
        Self {
            tick: "".to_string(),
            reason: "".to_string(),
            added_at: 0,
            added_by: "".to_string(),
            address: "".to_string(),
        }
    }
}

impl Default for StateBlacklistType {
    fn default() -> Self {
        Self {
            tick: String::new(),
            address: String::new(),
            op_add: 0,
        }
    }
}

/// State map type, corresponding to Go version's DataStateMapType
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DataStateMapType {
    pub state_token_map: std::collections::HashMap<String, Option<StateTokenType>>,
    pub state_balance_map: std::collections::HashMap<String, Option<StateBalanceType>>,
    pub state_market_map: std::collections::HashMap<String, Option<StateMarketType>>,
    pub state_blacklist_map: std::collections::HashMap<String, Option<StateBlacklistType>>,
}

impl DataStateMapType {
    pub fn new() -> Self {
        Self {
            state_token_map: std::collections::HashMap::new(),
            state_balance_map: std::collections::HashMap::new(),
            state_market_map: std::collections::HashMap::new(),
            state_blacklist_map: std::collections::HashMap::new(),
        }
    }
}
