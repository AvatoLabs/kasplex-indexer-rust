use anyhow::Result;
use kaspa_indexer_rust::config::types::*;
use kaspa_indexer_rust::operations::send::SendOperation;
use kaspa_indexer_rust::storage::types::*;
use serde_json::json;

/// DRAGON token transfer demo
/// Transfer issued DRAGON tokens to test address
#[tokio::main]
async fn main() -> Result<()> {
    // Initialize logging
    tracing_subscriber::fmt::init();

    println!("🔄 DRAGON Token Transfer Demo");
    println!("=============================");

    // Testnet configuration
    let testnet_config = Config {
        startup: StartupConfig {
            hysteresis: 3,
            daa_score_range: vec![],
            tick_reserved: vec![],
            kaspa_node_url: "https://testnet.kaspa.org:16210".to_string(),
            is_testnet: true,
        },
        rocksdb: RocksConfig {
            path: "./testnet_data".to_string(),
        },
        distributed: DistributedConfig::default(),
        http: HttpConfig::default(),
        rest: RestConfig::default(),
        debug: 2,
        testnet: true,
        is_testnet: true,
    };

    println!("📋 Configuration information:");
    println!("  Network: Testnet");
    println!("  Node URL: {}", testnet_config.startup.kaspa_node_url);

    // Wallet information
    let sender_address = "kaspatest:qrf7saw4vlc006mmcpfa29mrgs79ez5glszc4ytx9hq7wdu2a5d4kvl7wz5pz";
    let receiver_address =
        "kaspatest:qrf7saw4vlc006mmcpfa29mrgs79ez5glszc4ytx9hq7wdu2a5d4kvl7wz5pz"; // Can be changed to other test address

    println!("\n💰 Transfer information:");
    println!("  Sender: {}", sender_address);
    println!("  Receiver: {}", receiver_address);
    println!("  Token: DRAGON");
    println!("  Amount: 10000");

    // Build KRC-20 transfer script
    println!("\n📝 Building KRC-20 transfer script...");
    let transfer_script = build_transfer_script("DRAGON", "10000", receiver_address)?;
    println!("  Script: {}", transfer_script);

    // Create script data structure
    let mut script_data = DataScriptType {
        p: "KRC-20".to_string(),
        op: "send".to_string(),
        tick: Some("DRAGON".to_string()),
        from: Some(sender_address.to_string()),
        to: Some(receiver_address.to_string()),
        amt: Some("10000".to_string()),
        lim: None,
        pre: None,
        utxo: None,
        price: None,
        mod_type: "send".to_string(),
        ca: None,
        max: None,
        dec: None,
        name: None,
    };

    // Validate transfer script
    println!("\n✅ Validating transfer script...");
    let is_valid = SendOperation::validate(&mut script_data, "test_tx_id", 110165000, true);
    if is_valid {
        println!("  ✅ Script validation passed");
    } else {
        println!("  ❌ Script validation failed");
        return Ok(());
    }

    // Create state map (simulating existing DRAGON tokens)
    let mut state_map = DataStateMapType {
        state_token_map: std::collections::HashMap::new(),
        state_balance_map: std::collections::HashMap::new(),
        state_market_map: std::collections::HashMap::new(),
        state_blacklist_map: std::collections::HashMap::new(),
    };

    // Simulate sender already has DRAGON token balance
    let sender_balance_key = format!("balance:{}:{}", sender_address, "DRAGON");
    let sender_balance = StateBalanceType {
        address: sender_address.to_string(),
        tick: "DRAGON".to_string(),
        dec: 8,
        balance: "50000".to_string(), // Assume 50000 DRAGON tokens
        locked: "0".to_string(),
        op_mod: 0,
    };
    state_map
        .state_balance_map
        .insert(sender_balance_key, Some(sender_balance));

    // Prepare state
    println!("\n🔧 Preparing transfer state...");
    if let Err(e) = SendOperation::prepare_state(&script_data, &mut state_map) {
        println!("  ❌ State preparation failed: {}", e);
        return Ok(());
    }
    println!("  ✅ State preparation completed");

    // Execute transfer operation
    println!("\n🚀 Executing token transfer...");
    if let Err(e) = SendOperation::execute(&script_data, &mut state_map) {
        println!("  ❌ Transfer failed: {}", e);
        return Ok(());
    }
    println!("  ✅ Token transfer successful!");

    // Display results
    println!("\n📊 Transfer results:");

    // Sender balance
    let sender_balance_key = format!("balance:{}:{}", sender_address, "DRAGON");
    if let Some(balance) = state_map.state_balance_map.get(&sender_balance_key) {
        if let Some(balance_data) = balance {
            println!("  Sender balance: {} {}", balance_data.balance, "DRAGON");
        }
    }

    // Receiver balance
    let receiver_balance_key = format!("balance:{}:{}", receiver_address, "DRAGON");
    if let Some(balance) = state_map.state_balance_map.get(&receiver_balance_key) {
        if let Some(balance_data) = balance {
            println!("  Receiver balance: {} {}", balance_data.balance, "DRAGON");
        }
    }

    println!("\n🎉 DRAGON token transfer demo completed!");
    println!("\n📋 Next steps:");
    println!("  1. Use Kaspa wallet to create transaction containing this script");
    println!("  2. Sign and broadcast transaction to testnet");
    println!("  3. Wait for transaction confirmation");
    println!("  4. Use indexer to monitor transfer status");

    println!("\n💡 Actual transfer steps:");
    println!("  1. Create new transaction in Kaspa wallet");
    println!("  2. Add output with amount set to 0.01 KAS");
    println!("  3. Enter in script field: {}", transfer_script);
    println!("  4. Sign and send transaction");

    Ok(())
}

/// Build KRC-20 transfer script
fn build_transfer_script(tick: &str, amount: &str, to_address: &str) -> Result<String> {
    // KRC-20 transfer script format
    let script_data = json!({
        "p": "KRC-20",
        "op": "send",
        "tick": tick,
        "amt": amount,
        "to": to_address
    });

    // Convert JSON to script string
    let script_str = serde_json::to_string(&script_data)?;

    // Encode to hexadecimal
    let script_hex = hex::encode(script_str.as_bytes());

    Ok(script_hex)
}
