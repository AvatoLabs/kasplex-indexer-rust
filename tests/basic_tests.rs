use kaspa_indexer_rust::config::types::*;
use kaspa_indexer_rust::storage::types::*;

#[test]
fn test_config_defaults() {
    let config = Config::default();

    assert_eq!(config.debug, 2);
    assert_eq!(config.testnet, false);
    assert_eq!(config.startup.hysteresis, 3);
    assert_eq!(config.rocksdb.path, "./data");
}

#[test]
fn test_token_data_serialization() {
    let token = TokenData {
        tick: "TEST".to_string(),
        max_supply: 1000000,
        circulating_supply: 500000,
        decimals: 18,
        owner: "test_address".to_string(),
        is_blacklisted: false,
        is_reserved: false,
        deploy_tx_hash: "tx_hash".to_string(),
        deploy_block_hash: "block_hash".to_string(),
        deploy_timestamp: 1234567890,
        last_updated: 1234567890,
        minted_supply: "500000".to_string(),
        mode: "deploy".to_string(),
        lim: None,
        pre: None,
    };

    let json = serde_json::to_string(&token).unwrap();
    let deserialized: TokenData = serde_json::from_str(&json).unwrap();

    assert_eq!(token.tick, deserialized.tick);
    assert_eq!(token.max_supply, deserialized.max_supply);
    assert_eq!(token.circulating_supply, deserialized.circulating_supply);
}

#[test]
fn test_balance_data_serialization() {
    let balance = BalanceData {
        address: "test_address".to_string(),
        tick: "TEST".to_string(),
        balance: 1000,
        last_updated: 1234567890,
        locked: "0".to_string(),
    };

    let json = serde_json::to_string(&balance).unwrap();
    let deserialized: BalanceData = serde_json::from_str(&json).unwrap();

    assert_eq!(balance.address, deserialized.address);
    assert_eq!(balance.tick, deserialized.tick);
    assert_eq!(balance.balance, deserialized.balance);
}

#[test]
fn test_operation_data_serialization() {
    let operation = OperationData {
        operation_type: "mint".to_string(),
        tick: "TEST".to_string(),
        from_address: Some("from_address".to_string()),
        to_address: Some("to_address".to_string()),
        amount: Some(1000),
        tx_hash: "tx_hash".to_string(),
        block_hash: "block_hash".to_string(),
        timestamp: 1234567890,
        block_daa_score: 1000,
        script: None,
        is_testnet: false,
        daa_score: 1000,
        tx_id: "tx_hash".to_string(),
        ca: None,
    };

    let json = serde_json::to_string(&operation).unwrap();
    let deserialized: OperationData = serde_json::from_str(&json).unwrap();

    assert_eq!(operation.operation_type, deserialized.operation_type);
    assert_eq!(operation.tick, deserialized.tick);
    assert_eq!(operation.amount, deserialized.amount);
}

#[test]
fn test_storage_operation_clone() {
    let token = TokenData {
        tick: "TEST".to_string(),
        max_supply: 1000000,
        circulating_supply: 500000,
        decimals: 18,
        owner: "test_address".to_string(),
        is_blacklisted: false,
        is_reserved: false,
        deploy_tx_hash: "tx_hash".to_string(),
        deploy_block_hash: "block_hash".to_string(),
        deploy_timestamp: 1234567890,
        last_updated: 1234567890,
        minted_supply: "500000".to_string(),
        mode: "deploy".to_string(),
        lim: None,
        pre: None,
    };

    let op = StorageOperation::UpdateToken(token);
    let cloned_op = op.clone();

    match (op, cloned_op) {
        (StorageOperation::UpdateToken(t1), StorageOperation::UpdateToken(t2)) => {
            assert_eq!(t1.tick, t2.tick);
            assert_eq!(t1.max_supply, t2.max_supply);
        }
        _ => panic!("Expected UpdateToken operations"),
    }
}
