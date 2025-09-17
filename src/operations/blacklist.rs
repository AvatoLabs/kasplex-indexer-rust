use crate::storage::types::*;
use crate::utils::script_builder::ScriptBuilder;
use anyhow::Result;

/// Blacklist operation implementation, corresponding to Go version OpMethodBlacklist
pub struct BlacklistOperation;

impl BlacklistOperation {
    /// Build blacklist script, ensuring consistency with Go version
    pub fn build_script(tick: &str, blacklist: &str) -> Result<String> {
        ScriptBuilder::build_blacklist_script(tick, blacklist)
    }

    /// Validate blacklist operation, corresponding to Go version Validate method
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
        if script.op != "blacklist" {
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

    /// Prepare blacklist operation state, corresponding to Go version PrepareState method
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

        // Check operator permissions (only token owner can manage blacklist, determined by from field)
        if let Some(token) = state_map.state_token_map.get(&token_key) {
            if let Some(token_data) = token {
                if token_data.from != *from {
                    return Err(anyhow::anyhow!("Only token owner can manage blacklist"));
                }
            }
        }

        Ok(())
    }

    /// Execute blacklist operation, corresponding to Go version Execute method
    pub fn execute(script: &DataScriptType, state_map: &mut DataStateMapType) -> Result<()> {
        let tick = script
            .tick
            .as_ref()
            .ok_or_else(|| anyhow::anyhow!("Missing tick"))?;
        let to = script
            .to
            .as_ref()
            .ok_or_else(|| anyhow::anyhow!("Missing to address"))?;

        // Create or update blacklist record
        let blacklist_key = format!("blacklist:{}:{}", to, tick);
        let blacklist_data = StateBlacklistType {
            tick: tick.clone(),
            address: to.clone(),
            op_add: 0,
        };
        state_map
            .state_blacklist_map
            .insert(blacklist_key, Some(blacklist_data));

        Ok(())
    }

    /// Script collection extension
    pub fn script_collect_ex(
        _index: usize,
        _script: &mut DataScriptType,
        _tx_data: &DataTransactionType,
        _testnet: bool,
    ) {
        // Temporarily empty implementation
    }

    /// Get operation fee
    pub fn fee_least(_daa_score: u64) -> u64 {
        600000000 // Blacklist operation fee
    }

    /// Prepare state keys
    pub fn prepare_state_key(script: &DataScriptType, state_map: &mut DataStateMapType) {
        if let Some(tick) = &script.tick {
            state_map.state_token_map.insert(tick.clone(), None);
        }
        if let Some(to) = &script.to {
            if let Some(tick) = &script.tick {
                let key = format!("blacklist:{}:{}", to, tick);
                state_map.state_blacklist_map.insert(key, None);
            }
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
