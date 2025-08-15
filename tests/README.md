# Roelang Test Suite

This directory contains comprehensive tests for the Roelang programming language compiler and runtime.

## Test Categories

### Core Language Tests

- **test_simple.droe** - Basic functionality test
- **test_dsl.droe** - Domain-specific language features
- **test_comprehensive.droe** - Complete feature integration test

### Type System Tests (`type_system/`)

- **test_boolean.droe** - Boolean type handling
- **test_boolean_false.droe** - False value testing
- **test_boolean_multiple.droe** - Multiple boolean variables
- **test_collections.droe** - Array and collection types
- **test_new_types.droe** - Modern type system features
- **test_types_simple.droe** - Basic type declarations

### String Processing Tests (`string_interpolation/`)

- **test_interpolation.droe** - Variable interpolation in strings
- **test_simple_interpolation.droe** - Basic [variable] syntax
- **test_mixed_interpolation.droe** - Complex interpolation scenarios

### Advanced Features

- **test_param_action.droe** - Parameterized actions and modules
- **test_string_concatenation.droe** - String concatenation with + operator
- **test_data.droe** - Data structure definitions
- **test_data_structures.droe** - Complex data types with property access
- **test_data_simple.droe** - Basic data structure usage

### Error Handling Tests (`error_cases/`)

- **test_type_errors.droe** - Type mismatch error handling
- **test_variable_type_errors.droe** - Variable type validation
- **test_reassignment_errors.droe** - Variable reassignment restrictions

### Integration Tests (`integration/`)

- **test_complete_program.droe** - Full program integration test

### Performance Tests (`performance/`)

- **test_large_arrays.droe** - Large data structure performance

### Unit Tests (`unit/`)

- **test_arrays.droe** - Array operation unit tests
- **test_loops.droe** - Loop construct testing
- **test_variables.droe** - Variable handling unit tests

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
python -m compiler.compiler tests/test_simple.droe
wat2wasm tests/test_simple.wat
node run.js tests/test_simple.wasm

# Or using droe CLI (from tests directory)
cd tests
droe run test_simple.droe
```

### Run Test Categories

```bash
# Type system tests
droe run type_system/test_boolean.droe
droe run type_system/test_collections.droe

# String tests
droe run string_interpolation/test_interpolation.droe

# Error tests (should fail with specific error messages)
droe run error_cases/test_type_errors.droe
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
2. Follow naming convention: `test_[feature].droe`
3. Include comments explaining what's being tested
4. Update this README if adding new categories

## Notes

- `.wat` and `.wasm` files are generated artifacts and not stored in version control
- Tests are designed to be self-contained and independent
- Error tests are expected to fail with specific error messages
- Performance tests help identify regressions
