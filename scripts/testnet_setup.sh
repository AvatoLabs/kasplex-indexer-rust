#!/bin/bash

# Kaspa Testnet Indexer Setup Script

echo "=== Kaspa Testnet Indexer Setup ==="

# 1. Clean existing configuration
echo "1. Cleaning existing configuration..."
rm -f mainnet.toml config.toml

# 2. Clean existing data
echo "2. Cleaning existing data..."
rm -rf data/

# 3. Create data directory
echo "3. Creating data directory..."
mkdir -p data

# 4. Verify testnet configuration
echo "4. Verifying testnet configuration..."
if [ -f "testnet.toml" ]; then
    echo "✅ testnet.toml configuration exists"
    echo "Kaspa Node URL: $(grep 'kaspaNodeURL' testnet.toml | cut -d'=' -f2 | tr -d ' "')"
else
    echo "❌ testnet.toml configuration does not exist"
    exit 1
fi

# 5. Check network connection
echo "5. Checking network connection..."
NODE_URL=$(grep 'kaspaNodeURL' testnet.toml | cut -d'=' -f2 | tr -d ' "')
if curl -s --connect-timeout 10 "$NODE_URL" > /dev/null; then
    echo "✅ Can connect to Kaspa testnet node"
else
    echo "❌ Cannot connect to Kaspa testnet node"
    echo "Please check network connection or node URL"
fi

# 6. Compile project
echo "6. Compiling project..."
cargo build --no-default-features

if [ $? -eq 0 ]; then
    echo "✅ Compilation successful"
else
    echo "❌ Compilation failed"
    exit 1
fi

echo ""
echo "=== Setup Complete ==="
echo ""
echo "Next steps:"
echo "1. Start indexer: cargo run --no-default-features"
echo "2. Get testnet KAS: https://testnet.kaspa.org/faucet"
echo "3. Prepare Kaspa wallet for testing"
echo ""
echo "Testing guide please refer to: testnet_test_guide.md"
