# Droe Language Specification

**Version:** 4.0  
**Date:** January 2025  
**Authors:** Droelang Development Team

## Overview

Droelang is a modern DSL for building cross-platform applications with a fragment-based UI system that compiles to web (HTML/JavaScript), Android (Kotlin), and iOS (Swift) using consistent syntax optimized for mobile-first, responsive design.

## Table of Contents

1. [Overview](#overview)
2. [Language Rules and Syntax](#language-rules-and-syntax)
3. [Lexical Structure](#lexical-structure)
4. [Metadata Annotations](#metadata-annotations)
5. [Data Types](#data-types)
6. [Variables and Assignment](#variables-and-assignment)
7. [Expressions](#expressions)
8. [Control Flow](#control-flow)
9. [Functions and Actions](#functions-and-actions)
10. [Modules](#modules)
11. [Data Structures](#data-structures)
12. [Database DSL](#database-dsl)
13. [API DSL](#api-dsl)
14. [UI Components and Screens](#ui-components-and-screens)
15. [Mobile Platform Features](#mobile-platform-features)
16. [String Operations](#string-operations)
17. [Format Expressions](#format-expressions)
18. [Include System](#include-system)
19. [Comments](#comments)
20. [Keywords](#keywords)
21. [Compilation Targets](#compilation-targets)
22. [Framework Adapters](#framework-adapters)
23. [Example Programs](#example-programs)

---

## Overview

Droelang is a domain-specific language designed for business logic, process automation, API development, database operations, and cross-platform application development. It features a readable, English-like syntax with strong typing, modern language constructs, built-in DSLs for databases and APIs, and cross-platform compilation to multiple target languages including WebAssembly, Python, Java (with Spring Boot support), JavaScript, HTML, and mobile platforms (Android/iOS).

### Design Principles

- **Readability First**: Natural, English-like syntax
- **Strong Typing**: Explicit type declarations prevent runtime errors
- **Multi-Target**: Compile to multiple languages and platforms
- **Modular**: Support for modules and code reuse
- **Modern**: Support for collections, string interpolation, and format expressions
- **Metadata-Driven**: Built-in support for metadata annotations for compilation control and documentation
- **Database-Native**: Built-in DSL for database operations with ORM generation
- **HTTP-First**: Native support for API calls and HTTP server endpoints
- **Framework Integration**: Automatic generation of Spring Boot, Android, and iOS projects
- **Cross-Platform UI**: Single DSL syntax generates web, Android, and iOS applications
- **Mobile-First**: Built-in support for mobile-specific features like camera, GPS, sensors, and notifications

---

## Language Rules and Syntax

### Core Principles

1. **Lowercase Keywords Only**: All keywords must be lowercase (`display`, `module`, `when`, etc.)
2. **Word-Based Operators**: No mathematical symbols - use `plus`, `minus`, `times`, `divided by` instead of `+`, `-`, `*`, `/`
3. **Natural Language**: Comparisons use `is greater than`, `equals`, `does not equal` instead of `>`, `==`, `!=`
4. **Square Bracket Interpolation**: String interpolation uses `[variable]` syntax, not `+` concatenation
5. **@ Prefix for Imports**: Only `@include` uses `@` prefix to avoid ambiguity
6. **No Colons**: Clean syntax without colon suffixes

### Syntax Rules

```droe
// ✅ Correct Droelang Syntax
module MyModule
data User
@include MathUtils from "utils/math.droe"

set total to 10 plus 5 times 2
when age is greater than 18 then display "Adult"
display "Hello [name], you have [points] points"

// ❌ Incorrect - these will not work
Module: MyModule                    // No colons, no uppercase
include "file.droe"                 // Must use @include
set total to 10 + 5 * 2            // No symbols, use words
when age > 18 then display "Adult" // No symbols, use words
display "Hello " + name             // No +, use [name]
```

---

## Lexical Structure

### Case Sensitivity

Droelang is **case-sensitive**. Keywords must be lowercase.

### Identifiers

- Start with a letter (a-z, A-Z) or underscore (\_)
- Followed by letters, digits (0-9), or underscores
- Cannot be reserved keywords

```droe
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

## Metadata Annotations

Metadata annotations provide compile-time information about Droelang programs. They are specified at the beginning of a file using the `@key value` syntax.

### Syntax

```droe
@key value
@key "quoted value"
@key 'single quoted value'
```

### Standard Metadata Keys

#### @target

Specifies the compilation target, overriding command-line options and project configuration. If not specified, defaults to `bytecode`.

**Supported targets:**

- `wasm` - WebAssembly
- `python` - Python
- `java` - Java
- `javascript` or `node` - JavaScript/Node.js
- `go` - Go
- `html` - HTML with JavaScript
- `bytecode` - Droelang bytecode

**Note:** Multi-target compilation should be configured in `droeconfig.json` instead of file-level annotations:

```json
{
  "target": "mobile",
  "mobile": {
    "platforms": ["android", "ios"]
  }
}
```

#### @name

Specifies the module or component name. Useful for documentation and tooling.

```droe
@name "Shopping Cart"
@name shopping_cart_module
```

#### @description

Provides a human-readable description of the module's purpose.

```droe
@description "Handles user profile management and validation"
@description "A utility module for mathematical operations"
```

#### @package

Specifies the package name for applications (mainly used in `droeconfig.json`).

```droe
@package "com.example.myapp"
```

### Rules and Constraints

1. **Placement**: Metadata annotations must appear at the top of the file before any code statements
2. **Order**: Metadata can appear in any order
3. **Uniqueness**: Each metadata key should appear only once per file
4. **Values**: Values can be unquoted (single word) or quoted (for multi-word values)
5. **Case Sensitivity**: Metadata keys are case-sensitive

### Examples

**Simple file-level metadata:**

```droe
@target java
@name PaymentProcessor
@description "Processes payment transactions and validations"

// Your code here
display "Payment system initialized"
```

**Note:** For most projects, use `droeconfig.json` instead of file-level metadata:

```json
{
  "target": "java",
  "framework": "spring",
  "package": "com.example.payments",
  "database": {
    "type": "postgres",
    "url": "postgresql://localhost/payments_db"
  }
}
```

### Programmatic Access

Metadata can be accessed programmatically through the compiler API:

```python
from compiler.parser import parse
from compiler.compiler import get_metadata_value

ast = parse(source_code)
target = get_metadata_value(ast, "target")
name = get_metadata_value(ast, "name")
description = get_metadata_value(ast, "description")
```

---

## Data Types

Droelang supports the following built-in data types:

### Primitive Types

| Type      | Description                         | Examples                       |
| --------- | ----------------------------------- | ------------------------------ |
| `int`     | 32-bit signed integer               | `42`, `-10`, `0`               |
| `decimal` | Double-precision floating point     | `3.14`, `-0.5`, `99.99`        |
| `text`    | Unicode string                      | `"Hello"`, `"Droelang"`        |
| `flag`    | Boolean true/false                  | `true`, `false`                |
| `yesno`   | Boolean true/false (alias for flag) | `true`, `false`                |
| `date`    | ISO date string                     | `"2024-08-06"`, `"1990-01-15"` |
| `file`    | File path string                    | `"/path/to/file.txt"`          |

### Legacy Types (for compatibility)

- `number` → `int`
- `string` → `text`
- `boolean` → `flag`

### Collection Types

| Type              | Description          | Example                       |
| ----------------- | -------------------- | ----------------------------- |
| `list of <type>`  | Ordered collection   | `[1, 2, 3, 4, 5]`             |
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

```droe
set <variable> which is <type> to <value>
```

**Examples:**

```droe
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

```droe
set result to 10 plus 5     // Infers int type
set message to "Hello"      // Infers text type
```

### Reassignment

```droe
set age which is int to 25
set age to 26               // Type already declared
```

---

## Expressions

### Arithmetic Expressions

```droe
set sum to 10 plus 5                    // Addition
set difference to 20 minus 8            // Subtraction
set product to 6 times 7                // Multiplication
set quotient to 15 divided by 3         // Division

// With variables
set x which is int to 10
set y which is int to 3
set result to x plus y times 2          // 16 (multiplication has higher precedence)
```

### String Interpolation

```droe
set first_name which is text to "John"
set last_name which is text to "Doe"
set age which is int to 25

// String interpolation with variables
set full_name to "[first_name] [last_name]"
set message to "I am [age] years old"
set greeting to "Hello [first_name], nice to meet you!"
```

### Comparison Expressions

```droe
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

### Comparison Operators (Word-Based Only)

- `is greater than` - Tests if left value is greater than right value
- `is less than` - Tests if left value is less than right value
- `is greater than or equal to` - Tests if left value is greater than or equal to right value
- `is less than or equal to` - Tests if left value is less than or equal to right value
- `equals` - Tests if values are equal
- `does not equal` - Tests if values are not equal

**Note**: Droelang uses only word-based operators to maintain readability and non-technical friendliness. Symbol-based operators (`>`, `<`, `==`, etc.) are not supported.

---

## Control Flow

### Conditional Statements

Droelang provides comprehensive conditional logic with natural language syntax, supporting basic if-else, multiple else-if chains, and complex compound conditions.

#### Basic Conditional

**Single-line condition:**
```droe
when <condition> then <statement>
```

**If-else block:**
```droe
when <condition> then
    // statements
otherwise
    // alternative statements
end when
```

#### Comparison Operators

Droelang uses natural language comparison operators (longest-match-first parsing):

| Operator | Description | Example |
|----------|-------------|---------|
| `is greater than` | Greater than comparison | `age is greater than 18` |
| `is less than` | Less than comparison | `score is less than 100` |
| `is greater than or equal to` | Greater than or equal | `age is greater than or equal to 21` |
| `is less than or equal to` | Less than or equal | `score is less than or equal to 100` |
| `equals` | Equality comparison | `status equals "active"` |
| `does not equal` | Inequality comparison | `role does not equal "guest"` |
| `is` | Basic equality | `flag is true` |

#### Compound Conditions

**Logical Operators:**
- `and` (higher precedence) - Both conditions must be true
- `or` (lower precedence) - Either condition must be true
- `()` parentheses for explicit grouping

**Examples:**
```droe
// AND condition
when age is greater than 18 and status equals "active" then
    display "Qualified user"
end when

// OR condition  
when role equals "admin" or score is greater than 90 then
    display "Special access granted"
end when

// Complex conditions with parentheses
when (age is greater than 21 and status equals "active") or role equals "admin" then
    display "Full access granted"
end when

// Multiple conditions
when age is greater than or equal to 18 and status equals "active" and score is greater than 75 then
    display "All requirements met"
end when
```

#### Else-If Chains (Otherwise When)

Droelang supports multiple conditional branches using `otherwise when`:

**Syntax:**
```droe
when <condition1> then
    // statements
otherwise when <condition2> then
    // statements  
otherwise when <condition3> then
    // statements
otherwise
    // final else statements
end when
```

**Complete Example:**
```droe
set score which is int to 85

when score is greater than or equal to 90 then
    display "Grade: A - Excellent!"
    display "Outstanding performance"
otherwise when score is greater than or equal to 80 then
    display "Grade: B - Very Good!"
    display "Great work"
otherwise when score is greater than or equal to 70 then
    display "Grade: C - Good"
    display "Keep it up"
otherwise when score is greater than or equal to 60 then
    display "Grade: D - Passing"
    display "You passed"
otherwise
    display "Grade: F - Needs Improvement"
    display "Please study more"
end when
```

#### Nested Conditions

Conditions can be nested for complex decision trees:

```droe
when age is greater than or equal to 18 then
    when (status equals "active" and score is greater than 70) or role equals "admin" then
        display "Full access granted"
        when role equals "admin" then
            display "Admin privileges enabled"
        otherwise
            display "Standard user access"
        end when
    otherwise
        display "Limited access - requirements not met"
    end when
otherwise
    display "Access denied - must be 18 or older"
end when
```

#### Advanced Examples

**User Authentication Logic:**
```droe
when username is not empty and password is not empty then
    when (role equals "admin" or role equals "moderator") and account_status equals "active" then
        display "Login successful - management access"
    otherwise when role equals "user" and account_status equals "active" then
        display "Login successful - user access"
    otherwise when account_status equals "suspended" then
        display "Account suspended - contact administrator"
    otherwise
        display "Invalid account status"
    end when
otherwise
    display "Please enter both username and password"
end when
```

**Shopping Cart Logic:**
```droe
when cart_total is greater than 100 then
    display "Free shipping applied!"
otherwise when cart_total is greater than 50 and (customer_type equals "premium" or has_coupon equals true) then
    display "Reduced shipping: $2.99"
otherwise when cart_total is greater than 25 then
    display "Standard shipping: $5.99"
otherwise
    display "Add $[25 minus cart_total] more for reduced shipping"
end when
```

### While Loops

**Syntax:**
```droe
while <condition>
    // statements
end while
```

**Examples:**
```droe
// Counting loop
set counter to 1
while counter is less than or equal to 5
    display counter
    set counter to counter plus 1
end while

// Accumulator pattern
set i to 1
set total to 0
while i is less than or equal to 10
    set total to total plus i
    set i to i plus 1
end while
display "Sum: [total]"

// Complex condition in while loop
set attempts to 0
set success to false
while attempts is less than 3 and success is not true
    display "Attempt [attempts plus 1]"
    // ... attempt logic here ...
    set attempts to attempts plus 1
end while
```

### For-Each Loops

#### Standard For-Each
```droe
set numbers which are list of int to [1, 2, 3, 4, 5]
for each num in numbers
    display "Number: [num]"
end for
```

#### Character Iteration
Iterate over each character in a string:
```droe
set word which is text to "hello"
for each char in word
    display "Character: [char]"
end for
```

#### Advanced For-Each Examples
```droe
// Processing user list
set users which are list of text to ["Alice", "Bob", "Charlie"]
for each user in users
    when user equals "Alice" then
        display "Welcome back, Alice!"
    otherwise
        display "Hello, [user]"
    end when
end for

// String processing
set message which is text to "Droelang"
display "Processing each character:"
for each char in message
    when char equals "a" or char equals "e" or char equals "i" or char equals "o" or char equals "u" then
        display "[char] is a vowel"
    otherwise
        display "[char] is a consonant"
    end when
end for
```

---

## Functions and Actions

### Actions (Functions)

Actions are reusable blocks of code that can accept parameters and return values.

**Basic Action (no parameters):**

```droe
action greet_user
    display "Hello, welcome to Droelang!"
end action

// Call the action
run greet_user
```

**Parameterized Action:**

```droe
action greet_person with name which is text
    display "Hello, [name]!"
    display "Welcome to Droelang!"
end action

// Call with parameter
run greet_person with "Alice"
```

**Action with Return Value:**

```droe
action calculate_area with width which is int, height which is int gives int
    give width * height
end action

// Use return value
set area which is int from calculate_area with 10, 5
display "Area: [area]"
```

**Action with Multiple Parameters:**

```droe
action create_greeting with name which is text, age which is int gives text
    give "Hello " + name + ", you are " + age + " years old!"
end action

set message which is text from create_greeting with "Bob", 30
display message
```

### Tasks

Tasks are actions that don't return values, used for procedural execution.

**Task Syntax:**

```droe
task send_reminder
    display "Don't forget to complete your tasks!"
end task

task process_order with item which is text, quantity which is int
    display "Processing order for [quantity] [item]"
end task

// Execute tasks
run send_reminder
run process_order with "widgets", 5
```

---

## Modules

Modules provide namespacing and code organization.

**Module Definition:**

```droe
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

```droe
// Call module action
set result which is int from run math_utils.add with 10, 5
display "Result: [result]"

// Direct execution
display run math_utils.add with 20, 15
```

---

## Data Structures

Define custom data types with named fields.

**Data Definition:**

```droe
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

## Database DSL

Droelang provides a native Database DSL for defining data models and performing database operations with automatic ORM generation for target frameworks.

### Data Definitions with Annotations

Define data structures with field annotations for database schema generation:

```droe
data User
    id is text key auto                    // Primary key with auto-generation
    name is text required                   // Required field
    email is text required unique           // Unique constraint
    age is int optional                     // Nullable field
    active is flag default true             // Default value
    created_at is date auto                 // Auto-timestamp
end data

data Post
    id is text key auto
    title is text required
    content is text required
    author_id is text required              // Foreign key reference
    published is flag default false
    created_at is date auto
end data
```

### Field Annotations

| Annotation        | Description          | Example                       |
| ----------------- | -------------------- | ----------------------------- |
| `key`             | Primary key field    | `id is text key`              |
| `auto`            | Auto-generated value | `id is text key auto`         |
| `required`        | Non-nullable field   | `name is text required`       |
| `optional`        | Nullable field       | `age is int optional`         |
| `unique`          | Unique constraint    | `email is text unique`        |
| `default <value>` | Default value        | `active is flag default true` |

### Database Operations

#### Create (INSERT)

```droe
// Create new record
db create User with name is "Alice", email is "alice@example.com", age is 25

// Create with variable
set new_user which is User
set new_user.name to "Bob"
set new_user.email to "bob@example.com"
db create User from new_user
```

#### Read (SELECT)

```droe
// Find single record
set user from db find User where id equals "user123"

// Find with multiple conditions
set active_user from db find User where email equals "alice@example.com" and active equals true

// Find all records
set all_users from db find all User

// Find with conditions
set adult_users from db find all User where age is greater than 18
```

#### Update

```droe
// Update single record
db update User where id equals "user123" set name is "Alice Smith", age is 26

// Update with variables
set user_id which is text to "user123"
set new_name which is text to "Alice Johnson"
db update User where id equals user_id set name is new_name
```

#### Delete

```droe
// Delete single record
db delete User where id equals "user123"

// Delete with conditions
db delete User where active equals false and age is less than 18
```

### Complete Database Example

`droeconfig.json`:

```json
{
  "target": "java",
  "framework": "spring",
  "database": { "type": "postgres" }
}
```

`src/user_management.droe`:

```droe
module user_management

    // Define User entity
    data User
        id is text key auto
        username is text required unique
        email is text required unique
        password is text required
        active is flag default true
        created_at is date auto
    end data

    // User registration
    action register_user with username which is text, email which is text, password which is text
        // Check if user exists
        set existing from db find User where email equals email
        when existing is not empty then
            display "User already exists"
            give false
        end when

        // Create new user
        db create User with username is username, email is email, password is password
        display "User registered successfully"
        give true
    end action

    // User authentication
    action authenticate with email which is text, password which is text gives User
        set user from db find User where email equals email and password equals password
        when user is empty then
            display "Invalid credentials"
            give empty
        end when
        give user
    end action

    // Update user profile
    action update_profile with user_id which is text, new_email which is text
        db update User where id equals user_id set email is new_email
        display "Profile updated"
    end action

end module
```

---

## HTTP and API DSL

Droelang provides native support for making API calls and defining HTTP server endpoints.

### Making API Calls

#### Basic API Call Syntax

```droe
call <endpoint> method <HTTP_METHOD> [with <data>] [using headers <headers>] into <response_variable>
```

#### GET Request

```droe
// Simple GET request
call "https://api.example.com/users" method GET into response
display response

// GET with headers
call "https://api.example.com/profile" method GET using headers
    Authorization: "Bearer token123"
    Accept: "application/json"
end headers into profile_data
```

#### POST Request

```droe
// POST with JSON data
set user_data which is text to '{"name": "Alice", "email": "alice@example.com"}'
call "https://api.example.com/users" method POST with user_data using headers
    Content-Type: "application/json"
    Authorization: "Bearer token123"
end headers into create_response

// POST with form data
set form_data which is text to "username=alice&password=secret"
call "https://api.example.com/login" method POST with form_data using headers
    Content-Type: "application/x-www-form-urlencoded"
end headers into login_response
```

#### PUT Request

```droe
set update_data which is text to '{"name": "Alice Smith"}'
call "https://api.example.com/users/123" method PUT with update_data using headers
    Content-Type: "application/json"
    Authorization: "Bearer token123"
end headers into update_response
```

#### DELETE Request

```droe
call "https://api.example.com/users/123" method DELETE using headers
    Authorization: "Bearer token123"
end headers into delete_response
```

### API Response Handling

```droe
// Make API call
call "https://api.example.com/data" method GET into response

// Check response status
when response.status equals 200 then
    display "Success: [response.body]"
otherwise when response.status equals 404 then
    display "Not found"
otherwise
    display "Error: [response.status]"
end when

// Parse JSON response (automatic in supported targets)
set data from response.body
display "User name: [data.name]"
```

### Defining HTTP Endpoints

You can define REST HTTP endpoints using the `serve` statement:

```droe
// GET endpoint
serve get /users/:id
    set user from db find User where id equals id
    when user is empty then
        respond 404 with "User not found"
    end when
    respond 200 with user
end serve

// POST endpoint
serve post /users
    db create User from request.body
    respond 201 with created_user
end serve

// PUT endpoint
serve put /users/:id
    db update User where id equals id set name is request.name
    respond 200 with updated_user
end serve

// DELETE endpoint
serve delete /users/:id
    db delete User where id equals id
    respond 204
end serve
```

### Complete HTTP API Example

```droe
@target droe

module blog_api

    data Article
        id is text key auto
        title is text required
        content is text required
        author is text required
        published is flag default false
        created_at is date auto
    end data

    // List all articles
    serve get /articles
        set articles from db find all Article where published equals true
        respond 200 with articles
    end serve

    // Get single article
    serve get /articles/:id
        set article from db find Article where id equals id
        when article is empty then
            respond 404 with '{"error": "Article not found"}'
        end when
        respond 200 with article
    end serve

    // Create article
    serve post /articles
        set new_article from request.body
        db create Article from new_article
        respond 201 with new_article
    end serve

    // Update article
    serve put /articles/:id
        set article from db find Article where id equals id
        when article is empty then
            respond 404 with '{"error": "Article not found"}'
        end when
        db update Article where id equals id set title is request.title
        respond 200 with updated_article
    end serve

    // Delete article
    serve delete /articles/:id
        db delete Article where id equals id
        respond 204
    end serve

    // Publish article
    serve post /articles/:id/publish
        db update Article where id equals id set published is true
        respond 200 with '{"message": "Article published"}'
    end serve

end module
```

---

## UI Components and Screens

Droelang provides a comprehensive UI DSL that compiles to web (HTML/JavaScript), Android (Kotlin), and iOS (Swift) applications using a single, consistent syntax. The system uses a fragment-based architecture with slot-based content injection for building responsive, cross-platform interfaces.

### Screens and Fragments

Screens define application views that compose fragments. Fragments provide reusable UI components with named slots for content injection, designed for scalability and mobile-first development.

#### Fragment-Based UI System

Fragments are reusable UI components that define structure with named slots, providing unlimited flexibility in UI composition for responsive design across web and mobile platforms.

```droe
module AppModule
    // Define reusable fragments with slots
    fragment AppHeader
        slot "branding" classes "brand-area" styles "padding: 10px"
        slot "navigation" classes "nav-menu"
        slot "actions" classes "header-actions" styles "margin-left: auto"
    end fragment
    
    fragment ContentArea
        slot "sidebar" classes "sidebar-panel" styles "width: 250px"
        slot "main" classes "main-content" styles "flex: 1"
    end fragment
    
    fragment AppFooter
        slot "links" classes "footer-links"
        slot "copyright" classes "copyright-text" styles "text-align: center"
    end fragment

    // Screen using fragments with slot content mapping
    screen DashboardScreen classes "dashboard-page" styles "min-height: 100vh"
        fragment AppHeader
            slot "branding":
                title "MyApp" classes "app-title, bold" styles "color: #333"
            end slot
            slot "navigation":
                button "Home" classes "nav-btn, active"
                button "Reports" classes "nav-btn"
            end slot
            slot "actions":
                button "Profile" classes "profile-btn"
            end slot
        end fragment
        
        fragment ContentArea
            slot "sidebar":
                title "Quick Actions" classes "sidebar-title"
                button "New Report" classes "action-btn"
            end slot
            slot "main":
                title "Dashboard Overview" classes "page-title"
                text "Welcome to your dashboard" classes "welcome-text"
            end slot
        end fragment
        
        fragment AppFooter
            slot "links":
                button "Privacy" classes "footer-link"
                button "Terms" classes "footer-link"
            end slot
            slot "copyright":
                text "© 2024 MyApp Inc." classes "copyright-text"
            end slot
        end fragment
    end screen
end module
```

#### Styling Attributes

The fragment system uses two distinct styling attributes that align with HTML conventions:

- **`classes`** - CSS class names (comma-separated for multiple classes)
- **`styles`** - Inline CSS styles

```droe
// Components with styling
title "Page Title" classes "heading, primary" styles "color: blue; margin-top: 20px"
text "Content" classes "paragraph" styles "font-size: 14px"
button "Submit" classes "btn, btn-primary" styles "padding: 10px 20px"

// Fragments and slots with styling  
fragment HeaderSection
    slot "brand" classes "brand-area" styles "padding: 15px"
    slot "menu" classes "navigation" styles "display: flex"
end fragment
```

#### Semantic HTML Generation

Fragments automatically generate appropriate semantic HTML tags based on their names:

| Fragment Name Contains | Generated HTML Tag |
|----------------------|-------------------|
| `header` | `<header>` |
| `footer` | `<footer>` |
| `nav`, `navigation` | `<nav>` |
| `main`, `content` | `<main>` |
| `sidebar`, `aside` | `<aside>` |
| `article` | `<article>` |
| `section` | `<section>` |
| (other) | `<div>` |

#### Fragment Definition Syntax

```droe
fragment FragmentName [classes "css-classes"] [styles "inline-styles"]
    slot "slotName" [classes "slot-classes"] [styles "slot-styles"]
    slot "anotherSlot"
    // ... more slots
end fragment
```

#### Screen Definition Syntax

```droe
screen ScreenName [classes "screen-classes"] [styles "screen-styles"]
    fragment FragmentName
        slot "slotName":
            // content components go here
        end slot
    end fragment
    // ... more fragments
end screen
```

#### Slot System in Fragments

Slots provide content injection points in fragments:

```droe
// Define slots in fragment
fragment ContentSection
    slot "title" classes "title-area"
    slot "body" classes "content-body" styles "padding: 20px"
end fragment

// Fill slots in screen
screen MyPage
    fragment ContentSection
        slot "title":
            title "Welcome" classes "page-title"
        end slot
        slot "body":
            text "Page content here" classes "body-text"
            button "Action" classes "primary-btn"
        end slot
    end fragment
end screen
```

**Slot Rules:**
- Slots without content assignments remain empty
- Multiple components can be placed in a single slot
- Slots support all UI components with their styling attributes
- Default content can be defined in the fragment definition


#### Including Fragments from Other Files

Fragments can be defined in separate files and included:

```droe
// File: fragments/app_fragments.droe
module AppFragmentModule
    fragment AppHeader
        slot "branding" classes "brand-area"
        slot "navigation" classes "nav-menu"
    end fragment
end module

// File: screens/home.droe
@include AppFragmentModule from "fragments/app_fragments.droe"

module HomeModule
    screen HomeScreen
        fragment AppHeader
            slot "branding":
                title "My App"
            end slot
        end fragment
    end screen
end module
```

### UI Components

#### Text Components

```droe
// Title component with new styling attributes
title "Page Title" classes "page-header, bold" styles "color: #333; margin-bottom: 20px"

// Text component with new styling attributes
text "This is regular text content" classes "text-content" styles "line-height: 1.6"
text "Paragraph text that can span multiple lines"
```

**Implementation Status**: Both `title` and `text` components support the new `classes` and `styles` attributes. The `title` component generates `<h2>` elements in HTML, while `text` generates `<p>` elements.

#### Input Components

```droe
// Input component with new styling attributes
input id username_field type text placeholder "Enter username" bind UserProfile.userName validate required classes "form-input, required" styles "border: 1px solid #ccc"

// Email input
input id email_field type email placeholder "your@email.com" bind UserProfile.email validate email classes "form-input"

// Password input
input id password_field type password placeholder "Password" bind LoginForm.password validate required classes "form-input"
```

**Implementation Status**: Input spec syntax is fully implemented with support for id, type, placeholder, bind, validate, classes, and styles attributes.

#### Button Components

```droe
// Button component (fully implemented)
button "Submit" action submitForm classes "submit-btn primary"
button "Save" action saveData id save_btn classes "save-btn"
```

**Implementation Status**: Button spec syntax is fully implemented with support for text, action, id, and class attributes.

#### Toggle and Selection Components

```droe
// Toggle component (fully implemented)
toggle id notifications_toggle "Enable Notifications" bind UserSettings.notificationsEnabled default on classes "toggle-field"

// Dropdown component (fully implemented)
dropdown id quality_dropdown bind UserSettings.photoQuality default "Medium Quality" classes "dropdown-field"

// Checkbox component (fully implemented)
checkbox id accept_terms "I accept the terms" bind UserForm.acceptTerms classes "checkbox-field"

// Radio button component (fully implemented)
radio id theme_option group "appTheme" "Dark Theme" bind UserSettings.appTheme default "Light" classes "radio-option"

// Textarea component (fully implemented)
textarea id description placeholder "Enter description" bind UserForm.description rows 6 classes "textarea-field"
```

**Implementation Status**: All selection and form component spec syntax methods are fully implemented with support for id, bind, default, class, and component-specific attributes.

#### Image and Media Components

```droe
// Image component (fully implemented)
image source "logo.png" alt "Company Logo" classes "logo-image" id main_logo

// Video component (fully implemented)
video source "video.mp4" controls autoplay loop muted classes "video-player" id intro_video

// Audio component (fully implemented)
audio source "audio.mp3" controls autoplay loop classes "audio-player" id background_music
```

**Implementation Status**: Image, video, and audio spec syntax methods are fully implemented with support for source, alt, controls, autoplay, loop, muted, class, and id attributes.

### Forms

Forms provide structured data collection with validation and submission handling.

```droe
form SettingsForm
  column classes "settings-container"
    title "User Settings" classes "form-title"

    column classes "form-fields"
      input id name_field text placeholder "Full Name" bind UserProfile.fullName validate required classes "form-input"
      input id email_field email placeholder "Email Address" bind UserProfile.email validate email classes "form-input"

      toggle id marketing_toggle "Receive Marketing Emails" bind UserSettings.marketingOptIn default off classes "toggle-field"

      dropdown id language_dropdown bind UserSettings.language default "English" classes "dropdown-field"
        option "English"
        option "Spanish"
        option "French"
      end dropdown

      button "Save Settings" action saveUserSettings classes "save-btn primary"
    end column
  end column
end form
```

### Data Binding

UI components can bind to data models for automatic updates:

```droe
// Data binding syntax
bind VariableName.propertyName
bind UserProfile.userName
bind Settings.isDarkMode

// Conditional binding
enabled when variable is condition
enabled when form is valid
enabled when capturedImage is not empty
```

---

## Mobile Platform Features

When targeting mobile platforms (`@metadata(platform="mobile")` or `@targets "android, ios"`), Droelang provides access to native mobile capabilities.

### Camera Integration

```droe
// Camera button component
button "Take Photo" type camera action capturePhoto permissions "camera, storage" classes "camera-btn"

// Camera action with permission handling
action capturePhoto
  when device has camera permission then
    show message "Opening camera..."
    run native camera capture
    set capturedImage to camera result
  otherwise
    show alert "Camera permission required"
  end when
end action
```

### Location Services

```droe
// Location button component
button "Get Location" type location action getLocation permissions "location" accuracy high classes "location-btn"

// Location action
action getLocation
  when device has location permission then
    show message "Getting location..."
    run native location service with accuracy high
    set currentLocation to location result
  otherwise
    show alert "Location permission required"
  end when
end action
```

### Notifications

```droe
// Show notifications
show notification "Task completed successfully!"
show notification "New message received" with sound

// Request notification permissions
when app starts then
  request notification permissions
end when
```

### Data Persistence

```droe
// Local storage operations
action saveUserData
  store UserProfile in local database
  when online then sync with cloud storage
end action

action loadUserData
  set userData from local database where userId equals currentUser.id
  set UserProfile to userData
end action
```

### Sensors and Hardware

```droe
// Device sensors (mobile only)
action detectMotion
  when device has motion sensor then
    run native motion detection
    set motionData to sensor result
  end when
end action

// Hardware features
action vibrate
  run native vibration with pattern short
end action

action playSound with soundFile which is text
  run native audio playback with soundFile
end action
```

### Native Platform Integration

Mobile compilation automatically handles:

**Android (Kotlin) Features:**

- `MainActivity.kt` with proper lifecycle management
- Android XML layouts with responsive design
- `AndroidManifest.xml` with required permissions
- Material Design components
- Gradle build configuration

**iOS (Swift) Features:**

- SwiftUI `ContentView` with navigation
- iOS-specific UI components
- `Info.plist` with privacy usage descriptions
- Core frameworks integration (CoreLocation, AVFoundation)
- Xcode project structure

### Permission Management

Droelang automatically detects required permissions based on component usage:

```droe
// These components automatically add permissions:
button "Take Photo" type camera         // → CAMERA permission
button "Get Location" type location     // → LOCATION permissions
show notification "Message"             // → NOTIFICATION permission
action storeData                        // → STORAGE permissions

// Manual permission requests
request permissions "camera, location, notifications"
```

**Android Permissions Generated:**

- `android.permission.CAMERA`
- `android.permission.ACCESS_FINE_LOCATION`
- `android.permission.ACCESS_COARSE_LOCATION`
- `android.permission.WRITE_EXTERNAL_STORAGE`

**iOS Privacy Descriptions Generated:**

- `NSCameraUsageDescription`
- `NSLocationWhenInUseUsageDescription`
- `NSPhotoLibraryUsageDescription`

---

## String Operations

### String Interpolation

Embed variables and expressions within strings using square brackets:

```droe
set name which is text to "Alice"
set age which is int to 25
set balance which is decimal to 150.50

display "Hello [name]!"                           // Hello Alice!
display "Age: [age]"                             // Age: 25
display "Balance: $[balance]"                    // Balance: $150.50
display "Status: [name] ([age] years old)"      // Status: Alice (25 years old)
```

### String Building with Interpolation Only

```droe
set first which is text to "Hello"
set second which is text to "World"
set combined to "[first] [second]"              // "Hello World"

// Mixed type interpolation
set count which is int to 5
set message to "You have [count] items"         // "You have 5 items"

// Complex expressions in interpolation
set user which is text to "John"
set points which is int to 100
set bonus which is int to 25
set status to "[user] earned [points plus bonus] total points"
```

**Note**: String concatenation using the `+` operator is not supported. Use square bracket interpolation `[variable]` for all string composition.

---

## Format Expressions

Format expressions allow precise control over how data is displayed.

### Date Formatting

```droe
set event_date which is date to "2024-12-25"

display format event_date as "MM/dd/yyyy"       // 12/25/2024
display format event_date as "dd/MM/yyyy"       // 25/12/2024
display format event_date as "MMM dd, yyyy"     // Dec 25, 2024
display format event_date as "long"             // Wednesday, December 25, 2024
```

### Decimal Formatting

```droe
set price which is decimal to 1234.56

display format price as "0.00"                  // 1234.56
display format price as "#,##0.00"              // 1,234.56
display format price as "$0.00"                 // $1234.56
```

### Number Formatting

```droe
set quantity which is int to 12345
set code which is int to 255

display format quantity as "#,##0"              // 12,345
display format code as "hex"                    // 0xFF
display format code as "0000"                   // 0255
```

### Format in Assignments

```droe
set formatted_date which is text to format event_date as "long"
set formatted_price which is text to format price as "#,##0.00"
```

---

## Include System

Import and use code from other files.

**Include Syntax:**

```droe
@include ModuleName from "path/to/ModuleName.droe"
```

**Using Included Modules:**

```droe
// File: utils/MathUtils.droe
module utils_MathUtils
    action add with a which is int, b which is int gives int
        give a plus b
    end action
end module

// File: main.droe
@include utils_MathUtils from "utils/MathUtils.droe"

set result which is int from run utils_MathUtils.add with 10, 5
display "Sum: [result]"
```

---

## Comments

### Single-line Comments

```droe
// This is a single-line comment
set name which is text to "Alice"  // End-of-line comment
```

### Multi-line Comments

```droe
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

| Category          | Keywords                                                                                                                                                                              |
| ----------------- | ------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------- |
| **Variables**     | `set`, `which`, `is`, `to`, `are`                                                                                                                                                     |
| **Control Flow**  | `when`, `then`, `otherwise`, `end`, `while`, `for`, `each`, `in`                                                                                                                      |
| **Actions**       | `action`, `task`, `with`, `gives`, `give`, `run`, `from`                                                                                                                              |
| **Modules**       | `module`, `data`                                                                                                                                                                      |
| **Imports**       | `@include`                                                                                                                                                                            |
| **Display**       | `display`                                                                                                                                                                             |
| **Types**         | `int`, `decimal`, `text`, `flag`, `yesno`, `date`, `file`, `list`, `group`, `of`                                                                                                      |
| **Logic**         | `true`, `false`, `and`, `or`, `not`, `empty`                                                                                                                                          |
| **Arithmetic**    | `plus`, `minus`, `times`, `divided`                                                                                                                                                   |
| **Comparisons**   | `equals`, `greater`, `less`, `than`, `equal`, `does`                                                                                                                                  |
| **Format**        | `format`, `as`                                                                                                                                                                        |
| **Database**      | `db`, `find`, `create`, `update`, `delete`, `where`, `all`, `key`, `auto`, `required`, `optional`, `unique`                                                                           |
| **HTTP/API**      | `call`, `method`, `GET`, `POST`, `PUT`, `DELETE`, `using`, `headers`, `into`, `respond`, `serve`, `accept`, `request`, `body`, `status`                                               |
| **UI Components** | `layout`, `screen`, `slot`, `form`, `column`, `row`, `grid`, `stack`, `overlay`, `header`, `main`, `footer`, `nav`, `section`, `article`, `aside`, `title`, `text`, `input`, `button`, `toggle`, `dropdown`, `radio`, `image`, `video`, `audio`, `textarea`, `checkbox` |
| **UI Attributes** | `id`, `class`, `placeholder`, `bind`, `validate`, `default`, `enabled`, `action`, `type`, `permissions`, `accuracy`, `source`, `alt`, `controls`, `autoplay`, `loop`, `muted`, `rows` |
| **Mobile**        | `camera`, `location`, `notification`, `native`, `device`, `sensor`, `storage`, `cloud`, `vibration`, `audio`                                                                          |

### Operators (Word-Based Only)

| Word Operator     | Meaning                 | Usage                                               |
| ----------------- | ----------------------- | --------------------------------------------------- |
| `plus`            | Addition                | `5 plus 3`                                          |
| `minus`           | Subtraction             | `10 minus 4`                                        |
| `times`           | Multiplication          | `6 times 7`                                         |
| `divided by`      | Division                | `15 divided by 3`                                   |
| `equals`          | Equality comparison     | `x equals 5`                                        |
| `is greater than` | Greater than comparison | `age is greater than 18`                            |
| `is less than`    | Less than comparison    | `score is less than 100`                            |
| `and`             | Logical AND             | `age is greater than 18 and score is less than 100` |
| `or`              | Logical OR              | `status equals "active" or status equals "pending"` |

**Special Syntax:**
| Symbol | Meaning | Usage |
|--------|---------|-------|
| `[]` | String interpolation | `"Hello [name]"` |
| `[]` | Array literals | `[1, 2, 3]` |

**Note**: Droelang intentionally avoids mathematical symbols (`+`, `-`, `*`, `/`, `>`, `<`, etc.) to maintain readability for non-technical users.

---

## Compilation Targets

Droelang compiles to multiple target languages and frameworks:

### Core Compilation Targets

| Target     | Extension      | Description                             | Framework Support    | Native Support                          |
| ---------- | -------------- | --------------------------------------- | -------------------- | --------------------------------------- |
| `wasm`     | `.wat`/`.wasm` | WebAssembly Text and Binary formats     | -                    | -                                       |
| `python`   | `.py`          | Python 3.x source code                  | FastAPI + SQLAlchemy | `http.client`, `http.server`, `sqlite3` |
| `java`     | `.java`        | Java 11+ source code                    | Spring Boot + JPA    | `HttpClient`, `HttpServer`, JDBC        |
| `node`     | `.js`          | Node.js JavaScript ES6+                 | Fastify + Prisma     | `http`, `https` (no native DB)          |
| `go`       | `.go`          | Go 1.18+ source code                    | Fiber + GORM         | `net/http`, `database/sql`              |
| `rust`     | `.rs`          | Rust source code                        | Axum + SQLx          | Framework-only                          |
| `html`     | `.html`        | HTML5 with embedded JavaScript          | Vue.js integration   | -                                       |
| `bytecode` | `.droebc`      | Droelang VM bytecode format (default)   | -                    | -                                       |
| `mobile`   | Multiple       | Android (Kotlin) + iOS (Swift) projects | Native SDKs          | -                                       |

### Target Resolution

The compiler resolves compilation targets using the following priority order:

1. **@target metadata** in source file (highest priority)
2. **target** setting in `droeconfig.json` 
3. **Default: bytecode** (for compilation speed)

### Framework-Specific Generation

When using framework configuration in `droeconfig.json`:

| Framework       | Target   | Generated Output                                                                                                              |
| --------------- | -------- | ----------------------------------------------------------------------------------------------------------------------------- |
| **Spring Boot** | `java`   | Complete Spring Boot project with JPA entities, REST controllers, services, repositories, `pom.xml`, `application.properties` |
| **FastAPI**     | `python` | FastAPI project with SQLAlchemy models, database configuration, routers, `requirements.txt`, `main.py`                        |
| **Fiber**       | `go`     | Go Fiber project with GORM models, handlers, routes, database connection, `go.mod`                                            |
| **Fastify**     | `node`   | Node.js Fastify project with Prisma schema, handlers, routes, server configuration, `package.json`                            |
| **Axum**        | `rust`   | Rust Axum project with SQLx models, handlers, routes, `Cargo.toml`                                                            |
| **Android**     | `mobile` | Android Studio project with Kotlin activities, XML layouts, Gradle build files, `AndroidManifest.xml`                         |
| **iOS**         | `mobile` | Xcode project with SwiftUI views, `Info.plist`, project configuration                                                         |

### Native vs Framework Mode

Droelang supports both framework-based and native standard library code generation:

#### Framework Mode (Default)

Uses web frameworks and ORMs for full-featured applications:

```bash
droe compile api.droe --target java --framework spring
droe compile api.droe --target python --framework fastapi
droe compile api.droe --target go --framework fiber
```

#### Native Mode

Uses only standard library features for lightweight applications:

```bash
droe compile api.droe --target java --framework plain
droe compile api.droe --target python --framework plain
droe compile api.droe --target go --framework plain
```

| Feature     | Framework Mode              | Native Mode                               |
| ----------- | --------------------------- | ----------------------------------------- |
| **Java**    | Spring Boot + JPA/Hibernate | `HttpClient` + `HttpServer` + JDBC        |
| **Python**  | FastAPI + SQLAlchemy        | `http.client` + `http.server` + `sqlite3` |
| **Go**      | Fiber + GORM                | `net/http` + `database/sql`               |
| **Node.js** | Fastify + Prisma            | `http`/`https` modules                    |
| **Rust**    | Axum + SQLx                 | Not supported                             |

### Compilation Commands

```bash
# Basic compilation
droe compile program.droe                    # Uses default or metadata target
droe compile program.droe --target python    # Compile to Python
droe compile program.droe --target java      # Compile to Java

# Framework-specific compilation
droe compile api.droe --target java --framework spring    # Spring Boot project
droe compile app.droe --target mobile                     # Android + iOS apps

# Project-based compilation (uses droeconfig.json)
droe compile                                # Uses settings from ddroeconfig.json

# Compile and run
droe run program.droe                        # Compile and execute
```

### Configuration File (droeconfig.json)

```json
{
  "src": "src",
  "build": "build",
  "dist": "dist",
  "modules": "modules",
  "main": "src/main.droe",
  "target": "java",
  "framework": "spring",
  "package": "com.example.myproject",
  "database": {
    "type": "postgres",
    "url": "postgresql://localhost/myproject_db"
  }
}
```

#### Database Configuration

All frameworks use a standardized database configuration format:

```json
{
  "database": {
    "type": "postgres|mysql|sqlite|h2",
    "url": "database_connection_url"
  }
}
```

**Examples:**

- PostgreSQL: `"url": "postgresql://localhost/mydb"`
- MySQL: `"url": "mysql://localhost/mydb"`
- SQLite: `"url": "sqlite:///path/to/db.sqlite"`
- H2 (in-memory): `"url": "jdbc:h2:mem:testdb"`

#### Framework-Specific Examples

**Spring Boot (Java):**

```json
{
  "target": "java",
  "framework": "spring",
  "package": "com.example.api",
  "database": {
    "type": "postgres",
    "url": "postgresql://localhost/springapi_db"
  }
}
```

**FastAPI (Python):**

```json
{
  "target": "python",
  "framework": "fastapi",
  "package": "my_api",
  "database": {
    "type": "postgres",
    "url": "postgresql://localhost/fastapi_db"
  }
}
```

**Fiber (Go):**

```json
{
  "target": "go",
  "framework": "fiber",
  "package": "myapi",
  "database": {
    "type": "postgres",
    "url": "postgresql://localhost/fiber_db"
  }
}
```

**Native Mode:**

```json
{
  "target": "python",
  "framework": "plain",
  "package": "simple_app",
  "database": {
    "type": "sqlite",
    "url": "sqlite:///app.db"
  }
}
```

### Spring Boot Project Structure

When compiling with `--framework spring`:

```
dist/
├── pom.xml
├── src/
│   └── main/
│       ├── java/
│       │   └── com/example/myproject/
│       │       ├── Application.java
│       │       ├── entities/
│       │       │   └── User.java
│       │       ├── repositories/
│       │       │   └── UserRepository.java
│       │       ├── services/
│       │       │   └── UserService.java
│       │       └── controllers/
│       │           └── UserController.java
│       └── resources/
│           └── application.properties
```

### Mobile Project Structure

**Android Project (Kotlin):**

```
dist/android/
├── app/
│   ├── src/main/
│   │   ├── java/com/example/myapp/
│   │   │   ├── MainActivity.kt
│   │   │   └── components/
│   │   ├── res/
│   │   │   ├── layout/
│   │   │   │   └── activity_main.xml
│   │   │   └── values/
│   │   │       └── strings.xml
│   │   └── AndroidManifest.xml
│   └── build.gradle
├── build.gradle
└── settings.gradle
```

**iOS Project (Swift):**

```
dist/ios/
├── MyApp.xcodeproj/
├── MyApp/
│   ├── ContentView.swift
│   ├── MainscreenView.swift
│   ├── Models/
│   ├── Views/
│   └── Info.plist
└── Assets.xcassets/
```

### Type Mapping Across Targets

| Droelang Type | Python     | Java        | JavaScript | Go          | Kotlin      | Swift    |
| ------------- | ---------- | ----------- | ---------- | ----------- | ----------- | -------- |
| `int`         | `int`      | `int`       | `number`   | `int`       | `Int`       | `Int`    |
| `decimal`     | `float`    | `double`    | `number`   | `float64`   | `Double`    | `Double` |
| `text`        | `str`      | `String`    | `string`   | `string`    | `String`    | `String` |
| `flag`        | `bool`     | `boolean`   | `boolean`  | `bool`      | `Boolean`   | `Bool`   |
| `date`        | `datetime` | `LocalDate` | `Date`     | `time.Time` | `LocalDate` | `Date`   |
| `list of T`   | `List[T]`  | `List<T>`   | `T[]`      | `[]T`       | `List<T>`   | `[T]`    |

### Database Type Mapping (Spring Boot)

| Droelang Annotation | JPA Annotation                     | SQL Type       |
| ------------------- | ---------------------------------- | -------------- |
| `key auto`          | `@Id @GeneratedValue`              | `SERIAL/UUID`  |
| `required`          | `@Column(nullable=false)`          | `NOT NULL`     |
| `unique`            | `@Column(unique=true)`             | `UNIQUE`       |
| `text`              | `@Column(columnDefinition="TEXT")` | `TEXT/VARCHAR` |
| `int`               | `@Column`                          | `INTEGER`      |
| `decimal`           | `@Column`                          | `DECIMAL`      |
| `flag`              | `@Column`                          | `BOOLEAN`      |
| `date auto`         | `@CreatedDate`                     | `TIMESTAMP`    |

---

## Example Programs

### Complete Example

```droe
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

Droelang enforces strong typing and will generate compilation errors for:

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

---

## Cross-Platform Mobile Example

Complete example of a Droelang application that compiles to web, Android, and iOS:

```droe
// Cross-platform photo sharing app
@name "PhotoShare"
@description "Cross-platform photo sharing application"
@package "com.example.photoshare"

// Mobile configuration should be in droeconfig.json:
// {
//   "target": "mobile",
//   "mobile": {
//     "platforms": ["android", "ios"],
//     "package": "com.example.photoshare"
//   }
// }

module photoshare

  // Main application layout
  layout MainScreen
    column classes "main-container"
      title "PhotoShare App" classes "app-title"

      column classes "form-container"
        input id name_field text placeholder "Enter your name" bind UserProfile.userName validate required classes "form-input"

        // Mobile-specific components work seamlessly
        button "Take Photo" type camera action capturePhoto permissions "camera, storage" classes "camera-btn"
        button "Get Location" type location action getLocation permissions "location" accuracy high classes "location-btn"

        image source "placeholder.jpg" alt "Your captured photo" bind UserProfile.capturedImage classes "photo-preview"

        button "Share Photo" action sharePhoto enabled when capturedImage is not empty classes "share-btn primary"
      end column
    end column
  end layout

  // Settings form with cross-platform controls
  form SettingsForm
    column classes "settings-container"
      title "App Settings" classes "form-title"

      column classes "form-fields"
        toggle id notifications_toggle "Enable Notifications" bind UserSettings.notificationsEnabled default off classes "toggle-field"

        dropdown id quality_dropdown bind UserSettings.photoQuality default "Medium Quality" classes "dropdown-field"
          option "High Quality"
          option "Medium Quality"
          option "Low Quality"
        end dropdown

        button "Save Settings" action saveSettings classes "save-btn primary"
      end column
    end column
  end form

  // Actions with platform-specific implementations
  action capturePhoto
    when device has camera permission then
      show message "Opening camera..."
      run native camera capture
      set capturedImage to camera result
    otherwise
      show alert "Camera permission required"
    end when
  end action

  action getLocation
    when device has location permission then
      show message "Getting location..."
      run native location service with accuracy high
      set currentLocation to location result
    otherwise
      show alert "Location permission required"
    end when
  end action

  action sharePhoto
    when capturedImage is not empty then
      show message "Sharing photo..."
      run task UploadPhoto
    otherwise
      show alert "Please take a photo first"
    end when
  end action

  // Data persistence across platforms
  action saveSettings
    store UserSettings in local database
    when online then sync with cloud storage
    show notification "Settings saved successfully!"
  end action

  // Background tasks
  task UploadPhoto
    step "Compress image for upload"
    step "Upload to cloud server"
    step "Generate shareable link"

    when task completed then
      show notification "Photo shared successfully!"
    end when

    when task failed then
      show alert "Upload failed. Please try again."
    end when
  end task

  // Cross-platform data models
  data Photo
    id is text required
    imagePath is text required
    location is text optional
    timestamp is date required
    userName is text required
    isShared is flag default false
  end data

end module
```

This single Droelang file generates:

- **Web**: HTML/JavaScript with responsive design
- **Android**: Complete Kotlin project with Material Design
- **iOS**: SwiftUI project with native iOS components

---

## Framework Adapters

Droelang provides framework adapters that automatically generate idiomatic code for popular frameworks.

### Backend Framework Adapters

#### Spring Boot Adapter (Java)

The Spring Boot adapter generates a complete, production-ready Spring Boot application:

**Features:**

- **JPA Entities**: Automatic generation from `data` definitions with proper annotations
- **REST Controllers**: Full CRUD endpoints from `serve` statements
- **Service Layer**: Business logic separation
- **Repository Layer**: Spring Data JPA repositories
- **Configuration**: `application.properties` with database configuration
- **Build System**: Maven `pom.xml` with all required dependencies

**Configuration:**

```json
{
  "target": "java",
  "framework": "spring",
  "package": "com.example.api",
  "database": {
    "type": "postgres",
    "url": "postgresql://localhost/mydb"
  }
}
```

#### FastAPI Adapter (Python)

The FastAPI adapter generates a modern, async Python web API:

**Features:**

- **SQLAlchemy Models**: ORM models from `data` definitions
- **Pydantic Schemas**: Request/response validation
- **REST Endpoints**: Async CRUD APIs from `serve` statements
- **Database Integration**: SQLAlchemy session management
- **Dependencies**: `requirements.txt` with FastAPI, SQLAlchemy, and database drivers

**Configuration:**

```json
{
  "target": "python",
  "framework": "fastapi",
  "package": "my_api",
  "database": {
    "type": "postgres",
    "url": "postgresql://localhost/mydb"
  }
}
```

#### Axum Adapter (Rust)

The Axum adapter generates a high-performance Rust web server:

**Features:**

- **SQLx Models**: Compile-time checked database queries
- **Axum Handlers**: Type-safe HTTP handlers
- **REST Endpoints**: Async CRUD APIs from `serve` statements
- **Database Pool**: Connection pooling with SQLx
- **Build System**: `Cargo.toml` with Axum, SQLx, and database drivers

**Configuration:**

```json
{
  "target": "rust",
  "framework": "axum",
  "package": "my-api",
  "database": {
    "type": "postgres",
    "url": "postgresql://localhost/mydb"
  }
}
```

#### Fiber Adapter (Go)

The Fiber adapter generates a fast Go web API with GORM:

**Features:**

- **GORM Models**: Go structs with database tags from `data` definitions
- **Fiber Handlers**: High-performance HTTP handlers
- **REST Endpoints**: Complete CRUD APIs from `serve` statements
- **Database Integration**: GORM ORM with auto-migration
- **Go Module**: `go.mod` with Fiber, GORM, and database drivers

**Configuration:**

```json
{
  "target": "go",
  "framework": "fiber",
  "package": "my_api",
  "database": {
    "type": "postgres",
    "url": "postgresql://localhost/mydb"
  }
}
```

#### Fastify Adapter (Node.js)

The Fastify adapter generates a Node.js web API with Prisma:

**Features:**

- **Prisma Schema**: Type-safe database schema from `data` definitions
- **Fastify Routes**: Fast HTTP routing from `serve` statements
- **REST Endpoints**: Complete CRUD APIs with validation
- **Database Integration**: Prisma ORM with type generation
- **Package System**: `package.json` with Fastify, Prisma, and database drivers

**Configuration:**

```json
{
  "target": "node",
  "framework": "fastify",
  "package": "my-api",
  "database": {
    "type": "postgres",
    "url": "postgresql://localhost/mydb"
  }
}
```

### Mobile Framework Adapters

#### Android Adapter (Kotlin)

Generates a complete Android Studio project:

- **Activities**: Kotlin activities with lifecycle management
- **Layouts**: XML layouts with Material Design
- **Permissions**: Automatic `AndroidManifest.xml` configuration
- **Build System**: Gradle with dependencies
- **Components**: Native Android components from Droelang UI DSL

#### iOS Adapter (Swift)

Generates a complete Xcode project:

- **SwiftUI Views**: Modern declarative UI
- **Navigation**: Automatic navigation setup
- **Permissions**: `Info.plist` with usage descriptions
- **Project Structure**: Standard iOS app structure
- **Components**: Native iOS components from Droelang UI DSL

### Supported Framework Features

| Target  | Framework    | Database ORM    | API Type              | Generated Files   | Status         |
| ------- | ------------ | --------------- | --------------------- | ----------------- | -------------- |
| Java    | Spring Boot  | JPA/Hibernate   | REST Controllers      | Maven project     | ✅ Implemented |
| Python  | FastAPI      | SQLAlchemy      | REST + Async          | Python package    | ✅ Implemented |
| Rust    | Axum         | SQLx            | REST + Async          | Cargo project     | ✅ Implemented |
| Go      | Fiber        | GORM            | REST + Fast HTTP      | Go module         | ✅ Implemented |
| Node.js | Fastify      | Prisma          | REST + TypeScript     | npm package       | ✅ Implemented |
| HTML    | None         | N/A             | Client-side JS        | Static HTML/CSS   | ✅ Implemented |
| Mobile  | Kotlin/Swift | Room/CoreData\* | Retrofit/URLSession\* | Native projects\* | 🚧 Partial     |

### Database Support by Framework

| Framework   | PostgreSQL | MySQL | SQLite | In-Memory |
| ----------- | ---------- | ----- | ------ | --------- |
| Spring Boot | ✅         | ✅    | ✅     | ✅ (H2)   |
| FastAPI     | ✅         | ✅    | ✅     | ❌        |
| Axum        | ✅         | ✅    | ✅     | ❌        |
| Fiber       | ✅         | ✅    | ✅     | ❌        |
| Fastify     | ✅         | ✅    | ✅     | ❌        |

**Note**: Framework selection is configured in `droeconfig.json`, not as file-level annotations. HTML target generates static frontend code without framework dependencies.

### Framework Usage Examples

#### Example: User Management API

**Shared Droelang Code (`src/api.droe`):**

```droe
data User
    id is text key auto
    name is text required
    email is text unique required
    created_at is date auto
end data

serve get /api/users
end serve

serve post /api/users
end serve

serve get /api/users/:id
end serve

serve put /api/users/:id
end serve

serve delete /api/users/:id
end serve
```

**Java Spring Boot Configuration:**

```json
{
  "target": "java",
  "framework": "spring",
  "package": "com.example.userapi",
  "database": {
    "type": "postgres",
    "url": "postgresql://localhost/userdb"
  }
}
```

**Python FastAPI Configuration:**

```json
{
  "target": "python",
  "framework": "fastapi",
  "package": "user_api",
  "database": {
    "type": "postgres",
    "url": "postgresql://localhost/userdb"
  }
}
```

**Go Fiber Configuration:**

```json
{
  "target": "go",
  "framework": "fiber",
  "package": "user_api",
  "database": {
    "type": "postgres",
    "url": "postgresql://localhost/userdb"
  }
}
```

**Rust Axum Configuration:**

```json
{
  "target": "rust",
  "framework": "axum",
  "package": "user-api",
  "database": {
    "type": "postgres",
    "url": "postgresql://localhost/userdb"
  }
}
```

**Node.js Fastify Configuration:**

```json
{
  "target": "node",
  "framework": "fastify",
  "package": "user-api",
  "database": {
    "type": "postgres",
    "url": "postgresql://localhost/userdb"
  }
}
```

### Best Practices

1. **Consistent Configuration**: Use the same database URL format across all frameworks
2. **Type Safety**: Leverage Droelang's strong typing for database mapping
3. **REST Conventions**: Use standard HTTP methods in serve statements
4. **Database Support**: Choose appropriate database types for your framework
5. **Package Naming**: Follow language conventions for package names (snake_case for Python, kebab-case for Rust, etc.)

### Generated Project Structure

Each framework adapter generates a complete, production-ready project:

**Java Spring Boot:**

```
src/main/java/com/example/userapi/
├── Application.java          # Main application class
├── model/User.java           # JPA entity
├── repository/UserRepository.java  # Spring Data repository
├── service/UserService.java # Business logic
└── controller/UserController.java  # REST endpoints
src/main/resources/
├── application.properties    # Database configuration
pom.xml                      # Maven dependencies
```

**Python FastAPI:**

```
user_api/
├── main.py                  # FastAPI application
├── models.py                # SQLAlchemy models
├── database.py              # Database session
├── routers.py               # API endpoints
└── __init__.py
requirements.txt             # Dependencies
```

**Go Fiber:**

```
├── main.go                  # Fiber server
├── models.go                # GORM models
├── database.go              # Database connection
├── routes.go                # CRUD routes
├── handlers.go              # Custom handlers
├── go.mod                   # Go module
└── .env                     # Environment variables
```

            for each post in posts
                column classes "post-card"
                    title bind post.title classes "post-title"
                    text bind post.content classes "post-content"
                    text "By: " + post.author classes "post-author"
                end column
            end for

            button "Load Posts" action loadPosts classes "btn-primary"
        end column
    end layout

    action loadPosts
        call "/api/posts" method GET into response
        set posts from response.body
    end action

end module

```

This single Droelang file generates:
- Complete Spring Boot backend with database
- Responsive HTML/JavaScript frontend
- Automatic API integration
- Type-safe data transfer

### Complete Screen/Layout Example

This example demonstrates the full screen/layout system with slot-based content injection:

```droe
// Complete dashboard application with modular layout system
module DashboardApp

    // Define reusable layout with semantic HTML structure
    layout ResponsiveLayout
        header classes "app-header"
            nav classes "main-navigation"
                slot "navigation"
            end nav
            aside classes "user-section"
                slot "user-info"
            end aside
        end header
        
        main classes "content-wrapper"
            section classes "sidebar"
                slot "sidebar-content"
            end section
            article classes "main-content"
                slot "content"  // Default content slot
            end article
        end main
        
        footer classes "app-footer"
            slot "footer-content"
        end footer
    end layout

    // Dashboard screen using the layout with multiple slot assignments
    screen DashboardScreen layout="ResponsiveLayout"
        // Fill navigation slot
        slot "navigation":
            title "MyDashboard"
            text "Admin Panel"
            button "Home" action navigateHome classes "nav-btn"
            button "Analytics" action showAnalytics classes "nav-btn"
            button "Reports" action showReports classes "nav-btn"
        end slot
        
        // Fill user info slot
        slot "user-info":
            text "Welcome, Administrator!"
            text "Last login: Today"
            button "Profile" action openProfile classes "user-btn"
            button "Logout" action logout classes "logout-btn"
        end slot
        
        // Fill sidebar slot
        slot "sidebar-content":
            title "Quick Actions"
            text "• View Statistics"
            text "• Generate Reports"
            text "• Manage Users"
            text "• System Settings"
            
            form "quick-filter"
                title "Filter Data"
                input "Date Range" type="date" placeholder="Select date"
                button "Apply Filter" action applyFilter class="filter-btn"
            end form
        end slot
        
        // Fill footer slot
        slot "footer-content":
            text "© 2024 Dashboard Inc."
            text "Version 2.1.0"
            text "Support: help@dashboard.com"
        end slot
        
        // Default content (goes to main "content" slot)
        title "Dashboard Overview"
        text "Welcome to your comprehensive dashboard with real-time analytics."
        
        column classes "dashboard-widgets"
            title "Key Metrics"
            text "Total Users: 1,247"
            text "Active Sessions: 89"
            text "Revenue: $45,892"
            
            form "dashboard-actions"
                title "Quick Actions"
                input "Search" type="text" placeholder="Search users..."
                button "Search" action performSearch class="search-btn"
                button "Export Data" action exportData class="export-btn"
            end form
        end column
    end screen

    // Alternative screen using the same layout
    screen ReportsScreen layout="ResponsiveLayout"
        slot "navigation":
            title "Reports"
            button "Dashboard" action navigateHome class="nav-btn"
            button "Analytics" action showAnalytics class="nav-btn active"
        end slot
        
        slot "user-info":
            text "Reports Module"
            button "Settings" action openSettings class="user-btn"
        end slot
        
        slot "sidebar-content":
            title "Report Types"
            text "• Sales Reports"
            text "• User Analytics"
            text "• Performance Data"
        end slot
        
        slot "footer-content":
            text "© 2024 Dashboard Inc."
        end slot
        
        // Main content for reports
        title "Analytics Reports"
        text "Generate and view comprehensive reports."
        
        form "report-generator"
            title "Generate Report"
            input "Report Type" type="text" placeholder="Enter report type"
            input "Date From" type="date"
            input "Date To" type="date"
            button "Generate" action generateReport class="generate-btn"
        end form
    end screen

    // Actions for the application
    action navigateHome
        // Navigation logic
    end action
    
    action showAnalytics
        // Show analytics view
    end action
    
    action logout
        // Logout logic
    end action

end module
```

**Generated Output Features:**

This single Droelang module generates:

- **Semantic HTML**: Proper `<header>`, `<nav>`, `<aside>`, `<main>`, `<section>`, `<article>`, `<footer>` structure
- **Responsive Layout**: CSS classes for mobile-responsive design
- **Slot-Based Content**: Flexible content injection with named slots
- **Reusable Layouts**: Same layout used by multiple screens
- **Component Composition**: Forms, inputs, buttons with proper bindings
- **Cross-Platform**: Same code generates web, Android, and iOS applications

**Key Features Demonstrated:**

1. **Nested Container Support**: Navigation inside header, sections inside main
2. **Multiple Slot Assignments**: Each screen fills 4+ different slots
3. **Default Content Handling**: Unassigned content goes to "content" slot
4. **Layout Reusability**: Single layout used by multiple screens
5. **Rich Component Support**: Titles, text, buttons, forms, and inputs
6. **CSS Class Integration**: Styling preserved across all components

---

*This specification covers Droelang version 3.0 with Database DSL, API DSL, Screen/Layout System, and Framework Adapter support. For updates and examples, visit the official Droelang documentation.*
