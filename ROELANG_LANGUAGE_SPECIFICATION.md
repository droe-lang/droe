# Roelang Language Specification

**Version:** 1.0  
**Date:** August 2024  
**Authors:** Roelang Development Team

## Table of Contents

1. [Overview](#overview)
2. [Lexical Structure](#lexical-structure)
3. [Data Types](#data-types)
4. [Variables and Assignment](#variables-and-assignment)
5. [Expressions](#expressions)
6. [Control Flow](#control-flow)
7. [Functions and Actions](#functions-and-actions)
8. [Modules](#modules)
9. [Data Structures](#data-structures)
10. [String Operations](#string-operations)
11. [Format Expressions](#format-expressions)
12. [Include System](#include-system)
13. [Comments](#comments)
14. [Keywords](#keywords)
15. [Compilation Targets](#compilation-targets)

---

## Overview

Roelang is a domain-specific language designed for business logic and process automation. It features a readable, English-like syntax with strong typing, modern language constructs, and cross-platform compilation to multiple target languages including WebAssembly, Python, Java, JavaScript, and more.

### Design Principles

- **Readability First**: Natural, English-like syntax
- **Strong Typing**: Explicit type declarations prevent runtime errors
- **Multi-Target**: Compile to multiple languages and platforms
- **Modular**: Support for modules and code reuse
- **Modern**: Support for collections, string interpolation, and format expressions

---

## Lexical Structure

### Case Sensitivity
Roelang is **case-sensitive**. Keywords must be lowercase.

### Identifiers
- Start with a letter (a-z, A-Z) or underscore (_)
- Followed by letters, digits (0-9), or underscores
- Cannot be reserved keywords

```roe
// Valid identifiers
user_name
firstName
age2
_private_var

// Invalid identifiers
2name       // starts with digit
user-name   // contains hyphen
```

### Line Termination
Statements are terminated by newlines. No semicolons required.

---

## Data Types

Roelang supports the following built-in data types:

### Primitive Types

| Type | Description | Examples |
|------|-------------|----------|
| `int` | 32-bit signed integer | `42`, `-10`, `0` |
| `decimal` | Double-precision floating point | `3.14`, `-0.5`, `99.99` |
| `text` | Unicode string | `"Hello"`, `"Roelang"` |
| `flag` | Boolean true/false | `true`, `false` |
| `yesno` | Boolean true/false (alias for flag) | `true`, `false` |
| `date` | ISO date string | `"2024-08-06"`, `"1990-01-15"` |
| `file` | File path string | `"/path/to/file.txt"` |

### Legacy Types (for compatibility)
- `number` → `int`
- `string` → `text`  
- `boolean` → `flag`

### Collection Types

| Type | Description | Example |
|------|-------------|---------|
| `list of <type>` | Ordered collection | `[1, 2, 3, 4, 5]` |
| `group of <type>` | Unordered collection | `["Alice", "Bob", "Charlie"]` |

### Collection Element Types
Collections can contain any primitive type:
- `list of int` → `[1, 2, 3]`
- `list of text` → `["hello", "world"]`
- `group of decimal` → `[3.14, 2.71, 1.41]`
- `list of date` → `["2024-01-01", "2024-12-25"]`

---

## Variables and Assignment

### Variable Declaration and Assignment

**Basic Syntax:**
```roe
set <variable> which is <type> to <value>
```

**Examples:**
```roe
// Primitive types
set name which is text to "Alice"
set age which is int to 25
set salary which is decimal to 75000.50
set active which is flag to true
set birth_date which is date to "1998-03-15"

// Collections
set numbers which are list of int to [1, 2, 3, 4, 5]
set names which are group of text to ["Alice", "Bob", "Charlie"]
set scores which are list of decimal to [95.5, 87.2, 91.8]
set holidays which are list of date to ["2024-01-01", "2024-12-25"]
```

### Type Inference Assignment
```roe
set result to 10 + 5        // Infers int type
set message to "Hello"      // Infers text type
```

### Reassignment
```roe
set age which is int to 25
set age to 26               // Type already declared
```

---

## Expressions

### Arithmetic Expressions
```roe
set sum to 10 + 5          // Addition
set difference to 20 - 8    // Subtraction  
set product to 6 * 7        // Multiplication
set quotient to 15 / 3      // Division

// With variables
set x which is int to 10
set y which is int to 3
set result to x + y * 2     // 16 (multiplication has higher precedence)
```

### String Concatenation
```roe
set first_name which is text to "John"
set last_name which is text to "Doe"
set full_name to first_name + " " + last_name

// Mixed types (automatic conversion)
set age which is int to 25
set message to "I am " + age + " years old"
```

### Comparison Expressions
```roe
// Numeric comparisons
when 5 is greater than 3 then display "True"
when 10 is less than 20 then display "True"
when 7 is greater than or equal to 7 then display "True"
when 8 is less than or equal to 9 then display "True"
when 42 equals 42 then display "True"
when 1 does not equal 2 then display "True"

// Variable comparisons
set age which is int to 25
set limit which is int to 18
when age is greater than limit then display "Adult"
```

### Comparison Operators
- `is greater than` → `>`
- `is less than` → `<`
- `is greater than or equal to` → `>=`
- `is less than or equal to` → `<=`
- `equals` → `==`
- `does not equal` → `!=`

---

## Control Flow

### Conditional Statements

**Basic Conditional:**
```roe
when <condition> then <statement>
```

**If-Else Structure:**
```roe
when <condition> then
    // statements
otherwise
    // alternative statements
end when
```

**Examples:**
```roe
// Simple condition
when age is greater than 18 then display "Adult"

// If-else block
when score is greater than or equal to 90 then
    display "Grade A"
    display "Excellent work!"
otherwise
    display "Keep trying"
end when
```

### While Loops

**Syntax:**
```roe
while <condition>
    // statements
end while
```

**Examples:**
```roe
// Counting loop
set counter to 1
while counter is less than or equal to 5
    display counter
    set counter to counter + 1
end while

// Accumulator pattern
set i to 1
set total to 0
while i is less than or equal to 10
    set total to total + i
    set i to i + 1
end while
display "Sum: " + total
```

### For-Each Loops
```roe
set numbers which are list of int to [1, 2, 3, 4, 5]
for each num in numbers
    display "Number: " + num
end for
```

---

## Functions and Actions

### Actions (Functions)

Actions are reusable blocks of code that can accept parameters and return values.

**Basic Action (no parameters):**
```roe
action greet_user
    display "Hello, welcome to Roelang!"
end action

// Call the action
run greet_user
```

**Parameterized Action:**
```roe
action greet_person with name which is text
    display "Hello, " + name + "!"
    display "Welcome to Roelang!"
end action

// Call with parameter
run greet_person with "Alice"
```

**Action with Return Value:**
```roe
action calculate_area with width which is int, height which is int gives int
    give width * height
end action

// Use return value
set area which is int from calculate_area with 10, 5
display "Area: " + area
```

**Action with Multiple Parameters:**
```roe
action create_greeting with name which is text, age which is int gives text
    give "Hello " + name + ", you are " + age + " years old!"
end action

set message which is text from create_greeting with "Bob", 30
display message
```

### Tasks

Tasks are actions that don't return values, used for procedural execution.

**Task Syntax:**
```roe
task send_reminder
    display "Don't forget to complete your tasks!"
end task

task process_order with item which is text, quantity which is int
    display "Processing order for " + quantity + " " + item
end task

// Execute tasks
run send_reminder
run process_order with "widgets", 5
```

---

## Modules

Modules provide namespacing and code organization.

**Module Definition:**
```roe
module math_utils

    action add with a which is int, b which is int gives int
        give a + b
    end action
    
    action multiply with x which is decimal, y which is decimal gives decimal
        give x * y
    end action

end module
```

**Using Module Actions:**
```roe
// Call module action
set result which is int from run math_utils.add with 10, 5
display "Result: " + result

// Direct execution
display run math_utils.add with 20, 15
```

---

## Data Structures

Define custom data types with named fields.

**Data Definition:**
```roe
module user_system

    data User
        name is text
        age is int
        active is flag
    end data
    
    action create_user with user_name which is text, user_age which is int gives User
        // Return user instance (implementation varies by target)
        give User with name is user_name, age is user_age, active is true
    end action

end module
```

---

## String Operations

### String Interpolation

Embed variables and expressions within strings using square brackets:

```roe
set name which is text to "Alice"
set age which is int to 25
set balance which is decimal to 150.50

display "Hello [name]!"                           // Hello Alice!
display "Age: [age]"                             // Age: 25  
display "Balance: $[balance]"                    // Balance: $150.50
display "Status: [name] ([age] years old)"      // Status: Alice (25 years old)
```

### String Concatenation
```roe
set first which is text to "Hello"
set second which is text to "World"
set combined to first + " " + second            // "Hello World"

// Mixed type concatenation
set count which is int to 5
set message to "You have " + count + " items"   // "You have 5 items"
```

---

## Format Expressions

Format expressions allow precise control over how data is displayed.

### Date Formatting
```roe
set event_date which is date to "2024-12-25"

display format event_date as "MM/dd/yyyy"       // 12/25/2024
display format event_date as "dd/MM/yyyy"       // 25/12/2024  
display format event_date as "MMM dd, yyyy"     // Dec 25, 2024
display format event_date as "long"             // Wednesday, December 25, 2024
```

### Decimal Formatting
```roe
set price which is decimal to 1234.56

display format price as "0.00"                  // 1234.56
display format price as "#,##0.00"              // 1,234.56
display format price as "$0.00"                 // $1234.56
```

### Number Formatting  
```roe
set quantity which is int to 12345
set code which is int to 255

display format quantity as "#,##0"              // 12,345
display format code as "hex"                    // 0xFF
display format code as "0000"                   // 0255
```

### Format in Assignments
```roe
set formatted_date which is text to format event_date as "long"
set formatted_price which is text to format price as "#,##0.00"
```

---

## Include System

Import and use code from other files.

**Include Syntax:**
```roe
include "path/to/ModuleName.roe"
```

**Using Included Modules:**
```roe
// File: utils/MathUtils.roe
module utils_MathUtils
    action add with a which is int, b which is int gives int
        give a + b
    end action
end module

// File: main.roe
include "utils/MathUtils.roe"

set result which is int from run utils_MathUtils.add with 10, 5
display "Sum: " + result
```

---

## Comments

### Single-line Comments
```roe
// This is a single-line comment
set name which is text to "Alice"  // End-of-line comment
```

### Multi-line Comments
```roe
/*
This is a multi-line comment
that spans multiple lines.
Useful for documentation.
*/

set value which is int to 42
```

---

## Keywords

### Reserved Words

| Category | Keywords |
|----------|----------|
| **Variables** | `set`, `which`, `is`, `to`, `are` |
| **Control Flow** | `when`, `then`, `otherwise`, `end`, `while`, `for`, `each`, `in` |
| **Actions** | `action`, `task`, `with`, `gives`, `give`, `run`, `from` |
| **Modules** | `module`, `include`, `data` |
| **Display** | `display`, `show` |  
| **Types** | `int`, `decimal`, `text`, `flag`, `yesno`, `date`, `file`, `list`, `group`, `of` |
| **Logic** | `true`, `false`, `and`, `or`, `not` |
| **Comparisons** | `equals`, `greater`, `less`, `than`, `equal`, `does` |
| **Format** | `format`, `as` |

### Operators

| Symbol | Meaning | Usage |
|--------|---------|-------|
| `+` | Addition/Concatenation | `5 + 3`, `"Hello" + " World"` |
| `-` | Subtraction | `10 - 4` |
| `*` | Multiplication | `6 * 7` |
| `/` | Division | `15 / 3` |
| `[]` | String interpolation | `"Hello [name]"` |
| `[]` | Array literals | `[1, 2, 3]` |

---

## Compilation Targets

Roelang compiles to multiple target languages:

### Supported Targets

| Target | Extension | Description |
|--------|-----------|-------------|
| `wasm` | `.wat` | WebAssembly Text format |
| `python` | `.py` | Python source code |
| `java` | `.java` | Java source code |
| `node` | `.js` | Node.js JavaScript |  
| `go` | `.go` | Go source code |
| `html` | `.html` | HTML with embedded JavaScript |
| `kotlin` | `.kt` | Kotlin source code |
| `swift` | `.swift` | Swift source code |
| `bytecode` | `.roebc` | Roelang VM bytecode |

### Compilation Commands
```bash
# Compile to specific target
roe compile program.roe --target java
roe compile program.roe --target python
roe compile program.roe --target wasm

# Compile and run
roe run program.roe
```

### Type Mapping

| Roelang Type | Python | Java | JavaScript | Go |
|--------------|--------|------|------------|----| 
| `int` | `int` | `int` | `number` | `int` |
| `decimal` | `float` | `double` | `number` | `float64` |
| `text` | `str` | `String` | `string` | `string` |
| `flag` | `bool` | `boolean` | `boolean` | `bool` |
| `list of int` | `List[int]` | `List<Integer>` | `number[]` | `[]int` |
| `group of text` | `List[str]` | `List<String>` | `string[]` | `[]string` |

---

## Example Programs

### Complete Example
```roe
// Customer management system
module customer_system

    data Customer
        name is text
        age is int
        balance is decimal
        active is flag
    end data
    
    action create_welcome_message with customer_name which is text, balance which is decimal gives text
        when balance is greater than 1000 then
            give "Welcome, " + customer_name + "! You're a premium member."
        otherwise  
            give "Welcome, " + customer_name + "! Thank you for joining us."
        end when
    end action

end module

// Main program
set customer_name which is text to "Alice Johnson"
set account_balance which is decimal to 1250.75

set welcome which is text from run customer_system.create_welcome_message with customer_name, account_balance

display "=== Customer Portal ==="
display welcome
display "Account Balance: [format account_balance as '$#,##0.00']"

// Process transactions
set transactions which are list of decimal to [50.00, -25.50, 100.00]

display "Recent transactions:"
for each amount in transactions
    when amount is greater than 0 then
        display "+ [format amount as '$0.00'] (deposit)"
    otherwise
        display "- [format amount as '$0.00'] (withdrawal)"
    end when
end for
```

---

## Error Handling

Roelang enforces strong typing and will generate compilation errors for:

- **Type mismatches**: Assigning wrong type to variable
- **Undefined variables**: Using undeclared variables  
- **Unknown actions**: Calling non-existent functions
- **Parameter mismatches**: Wrong number or types of parameters
- **Syntax errors**: Invalid language constructs

---

## Best Practices

### Code Organization
- Use modules to group related functionality
- Use meaningful variable and action names
- Include type annotations for clarity

### Performance
- Use appropriate collection types (`list of` vs `group of`)
- Avoid deep nesting in control structures
- Use format expressions for efficient string formatting

### Readability  
- Use consistent indentation
- Add comments for complex logic
- Use string interpolation for dynamic messages

---

*This specification covers Roelang version 1.0. For updates and examples, visit the official Roelang documentation.*