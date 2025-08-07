# Roelang Language Specification

**Version:** 2.0  
**Date:** August 2024  
**Authors:** Roelang Development Team

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
11. [UI Components and Layouts](#ui-components-and-layouts)
12. [Mobile Platform Features](#mobile-platform-features)
13. [String Operations](#string-operations)
14. [Format Expressions](#format-expressions)
15. [Include System](#include-system)
16. [Comments](#comments)
17. [Keywords](#keywords)
18. [Compilation Targets](#compilation-targets)

---

## Overview

Roelang is a domain-specific language designed for business logic, process automation, and cross-platform application development. It features a readable, English-like syntax with strong typing, modern language constructs, and cross-platform compilation to multiple target languages including WebAssembly, Python, Java, JavaScript, HTML, and mobile platforms (Android/iOS).

### Design Principles

- **Readability First**: Natural, English-like syntax
- **Strong Typing**: Explicit type declarations prevent runtime errors
- **Multi-Target**: Compile to multiple languages and platforms
- **Modular**: Support for modules and code reuse
- **Modern**: Support for collections, string interpolation, and format expressions
- **Metadata-Driven**: Built-in support for metadata annotations for compilation control and documentation
- **Cross-Platform UI**: Single DSL syntax generates web, Android, and iOS applications
- **Mobile-First**: Built-in support for mobile-specific features like camera, GPS, sensors, and notifications

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

## Metadata Annotations

Metadata annotations provide compile-time information about Roelang programs. They are specified at the beginning of a file using the `@key value` syntax.

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
- `bytecode` - Roelang bytecode

#### @targets
Specifies multiple compilation targets as a comma-separated string.

```roe
@targets "web, android, ios"
@targets "python, java"
```

#### @metadata
Complex metadata with key-value parameters for advanced configuration.

```roe
@metadata(platform="mobile", name="MyApp", package="com.example.myapp")
@metadata(platform="web", framework="react")
```

**Note:** For mobile development, it's recommended to use project-level configuration in `roeconfig.json` rather than file-level metadata annotations.

```roe
@target python
@name user_authentication
@description "User authentication and authorization module"

display "This will compile to Python"
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

## UI Components and Layouts

Roelang provides a comprehensive UI DSL that compiles to web (HTML/JavaScript), Android (Kotlin), and iOS (Swift) applications using a single, consistent syntax.

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

When targeting mobile platforms (`@metadata(platform="mobile")` or `@targets "android, ios"`), Roelang provides access to native mobile capabilities.

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

Roelang automatically detects required permissions based on component usage:

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
| `bytecode` | `.roebc` | Roelang VM bytecode |

### Mobile Platform Targets

When using `@metadata(platform="mobile")` or `@targets "android, ios"`, Roelang generates complete mobile projects:

| Platform | Generated Files | Description |
|----------|----------------|-------------|
| **Android** | Project folder with Kotlin | Complete Android Studio project with `MainActivity.kt`, XML layouts, `AndroidManifest.xml`, Gradle build files |
| **iOS** | Project folder with Swift | Complete Xcode project with SwiftUI `ContentView.swift`, `Info.plist`, project configuration |

### Compilation Commands
```bash
# Compile to specific target
roe compile program.roe --target java
roe compile program.roe --target python
roe compile program.roe --target wasm

# Compile to HTML (web application)
roe compile webapp.roe --target html

# Compile mobile application (generates both Android and iOS)
roe compile mobile_app.roe --target mobile

# Compile and run
roe run program.roe
```

### Mobile Project Structure

Mobile compilation creates complete, buildable projects:

**Android Project Structure:**
```
dist/android/
├── app/
│   ├── src/main/
│   │   ├── java/com/example/myapp/
│   │   │   └── MainActivity.kt
│   │   ├── res/
│   │   │   └── layout/
│   │   │       └── activity_main.xml
│   │   └── AndroidManifest.xml
│   └── build.gradle
├── build.gradle
└── settings.gradle
```

**iOS Project Structure:**
```
dist/ios/
├── MyApp.xcodeproj/
├── ContentView.swift
├── MainscreenView.swift
├── Info.plist
└── Assets.xcassets/
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

---

## Cross-Platform Mobile Example

Complete example of a Roelang application that compiles to web, Android, and iOS:

```roe
// Cross-platform photo sharing app
@name "PhotoShare"
@description "Cross-platform photo sharing application"
@targets "web, android, ios"
@package "com.example.photoshare"

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

This single Roelang file generates:
- **Web**: HTML/JavaScript with responsive design
- **Android**: Complete Kotlin project with Material Design
- **iOS**: SwiftUI project with native iOS components

---

*This specification covers Roelang version 2.0 with mobile DSL support. For updates and examples, visit the official Roelang documentation.*