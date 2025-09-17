use crate::operations::validate_tick;
use crate::storage::types::*;
use crate::utils::address::verify_address;
use crate::utils::script_builder::ScriptBuilder;
use anyhow::Result;

/// Mint operation implementation, corresponding to Go version's OpMethodMint
pub struct MintOperation;

impl MintOperation {
    /// Build mint script, ensuring consistency with Go version
    pub fn build_script(tick: &str, to_address: &str, amount: &str) -> Result<String> {
        ScriptBuilder::build_mint_script(tick, to_address, amount)
    }

    /// Get minimum fee, corresponding to Go version's FeeLeast
    pub fn fee_least(_daa_score: u64) -> u64 {
        // if daa_score ...
        100000000
    }

    /// Script collection extension, corresponding to Go version's ScriptCollectEx
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
        if script.from.as_ref().map(|s| s.is_empty()).unwrap_or(true)
            || script.p != "KRC-20"
            || !validate_tick(script.tick.as_mut().unwrap_or(&mut String::new()))
        {
            return false;
        }
        script.amt = Some("".to_string());
        if script.to.as_ref().map(|s| s.is_empty()).unwrap_or(true) {
            script.to = script.from.clone();
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

    /// Prepare state key, corresponding to Go version PrepareStateKey
    pub fn prepare_state_key(op_script: &DataScriptType, state_map: &mut DataStateMapType) {
        if let Some(tick) = &op_script.tick {
            // Only insert None when key does not exist, avoid overwriting loaded state
            if !state_map.state_token_map.contains_key(tick) {
                state_map.state_token_map.insert(tick.clone(), None);
            }
        }
        if let Some(to) = &op_script.to {
            if let Some(tick) = &op_script.tick {
                let key_balance = format!("{}_{}", to, tick);
                state_map.state_balance_map.insert(key_balance, None);
            }
        }
    }

    /// Execute minting operation, corresponding to Go version Do
    pub fn do_operation(
        index: usize,
        op_data: &mut DataOperationType,
        state_map: &mut DataStateMapType,
        testnet: bool,
    ) -> Result<()> {
        let op_script = &op_data.op_script[index];

        // Check if token exists
        if let Some(tick) = &op_script.tick {
            println!("    Debug: Checking token '{}'", tick);
            println!(
                "    Debug: state_map key count: {}",
                state_map.state_token_map.len()
            );
            for (key, value) in &state_map.state_token_map {
                println!(
                    "    Debug: Key: '{}', Value: {}",
                    key,
                    if value.is_some() { "Some" } else { "None" }
                );
            }

            let token_result = state_map.state_token_map.get(tick);
            println!("    Debug: get('{}') result: {:?}", tick, token_result);

            if let Some(token_option) = token_result {
                if token_option.is_none() {
                    println!("    Debug: Token '{}' not found (value is None)", tick);
                    op_data.op_accept = -1;
                    op_data.op_error = "tick not found".to_string();
                    return Ok(());
                }
            } else {
                println!("    Debug: Token '{}' not found (key does not exist)", tick);
                op_data.op_accept = -1;
                op_data.op_error = "tick not found".to_string();
                return Ok(());
            }

            // Check token mode
            if let Some(token) = state_map.state_token_map.get(tick).unwrap_or(&None) {
                // Allow minting when mod_type is "0" or empty string
                if !token.mod_type.is_empty() && token.mod_type != "0" {
                    op_data.op_accept = -1;
                    op_data.op_error = "mode invalid".to_string();
                    return Ok(());
                }
            }
        }

        // Check fee
        if op_data.fee == 0 {
            op_data.op_accept = -1;
            op_data.op_error = "fee unknown".to_string();
            return Ok(());
        }
        if op_data.fee < Self::fee_least(op_data.op_score) {
            op_data.op_accept = -1;
            op_data.op_error = "fee not enough".to_string();
            return Ok(());
        }

        // Validate address
        if !verify_address(op_script.to.as_ref().map_or("", |v| v), testnet) {
            op_data.op_accept = -1;
            op_data.op_error = "address invalid".to_string();
            return Ok(());
        }

        // Get state data
        let key_balance = format!(
            "{}_{}",
            op_script.to.as_ref().map_or("", |v| v),
            op_script.tick.as_ref().map_or("", |v| v)
        );
        let st_token = state_map
            .state_token_map
            .get(op_script.tick.as_ref().map_or("", |v| v))
            .unwrap_or(&None)
            .clone();
        let st_balance = state_map
            .state_balance_map
            .get(&key_balance)
            .unwrap_or(&None)
            .clone();

        // Calculate minting amount
        let token = st_token.as_ref().unwrap();
        let amt = token.lim.clone();
        let max_big: u128 = token.max.parse().unwrap_or(0);
        let minted_big: u128 = token.minted.parse().unwrap_or(0);
        let left_big = max_big.saturating_sub(minted_big);

        if left_big == 0 {
            op_data.op_accept = -1;
            op_data.op_error = "mint finished".to_string();
            return Ok(());
        }

        let lim_big: u128 = amt.parse().unwrap_or(0);
        let final_amt = if lim_big > left_big {
            left_big
        } else {
            lim_big
        };
        let _final_amt_str = final_amt.to_string();

        let new_minted = minted_big + final_amt;
        let new_minted_str = new_minted.to_string();

        // Set pre-operation state
        op_data.st_before = Vec::new();
        op_data.st_before = crate::operations::append_st_line_token(
            &mut op_data.st_before,
            op_script.tick.as_ref().map_or("", |v| v),
            st_token.as_ref(),
            false,
            false,
        );
        op_data.st_before = crate::operations::append_st_line_balance(
            &mut op_data.st_before,
            &key_balance,
            st_balance.as_ref(),
            false,
        );

        // Update Token state
        let mut new_st_token = st_token.unwrap().clone();
        new_st_token.minted = new_minted_str.clone();
        new_st_token.op_mod = op_data.op_score;
        new_st_token.mts_mod = op_data.mts_add;
        state_map.state_token_map.insert(
            op_script.tick.as_ref().map_or("", |v| v).to_string(),
            Some(new_st_token.clone()),
        );

        // Update or create balance state
        let mut new_st_balance = if let Some(ref balance) = st_balance {
            balance.clone()
        } else {
            StateBalanceType {
                address: op_script.to.as_ref().map_or("", |v| v).to_string(),
                tick: op_script.tick.as_ref().map_or("", |v| v).to_string(),
                dec: new_st_token.dec,
                balance: "0".to_string(),
                locked: "0".to_string(),
                op_mod: op_data.op_score,
            }
        };

        let balance_big: u128 = new_st_balance.balance.parse().unwrap_or(0);
        let new_balance = balance_big + final_amt;
        new_st_balance.balance = new_balance.to_string();
        new_st_balance.op_mod = op_data.op_score;

        state_map
            .state_balance_map
            .insert(key_balance.clone(), Some(new_st_balance.clone()));

        // Update statistics
        let n_tick_affc = if st_balance.is_none() { 1 } else { 0 };
        op_data.ss_info.as_mut().unwrap().tick_affc = crate::operations::append_ss_info_tick_affc(
            &mut op_data.ss_info.as_mut().unwrap().tick_affc,
            op_script.tick.as_ref().map_or("", |v| v),
            n_tick_affc,
        );

        let locked_big: u128 = new_st_balance.locked.parse().unwrap_or(0);
        let balance_total = new_balance + locked_big;
        op_data.ss_info.as_mut().unwrap().address_affc =
            crate::operations::append_ss_info_address_affc(
                &mut op_data.ss_info.as_mut().unwrap().address_affc,
                &key_balance,
                &balance_total.to_string(),
            );

        // Set post-operation state
        op_data.st_after = Vec::new();
        op_data.st_after = crate::operations::append_st_line_token(
            &mut op_data.st_after,
            op_script.tick.as_ref().map_or("", |v| v),
            Some(&new_st_token),
            false,
            true,
        );
        op_data.st_after = crate::operations::append_st_line_balance(
            &mut op_data.st_after,
            &key_balance,
            Some(&new_st_balance),
            true,
        );

        op_data.op_accept = 1;
        Ok(())
    }
}
