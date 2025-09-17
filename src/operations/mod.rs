use once_cell::sync::Lazy;
use std::collections::HashMap;
use std::sync::Mutex;

// Global registry, corresponding to Go version's P_Registered and Op_Registered
static P_REGISTERED: Lazy<Mutex<HashMap<String, bool>>> = Lazy::new(|| {
    let mut map = HashMap::new();
    map.insert("KRC-20".to_string(), true);
    Mutex::new(map)
});

static OP_REGISTERED: Lazy<Mutex<HashMap<String, bool>>> = Lazy::new(|| {
    let mut map = HashMap::new();
    map.insert("deploy".to_string(), true);
    map.insert("mint".to_string(), true);
    map.insert("transfer".to_string(), true);
    map.insert("burn".to_string(), true);
    map.insert("send".to_string(), true);
    map.insert("issue".to_string(), true);
    map.insert("list".to_string(), true);
    map.insert("chown".to_string(), true);
    map.insert("blacklist".to_string(), true);
    Mutex::new(map)
});

/// Validate protocol, corresponding to Go version's ValidateP
pub fn validate_p(p: &mut String) -> bool {
    *p = p.to_uppercase();
    P_REGISTERED.lock().unwrap().contains_key(p)
}

/// Validate operation, corresponding to Go version's ValidateOp
pub fn validate_op(op: &mut String) -> bool {
    *op = op.to_lowercase();
    OP_REGISTERED.lock().unwrap().contains_key(op)
}

/// Validate ASCII characters, corresponding to Go version's ValidateAscii
pub fn validate_ascii(s: &str) -> bool {
    if s.is_empty() {
        return true;
    }
    s.chars().all(|c| c.is_ascii())
}

pub mod blacklist;
pub mod burn;
pub mod chown;
pub mod deploy;
pub mod handler;
pub mod issue;
pub mod list;
pub mod mint;
pub mod send;
pub mod transfer;

pub use blacklist::BlacklistOperation;
pub use burn::BurnOperation;
pub use chown::ChownOperation;
pub use deploy::DeployOperation;
pub use issue::IssueOperation;
pub use list::ListOperation;
pub use mint::MintOperation;
pub use send::SendOperation;
pub use transfer::TransferOperation;

/// Check if token is reserved, corresponding to Go version's is_tick_reserved
pub fn is_tick_reserved(tick: &str) -> bool {
    crate::config::is_tick_reserved(tick)
}

/// Check if token is ignored
pub fn is_tick_ignored(tick: &str) -> bool {
    let ignored_ticks = [
        "KASPA", "KASPLX", "KASP", "WKAS", "GIGA", "WBTC", "WETH", "USDT", "USDC", "FDUSD", "USDD",
        "TUSD", "USDP", "PYUSD", "EURC", "BUSD", "GUSD", "EURT", "XAUT", "TETHER",
    ];
    ignored_ticks.contains(&tick)
}

/// Get reserved token address, corresponding to Go version's get_reserved_tick_address
pub fn get_reserved_tick_address(tick: &str) -> Option<String> {
    crate::config::get_reserved_tick_address(tick)
}

/// Apply reserved token list, corresponding to Go version's ApplyTickReserved
pub fn apply_tick_reserved(reserved_list: &[String]) {
    crate::config::apply_tick_reserved(reserved_list);
}

/// Validate token name, corresponding to Go version's ValidateTick
pub fn validate_tick(tick: &mut String) -> bool {
    *tick = tick.to_uppercase();
    let len_tick = tick.len();
    if len_tick < 4 || len_tick > 6 {
        return false;
    }
    for ch in tick.chars() {
        if !ch.is_ascii_uppercase() {
            return false;
        }
    }
    true
}

/// Validate transaction ID, corresponding to Go version's ValidateTxId
pub fn validate_tx_id(tx_id: &mut String) -> bool {
    *tx_id = tx_id.to_lowercase();
    if tx_id.len() != 64 {
        return false;
    }
    // Check if it's a valid hexadecimal string
    tx_id.chars().all(|c| c.is_ascii_hexdigit())
}

/// Validate token or transaction ID, corresponding to Go version's ValidateTickTxId
pub fn validate_tick_tx_id(tick: &mut String) -> bool {
    if tick.len() < 64 {
        validate_tick(tick)
    } else {
        validate_tx_id(tick)
    }
}

/// Validate amount, corresponding to Go version's ValidateAmount
pub fn validate_amount(amount: &mut String) -> bool {
    if amount.is_empty() {
        *amount = "0".to_string();
        return false;
    }

    // Use string comparison to validate large integers, corresponding to Go version's big.Int validation logic
    let amount_str = amount.as_str();

    // Check if it's a valid numeric string
    if !amount_str.chars().all(|c| c.is_ascii_digit()) {
        return false;
    }

    // Check if it exceeds the limit
    let limit = "99999999999999999999999999999999";
    if amount_str == "0" {
        return false; // Corresponding to Go version's limitBig.Cmp(amountBig) >= 0
    }

    // Compare string length and value
    if amount_str.len() > limit.len() {
        return false;
    }
    if amount_str.len() == limit.len() && amount_str > limit {
        return false;
    }

    true
}

/// Validate decimal places, corresponding to Go version's ValidateDec
pub fn validate_dec(dec: &mut String, default: &str) -> bool {
    if dec.is_empty() {
        *dec = default.to_string();
        return true;
    }

    match dec.parse::<u8>() {
        Ok(d) => {
            let dec_string = d.to_string();
            if dec_string != *dec || d > 18 {
                return false;
            }
            true
        }
        Err(_) => false,
    }
}

/// Validate unsigned integer, corresponding to Go version's ValidationUint
pub fn validate_uint(value: &mut String, default: &str) -> bool {
    if value.is_empty() {
        *value = default.to_string();
        return true;
    }

    match value.parse::<u64>() {
        Ok(v) => {
            let value_string = v.to_string();
            value_string == *value
        }
        Err(_) => false,
    }
}

/// Generate token state line, corresponding to Go version's MakeStLineToken
pub fn make_st_line_token(
    key: &str,
    st_token: Option<&crate::storage::types::StateTokenType>,
    is_deploy: bool,
) -> String {
    let mut st_line = format!("sttoken_{}", key);

    if let Some(token) = st_token {
        st_line.push(',');
        let str_dec = token.dec.to_string();
        let op_score = token.op_mod; // Corresponding to Go version's opScore := stToken.OpMod
        let str_op_score = op_score.to_string();

        if is_deploy {
            st_line.push_str(&format!(
                "{},{},{},{},{},",
                token.max, token.lim, token.pre, str_dec, token.from
            ));
        }

        st_line.push_str(&format!("{},{},", token.minted, str_op_score));

        if token.mod_type == "issue" {
            st_line.push_str(&format!(
                "{},{},{}",
                token.mod_type, token.burned, token.name
            ));
        }
    }

    st_line
}

/// Append token state line, corresponding to Go version's AppendStLineToken
pub fn append_st_line_token(
    st_line: &mut Vec<String>,
    key: &str,
    st_token: Option<&crate::storage::types::StateTokenType>,
    is_deploy: bool,
    is_after: bool,
) -> Vec<String> {
    let key_full = format!("sttoken_{}", key);
    let mut result = st_line.clone();

    // Check if it already exists
    let mut i_exists = None;
    for (i, line) in result.iter().enumerate() {
        if line.starts_with(&key_full) {
            i_exists = Some(i);
            break;
        }
    }

    if let Some(i) = i_exists {
        if is_after {
            result[i] = make_st_line_token(key, st_token, is_deploy);
        }
    } else {
        result.push(make_st_line_token(key, st_token, is_deploy));
    }

    result
}

/// Generate balance state line, corresponding to Go version's MakeStLineBalance
pub fn make_st_line_balance(
    key: &str,
    st_balance: Option<&crate::storage::types::StateBalanceType>,
) -> String {
    let mut st_line = format!("stbalance_{}", key);

    if let Some(balance) = st_balance {
        st_line.push(',');
        let str_dec = balance.dec.to_string();
        let str_op_score = balance.op_mod.to_string();
        st_line.push_str(&format!(
            "{},{},{},{}",
            str_dec, balance.balance, balance.locked, str_op_score
        ));
    }

    st_line
}

/// Append balance state line, corresponding to Go version's AppendStLineBalance
pub fn append_st_line_balance(
    st_line: &mut Vec<String>,
    key: &str,
    st_balance: Option<&crate::storage::types::StateBalanceType>,
    is_after: bool,
) -> Vec<String> {
    let key_full = format!("stbalance_{}", key);
    let mut result = st_line.clone();

    // Check if it already exists
    let mut i_exists = None;
    for (i, line) in result.iter().enumerate() {
        if line.starts_with(&key_full) {
            i_exists = Some(i);
            break;
        }
    }

    if let Some(i) = i_exists {
        if is_after {
            result[i] = make_st_line_balance(key, st_balance);
        }
    } else {
        result.push(make_st_line_balance(key, st_balance));
    }

    result
}

/// Generate market state line, corresponding to Go version's MakeStLineMarket
pub fn make_st_line_market(
    key: &str,
    st_market: Option<&crate::storage::types::StateMarketType>,
) -> String {
    let mut st_line = format!("stmarket_{}", key);

    if let Some(market) = st_market {
        st_line.push(',');
        let str_op_score = market.op_add.to_string();
        st_line.push_str(&format!(
            "{},{},{},{}",
            market.u_addr, market.u_amt, market.t_amt, str_op_score
        ));
    }

    st_line
}

/// Append market state line, corresponding to Go version's AppendStLineMarket
pub fn append_st_line_market(
    st_line: &mut Vec<String>,
    key: &str,
    st_market: Option<&crate::storage::types::StateMarketType>,
    is_after: bool,
) -> Vec<String> {
    let key_full = format!("stmarket_{}", key);
    let mut result = st_line.clone();

    // Check if it already exists
    let mut i_exists = None;
    for (i, line) in result.iter().enumerate() {
        if line.starts_with(&key_full) {
            i_exists = Some(i);
            break;
        }
    }

    if let Some(i) = i_exists {
        if is_after {
            result[i] = make_st_line_market(key, st_market);
        }
    } else {
        result.push(make_st_line_market(key, st_market));
    }

    result
}

/// Generate blacklist state line, corresponding to Go version's MakeStLineBlacklist
pub fn make_st_line_blacklist(
    key: &str,
    st_blacklist: Option<&crate::storage::types::StateBlacklistType>,
) -> String {
    let mut st_line = format!("stblacklist_{}", key);

    if let Some(blacklist) = st_blacklist {
        st_line.push(',');
        let str_op_score = blacklist.op_add.to_string();
        st_line.push_str(&str_op_score);
    }

    st_line
}

/// Append blacklist state line, corresponding to Go version's AppendStLineBlacklist
pub fn append_st_line_blacklist(
    st_line: &mut Vec<String>,
    key: &str,
    st_blacklist: Option<&crate::storage::types::StateBlacklistType>,
    is_after: bool,
) -> Vec<String> {
    let key_full = format!("stblacklist_{}", key);
    let mut result = st_line.clone();

    // Check if it already exists
    let mut i_exists = None;
    for (i, line) in result.iter().enumerate() {
        if line.starts_with(&key_full) {
            i_exists = Some(i);
            break;
        }
    }

    if let Some(i) = i_exists {
        if is_after {
            result[i] = make_st_line_blacklist(key, st_blacklist);
        }
    } else {
        result.push(make_st_line_blacklist(key, st_blacklist));
    }

    result
}

/// Append tick effect information, corresponding to Go version's AppendSsInfoTickAffc
pub fn append_ss_info_tick_affc(tick_affc: &mut Vec<String>, key: &str, value: i64) -> Vec<String> {
    let mut result = tick_affc.clone();

    // Check if it already exists
    let mut i_exists = None;
    for (i, line) in result.iter().enumerate() {
        if line.starts_with(key) {
            i_exists = Some(i);
            break;
        }
    }

    if let Some(i) = i_exists {
        result[i] = format!("{}:{}", key, value);
    } else {
        result.push(format!("{}:{}", key, value));
    }

    result
}

/// Append address effect information, corresponding to Go version's AppendSsInfoAddressAffc
pub fn append_ss_info_address_affc(
    address_affc: &mut Vec<String>,
    key: &str,
    value: &str,
) -> Vec<String> {
    let mut result = address_affc.clone();
    let mut i_exists = None;

    for (i, affc) in result.iter().enumerate() {
        let parts: Vec<&str> = affc.split('=').collect();
        if parts.len() >= 2 && parts[0] == key {
            i_exists = Some(i);
            break;
        }
    }

    if let Some(i) = i_exists {
        result[i] = format!("{}={}", key, value);
    } else {
        result.push(format!("{}={}", key, value));
    }

    result
}

// Add missing validation functions, corresponding to Go version
