#!/bin/bash

# Indexer monitoring script

echo "=== Kaspa Indexer Monitoring ==="

# Check if process is running
if pgrep -f "kaspa-indexer" > /dev/null; then
    echo "✅ Indexer is running"
    PID=$(pgrep -f "kaspa-indexer")
    echo "Process ID: $PID"
else
    echo "❌ Indexer is not running"
    exit 1
fi

# Check data directory
if [ -d "data" ]; then
    echo "✅ Data directory exists"
    echo "Data size: $(du -sh data | cut -f1)"
else
    echo "❌ Data directory does not exist"
fi

# Check log file
if [ -f "data/LOG" ]; then
    echo "✅ Log file exists"
    echo "Log size: $(du -sh data/LOG | cut -f1)"
else
    echo "⚠️  Log file does not exist"
fi

# Check network connection
NODE_URL=$(grep 'kaspaNodeURL' testnet.toml | cut -d'=' -f2 | tr -d ' "')
if curl -s --connect-timeout 5 "$NODE_URL" > /dev/null; then
    echo "✅ Network connection is normal"
else
    echo "❌ Network connection is abnormal"
fi

# Show recent logs
echo ""
echo "=== Recent logs (last 10 lines) ==="
if [ -f "data/LOG" ]; then
    tail -10 data/LOG
else
    echo "Log file does not exist"
fi

echo ""
echo "=== System resource usage ==="
echo "CPU usage: $(top -bn1 | grep "Cpu(s)" | awk '{print $2}' | cut -d'%' -f1)%"
echo "Memory usage: $(free -h | grep Mem | awk '{print $3"/"$2}')"
echo "Disk usage: $(df -h . | tail -1 | awk '{print $5}')"

echo ""
echo "=== Monitoring commands ==="
echo "Real-time logs: tail -f data/LOG"
echo "Stop indexer: pkill -f kaspa-indexer"
echo "Restart indexer: cargo run --no-default-features"
