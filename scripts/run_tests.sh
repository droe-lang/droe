#!/bin/bash

# Ensure we're in the project root directory
cd "$(dirname "$0")/.."

echo "🧪 Running Droe Comprehensive Tests"
echo "====================================="

# Test counter
TOTAL=0
PASSED=0

run_test() {
    local test_file=$1
    local test_name=$2
    
    echo ""
    echo "🔍 Testing: $test_name"
    echo "File: $test_file"
    
    TOTAL=$((TOTAL + 1))
    
    # Compile using Rust droe compiler
    if droe compile "$test_file" --target wasm 2>/dev/null; then
        # Get the generated WASM file path
        local wasm_file="${test_file%/*}/build/${test_file##*/}"
        wasm_file="${wasm_file%.*}.wasm"
        
        # Run the WASM file
        if [ -f "$wasm_file" ] && node run.js "$wasm_file" >/dev/null 2>&1; then
                echo "✅ PASSED"
                PASSED=$((PASSED + 1))
            else
                echo "❌ FAILED (runtime error)"
            fi
        else
            echo "❌ FAILED (WAT to WASM conversion)"
        fi
    else
        echo "❌ FAILED (compilation error)"
    fi
}

# Run tests
run_test "tests/test_comprehensive.droe" "Comprehensive Features"
run_test "tests/string_interpolation/test_simple_interpolation.droe" "String Interpolation"
run_test "tests/type_system/test_types_simple.droe" "Type System"
run_test "tests/type_system/test_boolean_multiple.droe" "Boolean Variables"
run_test "tests/type_system/test_collections.droe" "Collections"
run_test "tests/test_string_concatenation.droe" "String Concatenation"

echo ""
echo "📊 Test Results"
echo "==============="
echo "Total tests: $TOTAL"
echo "Passed: $PASSED"
echo "Failed: $((TOTAL - PASSED))"

if [ $PASSED -eq $TOTAL ]; then
    echo "🎉 All tests passed!"
    exit 0
else
    echo "⚠️  Some tests failed"
    exit 1
fi