use anyhow::Result;
use kaspa_indexer_rust::config::types::*;
use kaspa_indexer_rust::operations::handler::OperationManager;
use kaspa_indexer_rust::operations::*;
use kaspa_indexer_rust::storage::StorageManager;
use kaspa_indexer_rust::storage::types::*;
use kaspa_indexer_rust::utils::address;
use std::sync::Arc;

#[tokio::main]
async fn main() -> Result<()> {
    // Initialize logging
    tracing_subscriber::fmt::init();

    println!("🚀 Kasplex Comprehensive Operations Test");
    println!("=========================================");

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
                timeout: 5,
                max_retries: 3,
                retry_interval: 1,
                enable_compression: false,
                compression_level: 1,
            },
            performance: PerformanceConfig::default(),
            monitoring: MonitoringConfig::default(),
            security: SecurityConfig::default(),
        },
        http: HttpConfig::default(),
        rest: RestConfig::default(),
        debug: 0,
        testnet: true,
        is_testnet: true,
    };

    // Test address information
    let owner_address = "kaspatest:qrf7saw4vlc006mmcpfa29mrgs79ez5glszc4ytx9hq7wdu2a5d4kvl7wz5pz";
    let user_address = "kaspatest:qq8guq855gxkfrj2w25skwgj7cp4hy08x6a8mz70tdtmgv5p2ngwqxpj4cknc";

    println!("\n📋 Testnet Configuration:");
    println!("  Network: Testnet");
    println!("  Node URL: {}", testnet_config.startup.kaspa_node_url);
    println!("  Data Directory: {}", testnet_config.rocksdb.path);

    println!("\n💰 Test Address Information:");
    println!("  Owner Address: {}", owner_address);
    println!("  User Address: {}", user_address);

    // Verify addresses
    let is_valid_owner = address::verify_address(owner_address, true);
    let is_valid_user = address::verify_address(user_address, true);
    println!(
        "  Owner Address Validation: {}",
        if is_valid_owner {
            "✅ Valid"
        } else {
            "❌ Invalid"
        }
    );
    println!(
        "  User Address Validation: {}",
        if is_valid_user {
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
    let operation_manager = OperationManager::new(storage.clone());
    println!("✅ Operation manager initialization completed");

    // Get supported operation types
    let supported_ops = operation_manager.get_supported_operations();
    println!("📝 Supported operation types: {:?}", supported_ops);

    // Create shared state map
    let mut state_map = DataStateMapType::new();

    // Test 1: Token deployment
    println!("\n🔨 Test 1: Token Deployment");
    test_deploy_operation(&operation_manager, owner_address, &mut state_map, &storage).await?;

    // Test 2: Token minting
    println!("\n⛏️ Test 2: Token Minting");
    test_mint_operation(&operation_manager, owner_address, &mut state_map, &storage).await?;

    // Test 3: Token transfer
    println!("\n💸 Test 3: Token Transfer");
    test_transfer_operation(
        &operation_manager,
        owner_address,
        user_address,
        &mut state_map,
        &storage,
    )
    .await?;

    // Test 4: Token issuance
    println!("\n📈 Test 4: Token Issuance");
    test_issue_operation(&operation_manager, owner_address, &mut state_map, &storage).await?;

    // Test 5: Ownership transfer
    println!("\n👑 Test 5: Ownership Transfer");
    test_chown_operation(
        &operation_manager,
        owner_address,
        user_address,
        &mut state_map,
        &storage,
    )
    .await?;

    // Test 6: Token burning
    println!("\n🔥 Test 6: Token Burning");
    test_burn_operation(&operation_manager, user_address, &storage).await?;

    println!("\n🎉 Comprehensive operations test completed!");
    Ok(())
}

async fn test_deploy_operation(
    operation_manager: &OperationManager,
    owner_address: &str,
    state_map: &mut DataStateMapType,
    storage: &Arc<StorageManager>,
) -> Result<()> {
    println!("  Creating token deployment script...");

    let mut deploy_script = DataScriptType {
        p: "KRC-20".to_string(),
        op: "deploy".to_string(),
        from: Some(owner_address.to_string()),
        to: Some(owner_address.to_string()),
        tick: Some("TEST".to_string()),
        max: Some("1000000".to_string()),
        lim: Some("1000".to_string()),
        pre: Some("10000".to_string()),
        dec: Some("8".to_string()),
        amt: None,
        name: None,
        mod_type: "".to_string(),
        ca: None,
        price: None,
        utxo: None,
    };

    // Validate deployment script
    let is_valid = DeployOperation::validate(
        &mut deploy_script,
        "test_deploy_tx_hash_1234567890abcdef",
        110165000,
        true,
    );

    println!(
        "  Script validation: {}",
        if is_valid { "✅ Passed" } else { "❌ Failed" }
    );

    if is_valid {
        let mut op_data =
            create_operation_data("test_deploy_tx_hash_1234567890abcdef", vec![deploy_script]);
        DeployOperation::prepare_state_key(&op_data.op_script[0], state_map);
        let result = operation_manager.execute_operation(0, &mut op_data, state_map, true);

        println!(
            "  Deployment execution: {}",
            if result.is_ok() {
                "✅ Success"
            } else {
                "❌ Failed"
            }
        );

        if let Err(e) = result {
            println!("  Error message: {}", e);
        }

        // Save state
        if let Err(e) = storage.state.save_state_batch_rocks_begin(state_map) {
            println!("  State save failed: {}", e);
        } else {
            println!("  State save: ✅ Success");
        }
    }

    Ok(())
}

async fn test_mint_operation(
    operation_manager: &OperationManager,
    owner_address: &str,
    state_map: &mut DataStateMapType,
    storage: &Arc<StorageManager>,
) -> Result<()> {
    println!("  Creating token minting script...");

    let mut mint_script = DataScriptType {
        p: "KRC-20".to_string(),
        op: "mint".to_string(),
        from: Some(owner_address.to_string()),
        to: Some(owner_address.to_string()),
        tick: Some("TEST".to_string()),
        max: None,
        lim: None,
        pre: None,
        dec: None,
        amt: Some("1000".to_string()),
        name: None,
        mod_type: "".to_string(),
        ca: None,
        price: None,
        utxo: None,
    };

    // Validate minting script
    let is_valid = MintOperation::validate(
        &mut mint_script,
        "test_mint_tx_hash_1234567890abcdef",
        110165000,
        true,
    );

    println!(
        "  Script validation: {}",
        if is_valid { "✅ Passed" } else { "❌ Failed" }
    );

    if is_valid {
        let mut op_data =
            create_operation_data("test_mint_tx_hash_1234567890abcdef", vec![mint_script]);
        MintOperation::prepare_state_key(&op_data.op_script[0], state_map);
        let result = operation_manager.execute_operation(0, &mut op_data, state_map, true);

        println!(
            "  Minting execution: {}",
            if result.is_ok() {
                "✅ Success"
            } else {
                "❌ Failed"
            }
        );

        if let Err(e) = result {
            println!("  Error message: {}", e);
        }

        // Save state
        if let Err(e) = storage.state.save_state_batch_rocks_begin(state_map) {
            println!("  State save failed: {}", e);
        } else {
            println!("  State save: ✅ Success");
        }
    }

    Ok(())
}

async fn test_transfer_operation(
    operation_manager: &OperationManager,
    from_address: &str,
    to_address: &str,
    state_map: &mut DataStateMapType,
    storage: &Arc<StorageManager>,
) -> Result<()> {
    println!("  Creating token transfer script...");

    let mut transfer_script = DataScriptType {
        p: "KRC-20".to_string(),
        op: "transfer".to_string(),
        from: Some(from_address.to_string()),
        to: Some(to_address.to_string()),
        tick: Some("TEST".to_string()),
        max: None,
        lim: None,
        pre: None,
        dec: None,
        amt: Some("500".to_string()),
        name: None,
        mod_type: "".to_string(),
        ca: None,
        price: None,
        utxo: None,
    };

    // Validate transfer script
    let is_valid = TransferOperation::validate(
        &mut transfer_script,
        "test_transfer_tx_hash_1234567890abcdef",
        110165000,
        true,
    );

    println!(
        "  Script validation: {}",
        if is_valid { "✅ Passed" } else { "❌ Failed" }
    );

    if is_valid {
        let mut op_data = create_operation_data(
            "test_transfer_tx_hash_1234567890abcdef",
            vec![transfer_script],
        );
        TransferOperation::prepare_state_key(&op_data.op_script[0], state_map);
        let result = operation_manager.execute_operation(0, &mut op_data, state_map, true);

        println!(
            "  Transfer execution: {}",
            if result.is_ok() {
                "✅ Success"
            } else {
                "❌ Failed"
            }
        );

        if let Err(e) = result {
            println!("  Error message: {}", e);
        }

        // Save state
        if let Err(e) = storage.state.save_state_batch_rocks_begin(state_map) {
            println!("  State save failed: {}", e);
        } else {
            println!("  State save: ✅ Success");
        }
    }

    Ok(())
}

async fn test_issue_operation(
    operation_manager: &OperationManager,
    owner_address: &str,
    state_map: &mut DataStateMapType,
    storage: &Arc<StorageManager>,
) -> Result<()> {
    println!("  Creating token issuance script...");

    let mut issue_script = DataScriptType {
        p: "KRC-20".to_string(),
        op: "issue".to_string(),
        from: Some(owner_address.to_string()),
        to: Some(owner_address.to_string()),
        tick: Some("TEST".to_string()),
        max: None,
        lim: None,
        pre: None,
        dec: None,
        amt: Some("2000".to_string()),
        name: None,
        mod_type: "".to_string(),
        ca: None,
        price: None,
        utxo: None,
    };

    // Validate issuance script
    let is_valid = IssueOperation::validate(
        &mut issue_script,
        "test_issue_tx_hash_1234567890abcdef",
        110165000,
        true,
    );

    println!(
        "  Script validation: {}",
        if is_valid { "✅ Passed" } else { "❌ Failed" }
    );

    if is_valid {
        let mut op_data =
            create_operation_data("test_issue_tx_hash_1234567890abcdef", vec![issue_script]);
        IssueOperation::prepare_state_key(&op_data.op_script[0], state_map);
        let result = operation_manager.execute_operation(0, &mut op_data, state_map, true);

        println!(
            "  Issuance execution: {}",
            if result.is_ok() {
                "✅ Success"
            } else {
                "❌ Failed"
            }
        );

        if let Err(e) = result {
            println!("  Error message: {}", e);
        }

        // Save state
        if let Err(e) = storage.state.save_state_batch_rocks_begin(state_map) {
            println!("  State save failed: {}", e);
        } else {
            println!("  State save: ✅ Success");
        }
    }

    Ok(())
}

async fn test_chown_operation(
    operation_manager: &OperationManager,
    from_address: &str,
    to_address: &str,
    state_map: &mut DataStateMapType,
    storage: &Arc<StorageManager>,
) -> Result<()> {
    println!("  Creating ownership transfer script...");

    let mut chown_script = DataScriptType {
        p: "KRC-20".to_string(),
        op: "chown".to_string(),
        from: Some(from_address.to_string()),
        to: Some(to_address.to_string()),
        tick: Some("TEST".to_string()),
        max: None,
        lim: None,
        pre: None,
        dec: None,
        amt: None,
        name: None,
        mod_type: "".to_string(),
        ca: None,
        price: None,
        utxo: None,
    };

    // Validate ownership transfer script
    let is_valid = ChownOperation::validate(
        &mut chown_script,
        "test_chown_tx_hash_1234567890abcdef",
        110165000,
        true,
    );

    println!(
        "  Script validation: {}",
        if is_valid { "✅ Passed" } else { "❌ Failed" }
    );

    if is_valid {
        let mut op_data =
            create_operation_data("test_chown_tx_hash_1234567890abcdef", vec![chown_script]);
        ChownOperation::prepare_state_key(&op_data.op_script[0], state_map);
        let result = operation_manager.execute_operation(0, &mut op_data, state_map, true);

        println!(
            "  Ownership transfer execution: {}",
            if result.is_ok() {
                "✅ Success"
            } else {
                "❌ Failed"
            }
        );

        if let Err(e) = result {
            println!("  Error message: {}", e);
        }

        // Save state
        if let Err(e) = storage.state.save_state_batch_rocks_begin(state_map) {
            println!("  State save failed: {}", e);
        } else {
            println!("  State save: ✅ Success");
        }
    }

    Ok(())
}

async fn test_burn_operation(
    operation_manager: &OperationManager,
    user_address: &str,
    storage: &Arc<StorageManager>,
) -> Result<()> {
    println!("  Creating token burning script...");

    let mut burn_script = DataScriptType {
        p: "KRC-20".to_string(),
        op: "burn".to_string(),
        from: Some(user_address.to_string()),
        to: Some(user_address.to_string()),
        tick: Some("TEST".to_string()),
        max: None,
        lim: None,
        pre: None,
        dec: None,
        amt: Some("100".to_string()),
        name: None,
        mod_type: "".to_string(),
        ca: None,
        price: None,
        utxo: None,
    };

    // Validate burning script
    let is_valid = BurnOperation::validate(
        &mut burn_script,
        "test_burn_tx_hash_1234567890abcdef",
        110165000,
        true,
    );

    println!(
        "  Script validation: {}",
        if is_valid { "✅ Passed" } else { "❌ Failed" }
    );

    if is_valid {
        let mut state_map = DataStateMapType::new();
        let mut op_data =
            create_operation_data("test_burn_tx_hash_1234567890abcdef", vec![burn_script]);
        BurnOperation::prepare_state_key(&op_data.op_script[0], &mut state_map);
        let result = operation_manager.execute_operation(0, &mut op_data, &mut state_map, true);

        println!(
            "  Burning execution: {}",
            if result.is_ok() {
                "✅ Success"
            } else {
                "❌ Failed"
            }
        );

        if let Err(e) = result {
            println!("  Error message: {}", e);
        }

        // Save state
        if let Err(e) = storage.state.save_state_batch_rocks_begin(&state_map) {
            println!("  State save failed: {}", e);
        } else {
            println!("  State save: ✅ Success");
        }
    }

    Ok(())
}

fn create_operation_data(tx_id: &str, scripts: Vec<DataScriptType>) -> DataOperationType {
    DataOperationType {
        tx_id: tx_id.to_string(),
        daa_score: 110165000,
        block_accept: "".to_string(),
        fee: 0,
        fee_least: 0,
        mts_add: 1234567890,
        op_score: 110165000,
        op_accept: 0,
        op_error: "".to_string(),
        op_script: scripts,
        script_sig: "".to_string(),
        st_before: vec![],
        st_after: vec![],
        checkpoint: "".to_string(),
        ss_info: None,
    }
}
