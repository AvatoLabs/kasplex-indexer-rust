use anyhow::Result;
use kaspa_indexer_rust::protobuf::client::KaspaRpcClient;

#[tokio::main]
async fn main() -> Result<()> {
    println!("Kaspa Protobuf Demo");

    // Create RPC client
    let mut client = KaspaRpcClient::new();

    // Connect to Kaspa node
    match client.connect("http://127.0.0.1:16110".to_string()).await {
        Ok(_) => println!("✅ Successfully connected to Kaspa node"),
        Err(e) => {
            println!("❌ Connection failed: {}", e);
            return Ok(());
        }
    }

    // Test basic functionality
    println!("\n=== Testing Basic Functionality ===");

    // Get network information
    match client.get_current_network().await {
        Ok(network) => println!("🌐 Current network: {}", network),
        Err(e) => println!("❌ Failed to get network information: {}", e),
    }

    // Get block DAG information
    match client.get_block_dag_info().await {
        Ok(info) => println!(
            "📊 Block DAG info: Network={}, Blocks={}, Difficulty={}",
            info.network_name, info.block_count, info.difficulty
        ),
        Err(e) => println!("❌ Failed to get block DAG info: {}", e),
    }

    // Get block count
    match client.get_block_count().await {
        Ok(count) => println!("🔢 Block count: {}", count),
        Err(e) => println!("❌ Failed to get block count: {}", e),
    }

    // Get sync status
    match client.get_sync_status().await {
        Ok(status) => println!("🔄 Sync status: Synced={}", status.is_synced),
        Err(e) => println!("❌ Failed to get sync status: {}", e),
    }

    // Test Ping
    match client.ping().await {
        Ok(_) => println!("🏓 Ping successful"),
        Err(e) => println!("❌ Ping failed: {}", e),
    }

    // Test serialization tools
    println!("\n=== Testing Serialization Tools ===");

    // Test transaction serialization
    let sample_tx = kaspa_indexer_rust::protobuf::protowire::RpcTransaction::default();
    println!("📝 Transaction created successfully: {:?}", sample_tx);

    // Test block serialization
    let sample_block = kaspa_indexer_rust::protobuf::protowire::RpcBlock::default();
    println!("📦 Block created successfully: {:?}", sample_block);

    println!("\n✅ Protobuf demo completed");
    Ok(())
}
