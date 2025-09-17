use crate::storage::types::*;
use crate::utils::script_builder::ScriptBuilder;
use anyhow::Result;

/// Issue operation implementation, corresponding to Go version OpMethodIssue
pub struct IssueOperation;

impl IssueOperation {
    /// Build issue script, ensuring consistency with Go version
    pub fn build_script(
        tick: &str,
        name: &str,
        max_supply: u64,
        decimals: u8,
        description: &str,
    ) -> Result<String> {
        ScriptBuilder::build_issue_script(tick, name, max_supply, decimals, description)
    }

    /// Validate issue operation, corresponding to Go version Validate method
    pub fn validate(
        script: &mut DataScriptType,
        _tx_id: &str,
        _daa_score: u64,
        _testnet: bool,
    ) -> bool {
        // Validate required fields
        if script.from.is_none() || script.tick.is_none() || script.amt.is_none() {
            return false;
        }

        // Validate protocol
        if script.p != "KRC-20" {
            return false;
        }

        // Validate operation type
        if script.op != "issue" {
            return false;
        }

        // Validate token name
        if !Self::validate_tick(&mut script.tick.clone().unwrap_or_default()) {
            return false;
        }

        // Validate issue amount
        if !Self::validate_amount(&mut script.amt.clone().unwrap_or_default()) {
            return false;
        }

        // Set recipient address
        if script.to.is_none() {
            script.to = script.from.clone();
        }

        // Clear unnecessary fields
        script.max = None;
        script.lim = None;
        script.dec = None;
        script.pre = None;
        script.mod_type = String::new();
        script.name = None;
        script.utxo = None;
        script.price = None;
        script.ca = None;

        true
    }

    /// Prepare issue operation state, corresponding to Go version PrepareState method
    pub fn prepare_state(script: &DataScriptType, state_map: &mut DataStateMapType) -> Result<()> {
        let tick = script
            .tick
            .as_ref()
            .ok_or_else(|| anyhow::anyhow!("Missing tick"))?;
        let from = script
            .from
            .as_ref()
            .ok_or_else(|| anyhow::anyhow!("Missing from address"))?;
        let amount = script
            .amt
            .as_ref()
            .ok_or_else(|| anyhow::anyhow!("Missing amount"))?;

        // Check if token exists
        let token_key = format!("token:{}", tick);
        if !state_map.state_token_map.contains_key(&token_key) {
            return Err(anyhow::anyhow!("Token {} does not exist", tick));
        }

        // Check token owner permissions (determined by from field)
        if let Some(token) = state_map.state_token_map.get(&token_key) {
            if let Some(token_data) = token {
                if token_data.from != *from {
                    return Err(anyhow::anyhow!("Only token owner can issue tokens"));
                }

                // Check if exceeds maximum supply
                let current_minted = token_data.minted.parse::<u64>().unwrap_or(0);
                let issue_amount = amount.parse::<u64>().unwrap_or(0);
                let max_supply = token_data.max.parse::<u64>().unwrap_or(0);

                if max_supply > 0 && current_minted + issue_amount > max_supply {
                    return Err(anyhow::anyhow!("Issue amount exceeds maximum supply"));
                }
            }
        }

        Ok(())
    }

    /// Execute issue operation, corresponding to Go version Execute method
    pub fn execute(script: &DataScriptType, state_map: &mut DataStateMapType) -> Result<()> {
        let tick = script
            .tick
            .as_ref()
            .ok_or_else(|| anyhow::anyhow!("Missing tick"))?;
        let to = script
            .to
            .as_ref()
            .ok_or_else(|| anyhow::anyhow!("Missing to address"))?;
        let amount = script
            .amt
            .as_ref()
            .ok_or_else(|| anyhow::anyhow!("Missing amount"))?;

        let issue_amount = amount.parse::<u64>().unwrap_or(0);
        if issue_amount == 0 {
            return Err(anyhow::anyhow!("Invalid issue amount"));
        }

        // Update token total supply
        let token_key = format!("token:{}", tick);
        if let Some(token) = state_map.state_token_map.get_mut(&token_key) {
            if let Some(token_data) = token {
                let current_minted = token_data.minted.parse::<u64>().unwrap_or(0);
                token_data.minted = (current_minted + issue_amount).to_string();
            }
        }

        // Increase recipient balance
        let balance_key = format!("balance:{}:{}", to, tick);
        if let Some(balance) = state_map.state_balance_map.get_mut(&balance_key) {
            if let Some(balance_data) = balance {
                let current_balance = balance_data.balance.parse::<u64>().unwrap_or(0);
                balance_data.balance = (current_balance + issue_amount).to_string();
            } else {
                // Create new balance record
                let new_balance = StateBalanceType {
                    address: to.clone(),
                    tick: tick.clone(),
                    dec: 0,
                    balance: issue_amount.to_string(),
                    locked: "0".to_string(),
                    op_mod: 0,
                };
                state_map
                    .state_balance_map
                    .insert(balance_key.clone(), Some(new_balance));
            }
        } else {
            // Create new balance record
            let new_balance = StateBalanceType {
                address: to.clone(),
                tick: tick.clone(),
                dec: 0,
                balance: issue_amount.to_string(),
                locked: "0".to_string(),
                op_mod: 0,
            };
            state_map
                .state_balance_map
                .insert(balance_key, Some(new_balance));
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
        400000000 // Issue operation fee
    }

    /// Prepare state keys
    pub fn prepare_state_key(script: &DataScriptType, state_map: &mut DataStateMapType) {
        if let Some(tick) = &script.tick {
            state_map.state_token_map.insert(tick.clone(), None);
        }
        if let Some(to) = &script.to {
            if let Some(tick) = &script.tick {
                let key = format!("balance:{}:{}", to, tick);
                state_map.state_balance_map.insert(key, None);
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

    /// Validate amount
    fn validate_amount(amount: &mut String) -> bool {
        if amount.is_empty() {
            return false;
        }

        // Check if it's a valid number
        if let Ok(amount_num) = amount.parse::<u64>() {
            amount_num > 0
        } else {
            false
        }
    }
}
