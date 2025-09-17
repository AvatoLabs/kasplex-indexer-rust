use kaspa_indexer_rust::config::types::*;
use kaspa_indexer_rust::storage::types::*;
use kaspa_indexer_rust::utils;
use tracing::info;

fn main() {
    println!("Kasplex Indexer Executor - Rust Demo");
    println!("=====================================");

    // Demo configuration
    let config = Config::default();
    println!("Default config loaded:");
    println!("  Debug level: {}", config.debug);
    println!("  Testnet: {}", config.testnet);
    println!("  RocksDB path: {}", config.rocksdb.path);

    // Demo token data
    let token = TokenData {
        tick: "DEMO".to_string(),
        max_supply: 1000000,
        circulating_supply: 500000,
        decimals: 18,
        owner: "demo_owner".to_string(),
        is_blacklisted: false,
        is_reserved: false,
        deploy_tx_hash: "demo_tx_hash".to_string(),
        deploy_block_hash: "demo_block_hash".to_string(),
        deploy_timestamp: 1234567890,
        mode: "deploy".to_string(),
        minted_supply: "500000".to_string(),
        last_updated: 1234567890,
        lim: Some("0".to_string()),
        pre: Some("0".to_string()),
    };

    println!("\nToken data:");
    println!("  Tick: {}", token.tick);
    println!("  Max supply: {}", token.max_supply);
    println!("  Circulating supply: {}", token.circulating_supply);
    println!("  Owner: {}", token.owner);

    // Demo balance data
    let balance = BalanceData {
        address: "demo_address".to_string(),
        tick: "DEMO".to_string(),
        balance: 1000,
        last_updated: 1234567890,
        locked: "0".to_string(),
    };

    println!("\nBalance data:");
    println!("  Address: {}", balance.address);
    println!("  Tick: {}", balance.tick);
    println!("  Balance: {}", balance.balance);

    // Demo operation data
    let operation = OperationData {
        operation_type: "mint".to_string(),
        tick: "DEMO".to_string(),
        from_address: Some("mint_from".to_string()),
        to_address: Some("mint_to".to_string()),
        amount: Some(100),
        tx_hash: "mint_tx_hash".to_string(),
        block_hash: "mint_block_hash".to_string(),
        timestamp: 1234567890,
        block_daa_score: 1000,
        script: None,
        is_testnet: false,
        tx_id: "mint_tx_hash".to_string(),
        daa_score: 1000,
        ca: None,
    };

    println!("\nOperation data:");
    println!("  Type: {}", operation.operation_type);
    println!("  Tick: {}", operation.tick);
    println!("  Amount: {:?}", operation.amount);
    println!("  From: {:?}", operation.from_address);
    println!("  To: {:?}", operation.to_address);

    // Demo utility functions
    println!("\nUtility functions:");
    info!("📊 Performance statistics:");
    info!("  Processing time: {}ms", 3661);
    info!("  Memory usage: {} bytes", 1024 * 1024);

    // Demo serialization
    let token_json = serde_json::to_string_pretty(&token).unwrap();
    println!("\nToken JSON:");
    println!("{}", token_json);

    println!("\nDemo completed successfully!");
}
