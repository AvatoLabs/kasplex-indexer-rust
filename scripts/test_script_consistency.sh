#!/bin/bash

echo "🔍 Go vs Rust Script Consistency Test"
echo "================================"

# Set colors
RED='\033[0;31m'
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
NC='\033[0m' # No Color

# Test function
test_script_consistency() {
    local test_name="$1"
    local go_script="$2"
    local rust_script="$3"
    
    echo -e "\n${YELLOW}🧪 Test: $test_name${NC}"
    
    if [ "$go_script" = "$rust_script" ]; then
        echo -e "${GREEN}✅ Scripts match${NC}"
        echo "  Go script:   $go_script"
        echo "  Rust script: $rust_script"
        return 0
    else
        echo -e "${RED}❌ Scripts do not match${NC}"
        echo "  Go script:   $go_script"
        echo "  Rust script: $rust_script"
        echo "  Length difference: Go(${#go_script}) vs Rust(${#rust_script})"
        return 1
    fi
}

# Run Go script to generate test data
echo "📝 Generating Go version scripts..."
go run scripts/compare_go_rust_scripts.go > /tmp/go_scripts.txt 2>&1

# Extract Go scripts
GO_ISSUE_SCRIPT=$(grep "Go script:" /tmp/go_scripts.txt | head -1 | cut -d' ' -f3)
GO_TRANSFER_SCRIPT=$(grep "Go transfer script:" /tmp/go_scripts.txt | cut -d' ' -f3)
GO_MINT_SCRIPT=$(grep "Go mint script:" /tmp/go_scripts.txt | cut -d' ' -f3)

echo "Go script extraction results:"
echo "  Issue script: $GO_ISSUE_SCRIPT"
echo "  Transfer script: $GO_TRANSFER_SCRIPT"
echo "  Mint script: $GO_MINT_SCRIPT"

# Run Rust tests
echo -e "\n🦀 Running Rust tests..."
cargo test script_consistency_tests -- --nocapture > /tmp/rust_test_output.txt 2>&1

# Check if Rust tests succeeded
if [ $? -eq 0 ]; then
    echo -e "${GREEN}✅ Rust tests ran successfully${NC}"
else
    echo -e "${RED}❌ Rust tests failed${NC}"
    echo "Error output:"
    cat /tmp/rust_test_output.txt
    exit 1
fi

# Extract Rust scripts (from test output)
RUST_ISSUE_SCRIPT=$(grep "Rust script:" /tmp/rust_test_output.txt | head -1 | cut -d' ' -f3)
RUST_TRANSFER_SCRIPT=$(grep "Rust script:" /tmp/rust_test_output.txt | sed -n '2p' | cut -d' ' -f3)
RUST_MINT_SCRIPT=$(grep "Rust script:" /tmp/rust_test_output.txt | sed -n '3p' | cut -d' ' -f3)

echo "Rust script extraction results:"
echo "  Issue script: $RUST_ISSUE_SCRIPT"
echo "  Transfer script: $RUST_TRANSFER_SCRIPT"
echo "  Mint script: $RUST_MINT_SCRIPT"

# Execute consistency tests
echo -e "\n🔍 Executing consistency tests..."
failed_tests=0

test_script_consistency "Issue script" "$GO_ISSUE_SCRIPT" "$RUST_ISSUE_SCRIPT"
if [ $? -ne 0 ]; then
    ((failed_tests++))
fi

test_script_consistency "Transfer script" "$GO_TRANSFER_SCRIPT" "$RUST_TRANSFER_SCRIPT"
if [ $? -ne 0 ]; then
    ((failed_tests++))
fi

test_script_consistency "Mint script" "$GO_MINT_SCRIPT" "$RUST_MINT_SCRIPT"
if [ $? -ne 0 ]; then
    ((failed_tests++))
fi

# Test results summary
echo -e "\n📊 Test results summary"
echo "=================="

if [ $failed_tests -eq 0 ]; then
    echo -e "${GREEN}🎉 All tests passed! Go and Rust version scripts are completely consistent${NC}"
    echo -e "${GREEN}✅ Both versions can be safely used for production deployment${NC}"
else
    echo -e "${RED}❌ $failed_tests tests failed${NC}"
    echo -e "${RED}⚠️  Script generation logic needs to be fixed to ensure consistency${NC}"
    
    echo -e "\n🔧 Fix suggestions:"
    echo "1. Check JSON serialization field order"
    echo "2. Ensure consistent hexadecimal encoding"
    echo "3. Verify string processing logic"
    echo "4. Check number formatting"
fi

# Clean up temporary files
rm -f /tmp/go_scripts.txt /tmp/rust_test_output.txt

echo -e "\n📋 Detailed test report:"
echo "=================="
echo "Test time: $(date)"
echo "Go version: $(go version 2>/dev/null || echo 'Go not installed')"
echo "Rust version: $(rustc --version 2>/dev/null || echo 'Rust not installed')"
echo "Test file: tests/script_consistency_test.rs"
echo "Comparison script: scripts/compare_go_rust_scripts.go"

exit $failed_tests
