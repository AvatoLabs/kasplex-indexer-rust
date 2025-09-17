use crate::storage::types::*;
use crate::utils::script_builder::ScriptBuilder;
use anyhow::Result;

/// Ownership transfer operation implementation, corresponding to Go version OpMethodChown
pub struct ChownOperation;

impl ChownOperation {
    /// Build ownership transfer script, ensuring consistency with Go version
    pub fn build_script(tick: &str, to_address: &str) -> Result<String> {
        ScriptBuilder::build_chown_script(tick, to_address)
    }

    /// Validate ownership transfer operation, corresponding to Go version Validate method
    pub fn validate(
        script: &mut DataScriptType,
        _tx_id: &str,
        _daa_score: u64,
        _testnet: bool,
    ) -> bool {
        // Validate required fields
        if script.from.is_none() || script.to.is_none() || script.tick.is_none() {
            return false;
        }

        // Validate protocol
        if script.p != "KRC-20" {
            return false;
        }

        // Validate operation type
        if script.op != "chown" {
            return false;
        }

        // Validate token name
        if !Self::validate_tick(&mut script.tick.clone().unwrap_or_default()) {
            return false;
        }

        // Validate address format
        if !Self::validate_address(&script.from.clone().unwrap_or_default()) {
            return false;
        }

        if !Self::validate_address(&script.to.clone().unwrap_or_default()) {
            return false;
        }

        // Clear unnecessary fields
        script.max = None;
        script.lim = None;
        script.dec = None;
        script.pre = None;
        script.mod_type = String::new();
        script.name = None;
        script.amt = None;
        script.utxo = None;
        script.price = None;
        script.ca = None;

        true
    }

    /// Prepare ownership transfer operation state, corresponding to Go version PrepareState method
    pub fn prepare_state(script: &DataScriptType, state_map: &mut DataStateMapType) -> Result<()> {
        let tick = script
            .tick
            .as_ref()
            .ok_or_else(|| anyhow::anyhow!("Missing tick"))?;
        let from = script
            .from
            .as_ref()
            .ok_or_else(|| anyhow::anyhow!("Missing from address"))?;

        // Check if token exists
        let token_key = format!("token:{}", tick);
        if !state_map.state_token_map.contains_key(&token_key) {
            return Err(anyhow::anyhow!("Token {} does not exist", tick));
        }

        // Check current owner permissions (determined by from field)
        if let Some(token) = state_map.state_token_map.get(&token_key) {
            if let Some(token_data) = token {
                if token_data.from != *from {
                    return Err(anyhow::anyhow!("Only token owner can transfer ownership"));
                }
            }
        }

        Ok(())
    }

    /// Execute ownership transfer operation, corresponding to Go version Execute method
    pub fn execute(script: &DataScriptType, state_map: &mut DataStateMapType) -> Result<()> {
        let tick = script
            .tick
            .as_ref()
            .ok_or_else(|| anyhow::anyhow!("Missing tick"))?;
        let to = script
            .to
            .as_ref()
            .ok_or_else(|| anyhow::anyhow!("Missing to address"))?;

        // Update token owner
        let token_key = format!("token:{}", tick);
        if let Some(token) = state_map.state_token_map.get_mut(&token_key) {
            if let Some(token_data) = token {
                token_data.from = to.clone();
            }
        }

        Ok(())
    }

    /// Script collection extension
    pub fn script_collect_ex(
        _index: usize,
        _script: &mut DataScriptType,
        _tx_data: &DataTransactionType,
        __testnet: bool,
    ) {
        // Temporarily empty implementation
    }

    /// Get operation fee
    pub fn fee_least(__daa_score: u64) -> u64 {
        800000000 // Ownership transfer operation fee
    }

    /// Prepare state keys
    pub fn prepare_state_key(script: &DataScriptType, state_map: &mut DataStateMapType) {
        if let Some(tick) = &script.tick {
            state_map.state_token_map.insert(tick.clone(), None);
        }
    }

    /// Execute operation
    pub fn do_operation(
        index: usize,
        op_data: &mut DataOperationType,
        state_map: &mut DataStateMapType,
        _testnet: bool,
    ) -> Result<()> {
        let script = op_data.op_script[index].clone();
        Self::execute(&script, state_map)
    }

    /// Validate token name
    fn validate_tick(tick: &mut String) -> bool {
        if tick.is_empty() || tick.len() > 10 {
            return false;
        }

        // Check if only contains letters and numbers
        tick.chars().all(|c| c.is_alphanumeric())
    }

    /// Validate address format
    fn validate_address(address: &str) -> bool {
        if address.is_empty() {
            return false;
        }

        // Check address format (simplified validation)
        address.starts_with("kaspa:") || address.starts_with("kaspatest:")
    }
}
