use anyhow::Result;
use kaspa_indexer_rust::protobuf::client::KaspaRpcClient;
use serde_json::json;

/// Frontend gRPC call demo
/// Demonstrates how frontend calls indexer service via gRPC
#[tokio::main]
async fn main() -> Result<()> {
    // Initialize logging
    tracing_subscriber::fmt::init();

    println!("🌐 Frontend gRPC Call Demo");
    println!("===========================");

    // 1. Create gRPC client
    println!("\n📡 Step 1: Create gRPC client");
    let mut client = KaspaRpcClient::new();

    // 2. Connect to Kaspa node
    println!("\n🔗 Step 2: Connect to Kaspa node");
    let endpoint = "https://testnet.kaspa.org:16210";
    match client.connect(endpoint.to_string()).await {
        Ok(_) => println!("  ✅ Successfully connected to Kaspa node: {}", endpoint),
        Err(e) => {
            println!("  ❌ Connection failed: {}", e);
            return Ok(());
        }
    }

    // 3. Get network information
    println!("\n🌐 Step 3: Get network information");
    match client.get_current_network().await {
        Ok(network) => println!("  ✅ Current network: {}", network),
        Err(e) => println!("  ❌ Failed to get network information: {}", e),
    }

    // 4. Get block information
    println!("\n📊 Step 4: Get block information");
    match client.get_block_count().await {
        Ok(count) => println!("  ✅ Block count: {}", count),
        Err(e) => println!("  ❌ Failed to get block count: {}", e),
    }

    match client.get_block_dag_info().await {
        Ok(info) => println!(
            "  ✅ Block DAG info: Network={}, Blocks={}, Difficulty={}",
            info.network_name, info.block_count, info.difficulty
        ),
        Err(e) => println!("  ❌ Failed to get block DAG info: {}", e),
    }

    // 5. Get sync status
    println!("\n🔄 Step 5: Get sync status");
    match client.get_sync_status().await {
        Ok(status) => println!("  ✅ Sync status: Synced={}", status.is_synced),
        Err(e) => println!("  ❌ Failed to get sync status: {}", e),
    }

    // 6. Test Ping
    println!("\n🏓 Step 6: Test connection");
    match client.ping().await {
        Ok(_) => println!("  ✅ Ping successful"),
        Err(e) => println!("  ❌ Ping failed: {}", e),
    }

    // 7. Get wallet balance
    println!("\n💰 Step 7: Get wallet balance");
    let wallet_address = "kaspatest:qrf7saw4vlc006mmcpfa29mrgs79ez5glszc4ytx9hq7wdu2a5d4kvl7wz5pz";
    match client
        .get_balance_by_address(wallet_address.to_string())
        .await
    {
        Ok(balance) => println!("  ✅ Wallet balance: {} KAS", balance),
        Err(e) => println!("  ❌ Failed to get balance: {}", e),
    }

    // 8. Get UTXO information
    println!("\n🔍 Step 8: Get UTXO information");
    let addresses = vec![wallet_address.to_string()];
    match client.get_utxos_by_addresses(addresses).await {
        Ok(utxos) => {
            println!("  ✅ UTXO count: {}", utxos.len());
            for (i, utxo) in utxos.iter().enumerate().take(3) {
                println!(
                    "    UTXO {}: Address={}, Amount={}",
                    i + 1,
                    utxo.address,
                    utxo.utxo_entry.as_ref().map_or(0, |e| e.amount)
                );
            }
        }
        Err(e) => println!("  ❌ Failed to get UTXOs: {}", e),
    }

    // 9. Get mempool information
    println!("\n📋 Step 9: Get mempool information");
    match client.get_mempool_entries().await {
        Ok(entries) => {
            println!("  ✅ Mempool transaction count: {}", entries.len());
            for (i, entry) in entries.iter().enumerate().take(3) {
                println!("    Transaction {}: ID={}, Fee={}", i + 1, "unknown", entry.fee);
            }
        }
        Err(e) => println!("  ❌ Failed to get mempool: {}", e),
    }

    // 10. Simulate frontend calling indexer for KRC-20 queries
    println!("\n🪙 Step 10: Simulate frontend KRC-20 queries");
    simulate_frontend_krc20_query(&mut client).await?;

    // 11. Disconnect
    println!("\n🔌 Step 11: Disconnect");
    match client.disconnect().await {
        Ok(_) => println!("  ✅ Connection disconnected"),
        Err(e) => println!("  ❌ Failed to disconnect: {}", e),
    }

    println!("\n🎉 Frontend gRPC call demo completed!");
    println!("\n📋 Frontend call summary:");
    println!("  1. Frontend connects to Kaspa node via gRPC client");
    println!("  2. Uses various methods provided by KaspaRpcClient");
    println!("  3. Gets blockchain data, balances, UTXOs, etc.");
    println!("  4. Queries KRC-20 token status and transactions");
    println!("  5. Submits transactions to Kaspa network");

    Ok(())
}

/// Simulate frontend KRC-20 queries
async fn simulate_frontend_krc20_query(_client: &mut KaspaRpcClient) -> Result<()> {
    println!("  🔍 Querying DRAGON token status...");

    // Simulate querying token information
    let token_info = json!({
        "tick": "DRAGON",
        "name": "Dragon Token",
        "max_supply": 1000000,
        "decimals": 8,
        "owner": "kaspatest:qrf7saw4vlc006mmcpfa29mrgs79ez5glszc4ytx9hq7wdu2a5d4kvl7wz5pz",
        "circulating_supply": 100000,
        "is_blacklisted": false
    });

    println!(
        "    ✅ Token information: {}",
        serde_json::to_string_pretty(&token_info)?
    );

    // Simulate querying token balance
    let balance_info = json!({
        "address": "kaspatest:qrf7saw4vlc006mmcpfa29mrgs79ez5glszc4ytx9hq7wdu2a5d4kvl7wz5pz",
        "tick": "DRAGON",
        "balance": 100000,
        "available": 100000,
        "locked": 0
    });

    println!(
        "    ✅ Token balance: {}",
        serde_json::to_string_pretty(&balance_info)?
    );

    // Simulate querying transaction history
    let tx_history = json!([
        {
            "tx_id": "tx_1234567890abcdef",
            "type": "issue",
            "tick": "DRAGON",
            "amount": 100000,
            "timestamp": 1757050180,
            "block_hash": "block_abcdef1234567890",
            "status": "confirmed"
        },
        {
            "tx_id": "tx_abcdef1234567890",
            "type": "send",
            "tick": "DRAGON",
            "amount": 10000,
            "timestamp": 1757050185,
            "block_hash": "block_1234567890abcdef",
            "status": "confirmed"
        }
    ]);

    println!(
        "    ✅ Transaction history: {}",
        serde_json::to_string_pretty(&tx_history)?
    );

    Ok(())
}

/// Frontend JavaScript call example
fn generate_frontend_example() {
    println!("\n💻 Frontend JavaScript call example:");
    println!("
// 1. Install dependencies
// npm install @grpc/grpc-js @grpc/proto-loader

// 2. Create gRPC client
const grpc = require('@grpc/grpc-js');
const protoLoader = require('@grpc/proto-loader');

const packageDefinition = protoLoader.loadSync('protowire.proto', {{
    keepCase: true,
    longs: String,
    enums: String,
    defaults: true,
    oneofs: true
}});

const protowire = grpc.loadPackageDefinition(packageDefinition).protowire;
const client = new protowire.RPC('https://testnet.kaspa.org:16210', 
    grpc.credentials.createInsecure());

// 3. Call gRPC methods
async function getBlockCount() {{
    return new Promise((resolve, reject) => {{
        const request = {{
            id: 1,
            getBlockCountRequest: {{}}
        }};
        
        client.messageStream(request, (error, response) => {{
            if (error) {{
                reject(error);
            }} else {{
                resolve(response.getBlockCountResponse.blockCount);
            }}
        }});
    }});
}}

// 4. Get balance
async function getBalance(address) {{
    return new Promise((resolve, reject) => {{
        const request = {{
            id: 2,
            getBalanceByAddressRequest: {{
                address: address
            }}
        }};
        
        client.messageStream(request, (error, response) => {{
            if (error) {{
                reject(error);
            }} else {{
                resolve(response.getBalanceByAddressResponse.balance);
            }}
        }});
    }});
}}

// 5. Submit transaction
async function submitTransaction(transaction) {{
    return new Promise((resolve, reject) => {{
        const request = {{
            id: 3,
            submitTransactionRequest: {{
                transaction: transaction
            }}
        }};
        
        client.messageStream(request, (error, response) => {{
            if (error) {{
                reject(error);
            }} else {{
                resolve(response.submitTransactionResponse.transactionId);
            }}
        }});
    }});
}}

// 6. Usage example
async function main() {{
    try {{
        const blockCount = await getBlockCount();
        console.log('Block count:', blockCount);
        
        const balance = await getBalance('kaspatest:qrf7saw4vlc006mmcpfa29mrgs79ez5glszc4ytx9hq7wdu2a5d4kvl7wz5pz');
        console.log('Balance:', balance);
        
        // Submit KRC-20 issuance transaction
        const issueTx = {{
            // Transaction data
        }};
        const txId = await submitTransaction(issueTx);
        console.log('Transaction ID:', txId);
        
    }} catch (error) {{
        console.error('Error:', error);
    }}
}}
");
}
