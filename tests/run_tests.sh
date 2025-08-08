#!/bin/bash

echo "🧪 Running Roelang Comprehensive Tests"
echo "======================================"

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
    
    # Compile
    if python -m compiler.compiler "$test_file" 2>/dev/null; then
        # Convert to WASM
        local wat_file="${test_file%.*}.wat"
        local wasm_file="${test_file%.*}.wasm"
        
        if wat2wasm "$wat_file" -o "$wasm_file" 2>/dev/null; then
            # Run
            if node run.js "$wasm_file" >/dev/null 2>&1; then
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
run_test "tests/test_comprehensive.roe" "Comprehensive Features"
run_test "tests/string_interpolation/test_simple_interpolation.roe" "String Interpolation"
run_test "tests/type_system/test_types_simple.roe" "Type System"
run_test "tests/type_system/test_boolean_multiple.roe" "Boolean Variables"
run_test "tests/type_system/test_collections.roe" "Collections"
run_test "tests/test_string_concatenation.roe" "String Concatenation"

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