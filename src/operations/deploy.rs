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
        if amount.is_empty() {
            *amount = "0".to_string();
            return false;
        }

        // Validate big integer format
        if let Ok(_) = amount.parse::<u64>() {
            true
        } else {
            false
        }
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
        let reserved_ticks = [
            (
                "NACHO",
                "kaspa:qzrsq2mfj9sf7uye3u5q7juejzlr0axk5jz9fpg4vqe76erdyvxxze84k9nk7",
            ),
            (
                "KCATS",
                "kaspa:qq8guq855gxkfrj2w25skwgj7cp4hy08x6a8mz70tdtmgv5p2ngwqxpj4cknc",
            ),
            (
                "KASTOR",
                "kaspa:qr8vt54764aaddejhjfwtsh07jcjr49v38vrw2vtmxxtle7j2uepynwy57ufg",
            ),
            (
                "KASPER",
                "kaspa:qppklkx2zyr2g2djg3uy2y2tsufwsqjk36pt27vt2xfu8uqm24pskk4p7tq5n",
            ),
            (
                "FUSUN",
                "kaspa:qzp30gu5uty8jahu9lq5vtplw2ca8m2k7p45ez3y8jf9yrm5qdxquq5nl45t5",
            ),
            (
                "KPAW",
                "kaspa:qpp0y685frmnlvhmnz5t6qljatumqm9zmppwnhwu9vyyl6w8nt30qjedekmdw",
            ),
            (
                "PPKAS",
                "kaspa:qrlx9377yje3gvj9qxvwnn697d209lshgcrvge3yzlxnvyrfyk3q583jh3cmz",
            ),
            (
                "GHOAD",
                "kaspa:qpkty3ymqs67t0z3g7l457l79f9k6drl55uf2qeq5tlkrpf3zwh85es0xtaj9",
            ),
            (
                "KEPE",
                "kaspa:qq45gur2grn80uuegg9qgewl0wg2ahz5n4qm9246laej9533f8e22x3xe6hkm",
            ),
            (
                "WORI",
                "kaspa:qzhgepc7mjscszkteeqhy99d3v96ftpg2wyy6r85nd0kg9m8rfmusqpp7mxkq",
            ),
            (
                "KEKE",
                "kaspa:qqq9m42mdcvlz8c7r9kmpqj59wkfx3nppqte8ay20m4p46x3z0lsyzz34h8uf",
            ),
            (
                "DOGK",
                "kaspa:qpsj64nxtlwceq4e7jvrsrkl0y6dayfyrqr49pep7pd2tq2uzvk7ks7n0qwxc",
            ),
            (
                "BTAI",
                "kaspa:qp0na29g4lysnaep5pmg9xkdzcn4xm4a35ha5naq79ns9mcgc3pccnf225qma",
            ),
            (
                "KASBOT",
                "kaspa:qrrcpdaev9augqwy8jnnp20skplyswa7ezz3m9ex3ryxw22frpzpj2xx99scq",
            ),
            (
                "SOMPS",
                "kaspa:qry7xqy6s7d449gqyl0dkr99x6df0q5jlj6u52p84tfv6rddxjrucnn066237",
            ),
            (
                "KREP",
                "kaspa:qzaclsmr5vttzlt0rz0x3shnudny8lnz5zpmjr4lp9v7aa7u7zvexh05eqwq0",
            ),
        ];

        for (reserved_tick, addr) in &reserved_ticks {
            if tick == *reserved_tick {
                return Some(addr.to_string());
            }
        }
        None
    }

    fn verify_address(address: &str, testnet: bool) -> bool {
        // Simple address validation logic
        if testnet {
            address.starts_with("kaspa:") && address.len() > 10
        } else {
            address.starts_with("kaspa:") && address.len() > 10
        }
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
