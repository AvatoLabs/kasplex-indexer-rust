use crate::storage::types::*;
use crate::utils::script_builder::ScriptBuilder;
use anyhow::Result;

/// Send operation implementation, corresponding to Go version's OpMethodSend
pub struct SendOperation;

impl SendOperation {
    /// Build transfer script, ensuring consistency with Go version
    pub fn build_script(tick: &str, to_address: &str, amount: &str) -> Result<String> {
        ScriptBuilder::build_send_script(tick, to_address, amount)
    }

    /// Validate send operation, corresponding to Go version's Validate method
    pub fn validate(
        script: &mut DataScriptType,
        _tx_id: &str,
        _daa_score: u64,
        _testnet: bool,
    ) -> bool {
        // Validate required fields
        if script.from.is_none()
            || script.to.is_none()
            || script.tick.is_none()
            || script.amt.is_none()
        {
            return false;
        }

        // Validate protocol
        if script.p != "KRC-20" {
            return false;
        }

        // Validate operation type
        if script.op != "send" {
            return false;
        }

        // Validate token name
        if !Self::validate_tick(&mut script.tick.clone().unwrap_or_default()) {
            return false;
        }

        // Validate send amount
        if !Self::validate_amount(&mut script.amt.clone().unwrap_or_default()) {
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
        script.utxo = None;
        script.price = None;
        script.ca = None;

        true
    }

    /// Prepare send operation state, corresponding to Go version PrepareState method
    pub fn prepare_state(script: &DataScriptType, state_map: &mut DataStateMapType) -> Result<()> {
        let tick = script
            .tick
            .as_ref()
            .ok_or_else(|| anyhow::anyhow!("Missing tick"))?;
        let from = script
            .from
            .as_ref()
            .ok_or_else(|| anyhow::anyhow!("Missing from address"))?;
        let to = script
            .to
            .as_ref()
            .ok_or_else(|| anyhow::anyhow!("Missing to address"))?;
        let amount = script
            .amt
            .as_ref()
            .ok_or_else(|| anyhow::anyhow!("Missing amount"))?;

        // Check if token exists
        let token_key = format!("token:{}", tick);
        if !state_map.state_token_map.contains_key(&token_key) {
            return Err(anyhow::anyhow!("Token {} does not exist", tick));
        }

        // Check if sender balance is sufficient
        let from_balance_key = format!("balance:{}:{}", from, tick);
        if let Some(balance) = state_map.state_balance_map.get(&from_balance_key) {
            if let Some(balance_data) = balance {
                let current_balance = balance_data.balance.parse::<u64>().unwrap_or(0);
                let send_amount = amount.parse::<u64>().unwrap_or(0);

                if current_balance < send_amount {
                    return Err(anyhow::anyhow!("Insufficient balance for send operation"));
                }
            }
        } else {
            return Err(anyhow::anyhow!(
                "Balance not found for address {} and token {}",
                from,
                tick
            ));
        }

        // Check if receiver is in blacklist
        let blacklist_key = format!("blacklist:{}:{}", to, tick);
        if let Some(blacklist) = state_map.state_blacklist_map.get(&blacklist_key) {
            if let Some(blacklist_data) = blacklist {
                // Check if in blacklist (judged by reason field)
                if !blacklist_data.tick.is_empty() {
                    return Err(anyhow::anyhow!(
                        "Recipient {} is blacklisted for token {}",
                        to,
                        tick
                    ));
                }
            }
        }

        Ok(())
    }

    /// Execute send operation, corresponding to Go version Execute method
    pub fn execute(script: &DataScriptType, state_map: &mut DataStateMapType) -> Result<()> {
        let tick = script
            .tick
            .as_ref()
            .ok_or_else(|| anyhow::anyhow!("Missing tick"))?;
        let from = script
            .from
            .as_ref()
            .ok_or_else(|| anyhow::anyhow!("Missing from address"))?;
        let to = script
            .to
            .as_ref()
            .ok_or_else(|| anyhow::anyhow!("Missing to address"))?;
        let amount = script
            .amt
            .as_ref()
            .ok_or_else(|| anyhow::anyhow!("Missing amount"))?;

        let send_amount = amount.parse::<u64>().unwrap_or(0);
        if send_amount == 0 {
            return Err(anyhow::anyhow!("Invalid send amount"));
        }

        // Decrease sender balance
        let from_balance_key = format!("balance:{}:{}", from, tick);
        if let Some(balance) = state_map.state_balance_map.get_mut(&from_balance_key) {
            if let Some(balance_data) = balance {
                let current_balance = balance_data.balance.parse::<u64>().unwrap_or(0);
                if current_balance >= send_amount {
                    balance_data.balance = (current_balance - send_amount).to_string();
                } else {
                    return Err(anyhow::anyhow!("Insufficient balance for send"));
                }
            }
        }

        // Increase receiver balance
        let to_balance_key = format!("balance:{}:{}", to, tick);
        if let Some(balance) = state_map.state_balance_map.get_mut(&to_balance_key) {
            if let Some(balance_data) = balance {
                let current_balance = balance_data.balance.parse::<u64>().unwrap_or(0);
                balance_data.balance = (current_balance + send_amount).to_string();
            } else {
                // Create new balance record
                let new_balance = StateBalanceType {
                    address: to.clone(),
                    tick: tick.clone(),
                    dec: 0,
                    balance: send_amount.to_string(),
                    locked: "0".to_string(),
                    op_mod: 0,
                };
                state_map
                    .state_balance_map
                    .insert(to_balance_key.clone(), Some(new_balance));
            }
        } else {
            // Create new balance record
            let new_balance = StateBalanceType {
                address: to.clone(),
                tick: tick.clone(),
                dec: 0,
                balance: send_amount.to_string(),
                locked: "0".to_string(),
                op_mod: 0,
            };
            state_map
                .state_balance_map
                .insert(to_balance_key, Some(new_balance));
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

        // Check if valid number
        if let Ok(amount_num) = amount.parse::<u64>() {
            amount_num > 0
        } else {
            false
        }
    }

    /// Validate address format
    fn validate_address(address: &str) -> bool {
        if address.is_empty() {
            return false;
        }

        // Check address format (simplified validation)
        address.starts_with("kaspa:") || address.starts_with("kaspatest:")
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
        200000000 // Send operation fee
    }

    /// Prepare state key
    pub fn prepare_state_key(script: &DataScriptType, state_map: &mut DataStateMapType) {
        if let Some(tick) = &script.tick {
            state_map.state_token_map.insert(tick.clone(), None);
        }
        if let Some(from) = &script.from {
            if let Some(tick) = &script.tick {
                let key = format!("balance:{}:{}", from, tick);
                state_map.state_balance_map.insert(key, None);
            }
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
}
