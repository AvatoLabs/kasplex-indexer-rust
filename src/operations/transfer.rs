use crate::operations::{validate_amount, validate_tick_tx_id, validate_tx_id};
use crate::storage::types::*;
use crate::utils::address::verify_address;
use anyhow::Result;

/// Transfer operation implementation, corresponding to Go version OpMethodTransfer
pub struct TransferOperation;

impl TransferOperation {
    /// Get minimum fee, corresponding to Go version FeeLeast
    pub fn fee_least(_daa_score: u64) -> u64 {
        0
    }

    /// Script collection extension, corresponding to Go version ScriptCollectEx
    pub fn script_collect_ex(
        _index: usize,
        _script: &mut DataScriptType,
        _tx_data: &DataTransactionType,
        _testnet: bool,
    ) {
        // Temporarily empty implementation
    }

    /// Validate script, corresponding to Go version Validate
    pub fn validate(
        script: &mut DataScriptType,
        _tx_id: &str,
        _daa_score: u64,
        _testnet: bool,
    ) -> bool {
        if let Some(ca) = &mut script.ca {
            if validate_tx_id(ca) {
                script.tick = script.ca.clone();
            }
        }
        if script.from.as_ref().map(|s| s.is_empty()).unwrap_or(true)
            || script.to.as_ref().map(|s| s.is_empty()).unwrap_or(true)
            || script.p != "KRC-20"
            || !validate_tick_tx_id(&mut script.tick.as_mut().unwrap_or(&mut String::new()))
            || !validate_amount(&mut script.amt.as_mut().unwrap_or(&mut String::new()))
        {
            return false;
        }
        script.max = Some("".to_string());
        script.lim = Some("".to_string());
        script.pre = Some("".to_string());
        script.dec = Some("".to_string());
        script.utxo = Some("".to_string());
        script.price = Some("".to_string());
        script.mod_type = "".to_string();
        script.name = Some("".to_string());
        script.ca = Some("".to_string());
        true
    }

    /// Prepare state keys, corresponding to Go version PrepareStateKey
    pub fn prepare_state_key(op_script: &DataScriptType, state_map: &mut DataStateMapType) {
        if let Some(tick) = &op_script.tick {
            // Only insert None when key does not exist, avoid overwriting loaded state
            if !state_map.state_token_map.contains_key(tick) {
                state_map.state_token_map.insert(tick.clone(), None);
            }
        }
        if let Some(from) = &op_script.from {
            if let Some(tick) = &op_script.tick {
                let key_balance_from = format!("{}_{}", from, tick);
                state_map.state_balance_map.insert(key_balance_from, None);
            }
        }
        if let Some(to) = &op_script.to {
            if let Some(tick) = &op_script.tick {
                let key_balance_to = format!("{}_{}", to, tick);
                state_map.state_balance_map.insert(key_balance_to, None);
            }
        }
        if let Some(tick) = &op_script.tick {
            if let Some(from) = &op_script.from {
                let key_blacklist = format!("{}_{}", tick, from);
                state_map.state_blacklist_map.insert(key_blacklist, None);
            }
        }
    }

    /// Execute transfer operation, corresponding to Go version Do
    pub fn do_operation(
        index: usize,
        op_data: &mut DataOperationType,
        state_map: &mut DataStateMapType,
        testnet: bool,
    ) -> Result<()> {
        let op_script = &mut op_data.op_script[index];

        // Check if token exists
        if let Some(tick) = &op_script.tick {
            let token_result = state_map.state_token_map.get(tick);
            if let Some(token_option) = token_result {
                if token_option.is_none() {
                    op_data.op_accept = -1;
                    op_data.op_error = format!("Token '{}' not found", tick);
                    return Ok(());
                }
            } else {
                op_data.op_accept = -1;
                op_data.op_error = format!("Token '{}' not found", tick);
                return Ok(());
            }
        }

        // Check blacklist
        if let Some(tick) = &op_script.tick {
            if let Some(from) = &op_script.from {
                let key_blacklist = format!("{}_{}", tick, from);
                if state_map
                    .state_blacklist_map
                    .get(&key_blacklist)
                    .unwrap_or(&None)
                    .is_some()
                {
                    op_data.op_accept = -1;
                    op_data.op_error = format!("Address '{}' is blacklisted for token '{}'", from, tick);
                    return Ok(());
                }
            }
        }

        // Validate address
        if op_script.from == op_script.to {
            op_data.op_accept = -1;
            op_data.op_error = "Cannot transfer to the same address".to_string();
            return Ok(());
        }

        if let Some(to) = &op_script.to {
            if !verify_address(to, testnet) {
                op_data.op_accept = -1;
                op_data.op_error = format!("Invalid address: {}", to);
                return Ok(());
            }
        }

        // Get state data
        let tick = op_script.tick.as_ref().unwrap();
        let from = op_script.from.as_ref().unwrap();
        let to = op_script.to.as_ref().unwrap();

        let key_balance_from = format!("{}_{}", from, tick);
        let key_balance_to = format!("{}_{}", to, tick);
        let st_balance_from = state_map
            .state_balance_map
            .get(&key_balance_from)
            .unwrap_or(&None)
            .clone();
        let st_balance_to = state_map
            .state_balance_map
            .get(&key_balance_to)
            .unwrap_or(&None)
            .clone();

        let mut n_tick_affc = 0i64;
        let token = state_map
            .state_token_map
            .get(tick)
            .unwrap_or(&None)
            .as_ref()
            .unwrap();
        op_script.name = Some(token.name.clone());

        // Check sender balance
        if st_balance_from.is_none() {
            op_data.op_accept = -1;
            op_data.op_error = format!("Insufficient balance for token '{}'", tick);
            return Ok(());
        }

        let balance_from = st_balance_from.as_ref().unwrap();
        let balance_big: u128 = balance_from.balance.parse().unwrap_or(0);
        let amt_big: u128 = op_script
            .amt
            .as_ref()
            .map_or("0", |v| v)
            .parse()
            .unwrap_or(0);

        if amt_big > balance_big {
            op_data.op_accept = -1;
            op_data.op_error = format!("Insufficient balance: required {}, available {}", amt_big, balance_big);
            return Ok(());
        } else if amt_big == balance_big && balance_from.locked == "0" {
            n_tick_affc = -1;
        }

        // Set pre-operation state
        op_data.st_before = Vec::new();
        op_data.st_before = crate::operations::append_st_line_balance(
            &mut op_data.st_before,
            &key_balance_from,
            st_balance_from.as_ref(),
            false,
        );
        op_data.st_before = crate::operations::append_st_line_balance(
            &mut op_data.st_before,
            &key_balance_to,
            st_balance_to.as_ref(),
            false,
        );

        // Update sender balance
        let new_balance_from = balance_big.saturating_sub(amt_big);
        let mut new_st_balance_from = balance_from.clone();
        new_st_balance_from.balance = new_balance_from.to_string();
        new_st_balance_from.op_mod = op_data.op_score;

        let locked_big: u128 = new_st_balance_from.locked.parse().unwrap_or(0);
        let balance_from_total = new_balance_from + locked_big;

        // Update or create recipient balance
        let mut new_st_balance_to = if let Some(balance_to) = st_balance_to {
            balance_to.clone()
        } else {
            StateBalanceType {
                address: op_script.to.clone().unwrap_or_default(),
                tick: op_script.tick.clone().unwrap_or_default(),
                dec: balance_from.dec,
                balance: "0".to_string(),
                locked: "0".to_string(),
                op_mod: op_data.op_score,
            }
        };

        let balance_to_big: u128 = new_st_balance_to.balance.parse().unwrap_or(0);
        let new_balance_to = balance_to_big + amt_big;
        new_st_balance_to.balance = new_balance_to.to_string();
        new_st_balance_to.op_mod = op_data.op_score;

        let locked_to_big: u128 = new_st_balance_to.locked.parse().unwrap_or(0);
        let balance_to_total = new_balance_to + locked_to_big;

        // Update state mapping
        state_map
            .state_balance_map
            .insert(key_balance_from.clone(), Some(new_st_balance_from.clone()));
        state_map
            .state_balance_map
            .insert(key_balance_to.clone(), Some(new_st_balance_to.clone()));

        // Update statistics
        op_data.ss_info.as_mut().unwrap().tick_affc = crate::operations::append_ss_info_tick_affc(
            &mut op_data.ss_info.as_mut().unwrap().tick_affc,
            op_script.tick.as_ref().map_or("", |v| v),
            n_tick_affc,
        );
        op_data.ss_info.as_mut().unwrap().address_affc =
            crate::operations::append_ss_info_address_affc(
                &mut op_data.ss_info.as_mut().unwrap().address_affc,
                &key_balance_from,
                &balance_from_total.to_string(),
            );
        op_data.ss_info.as_mut().unwrap().address_affc =
            crate::operations::append_ss_info_address_affc(
                &mut op_data.ss_info.as_mut().unwrap().address_affc,
                &key_balance_to,
                &balance_to_total.to_string(),
            );

        // Set post-operation state
        op_data.st_after = Vec::new();
        op_data.st_after = crate::operations::append_st_line_balance(
            &mut op_data.st_after,
            &key_balance_from,
            Some(&new_st_balance_from),
            true,
        );
        op_data.st_after = crate::operations::append_st_line_balance(
            &mut op_data.st_after,
            &key_balance_to,
            Some(&new_st_balance_to),
            true,
        );

        // If sender balance is 0 and no lock, delete balance record
        if new_st_balance_from.balance == "0" && new_st_balance_from.locked == "0" {
            state_map.state_balance_map.insert(key_balance_from, None);
        }

        op_data.op_accept = 1;
        Ok(())
    }
}
