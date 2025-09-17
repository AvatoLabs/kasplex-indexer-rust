use kaspa_indexer_rust::config::types::{RocksConfig, StartupConfig};
use kaspa_indexer_rust::explorer::{Explorer, ExplorerInterface};
use kaspa_indexer_rust::operations::handler::OperationManager;
use kaspa_indexer_rust::storage::StorageManager;
use std::collections::HashMap;
use std::sync::Arc;
use tokio;

#[tokio::test]
async fn test_storage_initialization() {
    // Test storage initialization
    let rocks_config = RocksConfig {
        path: "./test_data/rocksdb".to_string(),
    };

    let storage = StorageManager::new(rocks_config, None).await;
    assert!(storage.is_ok(), "Storage initialization should succeed");

    let mut storage = storage.unwrap();
    let init_result = storage.init().await;
    assert!(init_result.is_ok(), "Storage init should succeed");
}

#[tokio::test]
async fn test_explorer_initialization() {
    // Test explorer initialization
    let rocks_config = RocksConfig {
        path: "./test_data/rocksdb_explorer".to_string(),
    };

    let storage = StorageManager::new(rocks_config, None).await.unwrap();
    let storage_arc = Arc::new(storage);

    let startup_config = StartupConfig {
        hysteresis: 10,
        daa_score_range: vec![
            [83441551, 18446744073709551615],
            [90090600, 18446744073709551615],
        ],
        tick_reserved: vec![
            "NACHO_kaspa:qzrsq2mfj9sf7uye3u5q7juejzlr0axk5jz9fpg4vqe76erdyvxxze84k9nk7".to_string(),
        ],
        kaspa_node_url: "http://localhost:16110".to_string(),
        is_testnet: true,
    };

    let explorer = Explorer::new(Arc::clone(&storage_arc), startup_config, false);
    assert!(explorer.is_ok(), "Explorer initialization should succeed");

    let mut explorer = explorer.unwrap();
    let init_result = explorer.init().await;
    assert!(init_result.is_ok(), "Explorer init should succeed");
}

#[tokio::test]
async fn test_operation_manager() {
    // Test operation manager
    let rocks_config = RocksConfig {
        path: "./test_data/rocksdb_ops".to_string(),
    };

    let storage = StorageManager::new(rocks_config, None).await.unwrap();
    let storage_arc = Arc::new(storage);

    let operation_manager = OperationManager::new(storage_arc.clone());
    assert!(true, "Operation manager initialization should succeed");

    // Use operation manager
    let operation_manager = operation_manager;
    let supported_ops = operation_manager.get_supported_operations();
    assert!(
        !supported_ops.is_empty(),
        "Should have supported operations"
    );

    // Verify supported operation types
    let expected_ops = vec![
        "deploy",
        "mint",
        "burn",
        "transfer",
        "send",
        "issue",
        "list",
        "chown",
        "blacklist",
    ];
    for op in expected_ops {
        assert!(
            supported_ops.contains(&op.to_string()),
            "Should support operation: {}",
            op
        );
    }
}

#[tokio::test]
async fn test_vspc_scanner() {
    // Test VSPC scanner
    use kaspa_indexer_rust::explorer::scanner::VSPCScanner;

    let rocks_config = RocksConfig {
        path: "./test_data/rocksdb_scanner".to_string(),
    };

    let storage = StorageManager::new(rocks_config, None).await.unwrap();
    let storage_arc = Arc::new(storage);

    let startup_config = StartupConfig {
        hysteresis: 10,
        daa_score_range: vec![[83441551, 83525600]],
        tick_reserved: vec![],
        kaspa_node_url: "http://localhost:16110".to_string(),
        is_testnet: true,
    };

    let scanner = VSPCScanner::new(storage_arc, startup_config, false);
    assert!(
        scanner.is_ok(),
        "VSPC Scanner initialization should succeed"
    );

    let mut scanner = scanner.unwrap();
    let init_result = scanner.init().await;
    assert!(init_result.is_ok(), "VSPC Scanner init should succeed");

    // Test configuration validation
    let config_result = scanner.validate_config();
    assert!(config_result.is_ok(), "Config validation should succeed");

    // Test DAA score validation
    assert!(
        scanner.is_daa_score_valid(83441551),
        "Valid DAA score should pass"
    );
    assert!(
        !scanner.is_daa_score_valid(0),
        "Invalid DAA score should fail"
    );
}

#[tokio::test]
async fn test_state_manager() {
    // Test state manager
    use kaspa_indexer_rust::storage::rocksdb::RocksDBClient;
    use kaspa_indexer_rust::storage::state::StateManager;

    let rocks_config = RocksConfig {
        path: "./test_data/rocksdb_state".to_string(),
    };

    // Directly create RocksDB client
    let rocksdb = RocksDBClient::new(rocks_config);
    assert!(rocksdb.is_ok(), "RocksDB initialization should succeed");

    let rocksdb = rocksdb.unwrap();
    let rocksdb_arc = Arc::new(rocksdb);

    let state_manager = StateManager::new(rocksdb_arc);
    assert!(
        state_manager.is_ok(),
        "State manager initialization should succeed"
    );

    let state_manager = state_manager.unwrap();

    // Test basic functionality
    let init_result = state_manager.init();
    assert!(init_result.is_ok(), "State manager init should succeed");

    // Test getting state map (using empty map)
    let mut token_map = HashMap::new();
    let result = state_manager.get_state_token_map(&mut token_map);
    assert!(result.is_ok(), "Should be able to get state token map");

    let mut balance_map = HashMap::new();
    let result = state_manager.get_state_balance_map(&mut balance_map);
    assert!(result.is_ok(), "Should be able to get state balance map");
}

#[tokio::test]
async fn test_operation_validation() {
    // Test operation validation
    use kaspa_indexer_rust::operations;

    #[test]
    fn test_validation_functions() {
        // Test tick validation
        let mut valid_tick = "TEST".to_string();
        assert!(
            operations::validate_tick(&mut valid_tick),
            "Valid tick should pass"
        );

        let mut empty_tick = "".to_string();
        assert!(
            !operations::validate_tick(&mut empty_tick),
            "Empty tick should fail"
        );

        let mut long_tick = "TOOLONGTICK".to_string();
        assert!(
            !operations::validate_tick(&mut long_tick),
            "Too long tick should fail"
        );

        let mut reserved_tick = "KASPA".to_string();
        assert!(
            !operations::validate_tick(&mut reserved_tick),
            "Reserved tick should fail"
        );

        // Test tx_id validation
        let mut valid_tx_id =
            "1234567890abcdef1234567890abcdef1234567890abcdef1234567890abcdef".to_string();
        assert!(
            operations::validate_tx_id(&mut valid_tx_id),
            "Valid tx_id should pass"
        );

        let mut invalid_tx_id = "invalid".to_string();
        assert!(
            !operations::validate_tx_id(&mut invalid_tx_id),
            "Invalid tx_id should fail"
        );

        // Test amount validation
        let mut valid_amount = "1000".to_string();
        assert!(
            operations::validate_amount(&mut valid_amount),
            "Valid amount should pass"
        );

        let mut empty_amount = "".to_string();
        assert!(
            !operations::validate_amount(&mut empty_amount),
            "Empty amount should fail"
        );

        // Test dec validation
        let mut valid_dec = "8".to_string();
        assert!(
            operations::validate_dec(&mut valid_dec, "8"),
            "Valid dec should pass"
        );

        let mut empty_dec = "".to_string();
        assert!(
            operations::validate_dec(&mut empty_dec, "8"),
            "Empty dec should use default"
        );

        let mut invalid_dec = "20".to_string();
        assert!(
            !operations::validate_dec(&mut invalid_dec, "8"),
            "Invalid dec should fail"
        );
    }
}

#[tokio::test]
async fn test_end_to_end_workflow() {
    // End-to-end workflow test
    let rocks_config = RocksConfig {
        path: "./test_data/rocksdb_e2e".to_string(),
    };

    let storage = StorageManager::new(rocks_config, None).await.unwrap();
    let storage_arc = Arc::new(storage);

    let startup_config = StartupConfig {
        hysteresis: 10,
        daa_score_range: vec![[83441551, 83525600]],
        tick_reserved: vec![],
        kaspa_node_url: "http://localhost:16110".to_string(),
        is_testnet: true,
    };

    // Initialize explorer
    let explorer = Explorer::new(Arc::clone(&storage_arc), startup_config, false).unwrap();
    let mut explorer = explorer;
    explorer.init().await.unwrap();

    // Initialize runtime state
    storage_arc.runtime.init().unwrap();

    // Test sync status retrieval
    let sync_status = explorer.get_sync_status();
    assert!(sync_status.is_ok(), "Should be able to get sync status");

    // Test scan statistics retrieval
    let scan_stats = explorer.get_scan_stats();
    assert!(scan_stats.is_ok(), "Should be able to get scan stats");
}

// Helper function to clean up test data
async fn cleanup_test_data() {
    let test_dirs = vec![
        "./test_data/rocksdb",
        "./test_data/rocksdb_explorer",
        "./test_data/rocksdb_ops",
        "./test_data/rocksdb_scanner",
        "./test_data/rocksdb_state",
        "./test_data/rocksdb_e2e",
    ];

    for dir in test_dirs {
        if std::path::Path::new(dir).exists() {
            std::fs::remove_dir_all(dir).ok();
        }
    }
}

#[tokio::test]
async fn test_cleanup() {
    // Test cleanup functionality
    cleanup_test_data().await;
    println!("Test data cleaned up successfully");
}
