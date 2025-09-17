use crate::operations::{
    BlacklistOperation, BurnOperation, ChownOperation, DeployOperation, IssueOperation,
    ListOperation, MintOperation, SendOperation, TransferOperation,
};
use crate::storage::StorageManager;
use crate::storage::types::*;
use anyhow::Result;
use std::collections::HashMap;
use std::sync::Arc;
use tracing::info;

/// Operation manager, corresponding to Go version's operation registration and execution mechanism
pub struct OperationManager {
    storage: Arc<StorageManager>,
    // Registration mapping corresponding to Go version
    p_registered: HashMap<String, bool>,
    op_registered: HashMap<String, bool>,
    method_registered: HashMap<String, Box<dyn OperationMethod>>,
    op_recycle_registered: HashMap<String, bool>,
}

/// Operation method trait, corresponding to Go version's OpMethod interface
pub trait OperationMethod: Send + Sync {
    fn script_collect_ex(
        &self,
        index: usize,
        script: &mut DataScriptType,
        tx_data: &DataTransactionType,
        testnet: bool,
    );
    fn validate(
        &self,
        script: &mut DataScriptType,
        tx_id: &str,
        daa_score: u64,
        testnet: bool,
    ) -> bool;
    fn fee_least(&self, daa_score: u64) -> u64;
    fn prepare_state_key(&self, script: &DataScriptType, state_map: &mut DataStateMapType);
    fn do_operation(
        &self,
        index: usize,
        op_data: &mut DataOperationType,
        state_map: &mut DataStateMapType,
        testnet: bool,
    ) -> Result<()>;
}

impl OperationManager {
    pub fn new(storage: Arc<StorageManager>) -> Self {
        let mut manager = Self {
            storage,
            p_registered: HashMap::new(),
            op_registered: HashMap::new(),
            method_registered: HashMap::new(),
            op_recycle_registered: HashMap::new(),
        };

        // Register operations, corresponding to Go version init function
        manager.register_operations();

        manager
    }

    /// Register operations, corresponding to Go version's init function
    fn register_operations(&mut self) {
        // Register protocol
        self.p_registered.insert("KRC-20".to_string(), true);

        // Register deploy operation
        self.op_registered.insert("deploy".to_string(), true);
        self.method_registered
            .insert("deploy".to_string(), Box::new(DeployOperationHandler));

        // Register mint operation
        self.op_registered.insert("mint".to_string(), true);
        self.method_registered
            .insert("mint".to_string(), Box::new(MintOperationHandler));

        // Register transfer operation
        self.op_registered.insert("transfer".to_string(), true);
        self.method_registered
            .insert("transfer".to_string(), Box::new(TransferOperationHandler));

        // Register issue operation
        self.op_registered.insert("issue".to_string(), true);
        self.method_registered
            .insert("issue".to_string(), Box::new(IssueOperationHandler));

        // Register list operation
        self.op_registered.insert("list".to_string(), true);
        self.method_registered
            .insert("list".to_string(), Box::new(ListOperationHandler));

        // Register send operation (recyclable operation)
        self.op_registered.insert("send".to_string(), true);
        self.method_registered
            .insert("send".to_string(), Box::new(SendOperationHandler));
        self.op_recycle_registered.insert("send".to_string(), true);

        // Register burn operation
        self.op_registered.insert("burn".to_string(), true);
        self.method_registered
            .insert("burn".to_string(), Box::new(BurnOperationHandler));

        // Register blacklist operation
        self.op_registered.insert("blacklist".to_string(), true);
        self.method_registered
            .insert("blacklist".to_string(), Box::new(BlacklistOperationHandler));

        // Register ownership operation
        self.op_registered.insert("chown".to_string(), true);
        self.method_registered
            .insert("chown".to_string(), Box::new(ChownOperationHandler));

        info!(
            "Registered operations: {:?}",
            self.op_registered.keys().collect::<Vec<_>>()
        );
    }

    /// Get list of supported operations
    pub fn get_supported_operations(&self) -> Vec<String> {
        self.op_registered.keys().cloned().collect()
    }

    /// Check if operation is registered
    pub fn is_operation_registered(&self, op_name: &str) -> bool {
        self.op_registered.contains_key(op_name)
    }

    /// Check if operation is recyclable
    pub fn is_operation_recyclable(&self, op_name: &str) -> bool {
        self.op_recycle_registered.contains_key(op_name)
    }

    /// Get operation fee
    pub fn get_operation_fee(&self, op_name: &str, daa_score: u64) -> u64 {
        if let Some(method) = self.method_registered.get(op_name) {
            method.fee_least(daa_score)
        } else {
            0
        }
    }

    /// Validate operation
    pub fn validate_operation(
        &self,
        script: &mut DataScriptType,
        tx_id: &str,
        daa_score: u64,
        testnet: bool,
    ) -> bool {
        if let Some(method) = self.method_registered.get(&script.op) {
            method.validate(script, tx_id, daa_score, testnet)
        } else {
            false
        }
    }

    /// Execute operation
    pub fn execute_operation(
        &self,
        index: usize,
        op_data: &mut DataOperationType,
        state_map: &mut DataStateMapType,
        testnet: bool,
    ) -> Result<()> {
        if let Some(first_script) = op_data.op_script.first() {
            if let Some(method) = self.method_registered.get(&first_script.op) {
                method.do_operation(index, op_data, state_map, testnet)
            } else {
                Err(anyhow::anyhow!("Unknown operation: {}", first_script.op))
            }
        } else {
            Err(anyhow::anyhow!("No operation script found"))
        }
    }
}

// Operation handler implementation - using independent operation modules

struct DeployOperationHandler;

impl OperationMethod for DeployOperationHandler {
    fn script_collect_ex(
        &self,
        index: usize,
        script: &mut DataScriptType,
        tx_data: &DataTransactionType,
        testnet: bool,
    ) {
        DeployOperation::script_collect_ex(index, script, tx_data, testnet)
    }

    fn validate(
        &self,
        script: &mut DataScriptType,
        tx_id: &str,
        daa_score: u64,
        testnet: bool,
    ) -> bool {
        DeployOperation::validate(script, tx_id, daa_score, testnet)
    }

    fn fee_least(&self, daa_score: u64) -> u64 {
        DeployOperation::fee_least(daa_score)
    }

    fn prepare_state_key(&self, script: &DataScriptType, state_map: &mut DataStateMapType) {
        DeployOperation::prepare_state_key(script, state_map)
    }

    fn do_operation(
        &self,
        index: usize,
        op_data: &mut DataOperationType,
        state_map: &mut DataStateMapType,
        testnet: bool,
    ) -> Result<()> {
        let script = op_data.op_script[index].clone();
        DeployOperation::do_operation(&script, op_data, state_map, testnet)
    }
}

struct MintOperationHandler;

impl OperationMethod for MintOperationHandler {
    fn script_collect_ex(
        &self,
        index: usize,
        script: &mut DataScriptType,
        tx_data: &DataTransactionType,
        testnet: bool,
    ) {
        MintOperation::script_collect_ex(index, script, tx_data, testnet)
    }

    fn validate(
        &self,
        script: &mut DataScriptType,
        tx_id: &str,
        daa_score: u64,
        testnet: bool,
    ) -> bool {
        MintOperation::validate(script, tx_id, daa_score, testnet)
    }

    fn fee_least(&self, daa_score: u64) -> u64 {
        MintOperation::fee_least(daa_score)
    }

    fn prepare_state_key(&self, script: &DataScriptType, state_map: &mut DataStateMapType) {
        MintOperation::prepare_state_key(script, state_map)
    }

    fn do_operation(
        &self,
        index: usize,
        op_data: &mut DataOperationType,
        state_map: &mut DataStateMapType,
        testnet: bool,
    ) -> Result<()> {
        let script = op_data.op_script[index].clone();
        MintOperation::do_operation(index, op_data, state_map, testnet)
    }
}

struct TransferOperationHandler;

impl OperationMethod for TransferOperationHandler {
    fn script_collect_ex(
        &self,
        index: usize,
        script: &mut DataScriptType,
        tx_data: &DataTransactionType,
        testnet: bool,
    ) {
        TransferOperation::script_collect_ex(index, script, tx_data, testnet)
    }

    fn validate(
        &self,
        script: &mut DataScriptType,
        tx_id: &str,
        daa_score: u64,
        testnet: bool,
    ) -> bool {
        TransferOperation::validate(script, tx_id, daa_score, testnet)
    }

    fn fee_least(&self, daa_score: u64) -> u64 {
        TransferOperation::fee_least(daa_score)
    }

    fn prepare_state_key(&self, script: &DataScriptType, state_map: &mut DataStateMapType) {
        TransferOperation::prepare_state_key(script, state_map)
    }

    fn do_operation(
        &self,
        index: usize,
        op_data: &mut DataOperationType,
        state_map: &mut DataStateMapType,
        testnet: bool,
    ) -> Result<()> {
        let script = op_data.op_script[index].clone();
        TransferOperation::do_operation(index, op_data, state_map, testnet)
    }
}

struct BurnOperationHandler;

impl OperationMethod for BurnOperationHandler {
    fn script_collect_ex(
        &self,
        index: usize,
        script: &mut DataScriptType,
        tx_data: &DataTransactionType,
        testnet: bool,
    ) {
        BurnOperation::script_collect_ex(index, script, tx_data, testnet)
    }

    fn validate(
        &self,
        script: &mut DataScriptType,
        tx_id: &str,
        daa_score: u64,
        testnet: bool,
    ) -> bool {
        BurnOperation::validate(script, tx_id, daa_score, testnet)
    }

    fn fee_least(&self, daa_score: u64) -> u64 {
        BurnOperation::fee_least(daa_score)
    }

    fn prepare_state_key(&self, script: &DataScriptType, state_map: &mut DataStateMapType) {
        BurnOperation::prepare_state_key(script, state_map)
    }

    fn do_operation(
        &self,
        index: usize,
        op_data: &mut DataOperationType,
        state_map: &mut DataStateMapType,
        testnet: bool,
    ) -> Result<()> {
        let script = op_data.op_script[index].clone();
        BurnOperation::do_operation(index, op_data, state_map, testnet)
    }
}

struct SendOperationHandler;

impl OperationMethod for SendOperationHandler {
    fn script_collect_ex(
        &self,
        index: usize,
        script: &mut DataScriptType,
        tx_data: &DataTransactionType,
        testnet: bool,
    ) {
        SendOperation::script_collect_ex(index, script, tx_data, testnet)
    }

    fn validate(
        &self,
        script: &mut DataScriptType,
        tx_id: &str,
        daa_score: u64,
        testnet: bool,
    ) -> bool {
        SendOperation::validate(script, tx_id, daa_score, testnet)
    }

    fn fee_least(&self, daa_score: u64) -> u64 {
        SendOperation::fee_least(daa_score)
    }

    fn prepare_state_key(&self, script: &DataScriptType, state_map: &mut DataStateMapType) {
        SendOperation::prepare_state_key(script, state_map)
    }

    fn do_operation(
        &self,
        index: usize,
        op_data: &mut DataOperationType,
        state_map: &mut DataStateMapType,
        testnet: bool,
    ) -> Result<()> {
        let script = op_data.op_script[index].clone();
        SendOperation::do_operation(index, op_data, state_map, testnet)
    }
}

struct IssueOperationHandler;

impl OperationMethod for IssueOperationHandler {
    fn script_collect_ex(
        &self,
        index: usize,
        script: &mut DataScriptType,
        tx_data: &DataTransactionType,
        testnet: bool,
    ) {
        IssueOperation::script_collect_ex(index, script, tx_data, testnet)
    }

    fn validate(
        &self,
        script: &mut DataScriptType,
        tx_id: &str,
        daa_score: u64,
        testnet: bool,
    ) -> bool {
        IssueOperation::validate(script, tx_id, daa_score, testnet)
    }

    fn fee_least(&self, daa_score: u64) -> u64 {
        IssueOperation::fee_least(daa_score)
    }

    fn prepare_state_key(&self, script: &DataScriptType, state_map: &mut DataStateMapType) {
        IssueOperation::prepare_state_key(script, state_map)
    }

    fn do_operation(
        &self,
        index: usize,
        op_data: &mut DataOperationType,
        state_map: &mut DataStateMapType,
        testnet: bool,
    ) -> Result<()> {
        let script = op_data.op_script[index].clone();
        IssueOperation::do_operation(index, op_data, state_map, testnet)
    }
}

struct ListOperationHandler;

impl OperationMethod for ListOperationHandler {
    fn script_collect_ex(
        &self,
        index: usize,
        script: &mut DataScriptType,
        tx_data: &DataTransactionType,
        testnet: bool,
    ) {
        ListOperation::script_collect_ex(index, script, tx_data, testnet)
    }

    fn validate(
        &self,
        script: &mut DataScriptType,
        tx_id: &str,
        daa_score: u64,
        testnet: bool,
    ) -> bool {
        ListOperation::validate(script, tx_id, daa_score, testnet)
    }

    fn fee_least(&self, daa_score: u64) -> u64 {
        ListOperation::fee_least(daa_score)
    }

    fn prepare_state_key(&self, script: &DataScriptType, state_map: &mut DataStateMapType) {
        ListOperation::prepare_state_key(script, state_map)
    }

    fn do_operation(
        &self,
        index: usize,
        op_data: &mut DataOperationType,
        state_map: &mut DataStateMapType,
        testnet: bool,
    ) -> Result<()> {
        let script = op_data.op_script[index].clone();
        ListOperation::do_operation(index, op_data, state_map, testnet)
    }
}

struct ChownOperationHandler;

impl OperationMethod for ChownOperationHandler {
    fn script_collect_ex(
        &self,
        index: usize,
        script: &mut DataScriptType,
        tx_data: &DataTransactionType,
        testnet: bool,
    ) {
        ChownOperation::script_collect_ex(index, script, tx_data, testnet)
    }

    fn validate(
        &self,
        script: &mut DataScriptType,
        tx_id: &str,
        daa_score: u64,
        testnet: bool,
    ) -> bool {
        ChownOperation::validate(script, tx_id, daa_score, testnet)
    }

    fn fee_least(&self, daa_score: u64) -> u64 {
        ChownOperation::fee_least(daa_score)
    }

    fn prepare_state_key(&self, script: &DataScriptType, state_map: &mut DataStateMapType) {
        ChownOperation::prepare_state_key(script, state_map)
    }

    fn do_operation(
        &self,
        index: usize,
        op_data: &mut DataOperationType,
        state_map: &mut DataStateMapType,
        testnet: bool,
    ) -> Result<()> {
        let script = op_data.op_script[index].clone();
        ChownOperation::do_operation(index, op_data, state_map, testnet)
    }
}

struct BlacklistOperationHandler;

impl OperationMethod for BlacklistOperationHandler {
    fn script_collect_ex(
        &self,
        index: usize,
        script: &mut DataScriptType,
        tx_data: &DataTransactionType,
        testnet: bool,
    ) {
        BlacklistOperation::script_collect_ex(index, script, tx_data, testnet)
    }

    fn validate(
        &self,
        script: &mut DataScriptType,
        tx_id: &str,
        daa_score: u64,
        testnet: bool,
    ) -> bool {
        BlacklistOperation::validate(script, tx_id, daa_score, testnet)
    }

    fn fee_least(&self, daa_score: u64) -> u64 {
        BlacklistOperation::fee_least(daa_score)
    }

    fn prepare_state_key(&self, script: &DataScriptType, state_map: &mut DataStateMapType) {
        BlacklistOperation::prepare_state_key(script, state_map)
    }

    fn do_operation(
        &self,
        index: usize,
        op_data: &mut DataOperationType,
        state_map: &mut DataStateMapType,
        testnet: bool,
    ) -> Result<()> {
        let script = op_data.op_script[index].clone();
        BlacklistOperation::do_operation(index, op_data, state_map, testnet)
    }
}
