use crate::storage::types::*;
use crate::utils::script_builder::ScriptBuilder;
use anyhow::Result;

/// List operation implementation, corresponding to Go version OpMethodList
pub struct ListOperation;

impl ListOperation {
    /// Build list script, ensuring consistency with Go version
    pub fn build_script(tick: &str, list: &str) -> Result<String> {
        ScriptBuilder::build_list_script(tick, list)
    }

    /// Validate list operation, corresponding to Go version Validate method
    pub fn validate(
        script: &mut DataScriptType,
        _tx_id: &str,
        _daa_score: u64,
        _testnet: bool,
    ) -> bool {
        // Validate required fields
        if script.from.is_none()
            || script.tick.is_none()
            || script.amt.is_none()
            || script.price.is_none()
        {
            return false;
        }

        // Validate protocol
        if script.p != "KRC-20" {
            return false;
        }

        // Validate operation type
        if script.op != "list" {
            return false;
        }

        // Validate token name
        if !Self::validate_tick(&mut script.tick.clone().unwrap_or_default()) {
            return false;
        }

        // Validate list amount
        if !Self::validate_amount(&mut script.amt.clone().unwrap_or_default()) {
            return false;
        }

        // Validate price
        if !Self::validate_price(&mut script.price.clone().unwrap_or_default()) {
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
        script.ca = None;

        true
    }

    /// Prepare list operation state, corresponding to Go version PrepareState method
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

        // Check if lister balance is sufficient
        let balance_key = format!("balance:{}:{}", from, tick);
        if let Some(balance) = state_map.state_balance_map.get(&balance_key) {
            if let Some(balance_data) = balance {
                let current_balance = balance_data.balance.parse::<u64>().unwrap_or(0);
                let list_amount = amount.parse::<u64>().unwrap_or(0);

                if current_balance < list_amount {
                    return Err(anyhow::anyhow!("Insufficient balance for list operation"));
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

    /// Execute list operation, corresponding to Go version Execute method
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
        let price = script
            .price
            .as_ref()
            .ok_or_else(|| anyhow::anyhow!("Missing price"))?;

        let list_amount = amount.parse::<u64>().unwrap_or(0);
        let list_price = price.parse::<f64>().unwrap_or(0.0);

        if list_amount == 0 || list_price <= 0.0 {
            return Err(anyhow::anyhow!("Invalid list amount or price"));
        }

        // Reduce lister balance
        let balance_key = format!("balance:{}:{}", from, tick);
        if let Some(balance) = state_map.state_balance_map.get_mut(&balance_key) {
            if let Some(balance_data) = balance {
                let current_balance = balance_data.balance.parse::<u64>().unwrap_or(0);
                if current_balance >= list_amount {
                    balance_data.balance = (current_balance - list_amount).to_string();
                    balance_data.locked =
                        (balance_data.locked.parse::<u64>().unwrap_or(0) + list_amount).to_string();
                } else {
                    return Err(anyhow::anyhow!("Insufficient balance for list"));
                }
            }
        }

        // Create or update market listing
        let market_key = format!("market:{}:{}", from, tick);
        let market_data = StateMarketType {
            tick: tick.clone(),
            t_addr: from.clone(),
            u_tx_id: "".to_string(),
            u_addr: "".to_string(),
            u_amt: list_amount.to_string(),
            u_script: "".to_string(),
            t_amt: "0".to_string(),
            op_add: 0,
        };
        state_map
            .state_market_map
            .insert(market_key, Some(market_data));

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
        100000000 // List operation fee
    }

    /// Prepare state keys
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
        if let Some(from) = &script.from {
            if let Some(tick) = &script.tick {
                let key = format!("market:{}:{}", from, tick);
                state_map.state_market_map.insert(key, None);
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

    /// Validate price
    fn validate_price(price: &mut String) -> bool {
        if price.is_empty() {
            return false;
        }

        // Check if it's a valid floating point number
        if let Ok(price_num) = price.parse::<f64>() {
            price_num > 0.0
        } else {
            false
        }
    }
}
