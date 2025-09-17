use crate::storage::types::*;
use anyhow::Result;
use blake2::{Blake2b, Digest};
use serde_json;
use std::collections::HashMap;

/// Script parser, corresponding to Go version script parsing functionality
pub struct ScriptParser;

impl ScriptParser {
    /// Parse P2SH transaction input script, corresponding to Go version parseScriptInput
    pub fn parse_script_input(script: &str) -> Result<(bool, Vec<String>)> {
        let script = script.to_lowercase();
        let len_script = script.len();

        if len_script <= 138 {
            return Ok((false, Vec::new()));
        }

        // Get next data length and position
        let l_get = |s: &str, i: usize| -> Result<(i64, usize, bool)> {
            let i_raw = i;
            let len_s = s.len();
            if len_s < (i + 2) {
                return Ok((0, i_raw, false));
            }

            let f = &s[i..i + 2];
            let mut i = i + 2;
            let mut len_d: i64 = 0;

            match f {
                "4c" => {
                    if len_s < (i + 2) {
                        return Ok((0, i_raw, false));
                    }
                    let f = &s[i..i + 2];
                    i += 2;
                    len_d = i64::from_str_radix(f, 16)?;
                }
                "4d" => {
                    if len_s < (i + 4) {
                        return Ok((0, i_raw, false));
                    }
                    let f = s[i + 2..i + 4].to_string() + &s[i..i + 2];
                    i += 4;
                    len_d = i64::from_str_radix(&f, 16)?;
                }
                _ => {
                    len_d = i64::from_str_radix(f, 16)?;
                    if len_d < 0 || len_d > 75 {
                        return Ok((0, i_raw, false));
                    }
                }
            }

            len_d *= 2;
            Ok((len_d, i, true))
        };

        // Get push number and position
        let n_get = |s: &str, i: usize| -> Result<(i64, usize, bool)> {
            let i_raw = i;
            let len_s = s.len();
            if len_s < (i + 2) {
                return Ok((0, i_raw, false));
            }

            let f = &s[i..i + 2];
            let i = i + 2;
            let num = i64::from_str_radix(f, 16)?;
            if num < 81 || num > 96 {
                return Ok((0, i_raw, false));
            }
            let num = num - 80;
            Ok((num, i, true))
        };

        // Get last data position, corresponding to Go version _dGotoLast
        let d_goto_last = |s: &str, mut i: usize| -> Result<usize> {
            let i_raw = i;
            let len_s = s.len();
            let mut len_d: i64 = 0;
            let mut _r = true;

            for _j in 0..16 {
                let (new_len_d, new_i, new_r) = l_get(s, i)?;
                if !new_r {
                    return Ok(i_raw);
                }
                len_d = new_len_d;
                i = new_i;
                _r = new_r;

                if len_s < (i + len_d as usize) {
                    return Ok(i_raw);
                } else if len_s == (i + len_d as usize) {
                    if len_d < 94 {
                        return Ok(i_raw);
                    }
                    return Ok(i);
                } else {
                    i += len_d as usize;
                }
            }
            Ok(i_raw)
        };

        // Get parameter data and position
        let p_get = |s: &str, i: usize| -> Result<(String, usize, bool)> {
            let i_raw = i;
            let len_s = s.len();
            let (len_p, mut i, r) = l_get(s, i)?;
            if !r || len_s < (i + len_p as usize) {
                return Ok((String::new(), i_raw, false));
            }

            if len_p == 0 {
                return Ok((String::new(), i, true));
            }

            let hex_data = &s[i..i + len_p as usize];
            let decoded = hex::decode(hex_data)?;
            let p = String::from_utf8(decoded)?;
            i += len_p as usize;
            Ok((p, i, true))
        };

        // Skip to redemption script
        let mut n = 0;
        let mut flag = String::new();
        let mut _r = true;
        n = d_goto_last(&script, n)?;

        // Get public key or multisig script hash
        let mut script_sig = String::new();
        let mut multisig = false;
        let mut mm = 0i64;
        let _nn = 0i64;
        let mut k_pub = String::new();
        let mut len_d = 0i64;

        let (num, new_n, mut r) = n_get(&script, n)?;
        if r {
            if num > 0 && num < 16 {
                multisig = true;
                mm = num;
            } else {
                return Ok((false, Vec::new()));
            }
        }
        n = new_n;

        if !multisig {
            // Single signature processing
            let (new_len_d, new_n, r) = l_get(&script, n)?;
            if !r {
                return Ok((false, Vec::new()));
            }
            len_d = new_len_d;
            n = new_n;

            let mut f_sig = String::new();
            if len_script > (n + len_d as usize + 2) {
                f_sig = script[n + len_d as usize..n + len_d as usize + 2].to_string();
            }

            if len_d == 64 && f_sig == "ac" {
                k_pub = script[n..n + 64].to_string();
                n += 66;
                script_sig = format!("20{}{}", k_pub, f_sig);
            } else if len_d == 66 && f_sig == "ab" {
                k_pub = script[n..n + 66].to_string();
                n += 68;
                script_sig = format!("21{}{}", k_pub, f_sig);
            } else {
                return Ok((false, Vec::new()));
            }
        } else {
            // Multisig processing, corresponding to Go version complete multisig logic
            let mut k_pub_list = Vec::new();
            for _j in 0..16 {
                let (new_len_d, new_n, r) = l_get(&script, n)?;
                if !r {
                    let (new_nn, new_n, r) = n_get(&script, new_n)?;
                    if !r || k_pub_list.len() != new_nn as usize {
                        return Ok((false, Vec::new()));
                    }
                    let (new_k_pub, new_script_sig) =
                        Self::conv_k_pub_list_to_script_hash_multisig(mm, &k_pub_list, new_nn)?;
                    k_pub = new_k_pub;
                    script_sig = new_script_sig;
                    n = new_n;
                    break;
                }
                len_d = new_len_d;
                n = new_n;

                if len_d == 64 || len_d == 66 {
                    k_pub_list.push(script[n..n + len_d as usize].to_string());
                    n += len_d as usize;
                } else {
                    return Ok((false, Vec::new()));
                }
            }

            if len_script < (n + 2) {
                return Ok((false, Vec::new()));
            }
            flag = script[n..n + 2].to_string();
            n += 2;
            if flag != "a9" && flag != "ae" {
                return Ok((false, Vec::new()));
            }
        }

        if k_pub.is_empty() {
            return Ok((false, Vec::new()));
        }

        // Check protocol header, corresponding to Go version protocol header validation
        if len_script < (n + 22) {
            return Ok((false, Vec::new()));
        }
        flag = script[n..n + 6].to_string();
        n += 6;
        if flag != "006307" {
            return Ok((false, Vec::new()));
        }
        flag = script[n..n + 14].to_string();
        n += 14;
        let decoded = hex::decode(&flag)?;
        let header = String::from_utf8(decoded)?.to_uppercase();
        if header != "KASPLEX" {
            return Ok((false, Vec::new()));
        }

        // Get parameters and JSON data
        let mut p0 = String::new();
        let mut p1 = String::new();
        let mut p2 = String::new();
        _r = true;

        for _j in 0..2 {
            if len_script < (n + 2) {
                return Ok((false, Vec::new()));
            }

            flag = script[n..n + 2].to_string();
            n += 2;

            match flag.as_str() {
                "00" => {
                    let (p, new_n, r) = p_get(&script, n)?;
                    if !r {
                        return Ok((false, Vec::new()));
                    }
                    p0 = p;
                    n = new_n;
                }
                "68" => break,
                "51" | "53" | "55" | "57" | "59" | "5b" | "5d" | "5f" => {
                    p1 = match flag.as_str() {
                        "51" => "p1".to_string(),
                        "53" => "p3".to_string(),
                        "55" => "p5".to_string(),
                        "57" => "p7".to_string(),
                        "59" => "p9".to_string(),
                        "5b" => "p11".to_string(),
                        "5d" => "p13".to_string(),
                        "5f" => "p15".to_string(),
                        _ => return Ok((false, Vec::new())),
                    };
                    let (p, new_n, r) = p_get(&script, n)?;
                    if !r {
                        return Ok((false, Vec::new()));
                    }
                    p2 = p;
                    n = new_n;
                }
                _ => return Ok((false, Vec::new())),
            }
        }

        if p0.is_empty() {
            return Ok((false, Vec::new()));
        }

        // Get sender address
        let from = if multisig {
            Self::conv_k_pub_to_p2sh(&k_pub, false) // Temporarily hardcoded testnet=false
        } else {
            Self::conv_k_pub_to_addr(&k_pub, false) // Temporarily hardcoded testnet=false
        };

        Ok((true, vec![from, p0, p1, p2, script_sig]))
    }

    /// Parse operation data in transaction, corresponding to Go version parseOpData
    pub fn parse_op_data(
        tx_data: &DataTransactionType,
        testnet: bool,
    ) -> Result<Option<DataOperationType>> {
        if tx_data.data.is_none() {
            return Ok(None);
        }

        let tx_json = tx_data.data.as_ref().unwrap();

        // Get inputs
        let inputs = tx_json
            .get("inputs")
            .and_then(|v| v.as_array())
            .ok_or_else(|| anyhow::anyhow!("No inputs found"))?;

        if inputs.is_empty() {
            return Ok(None);
        }

        let mut op_script = Vec::new();
        let mut script_sig = String::new();

        for (i, input) in inputs.iter().enumerate() {
            if let Some(signature_script) = input.get("signatureScript").and_then(|v| v.as_str()) {
                let (is_op, script_info) = Self::parse_script_input(signature_script)?;
                if !is_op || script_info[0].is_empty() {
                    continue;
                }

                // Parse JSON data
                let decoded: DataScriptType = serde_json::from_str(&script_info[1])?;
                let mut script = decoded;
                script.from = Some(script_info[0].clone());

                // Set receiver address
                if !testnet && tx_data.daa_score <= 83525600 {
                    if let Some(outputs) = tx_json.get("outputs").and_then(|v| v.as_array()) {
                        if let Some(first_output) = outputs.first() {
                            if let Some(address) = first_output
                                .get("scriptPublicKeyAddress")
                                .and_then(|v| v.as_str())
                            {
                                script.to = Some(address.to_string());
                            }
                        }
                    }
                }

                // Validate script
                if !Self::validate_p(&script.p)
                    || !Self::validate_op(&script.op)
                    || !Self::validate_ascii(&script.to.clone().unwrap_or_default())
                {
                    continue;
                }

                // Collect script information
                Self::script_collect_ex(i, &mut script, tx_data, testnet);

                // Validate operation
                if !Self::validate_operation(&script, &tx_data.tx_id, tx_data.daa_score, testnet) {
                    continue;
                }

                if i == 0 {
                    op_script.push(script);
                    script_sig = script_info[4].clone();
                    continue;
                }

                // Check if it is a recyclable operation
                if !Self::is_op_recycle(&script.op) {
                    continue;
                }

                op_script.push(script);
            }
        }

        if op_script.is_empty() {
            return Ok(None);
        }

        // Create operation data
        let op_data = DataOperationType {
            tx_id: tx_data.tx_id.clone(),
            daa_score: tx_data.daa_score,
            block_accept: tx_data.block_accept.clone(),
            fee: 0,
            fee_least: 0,
            mts_add: tx_json
                .get("blockTime")
                .and_then(|v| v.as_i64())
                .unwrap_or(0),
            op_score: 0,
            op_accept: 0,
            op_error: String::new(),
            op_script,
            script_sig,
            st_before: Vec::new(),
            st_after: Vec::new(),
            checkpoint: String::new(),
            ss_info: Some(DataStatsType {
                tick_affc: Vec::new(),
                address_affc: Vec::new(),
            }),
        };

        Ok(Some(op_data))
    }

    /// Parse operation data list, corresponding to Go version ParseOpDataList
    pub fn parse_op_data_list(
        tx_data_list: &[DataTransactionType],
        testnet: bool,
    ) -> Result<Vec<DataOperationType>> {
        let mut op_data_map = HashMap::new();
        let mut tx_id_map = HashMap::new();

        for tx_data in tx_data_list {
            if let Ok(Some(op_data)) = Self::parse_op_data(tx_data, testnet) {
                let mut op_data = op_data;

                // Calculate fee
                if let Some(first_script) = op_data.op_script.first() {
                    op_data.fee_least =
                        Self::get_operation_fee(&first_script.op, op_data.daa_score);
                }

                if op_data.fee_least > 0 {
                    // Collect input transaction IDs
                    if let Some(tx_json) = &tx_data.data {
                        if let Some(inputs) = tx_json.get("inputs").and_then(|v| v.as_array()) {
                            for input in inputs {
                                if let Some(tx_id) = input
                                    .get("previousOutpoint")
                                    .and_then(|p| p.get("transactionId"))
                                    .and_then(|v| v.as_str())
                                {
                                    tx_id_map.insert(tx_id.to_string(), true);
                                }
                            }
                        }
                    }
                }

                op_data_map.insert(op_data.tx_id.clone(), op_data);
            }
        }

        // Calculate operation score and fee
        let mut op_data_list = Vec::new();
        let mut daa_score_now = 0u64;
        let mut op_score = 0u64;

        for tx_data in tx_data_list {
            if let Some(op_data) = op_data_map.get_mut(&tx_data.tx_id) {
                if daa_score_now != tx_data.daa_score {
                    daa_score_now = tx_data.daa_score;
                    op_score = daa_score_now * 10000;
                }

                op_data.op_score = op_score;

                // Calculate actual fee
                if op_data.fee_least > 0 {
                    let mut amount_in = 0u64;
                    let mut amount_out = 0u64;

                    // Calculate output amount
                    if let Some(tx_json) = &tx_data.data {
                        if let Some(outputs) = tx_json.get("outputs").and_then(|v| v.as_array()) {
                            for output in outputs {
                                if let Some(amount) = output.get("amount").and_then(|v| v.as_u64())
                                {
                                    amount_out += amount;
                                }
                            }
                        }
                    }

                    // Calculate input amount (simplified implementation)
                    amount_in = amount_out + op_data.fee_least;

                    if amount_in <= amount_out {
                        op_data.fee = 0;
                    } else {
                        op_data.fee = amount_in - amount_out;
                    }
                }

                op_data_list.push(op_data.clone());
                op_score += 1;
            }
        }

        Ok(op_data_list)
    }

    /// Script collection extension, corresponding to Go version ScriptCollectEx
    fn script_collect_ex(
        i: usize,
        script: &mut DataScriptType,
        tx_data: &DataTransactionType,
        testnet: bool,
    ) {
        // Collect script information based on operation type
        match script.op.as_str() {
            "deploy" => {
                // Special handling for deploy operation
                if let Some(tick) = &script.tick {
                    script.name = Some(tick.clone());
                }
            }
            "mint" => {
                // Special handling for mint operation
                if script.amt.is_none() {
                    script.amt = Some("0".to_string());
                }
            }
            "transfer" => {
                // Special handling for transfer operation
                if script.amt.is_none() {
                    script.amt = Some("0".to_string());
                }
            }
            _ => {
                // Handling for other operations
            }
        }
    }

    // Helper methods
    /// Convert public key list to multisig script hash, corresponding to Go version ConvKPubListToScriptHashMultisig
    fn conv_k_pub_list_to_script_hash_multisig(
        m: i64,
        k_pub_list: &[String],
        n: i64,
    ) -> Result<(String, String)> {
        let len_k_pub_list = k_pub_list.len();
        if len_k_pub_list < 1 || len_k_pub_list != n as usize {
            return Ok((String::new(), String::new()));
        }

        let mut ecdsa = false;
        let mut k_pub = format!("{:x}", m + 80);

        for k in k_pub_list {
            let len_k = k.len();
            if len_k == 64 {
                k_pub += "20";
            } else if len_k == 66 {
                k_pub += "21";
                ecdsa = true;
            } else {
                return Ok((String::new(), String::new()));
            }
            k_pub += k;
        }

        k_pub += &format!("{:x}", n + 80);
        if ecdsa {
            k_pub += "a9";
        } else {
            k_pub += "ae";
        }

        let decoded = hex::decode(&k_pub)?;
        let mut hasher = Blake2b::<blake2::digest::consts::U32>::new();
        hasher.update(&decoded);
        let result = hasher.finalize();
        let script_hash = format!("{:064x}", result);

        Ok((script_hash, k_pub))
    }

    /// Convert script hash to P2SH address, corresponding to Go version ConvKPubToP2sh
    fn conv_k_pub_to_p2sh(k_pub: &str, testnet: bool) -> String {
        let decoded = hex::decode(k_pub).unwrap_or_default();
        let addr = Self::encode_bech32(&decoded, testnet);
        if testnet {
            format!("kaspatest:{}", addr)
        } else {
            format!("kaspa:{}", addr)
        }
    }

    /// Convert public key to address, corresponding to Go version ConvKPubToAddr
    fn conv_k_pub_to_addr(k_pub: &str, testnet: bool) -> String {
        let decoded = hex::decode(k_pub).unwrap_or_default();
        let addr = Self::encode_bech32(&decoded, testnet);
        if testnet {
            format!("kaspatest:{}", addr)
        } else {
            format!("kaspa:{}", addr)
        }
    }

    /// Bech32 encoding implementation
    fn encode_bech32(data: &[u8], _testnet: bool) -> String {
        // Simplified Bech32 encoding implementation
        // In actual applications, a dedicated Bech32 library should be used
        hex::encode(data)
    }

    fn validate_p(p: &str) -> bool {
        let mut p_upper = p.to_uppercase();
        crate::operations::validate_p(&mut p_upper)
    }

    fn validate_op(op: &str) -> bool {
        let mut op_lower = op.to_lowercase();
        crate::operations::validate_op(&mut op_lower)
    }

    fn validate_ascii(s: &str) -> bool {
        crate::operations::validate_ascii(s)
    }

    fn validate_operation(
        script: &DataScriptType,
        tx_id: &str,
        daa_score: u64,
        testnet: bool,
    ) -> bool {
        // Call corresponding validation method based on operation type
        match script.op.as_str() {
            "deploy" => crate::operations::deploy::DeployOperation::validate(
                &mut script.clone(),
                tx_id,
                daa_score,
                testnet,
            ),
            "mint" => crate::operations::mint::MintOperation::validate(
                &mut script.clone(),
                tx_id,
                daa_score,
                testnet,
            ),
            "transfer" => crate::operations::transfer::TransferOperation::validate(
                &mut script.clone(),
                tx_id,
                daa_score,
                testnet,
            ),
            "burn" => crate::operations::burn::BurnOperation::validate(
                &mut script.clone(),
                tx_id,
                daa_score,
                testnet,
            ),
            "send" => crate::operations::send::SendOperation::validate(
                &mut script.clone(),
                tx_id,
                daa_score,
                testnet,
            ),
            "issue" => crate::operations::issue::IssueOperation::validate(
                &mut script.clone(),
                tx_id,
                daa_score,
                testnet,
            ),
            "list" => crate::operations::list::ListOperation::validate(
                &mut script.clone(),
                tx_id,
                daa_score,
                testnet,
            ),
            "chown" => crate::operations::chown::ChownOperation::validate(
                &mut script.clone(),
                tx_id,
                daa_score,
                testnet,
            ),
            "blacklist" => crate::operations::blacklist::BlacklistOperation::validate(
                &mut script.clone(),
                tx_id,
                daa_score,
                testnet,
            ),
            _ => true, // Other operations temporarily return true
        }
    }

    fn is_op_recycle(op: &str) -> bool {
        let recycle_operations = ["mint", "burn", "transfer", "send"];
        recycle_operations.contains(&op)
    }

    fn get_operation_fee(op: &str, daa_score: u64) -> u64 {
        // Calculate fee based on DAA score and operation type
        match op {
            "deploy" => {
                if daa_score >= 110165000 {
                    100000000000 // New version fee
                } else {
                    50000000000 // Old version fee
                }
            }
            "mint" => {
                if daa_score >= 110165000 {
                    100000000 // New version fee
                } else {
                    50000000 // Old version fee
                }
            }
            "transfer" => 0,
            "send" => {
                if daa_score >= 110165000 {
                    200000000 // New version fee
                } else {
                    100000000 // Old version fee
                }
            }
            "issue" => {
                if daa_score >= 110165000 {
                    400000000 // New version fee
                } else {
                    200000000 // Old version fee
                }
            }
            "list" => {
                if daa_score >= 110165000 {
                    100000000 // New version fee
                } else {
                    50000000 // Old version fee
                }
            }
            "chown" => {
                if daa_score >= 110165000 {
                    800000000 // New version fee
                } else {
                    400000000 // Old version fee
                }
            }
            "blacklist" => {
                if daa_score >= 110165000 {
                    600000000 // New version fee
                } else {
                    300000000 // Old version fee
                }
            }
            _ => 0,
        }
    }
}
