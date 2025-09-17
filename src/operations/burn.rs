use crate::storage::types::*;
use crate::utils::script_builder::ScriptBuilder;
use anyhow::Result;

/// Burn operation implementation, corresponding to Go version OpMethodBurn
pub struct BurnOperation;

impl BurnOperation {
    /// Build burn script, ensuring consistency with Go version
    pub fn build_script(tick: &str, amount: &str) -> Result<String> {
        ScriptBuilder::build_burn_script(tick, amount)
    }

    /// Validate burn operation, corresponding to Go version Validate method
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
        if script.op != "burn" {
            return false;
        }

        // Validate token name
        if !Self::validate_tick(&mut script.tick.clone().unwrap_or_default()) {
            return false;
        }

        // Validate burn amount
        if !Self::validate_amount(&mut script.amt.clone().unwrap_or_default()) {
            return false;
        }

        // Set recipient address (burn operations usually send to zero address)
        if script.to.is_none() {
            script.to = Some(
                "kaspa:0000000000000000000000000000000000000000000000000000000000000000"
                    .to_string(),
            );
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

    /// Prepare burn operation state, corresponding to Go version PrepareState method
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

        // Check if balance is sufficient
        let balance_key = format!("balance:{}:{}", from, tick);
        if let Some(balance) = state_map.state_balance_map.get(&balance_key) {
            if let Some(balance_data) = balance {
                let current_balance = balance_data.balance.parse::<u64>().unwrap_or(0);
                let burn_amount = amount.parse::<u64>().unwrap_or(0);

                if current_balance < burn_amount {
                    return Err(anyhow::anyhow!("Insufficient balance for burn operation"));
                }
            }
        } else {
            return Err(anyhow::anyhow!(
                "Balance not found for address {} and token {}",
                from,
                tick
            ));
        }

        Ok(())
    }

    /// Execute burn operation, corresponding to Go version Execute method
    pub fn execute(script: &DataScriptType, state_map: &mut DataStateMapType) -> Result<()> {
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

        let burn_amount = amount.parse::<u64>().unwrap_or(0);
        if burn_amount == 0 {
            return Err(anyhow::anyhow!("Invalid burn amount"));
        }

        // Update token total supply
        let token_key = format!("token:{}", tick);
        if let Some(token) = state_map.state_token_map.get_mut(&token_key) {
            if let Some(token_data) = token {
                let current_minted = token_data.minted.parse::<u64>().unwrap_or(0);
                if current_minted >= burn_amount {
                    token_data.minted = (current_minted - burn_amount).to_string();
                } else {
                    return Err(anyhow::anyhow!("Cannot burn more than minted"));
                }
            }
        }

        // Update user balance
        let balance_key = format!("balance:{}:{}", from, tick);
        if let Some(balance) = state_map.state_balance_map.get_mut(&balance_key) {
            if let Some(balance_data) = balance {
                let current_balance = balance_data.balance.parse::<u64>().unwrap_or(0);
                if current_balance >= burn_amount {
                    balance_data.balance = (current_balance - burn_amount).to_string();
                } else {
                    return Err(anyhow::anyhow!("Insufficient balance for burn"));
                }
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
        600000000 // Burn operation fee
    }

    /// Prepare state keys
    pub fn prepare_state_key(script: &DataScriptType, state_map: &mut DataStateMapType) {
        if let Some(tick) = &script.tick {
            // Only insert None when key does not exist, avoid overwriting loaded state
            if !state_map.state_token_map.contains_key(tick) {
                state_map.state_token_map.insert(tick.clone(), None);
            }
        }
        if let Some(from) = &script.from {
            if let Some(tick) = &script.tick {
                let key = format!("balance:{}:{}", from, tick);
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
        let script = &op_data.op_script[index];

        // Check if token exists
        if let Some(tick) = &script.tick {
            let token_result = state_map.state_token_map.get(tick);
            if let Some(token_option) = token_result {
                if token_option.is_none() {
                    op_data.op_accept = -1;
                    op_data.op_error = "tick not found".to_string();
                    return Ok(());
                }
            } else {
                op_data.op_accept = -1;
                op_data.op_error = "tick not found".to_string();
                return Ok(());
            }
        }

        // Execute burn operation
        match Self::execute(script, state_map) {
            Ok(_) => {
                op_data.op_accept = 1;
                op_data.op_error = "".to_string();
            }
            Err(e) => {
                op_data.op_accept = -1;
                op_data.op_error = e.to_string();
            }
        }

        Ok(())
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
