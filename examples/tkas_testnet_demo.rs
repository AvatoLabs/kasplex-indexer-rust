use anyhow::Result;
use kaspa_indexer_rust::config::types::*;
use kaspa_indexer_rust::operations::handler::OperationManager;
use kaspa_indexer_rust::operations::*;
use kaspa_indexer_rust::storage::StorageManager;
use kaspa_indexer_rust::storage::types::*;
use kaspa_indexer_rust::utils::address;
use kaspa_indexer_rust::utils::script;
use std::sync::Arc;

#[tokio::main]
async fn main() -> Result<()> {
    // Initialize logging
    tracing_subscriber::fmt::init();

    println!("🚀 Kasplex PEPEK Testnet Demo");
    println!("================================");

    // Testnet configuration
    let testnet_config = Config {
        startup: StartupConfig {
            hysteresis: 3,
            daa_score_range: vec![],
            tick_reserved: vec![],
            kaspa_node_url: "http://127.0.0.1:16210".to_string(),
            is_testnet: true,
        },
        rocksdb: RocksConfig {
            path: "./testnet_data".to_string(),
        },
        distributed: DistributedConfig {
            node: DistributedNodeConfig {
                enabled: false,
                node_id: "testnet_node".to_string(),
                data_dir: "./testnet_data/distributed".to_string(),
                shard_count: 4,
                replication_factor: 2,
                port: 8081,
                nodes: vec![],
                role: "standalone".to_string(),
                max_connections: 100,
            },
            shards: vec![],
            hash_ring: HashRingConfig {
                virtual_nodes: 100,
                hash_algorithm: "blake3".to_string(),
                enabled: false,
                ring_size: 1024,
            },
            replication: ReplicationConfig {
                strategy: "async".to_string(),
                timeout: 30,
                max_retries: 3,
                retry_interval: 5,
                enable_compression: true,
                compression_level: 6,
            },
            performance: PerformanceConfig::default(),
            monitoring: MonitoringConfig::default(),
            security: SecurityConfig::default(),
        },
        http: HttpConfig::default(),
        rest: RestConfig::default(),
        debug: 2,
        testnet: true,
        is_testnet: true,
    };

    println!("📋 Testnet Configuration:");
    println!("  Network: Testnet");
    println!("  Node URL: {}", testnet_config.startup.kaspa_node_url);
    println!("  Data Directory: {}", testnet_config.rocksdb.path);

    // Test address information
    let test_address = "kaspatest:qrf7saw4vlc006mmcpfa29mrgs79ez5glszc4ytx9hq7wdu2a5d4kvl7wz5pz";
    let tkas_balance = 30000u64;

    println!("\n💰 Test Address Information:");
    println!("  Address: {}", test_address);
    println!("  PEPEK Balance: {} PEPEK (Testnet Layer 1 Token)", tkas_balance);
    println!("  Testing Kasplex Token: PEPEK (Kasplex Layer 2 Token)");

    // Verify address
    let is_valid_address = address::verify_address(test_address, true);
    println!(
        "  Address Verification: {}",
        if is_valid_address {
            "✅ Valid"
        } else {
            "❌ Invalid"
        }
    );

    // Create storage manager
    println!("\n🗄️ Initializing storage manager...");
    let mut storage_manager =
        StorageManager::new(testnet_config.rocksdb, Some(testnet_config.distributed)).await?;
    storage_manager.init().await?;
    let storage: Arc<StorageManager> = Arc::new(storage_manager);

    // Create operation manager
    let operation_manager = OperationManager::new(Arc::clone(&storage));
    println!("✅ Operation manager initialization completed");

    // Display supported operations
    let supported_ops = operation_manager.get_supported_operations();
    println!("📝 Supported operation types: {:?}", supported_ops);

    // Create shared state mapping
    let mut state_map = DataStateMapType::new();

    // Test 1: PEPEK token deployment
    println!("\n🔨 Test 1: PEPEK Token Deployment");
    test_pepek_deploy(&operation_manager, test_address, &mut state_map, &storage).await?;

    // Test 2: PEPEK token minting
    println!("\n⛏️ Test 2: PEPEK Token Minting");
    test_pepek_mint(&operation_manager, test_address, &mut state_map, &storage).await?;

    // Test 3: PEPEK token transfer
    println!("\n💸 Test 3: PEPEK Token Transfer");
    test_pepek_transfer(&operation_manager, test_address, &mut state_map, &storage).await?;

    // Test 4: PEPEK token burning
    println!("\n🔥 Test 4: PEPEK Token Burning");
    test_pepek_burn(&operation_manager, test_address, &storage).await?;

    // Test 5: Script generation
    println!("\n📜 Test 5: Script Generation");
    test_pepek_script_generation(test_address).await?;

    // Test 6: Address handling
    println!("\n🏠 Test 6: Address Handling");
    test_address_handling().await?;

    println!("\n🎉 PEPEK Testnet Demo Completed!");
    Ok(())
}

async fn test_pepek_deploy(
    operation_manager: &OperationManager,
    test_address: &str,
    state_map: &mut DataStateMapType,
    storage: &Arc<StorageManager>,
) -> Result<()> {
    println!("  Creating PEPEK deployment script...");

    let mut deploy_script = DataScriptType {
        p: "KRC-20".to_string(),
        op: "deploy".to_string(),
        from: Some(test_address.to_string()),
        to: Some(test_address.to_string()),
        tick: Some("PEPEK".to_string()),
        max: Some("1000000".to_string()), // Maximum supply 1 million
        lim: Some("1000".to_string()),    // Single mint limit 1000
        pre: Some("30000".to_string()),   // Pre-mint 30000
        dec: Some("8".to_string()),       // 8 decimal places
        mod_type: "".to_string(),         // mint mode
        name: None,
        amt: None,
        utxo: None,
        price: None,
        ca: None,
    };

    // Validate deployment script
    let is_valid = DeployOperation::validate(
        &mut deploy_script,
        "tkas_deploy_tx_hash_1234567890abcdef",
        110165000,
        true, // testnet
    );

    println!(
        "  Script validation: {}",
        if is_valid { "✅ Passed" } else { "❌ Failed" }
    );

    if is_valid {
        // Create operation data
        let mut op_data =
            create_operation_data("tkas_deploy_tx_hash_1234567890abcdef", vec![deploy_script]);

        // Prepare state key
        DeployOperation::prepare_state_key(&op_data.op_script[0], state_map);

        // Execute deployment operation
        let result = operation_manager.execute_operation(0, &mut op_data, state_map, true);

        match result {
            Ok(_) => {
                println!(
                    "  Deployment execution: {}",
                    if op_data.op_accept == 1 {
                        "✅ Success"
                    } else {
                        "❌ Failed"
                    }
                );
                if op_data.op_accept != 1 {
                    println!("  Error message: {}", op_data.op_error);
                } else {
                    // Save state to storage
                    if let Err(e) = storage.state.save_state_batch_rocks_begin(state_map) {
                        println!("  State save failed: {}", e);
                    } else {
                        println!("  State save: ✅ Success");
                    }
                }
            }
            Err(e) => println!("  Deployment execution failed: {}", e),
        }
    }

    Ok(())
}

async fn test_pepek_mint(
    operation_manager: &OperationManager,
    test_address: &str,
    state_map: &mut DataStateMapType,
    storage: &Arc<StorageManager>,
) -> Result<()> {
    println!("  Creating PEPEK minting script...");

    // Load PEPEK token state from storage
    println!("  Loading PEPEK token state...");

    // Try different key formats
    let keys_to_try = vec!["PEPEK", "sttoken_PEPEK", "token:PEPEK"];

    let mut token_found = false;
    for key in keys_to_try {
        println!("    Trying key: {}", key);
        if let Ok(token_data) = storage.state.get_token(key) {
            if let Some(token) = token_data {
                println!("    Debug: TokenData content: {:?}", token);
                let state_token = token.into();
                println!("    Debug: StateTokenType content: {:?}", state_token);
                state_map
                    .state_token_map
                    .insert("PEPEK".to_string(), Some(state_token));
                println!("  PEPEK token state: ✅ Loaded (key: {})", key);
                token_found = true;
                break;
            }
        }
    }

    if !token_found {
        println!("  PEPEK token state: ❌ Not found");
    }

    // Debug: Display state mapping content
    println!("  State mapping debug info:");
    println!(
        "    state_token_map key count: {}",
        state_map.state_token_map.len()
    );
    for (key, value) in &state_map.state_token_map {
        println!(
            "    Key: '{}', Value: {}",
            key,
            if value.is_some() { "Some" } else { "None" }
        );
    }

    let mut mint_script = DataScriptType {
        p: "KRC-20".to_string(),
        op: "mint".to_string(),
        from: Some(test_address.to_string()),
        to: Some(test_address.to_string()),
        tick: Some("PEPEK".to_string()),
        max: Some("".to_string()),
        lim: Some("".to_string()),
        pre: Some("".to_string()),
        dec: Some("".to_string()),
        amt: Some("".to_string()),
        mod_type: "".to_string(),
        name: Some("".to_string()),
        utxo: Some("".to_string()),
        price: Some("".to_string()),
        ca: Some("".to_string()),
    };

    // Validate minting script
    let is_valid = MintOperation::validate(
        &mut mint_script,
        "tkas_mint_tx_hash_1234567890abcdef",
        110165000,
        true,
    );

    println!(
        "  Script validation: {}",
        if is_valid { "✅ Passed" } else { "❌ Failed" }
    );

    if is_valid {
        // Create operation data
        let mut op_data =
            create_operation_data("tkas_mint_tx_hash_1234567890abcdef", vec![mint_script]);

        // Prepare state key
        MintOperation::prepare_state_key(&op_data.op_script[0], state_map);

        // Check state mapping before minting
        let balance_key = format!("{}_{}", test_address, "PEPEK");
        println!("  Pre-minting state mapping check:");
        println!(
            "    state_token_map key count: {}",
            state_map.state_token_map.len()
        );
        println!(
            "    state_balance_map key count: {}",
            state_map.state_balance_map.len()
        );
        if let Some(Some(balance)) = state_map.state_balance_map.get(&balance_key) {
            println!("    Balance: {}", balance.balance);
        } else {
            println!("    Balance: Not found");
        }

        // Execute minting operation
        let result = operation_manager.execute_operation(0, &mut op_data, state_map, true);

        match result {
            Ok(_) => {
                println!(
                    "  Minting execution: {}",
                    if op_data.op_accept == 1 {
                        "✅ Success"
                    } else {
                        "❌ Failed"
                    }
                );
                if op_data.op_accept != 1 {
                    println!("  Error message: {}", op_data.op_error);
                } else {
                    // Check state mapping after minting
                    println!("  Post-minting state mapping check:");
                    println!(
                        "    state_token_map key count: {}",
                        state_map.state_token_map.len()
                    );
                    println!(
                        "    state_balance_map key count: {}",
                        state_map.state_balance_map.len()
                    );
                    if let Some(Some(balance)) = state_map.state_balance_map.get(&balance_key) {
                        println!("    Balance: {}", balance.balance);
                    } else {
                        println!("    Balance: Not found");
                    }

                    // Save state to storage
                    if let Err(e) = storage.state.save_state_batch_rocks_begin(state_map) {
                        println!("  State save failed: {}", e);
                    } else {
                        println!("  State save: ✅ Success");

                        // Check storage after saving
                        println!("  Post-save storage check:");
                        let mut balance_map = std::collections::HashMap::new();
                        balance_map.insert(balance_key.clone(), None);
                        if let Ok(_) = storage.state.get_state_balance_map(&mut balance_map) {
                            if let Some(Some(balance)) = balance_map.get(&balance_key) {
                                println!("    Balance in storage: {}", balance.balance);
                            } else {
                                println!("    Balance in storage: Not found");
                            }
                        } else {
                            println!("    Balance in storage: Load failed");
                        }
                    }
                }
            }
            Err(e) => println!("  Minting execution failed: {}", e),
        }
    }

    Ok(())
}

async fn test_pepek_transfer(
    operation_manager: &OperationManager,
    test_address: &str,
    state_map: &mut DataStateMapType,
    storage: &Arc<StorageManager>,
) -> Result<()> {
    println!("  Creating PEPEK transfer script...");

    // Load PEPEK token state from storage
    println!("  Loading PEPEK token state...");
    if let Ok(token_data) = storage.state.get_token("PEPEK") {
        if let Some(token) = token_data {
            let state_token = token.into();
            state_map
                .state_token_map
                .insert("PEPEK".to_string(), Some(state_token));
            println!("  PEPEK token state: ✅ Loaded");
        } else {
            println!("  PEPEK token state: ❌ Not found");
        }
    } else {
        println!("  PEPEK token state: ❌ Load failed");
    }

    // Create receiver address (example)
    let receiver_address =
        "kaspatest:qq8guq855gxkfrj2w25skwgj7cp4hy08x6a8mz70tdtmgv5p2ngwqxpj4cknc";

    let mut transfer_script = DataScriptType {
        p: "KRC-20".to_string(),
        op: "transfer".to_string(),
        from: Some(test_address.to_string()),
        to: Some(receiver_address.to_string()),
        tick: Some("PEPEK".to_string()),
        amt: Some("1000".to_string()), // Transfer 1000 PEPEK
        max: Some("".to_string()),
        lim: Some("".to_string()),
        pre: Some("".to_string()),
        dec: Some("".to_string()),
        utxo: Some("".to_string()),
        price: Some("".to_string()),
        mod_type: "".to_string(),
        name: Some("".to_string()),
        ca: Some("".to_string()),
    };

    // Validate transfer script
    let is_valid = TransferOperation::validate(
        &mut transfer_script,
        "tkas_transfer_tx_hash_1234567890abcdef",
        110165000,
        true,
    );

    println!(
        "  Script validation: {}",
        if is_valid { "✅ Passed" } else { "❌ Failed" }
    );
    println!(
        "  Transfer details: {} PEPEK -> {}",
        transfer_script.amt.as_ref().unwrap(),
        receiver_address
    );

    if is_valid {
        // Create operation data
        let mut op_data = create_operation_data(
            "tkas_transfer_tx_hash_1234567890abcdef",
            vec![transfer_script],
        );
        let mut state_map = DataStateMapType::new();

        // Load PEPEK token state
        println!("  Loading PEPEK token state...");
        let keys_to_try = ["PEPEK", "sttoken_PEPEK", "token:PEPEK"];

        let mut token_found = false;
        for key in keys_to_try {
            if let Ok(token_data) = storage.state.get_token(key) {
                if let Some(token) = token_data {
                    let state_token = token.into();
                    state_map
                        .state_token_map
                        .insert("PEPEK".to_string(), Some(state_token));
                    println!("  PEPEK token state: ✅ Loaded (key: {})", key);
                    token_found = true;
                    break;
                }
            }
        }

        if !token_found {
            println!("  PEPEK token state: ❌ Not found");
        }

        // Check balance status
        println!("  Checking balance status...");
        let balance_key = format!("{}_{}", test_address, "PEPEK");

        // Check if balance data already exists in current state mapping
        if let Some(Some(state_balance)) = state_map.state_balance_map.get(&balance_key) {
            println!(
                "  Balance status: ✅ Exists (key: {}, balance: {})",
                balance_key, state_balance.balance
            );
        } else {
            // If no balance data in state mapping, load from storage
            println!("  Balance status: ❌ Not found in state mapping, loading from storage...");
            let mut balance_map = std::collections::HashMap::new();
            balance_map.insert(balance_key.clone(), None);

            if let Ok(_) = storage.state.get_state_balance_map(&mut balance_map) {
                if let Some(Some(state_balance)) = balance_map.get(&balance_key) {
                    state_map
                        .state_balance_map
                        .insert(balance_key.clone(), Some(state_balance.clone()));
                    println!(
                        "  Balance status: ✅ Loaded (key: {}, balance: {})",
                        balance_key, state_balance.balance
                    );
                } else {
                    println!("  Balance status: ❌ Not found (key: {})", balance_key);
                }
            } else {
                println!("  Balance status: ❌ Load failed (key: {})", balance_key);
            }
        }

        // Prepare state key
        TransferOperation::prepare_state_key(&op_data.op_script[0], &mut state_map);

        // Execute transfer operation
        let result = operation_manager.execute_operation(0, &mut op_data, &mut state_map, true);

        match result {
            Ok(_) => {
                println!(
                    "  Transfer execution: {}",
                    if op_data.op_accept == 1 {
                        "✅ Success"
                    } else {
                        "❌ Failed"
                    }
                );
                if op_data.op_accept != 1 {
                    println!("  Error message: {}", op_data.op_error);
                }
            }
            Err(e) => println!("  Transfer execution failed: {}", e),
        }
    }

    Ok(())
}

async fn test_pepek_burn(
    operation_manager: &OperationManager,
    test_address: &str,
    storage: &Arc<StorageManager>,
) -> Result<()> {
    println!("  Creating PEPEK burn script...");

    let mut burn_script = DataScriptType {
        p: "KRC-20".to_string(),
        op: "burn".to_string(),
        from: Some(test_address.to_string()),
        to: Some(test_address.to_string()),
        tick: Some("PEPEK".to_string()),
        amt: Some("500".to_string()), // Burn 500 PEPEK
        max: Some("".to_string()),
        lim: Some("".to_string()),
        pre: Some("".to_string()),
        dec: Some("".to_string()),
        utxo: Some("".to_string()),
        price: Some("".to_string()),
        mod_type: "".to_string(),
        name: Some("".to_string()),
        ca: Some("".to_string()),
    };

    // Validate burn script
    let is_valid = BurnOperation::validate(
        &mut burn_script,
        "tkas_burn_tx_hash_1234567890abcdef",
        110165000,
        true,
    );

    println!(
        "  Script validation: {}",
        if is_valid { "✅ Passed" } else { "❌ Failed" }
    );
    println!("  Burn amount: {} PEPEK", burn_script.amt.as_ref().unwrap());

    if is_valid {
        // Create operation data
        let mut op_data =
            create_operation_data("tkas_burn_tx_hash_1234567890abcdef", vec![burn_script]);
        let mut state_map = DataStateMapType::new();

        // Load PEPEK token state
        println!("  Loading PEPEK token state...");
        let keys_to_try = ["PEPEK", "sttoken_PEPEK", "token:PEPEK"];

        let mut token_found = false;
        for key in keys_to_try {
            if let Ok(token_data) = storage.state.get_token(key) {
                if let Some(token) = token_data {
                    let state_token = token.into();
                    state_map
                        .state_token_map
                        .insert("PEPEK".to_string(), Some(state_token));
                    println!("  PEPEK token state: ✅ Loaded (key: {})", key);
                    token_found = true;
                    break;
                }
            }
        }

        if !token_found {
            println!("  PEPEK token state: ❌ Not found");
        }

        // Prepare state key
        BurnOperation::prepare_state_key(&op_data.op_script[0], &mut state_map);

        // Execute burn operation
        let result = operation_manager.execute_operation(0, &mut op_data, &mut state_map, true);

        match result {
            Ok(_) => {
                println!(
                    "  Burn execution: {}",
                    if op_data.op_accept == 1 {
                        "✅ Success"
                    } else {
                        "❌ Failed"
                    }
                );
                if op_data.op_accept != 1 {
                    println!("  Error message: {}", op_data.op_error);
                }
            }
            Err(e) => println!("  Burn execution failed: {}", e),
        }
    }

    Ok(())
}

async fn test_pepek_script_generation(_test_address: &str) -> Result<()> {
    println!("  Generating PEPEK deployment script...");

    // Create JSON data
    let json_data = format!(
        r#"{{"p":"KRC-20","op":"deploy","tick":"PEPEK","max":"1000000","lim":"1000","pre":"30000","dec":"8"}}"#
    );

    println!("  JSON data: {}", json_data);

    // Generate script hexadecimal
    let script_hex = script::make_script_hex(&json_data);
    println!("  Script hexadecimal: {}", script_hex);

    // Generate P2SH Kasplex script
    let (address, script) = script::make_p2sh_kaspa(
        "script_sig_hex_example",
        "script_pn_hex_example",
        &json_data,
        true, // testnet
    );

    println!("  Generated P2SH address: {}", address);
    println!("  Generated script: {}", script);

    Ok(())
}

async fn test_address_handling() -> Result<()> {
    println!("  Testing address validation...");

    let test_addresses = vec![
        "kaspatest:qrf7saw4vlc006mmcpfa29mrgs79ez5glszc4ytx9hq7wdu2a5d4kvl7wz5pz",
        "kaspatest:qq8guq855gxkfrj2w25skwgj7cp4hy08x6a8mz70tdtmgv5p2ngwqxpj4cknc",
        "kaspatest:invalid_address_test",
        "kaspa:mainnet_address_test",
    ];

    for addr in test_addresses {
        let is_valid = address::verify_address(addr, true);
        println!(
            "  {}: {}",
            addr,
            if is_valid { "✅ Valid" } else { "❌ Invalid" }
        );
    }

    Ok(())
}

fn create_operation_data(tx_id: &str, scripts: Vec<DataScriptType>) -> DataOperationType {
    DataOperationType {
        tx_id: tx_id.to_string(),
        daa_score: 110165000,
        block_accept: "test_block_hash".to_string(),
        fee: 100000000000, // 100 KAS
        fee_least: 100000000000,
        mts_add: 1234567890,
        op_score: 110165000,
        op_accept: 0,
        op_error: String::new(),
        op_script: scripts,
        script_sig: String::new(),
        st_before: Vec::new(),
        st_after: Vec::new(),
        checkpoint: String::new(),
        ss_info: Some(DataStatsType {
            tick_affc: Vec::new(),
            address_affc: Vec::new(),
        }),
    }
}
