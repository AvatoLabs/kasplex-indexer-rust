#!/bin/bash

# Quick test script

echo "=== Kaspa Testnet Quick Test ==="

# 1. Run setup script
echo "1. Running setup script..."
./scripts/testnet_setup.sh

if [ $? -ne 0 ]; then
    echo "❌ Setup failed"
    exit 1
fi

# 2. Start indexer
echo ""
echo "2. Starting indexer..."
echo "Start command: cargo run --no-default-features"
echo "Please run the above command in a new terminal"
echo ""

# 3. Wait for user confirmation
read -p "Has the indexer started and is running normally? (y/n): " -n 1 -r
echo
if [[ ! $REPLY =~ ^[Yy]$ ]]; then
    echo "Please start the indexer first"
    exit 1
fi

# 4. Check connection
echo ""
echo "3. Checking connection status..."
sleep 5
./scripts/monitor_indexer.sh

# 5. Provide testing suggestions
echo ""
echo "=== Testing Suggestions ==="
echo ""
echo "1. Get Testnet KAS:"
echo "   - Visit: https://testnet.kaspa.org/faucet"
echo "   - Enter your Kaspa address"
echo ""
echo "2. Prepare test wallet:"
echo "   - Download Kaspa wallet: https://kaspa.org/wallets/"
echo "   - Create testnet wallet"
echo "   - Record address and private key"
echo ""
echo "3. Test Token operations:"
echo "   - Issue Token (issue)"
echo "   - Transfer Token (send)"
echo "   - Burn Token (burn)"
echo "   - Transfer ownership (chown)"
echo "   - List Token (list)"
echo ""
echo "4. Monitor indexer:"
echo "   - Run: ./scripts/monitor_indexer.sh"
echo "   - View logs: tail -f data/LOG"
echo ""
echo "5. Verify data:"
echo "   - Check token information"
echo "   - Verify balance changes"
echo "   - Confirm transaction history"
echo ""
echo "Detailed testing guide please refer to: testnet_test_guide.md"
