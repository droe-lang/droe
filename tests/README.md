# Roelang Test Suite

This directory contains comprehensive tests for the Roelang programming language compiler and runtime.

## Test Categories

### Core Language Tests

- **test_simple.roe** - Basic functionality test
- **test_dsl.roe** - Domain-specific language features
- **test_comprehensive.roe** - Complete feature integration test

### Type System Tests (`type_system/`)

- **test_boolean.roe** - Boolean type handling
- **test_boolean_false.roe** - False value testing
- **test_boolean_multiple.roe** - Multiple boolean variables
- **test_collections.roe** - Array and collection types
- **test_new_types.roe** - Modern type system features
- **test_types_simple.roe** - Basic type declarations

### String Processing Tests (`string_interpolation/`)

- **test_interpolation.roe** - Variable interpolation in strings
- **test_simple_interpolation.roe** - Basic [variable] syntax
- **test_mixed_interpolation.roe** - Complex interpolation scenarios

### Advanced Features

- **test_param_action.roe** - Parameterized actions and modules
- **test_string_concatenation.roe** - String concatenation with + operator
- **test_data.roe** - Data structure definitions
- **test_data_structures.roe** - Complex data types with property access
- **test_data_simple.roe** - Basic data structure usage

### Error Handling Tests (`error_cases/`)

- **test_type_errors.roe** - Type mismatch error handling
- **test_variable_type_errors.roe** - Variable type validation
- **test_reassignment_errors.roe** - Variable reassignment restrictions

### Integration Tests (`integration/`)

- **test_complete_program.roe** - Full program integration test

### Performance Tests (`performance/`)

- **test_large_arrays.roe** - Large data structure performance

### Unit Tests (`unit/`)

- **test_arrays.roe** - Array operation unit tests
- **test_loops.roe** - Loop construct testing
- **test_variables.roe** - Variable handling unit tests

### Manual Tests (`manual/`)

- Manual testing artifacts and debug files

## Running Tests

### Run All Tests
```bash
./run_tests.sh
```

### Run Individual Tests
```bash
# From project root
python -m compiler.compiler tests/test_simple.roe
wat2wasm tests/test_simple.wat
node run.js tests/test_simple.wasm

# Or using roe CLI (from tests directory)
cd tests
roe run test_simple.roe
```

### Run Test Categories
```bash
# Type system tests
roe run type_system/test_boolean.roe
roe run type_system/test_collections.roe

# String tests  
roe run string_interpolation/test_interpolation.roe

# Error tests (should fail with specific error messages)
roe run error_cases/test_type_errors.roe
```

## Test Status

### ✅ Passing Tests
- All core language features
- Type system validation
- String interpolation and concatenation
- Data structures and property access
- Module system and parameterized actions
- Collection handling
- Control flow constructs

### 🔧 Test Infrastructure
- Automated test runner (`run_tests.sh`)
- Error case validation
- Performance benchmarks
- Integration test suite

## Adding New Tests

1. Create test file in appropriate subdirectory
2. Follow naming convention: `test_[feature].roe`
3. Include comments explaining what's being tested
4. Update this README if adding new categories

## Notes

- `.wat` and `.wasm` files are generated artifacts and not stored in version control
- Tests are designed to be self-contained and independent
- Error tests are expected to fail with specific error messages
- Performance tests help identify regressions