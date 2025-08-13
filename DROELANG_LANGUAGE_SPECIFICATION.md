# Droelang Language Specification

**Version:** 3.0  
**Date:** January 2025  
**Authors:** Droelang Development Team

## Table of Contents

1. [Overview](#overview)
2. [Lexical Structure](#lexical-structure)
3. [Metadata Annotations](#metadata-annotations)
4. [Data Types](#data-types)
5. [Variables and Assignment](#variables-and-assignment)
6. [Expressions](#expressions)
7. [Control Flow](#control-flow)
8. [Functions and Actions](#functions-and-actions)
9. [Modules](#modules)
10. [Data Structures](#data-structures)
11. [Database DSL](#database-dsl)
12. [API DSL](#api-dsl)
13. [UI Components and Layouts](#ui-components-and-layouts)
14. [Mobile Platform Features](#mobile-platform-features)
15. [String Operations](#string-operations)
16. [Format Expressions](#format-expressions)
17. [Include System](#include-system)
18. [Comments](#comments)
19. [Keywords](#keywords)
20. [Compilation Targets](#compilation-targets)
21. [Framework Adapters](#framework-adapters)

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

## Lexical Structure

### Case Sensitivity
Droelang is **case-sensitive**. Keywords must be lowercase.

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

## Metadata Annotations

Metadata annotations provide compile-time information about Droelang programs. They are specified at the beginning of a file using the `@key value` syntax.

### Syntax

```roe
@key value
@key "quoted value"
@key 'single quoted value'
```

### Standard Metadata Keys

#### @target
Specifies the compilation target, overriding command-line options and project configuration.

**Supported targets:**
- `wasm` - WebAssembly (default)
- `python` - Python
- `java` - Java  
- `javascript` or `node` - JavaScript/Node.js
- `go` - Go
- `html` - HTML with JavaScript
- `bytecode` - Droelang bytecode

#### @targets
**Note: This annotation is not currently implemented in the compiler.**

Multi-target compilation should be configured in `roeconfig.json` instead:

```json
{
    "target": "mobile",
    "mobile": {
        "platforms": ["android", "ios"]
    }
}
```

#### @metadata
Complex metadata with key-value parameters for advanced configuration.

```roe
@metadata(platform="mobile", name="MyApp", package="com.example.myapp")
@metadata(platform="web", framework="react")
```

**Note:** For mobile development, you must use project-level configuration in `roeconfig.json` rather than file-level metadata annotations.

```roe
@target roe
@name user_authentication
@description "User authentication and authorization module"

display "This will compile to RoeVM bytecode"
```

#### @name
Specifies the module or component name. Useful for documentation and tooling.

```roe
@name "Shopping Cart"
@name shopping_cart_module
```

#### @description  
Provides a human-readable description of the module's purpose.

```roe
@description "Handles user profile management and validation"
@description "A utility module for mathematical operations"
```

#### @package
Specifies the package name for mobile applications.

```roe
@package "com.example.myapp"
@package "org.company.project"
```

#### Custom Metadata
You can define custom metadata keys for your specific use cases:

```roe
@version "1.2.0"
@author "Development Team"
@license "MIT"
@category "utilities"
```

### Rules and Constraints

1. **Placement**: Metadata annotations must appear at the top of the file before any code statements
2. **Order**: Metadata can appear in any order
3. **Uniqueness**: Each metadata key should appear only once per file
4. **Values**: Values can be unquoted (single word) or quoted (for multi-word values)
5. **Case Sensitivity**: Metadata keys are case-sensitive

### Examples

**Basic metadata:**
```roe
@target java
@name PaymentProcessor
@description "Processes payment transactions and validations"

// Your code here
display "Payment system initialized"
```

**Web application module:**
```roe
@target javascript  
@name "User Profile Form"
@description "Interactive form for user profile management"
@version "2.1.0"

// Your code here
set user_name which is text to "John Doe"
display user_name
```

**Backend service:**
```roe
@target go
@name api_gateway
@description "API gateway with authentication and rate limiting"
@author "Backend Team"

// Your code here
display "API Gateway starting..."
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

| Type | Description | Examples |
|------|-------------|----------|
| `int` | 32-bit signed integer | `42`, `-10`, `0` |
| `decimal` | Double-precision floating point | `3.14`, `-0.5`, `99.99` |
| `text` | Unicode string | `"Hello"`, `"Droelang"` |
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
    display "Hello, welcome to Droelang!"
end action

// Call the action
run greet_user
```

**Parameterized Action:**
```roe
action greet_person with name which is text
    display "Hello, " + name + "!"
    display "Welcome to Droelang!"
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

## Database DSL

Droelang provides a native Database DSL for defining data models and performing database operations with automatic ORM generation for target frameworks.

### Data Definitions with Annotations

Define data structures with field annotations for database schema generation:

```roe
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

| Annotation | Description | Example |
|------------|-------------|---------|
| `key` | Primary key field | `id is text key` |
| `auto` | Auto-generated value | `id is text key auto` |
| `required` | Non-nullable field | `name is text required` |
| `optional` | Nullable field | `age is int optional` |
| `unique` | Unique constraint | `email is text unique` |
| `default <value>` | Default value | `active is flag default true` |

### Database Operations

#### Create (INSERT)
```roe
// Create new record
db create User with name is "Alice", email is "alice@example.com", age is 25

// Create with variable
set new_user which is User
set new_user.name to "Bob"
set new_user.email to "bob@example.com"
db create User from new_user
```

#### Read (SELECT)
```roe
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
```roe
// Update single record
db update User where id equals "user123" set name is "Alice Smith", age is 26

// Update with variables
set user_id which is text to "user123"
set new_name which is text to "Alice Johnson"
db update User where id equals user_id set name is new_name
```

#### Delete
```roe
// Delete single record
db delete User where id equals "user123"

// Delete with conditions
db delete User where active equals false and age is less than 18
```

### Complete Database Example

`roeconfig.json`:
```json
{
    "target": "java",
    "framework": "spring",
    "database": {"type": "postgres"}
}
```

`src/user_management.droe`:
```roe
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
```roe
call <endpoint> method <HTTP_METHOD> [with <data>] [using headers <headers>] into <response_variable>
```

#### GET Request
```roe
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
```roe
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
```roe
set update_data which is text to '{"name": "Alice Smith"}'
call "https://api.example.com/users/123" method PUT with update_data using headers
    Content-Type: "application/json"
    Authorization: "Bearer token123"
end headers into update_response
```

#### DELETE Request
```roe
call "https://api.example.com/users/123" method DELETE using headers
    Authorization: "Bearer token123"
end headers into delete_response
```

### API Response Handling

```roe
// Make API call
call "https://api.example.com/data" method GET into response

// Check response status
when response.status equals 200 then
    display "Success: " + response.body
otherwise when response.status equals 404 then
    display "Not found"
otherwise
    display "Error: " + response.status
end when

// Parse JSON response (automatic in supported targets)
set data from response.body
display "User name: " + data.name
```

### Defining HTTP Endpoints

You can define REST HTTP endpoints using the `serve` statement:

```roe
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

```roe
@target roe

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

## UI Components and Layouts

Droelang provides a comprehensive UI DSL that compiles to web (HTML/JavaScript), Android (Kotlin), and iOS (Swift) applications using a single, consistent syntax.

### Layouts

Layouts define the structure and organization of UI components.

**Column Layout:**
```roe
layout MainScreen
  column class "main-container"
    title "My Application" class "app-title"
    
    column class "content-section"
      text "Welcome to the app!" class "welcome-text"
      button "Get Started" action startApp class "primary-btn"
    end column
  end column
end layout
```

**Layout Attributes:**
- `class` - CSS class for web, styling hints for mobile
- Automatic responsive design for different screen sizes

### UI Components

#### Text Components
```roe
// Static text
text "Hello World" class "greeting"

// Dynamic text with data binding
text bind UserProfile.displayName class "user-name"

// Title text
title "Page Title" class "page-header"
```

#### Input Components
```roe
// Text input with validation
input id username_field text placeholder "Enter username" bind UserProfile.userName validate required class "form-input"

// Email input
input id email_field email placeholder "your@email.com" bind UserProfile.email validate email class "form-input"

// Password input
input id password_field password placeholder "Password" bind LoginForm.password validate required class "form-input"
```

#### Button Components
```roe
// Action button
button "Submit" action submitForm class "submit-btn primary"

// Button with conditional enabling
button "Save" action saveData enabled when form is valid class "save-btn"

// Mobile-specific buttons (see Mobile Platform Features)
button "Take Photo" type camera action capturePhoto permissions "camera, storage"
```

#### Toggle and Selection Components
```roe
// Toggle switch
toggle id notifications_toggle "Enable Notifications" bind UserSettings.notificationsEnabled default off class "toggle-field"

// Dropdown selection
dropdown id quality_dropdown bind UserSettings.photoQuality default "Medium Quality" class "dropdown-field"
  option "High Quality"
  option "Medium Quality" 
  option "Low Quality"
end dropdown

// Radio button group
radio id theme_radio group "appTheme" bind UserSettings.appTheme default "System Default" class "radio-group"
  option "Light Theme"
  option "Dark Theme" 
  option "System Default"
end radio
```

#### Image Components
```roe
// Static image
image source "logo.png" alt "Company Logo" class "logo-image"

// Dynamic image with data binding
image source bind UserProfile.profilePicture alt "Profile Picture" class "profile-pic"
```

### Forms

Forms provide structured data collection with validation and submission handling.

```roe
form SettingsForm
  column class "settings-container"
    title "User Settings" class "form-title"
    
    column class "form-fields"
      input id name_field text placeholder "Full Name" bind UserProfile.fullName validate required class "form-input"
      input id email_field email placeholder "Email Address" bind UserProfile.email validate email class "form-input"
      
      toggle id marketing_toggle "Receive Marketing Emails" bind UserSettings.marketingOptIn default off class "toggle-field"
      
      dropdown id language_dropdown bind UserSettings.language default "English" class "dropdown-field"
        option "English"
        option "Spanish"
        option "French"
      end dropdown
      
      button "Save Settings" action saveUserSettings class "save-btn primary"
    end column
  end column
end form
```

### Data Binding

UI components can bind to data models for automatic updates:

```roe
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

```roe
// Camera button component
button "Take Photo" type camera action capturePhoto permissions "camera, storage" class "camera-btn"

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

```roe
// Location button component
button "Get Location" type location action getLocation permissions "location" accuracy high class "location-btn"

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

```roe
// Show notifications
show notification "Task completed successfully!" 
show notification "New message received" with sound

// Request notification permissions
when app starts then
  request notification permissions
end when
```

### Data Persistence

```roe
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

```roe
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

```roe
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
include "path/to/ModuleName.droe"
```

**Using Included Modules:**
```roe
// File: utils/MathUtils.droe
module utils_MathUtils
    action add with a which is int, b which is int gives int
        give a + b
    end action
end module

// File: main.droe
include "utils/MathUtils.droe"

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
| **Logic** | `true`, `false`, `and`, `or`, `not`, `empty` |
| **Comparisons** | `equals`, `greater`, `less`, `than`, `equal`, `does` |
| **Format** | `format`, `as` |
| **Database** | `db`, `find`, `create`, `update`, `delete`, `where`, `all`, `key`, `auto`, `required`, `optional`, `unique` |
| **HTTP/API** | `call`, `method`, `GET`, `POST`, `PUT`, `DELETE`, `using`, `headers`, `into`, `respond`, `serve`, `accept`, `request`, `body`, `status` |
| **UI Components** | `layout`, `form`, `column`, `title`, `text`, `input`, `button`, `toggle`, `dropdown`, `radio`, `image`, `option` |
| **UI Attributes** | `id`, `class`, `placeholder`, `bind`, `validate`, `default`, `enabled`, `action`, `type`, `permissions`, `accuracy` |
| **Mobile** | `camera`, `location`, `notification`, `native`, `device`, `sensor`, `storage`, `cloud`, `vibration`, `audio` |

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

Droelang compiles to multiple target languages and frameworks:

### Core Compilation Targets

| Target | Extension | Description | Framework Support | Native Support |
|--------|-----------|-------------|-------------------|----------------|
| `wasm` | `.wat`/`.wasm` | WebAssembly Text and Binary formats | - | - |
| `python` | `.py` | Python 3.x source code | FastAPI + SQLAlchemy | `http.client`, `http.server`, `sqlite3` |
| `java` | `.java` | Java 11+ source code | Spring Boot + JPA | `HttpClient`, `HttpServer`, JDBC |
| `node` | `.js` | Node.js JavaScript ES6+ | Fastify + Prisma | `http`, `https` (no native DB) |  
| `go` | `.go` | Go 1.18+ source code | Fiber + GORM | `net/http`, `database/sql` |
| `rust` | `.rs` | Rust source code | Axum + SQLx | Framework-only |
| `html` | `.html` | HTML5 with embedded JavaScript | Vue.js integration | - |
| `bytecode` | `.droebc` | Droelang VM bytecode format | - | - |
| `mobile` | Multiple | Android (Kotlin) + iOS (Swift) projects | Native SDKs | - |

### Framework-Specific Generation

When using framework configuration in `roeconfig.json`:

| Framework | Target | Generated Output |
|-----------|--------|------------------|
| **Spring Boot** | `java` | Complete Spring Boot project with JPA entities, REST controllers, services, repositories, `pom.xml`, `application.properties` |
| **FastAPI** | `python` | FastAPI project with SQLAlchemy models, database configuration, routers, `requirements.txt`, `main.py` |
| **Fiber** | `go` | Go Fiber project with GORM models, handlers, routes, database connection, `go.mod` |
| **Fastify** | `node` | Node.js Fastify project with Prisma schema, handlers, routes, server configuration, `package.json` |
| **Axum** | `rust` | Rust Axum project with SQLx models, handlers, routes, `Cargo.toml` |
| **Android** | `mobile` | Android Studio project with Kotlin activities, XML layouts, Gradle build files, `AndroidManifest.xml` |
| **iOS** | `mobile` | Xcode project with SwiftUI views, `Info.plist`, project configuration |

### Native vs Framework Mode

Droelang supports both framework-based and native standard library code generation:

#### Framework Mode (Default)
Uses web frameworks and ORMs for full-featured applications:
```bash
roe compile api.droe --target java --framework spring
roe compile api.droe --target python --framework fastapi
roe compile api.droe --target go --framework fiber
```

#### Native Mode
Uses only standard library features for lightweight applications:
```bash
roe compile api.droe --target java --framework plain
roe compile api.droe --target python --framework plain  
roe compile api.droe --target go --framework plain
```

| Feature | Framework Mode | Native Mode |
|---------|----------------|-------------|
| **Java** | Spring Boot + JPA/Hibernate | `HttpClient` + `HttpServer` + JDBC |
| **Python** | FastAPI + SQLAlchemy | `http.client` + `http.server` + `sqlite3` |
| **Go** | Fiber + GORM | `net/http` + `database/sql` |
| **Node.js** | Fastify + Prisma | `http`/`https` modules |
| **Rust** | Axum + SQLx | Not supported |

### Compilation Commands

```bash
# Basic compilation
roe compile program.droe                    # Uses default or metadata target
roe compile program.droe --target python    # Compile to Python
roe compile program.droe --target java      # Compile to Java

# Framework-specific compilation
roe compile api.droe --target java --framework spring    # Spring Boot project
roe compile app.droe --target mobile                     # Android + iOS apps

# Project-based compilation (uses roeconfig.json)
roe compile                                # Uses settings from roeconfig.json

# Compile and run
roe run program.droe                        # Compile and execute
```

### Configuration File (roeconfig.json)

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

| Droelang Type | Python | Java | JavaScript | Go | Kotlin | Swift |
|--------------|--------|------|------------|-------|--------|-------|
| `int` | `int` | `int` | `number` | `int` | `Int` | `Int` |
| `decimal` | `float` | `double` | `number` | `float64` | `Double` | `Double` |
| `text` | `str` | `String` | `string` | `string` | `String` | `String` |
| `flag` | `bool` | `boolean` | `boolean` | `bool` | `Boolean` | `Bool` |
| `date` | `datetime` | `LocalDate` | `Date` | `time.Time` | `LocalDate` | `Date` |
| `list of T` | `List[T]` | `List<T>` | `T[]` | `[]T` | `List<T>` | `[T]` |

### Database Type Mapping (Spring Boot)

| Droelang Annotation | JPA Annotation | SQL Type |
|--------------------|----------------|----------|
| `key auto` | `@Id @GeneratedValue` | `SERIAL/UUID` |
| `required` | `@Column(nullable=false)` | `NOT NULL` |
| `unique` | `@Column(unique=true)` | `UNIQUE` |
| `text` | `@Column(columnDefinition="TEXT")` | `TEXT/VARCHAR` |
| `int` | `@Column` | `INTEGER` |
| `decimal` | `@Column` | `DECIMAL` |
| `flag` | `@Column` | `BOOLEAN` |
| `date auto` | `@CreatedDate` | `TIMESTAMP` |

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

```roe
// Cross-platform photo sharing app
@name "PhotoShare" 
@description "Cross-platform photo sharing application"
@package "com.example.photoshare"

// Mobile configuration should be in roeconfig.json:
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
    column class "main-container"
      title "PhotoShare App" class "app-title"
      
      column class "form-container"
        input id name_field text placeholder "Enter your name" bind UserProfile.userName validate required class "form-input"
        
        // Mobile-specific components work seamlessly
        button "Take Photo" type camera action capturePhoto permissions "camera, storage" class "camera-btn"
        button "Get Location" type location action getLocation permissions "location" accuracy high class "location-btn"
        
        image source "placeholder.jpg" alt "Your captured photo" bind UserProfile.capturedImage class "photo-preview"
        
        button "Share Photo" action sharePhoto enabled when capturedImage is not empty class "share-btn primary"
      end column
    end column
  end layout

  // Settings form with cross-platform controls
  form SettingsForm
    column class "settings-container"
      title "App Settings" class "form-title"
      
      column class "form-fields"
        toggle id notifications_toggle "Enable Notifications" bind UserSettings.notificationsEnabled default off class "toggle-field"
        
        dropdown id quality_dropdown bind UserSettings.photoQuality default "Medium Quality" class "dropdown-field"
          option "High Quality"
          option "Medium Quality" 
          option "Low Quality"
        end dropdown
        
        button "Save Settings" action saveSettings class "save-btn primary"
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

| Target | Framework | Database ORM | API Type | Generated Files | Status |
|--------|-----------|--------------|----------|-----------------|--------|
| Java | Spring Boot | JPA/Hibernate | REST Controllers | Maven project | ✅ Implemented |
| Python | FastAPI | SQLAlchemy | REST + Async | Python package | ✅ Implemented |
| Rust | Axum | SQLx | REST + Async | Cargo project | ✅ Implemented |
| Go | Fiber | GORM | REST + Fast HTTP | Go module | ✅ Implemented |
| Node.js | Fastify | Prisma | REST + TypeScript | npm package | ✅ Implemented |
| HTML | None | N/A | Client-side JS | Static HTML/CSS | ✅ Implemented |
| Mobile | Kotlin/Swift | Room/CoreData* | Retrofit/URLSession* | Native projects* | 🚧 Partial |

### Database Support by Framework

| Framework | PostgreSQL | MySQL | SQLite | In-Memory |
|-----------|------------|-------|--------|-----------|
| Spring Boot | ✅ | ✅ | ✅ | ✅ (H2) |
| FastAPI | ✅ | ✅ | ✅ | ❌ |
| Axum | ✅ | ✅ | ✅ | ❌ |
| Fiber | ✅ | ✅ | ✅ | ❌ |
| Fastify | ✅ | ✅ | ✅ | ❌ |

**Note**: Framework selection is configured in `roeconfig.json`, not as file-level annotations. HTML target generates static frontend code without framework dependencies.

### Framework Usage Examples

#### Example: User Management API

**Shared Droelang Code (`src/api.droe`):**
```roe
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
                column class "post-card"
                    title bind post.title class "post-title"
                    text bind post.content class "post-content"
                    text "By: " + post.author class "post-author"
                end column
            end for
            
            button "Load Posts" action loadPosts class "btn-primary"
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

---

*This specification covers Droelang version 3.0 with Database DSL, API DSL, and Framework Adapter support. For updates and examples, visit the official Droelang documentation.*