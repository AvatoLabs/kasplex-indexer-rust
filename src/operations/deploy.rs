use crate::storage::types::*;
use crate::utils::address::verify_address;
use crate::utils::script_builder::ScriptBuilder;
use anyhow::Result;

/// Deploy operation implementation, corresponding to Go version OpMethodDeploy
pub struct DeployOperation;

impl DeployOperation {
    /// Build deploy script, ensuring consistency with Go version
    pub fn build_script(
        tick: &str,
        name: &str,
        max_supply: u64,
        decimals: u8,
        description: &str,
    ) -> Result<String> {
        ScriptBuilder::build_deploy_script(tick, name, max_supply, decimals, description)
    }

    /// Validate deploy operation, corresponding to Go version Validate method
    pub fn validate(
        script: &mut DataScriptType,
        tx_id: &str,
        daa_score: u64,
        testnet: bool,
    ) -> bool {
        if (testnet || daa_score >= 110165000) && script.mod_type == "issue" {
            // Issue mode validation
            if script.from.is_none()
                || script.p != "KRC-20"
                || !Self::validate_tick(&mut script.name.clone().unwrap_or_default())
                || !Self::validate_dec(&mut script.dec.clone().unwrap_or_default(), "8")
            {
                return false;
            }
            if script.max.as_ref().map(|s| s != "0").unwrap_or(false)
                && !Self::validate_amount(&mut script.max.clone().unwrap_or_default())
            {
                return false;
            }
            script.tick = Some(tx_id.to_string());
            // Do not override lim field, keep original value
        } else {
            // Mint mode validation
            if script.from.is_none()
                || script.p != "KRC-20"
                || !Self::validate_tick(&mut script.tick.clone().unwrap_or_default())
                || !Self::validate_amount(&mut script.max.clone().unwrap_or_default())
                || !Self::validate_dec(&mut script.dec.clone().unwrap_or_default(), "8")
            {
                return false;
            }

            // Validate lim field
            let mut lim_value = script.lim.clone().unwrap_or_default();
            if !Self::validate_amount(&mut lim_value) {
                return false;
            }
            script.lim = Some(lim_value);
            script.mod_type = "".to_string();
            script.name = None;
        }

        if !Self::validate_amount(&mut script.pre.clone().unwrap_or_default()) {
            script.pre = Some("0".to_string());
        }

        if script.to.is_none() {
            script.to = script.from.clone();
        }

        // Clear unnecessary fields, but keep op field unchanged
        script.amt = None;
        script.utxo = None;
        script.price = None;
        script.ca = None;

        true
    }

    /// Prepare state keys, corresponding to Go version PrepareStateKey method
    pub fn prepare_state_key(script: &DataScriptType, state_map: &mut DataStateMapType) {
        if let Some(tick) = &script.tick {
            // Keep consistent with Go version: use tick directly as key
            if !state_map.state_token_map.contains_key(tick) {
                state_map.state_token_map.insert(tick.clone(), None);
            }

            if script.pre.as_ref().map(|s| s != "0").unwrap_or(false) {
                if let Some(to) = &script.to {
                    // Keep consistent with Go version: use "to_tick" format
                    let balance_key = format!("{}_{}", to, tick);
                    if !state_map.state_balance_map.contains_key(&balance_key) {
                        state_map.state_balance_map.insert(balance_key, None);
                    }
                }
            }
        }
    }

    /// Execute deploy operation, corresponding to Go version Do method
    pub fn do_operation(
        script: &DataScriptType,
        op_data: &mut DataOperationType,
        state_map: &mut DataStateMapType,
        testnet: bool,
    ) -> Result<()> {
        // Check if token already exists
        if let Some(tick) = &script.tick {
            if state_map
                .state_token_map
                .get(tick)
                .map(|t| t.is_some())
                .unwrap_or(false)
            {
                op_data.op_accept = -1;
                op_data.op_error = "tick existed".to_string();
                return Ok(());
            }

            // Check if it is an ignored token
            if Self::is_tick_ignored(tick) {
                op_data.op_accept = -1;
                op_data.op_error = "tick ignored".to_string();
                return Ok(());
            }

            // Check if it is a reserved token
            if let Some(reserved_addr) = Self::get_tick_reserved(tick) {
                if script
                    .from
                    .as_ref()
                    .map(|s| s != &reserved_addr)
                    .unwrap_or(true)
                {
                    op_data.op_accept = -1;
                    op_data.op_error = "tick reserved".to_string();
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
        if script.pre.as_ref().map(|s| s != "0").unwrap_or(false) {
            if let Some(to) = &script.to {
                if !verify_address(to, testnet) {
                    op_data.op_accept = -1;
                    op_data.op_error = "address invalid".to_string();
                    return Ok(());
                }
            }
        }

        // Create token state
        if let (Some(tick), Some(from), Some(to)) = (&script.tick, &script.from, &script.to) {
            let balance_key = format!("{}_{}", to, tick);

            let dec = script
                .dec
                .clone()
                .unwrap_or_default()
                .parse::<i32>()
                .unwrap_or(8);
            let max = script.max.clone().unwrap_or_default();
            let lim = script.lim.clone().unwrap_or_default();
            let pre = script.pre.clone().unwrap_or_default();

            let mod_ = script.mod_type.clone();
            let name = script.name.clone();

            let mut token = StateTokenType {
                tick: tick.clone(),
                max: max.clone(),
                lim: lim.clone(),
                pre: pre.clone(),
                dec,
                mod_type: mod_,
                from: from.clone(),
                to: to.clone(),
                minted: "0".to_string(),
                burned: "0".to_string(),
                name: name.unwrap_or_default(),
                tx_id: op_data.tx_id.clone(),
                op_add: op_data.op_score,
                op_mod: op_data.op_score,
                mts_add: op_data.mts_add,
                mts_mod: op_data.mts_add,
            };

            // Handle pre-minting
            if pre != "0" {
                let mut minted = pre.clone();
                if max != "0" {
                    let max_val: u64 = max.parse().unwrap_or(0);
                    let pre_val: u64 = pre.parse().unwrap_or(0);
                    if pre_val > max_val {
                        minted = max;
                    }
                }
                token.minted = minted.clone();

                // Create balance state
                let balance = StateBalanceType {
                    address: to.clone(),
                    tick: tick.clone(),
                    dec: dec.try_into().unwrap(),
                    balance: minted.clone(),
                    locked: "0".to_string(),
                    op_mod: op_data.op_score,
                };

                state_map
                    .state_balance_map
                    .insert(balance_key.clone(), Some(balance));

                // Update statistics
                Self::update_stats(op_data, tick, &balance_key, &minted);
            } else {
                // Update statistics
                Self::update_stats(op_data, tick, &balance_key, "0");
            }

            state_map.state_token_map.insert(tick.clone(), Some(token));
        }

        op_data.op_accept = 1;
        Ok(())
    }

    // Helper methods
    fn validate_tick(tick: &mut String) -> bool {
        *tick = tick.to_uppercase();
        let len_tick = tick.len();
        if len_tick < 4 || len_tick > 6 {
            return false;
        }
        tick.chars().all(|c| c.is_ascii_uppercase())
    }

    fn validate_amount(amount: &mut String) -> bool {
        // Use the centralized amount validation function
        crate::operations::validate_amount(amount)
    }

    fn validate_dec(dec: &mut String, default: &str) -> bool {
        if dec.is_empty() {
            *dec = default.to_string();
            return true;
        }

        if let Ok(val) = dec.parse::<i32>() {
            if val >= 0 && val <= 18 {
                *dec = val.to_string();
                true
            } else {
                false
            }
        } else {
            false
        }
    }

    fn is_tick_ignored(tick: &str) -> bool {
        let ignored_ticks = [
            "KASPA", "KASPLX", "KASP", "WKAS", "GIGA", "WBTC", "WETH", "USDT", "USDC", "FDUSD",
            "USDD", "TUSD", "USDP", "PYUSD", "EURC", "BUSD", "GUSD", "EURT", "XAUT", "TETHER",
        ];
        ignored_ticks.contains(&tick)
    }

    fn get_tick_reserved(tick: &str) -> Option<String> {
        // Use the centralized reserved token check from config module
        crate::config::get_reserved_tick_address(tick)
    }

    fn verify_address(address: &str, testnet: bool) -> bool {
        // Use the centralized address verification function
        crate::utils::address::verify_address(address, testnet)
    }

    fn update_stats(op_data: &mut DataOperationType, tick: &str, balance_key: &str, amount: &str) {
        // Update token impact statistics
        if let Some(ss_info) = &mut op_data.ss_info {
            if amount != "0" {
                ss_info.tick_affc.push(format!("{}={}", tick, 1));
                ss_info
                    .address_affc
                    .push(format!("{}={}", balance_key, amount));
            } else {
                ss_info.tick_affc.push(format!("{}={}", tick, 0));
            }
        }
    }

    /// Get minimum fee, corresponding to Go version FeeLeast
    pub fn fee_least(__daa_score: u64) -> u64 {
        // if daa_score ...
        100000000000
    }

    /// Script collection extension, corresponding to Go version ScriptCollectEx
    pub fn script_collect_ex(
        _index: usize,
        _script: &mut DataScriptType,
        _tx_data: &DataTransactionType,
        _testnet: bool,
    ) {
        // Temporarily empty implementation, corresponding to Go version empty implementation
    }
}
