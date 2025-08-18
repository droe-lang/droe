# Droelang Fragment-Based Screen System

**Version:** 4.0  
**Date:** January 2025  
**Status:** Active

## Overview

Droelang's fragment-based screen system provides a flexible, scalable approach to creating user interfaces. Unlike the previous fixed container system (`header`, `main`, `footer`), fragments allow developers to create named, reusable UI sections with custom slot arrangements.

## Core Concepts

### 1. Fragments

Fragments are reusable UI components that define a structure with named slots. They replace the old layout system.

```droe
fragment HeaderFragment
    slot "logo" classes "logo-container" styles "padding: 10px"
    slot "navigation" classes "nav-links"
    slot "actions" classes "header-actions" styles "margin-left: auto"
end fragment
```

### 2. Screens

Screens compose fragments and fill their slots with content.

```droe
screen Dashboard
    fragment HeaderFragment
        slot "logo":
            title "My App" classes "brand-title"
        end slot
        slot "navigation":
            button "Home" classes "nav-btn"
            button "Profile" classes "nav-btn"
        end slot
    end fragment
    
    fragment ContentFragment
        slot "main":
            title "Welcome to Dashboard" classes "page-title"
            text "Your dashboard content here" classes "description"
        end slot
    end fragment
end screen
```

### 3. Styling Attributes

The new system uses two distinct styling attributes:

- **`classes`** - CSS class names (comma-separated)
- **`styles`** - Inline CSS styles

```droe
title "Hello World" classes "primary-heading, bold" styles "color: blue; margin-top: 20px"
```

## Syntax Reference

### Fragment Definition

```droe
fragment FragmentName
    slot "slotName" [classes "class1, class2"] [styles "css: properties"]
    slot "anotherSlot"
    // ... more slots
end fragment
```

### Screen Definition

```droe
screen ScreenName [classes "screen-classes"] [styles "screen-styles"]
    fragment FragmentName
        slot "slotName":
            // content goes here
        end slot
    end fragment
end screen
```

### Component Syntax

All UI components now support both styling attributes:

```droe
title "Text" classes "class1, class2" styles "color: red"
text "Content" classes "paragraph" styles "font-size: 14px"
button "Click Me" classes "btn, primary" styles "padding: 10px"
input id "username" classes "form-input" styles "width: 100%"
```

## HTML Generation

The fragment system generates semantic HTML based on fragment names:

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

### Example Output

```droe
fragment HeaderSection
    slot "brand"
    slot "menu"
end fragment

screen Home
    fragment HeaderSection
        slot "brand":
            title "My App"
        end slot
    end fragment
end screen
```

Generates:

```html
<div id="screen-Home-1">
    <header id="fragment-HeaderSection-1" data-fragment="HeaderSection">
        <div id="slot-brand-1" data-slot="brand">
            <h2>My App</h2>
        </div>
        <div id="slot-menu-2" data-slot="menu">
            <!-- Empty slot -->
        </div>
    </header>
</div>
```

## Migration from Layout System

### Before (Layout System)
```droe
layout AppLayout
    header class "app-header"
        slot "logo"
    end header
    main class "content"
        slot "main"
    end main
end layout

screen Home layout="AppLayout"
    slot "logo":
        title "My App"
    end slot
    slot "main":
        text "Welcome"
    end slot
end screen
```

### After (Fragment System)
```droe
fragment AppHeader
    slot "logo" classes "app-header"
end fragment

fragment AppMain  
    slot "main" classes "content"
end fragment

screen Home
    fragment AppHeader
        slot "logo":
            title "My App"
        end slot
    end fragment
    
    fragment AppMain
        slot "main":
            text "Welcome"
        end slot
    end fragment
end screen
```

## Advantages

1. **Flexibility** - No predefined containers, create any structure
2. **Mobile-First** - Better responsive design capabilities  
3. **Reusability** - Fragments can be reused across screens
4. **Semantic HTML** - Automatic semantic tag generation
5. **Clear Separation** - Distinct `classes` and `styles` attributes
6. **Scalability** - Easily add new fragments and slots

## Complete Example

```droe
module AppModule
    // Define reusable fragments
    fragment PageHeader
        slot "branding" classes "brand-area"
        slot "navigation" classes "nav-area" 
        slot "search" classes "search-area"
    end fragment
    
    fragment ContentArea
        slot "sidebar" classes "sidebar" styles "width: 250px"
        slot "main" classes "main-content" styles "flex: 1"
        slot "widgets" classes "widget-panel"
    end fragment
    
    fragment PageFooter
        slot "links" classes "footer-links"
        slot "copyright" classes "copyright"
    end fragment
    
    // Define screen using fragments
    screen Dashboard
        fragment PageHeader
            slot "branding":
                title "My Application" classes "app-title" styles "font-size: 24px"
            end slot
            slot "navigation":
                button "Home" classes "nav-btn, active"
                button "Reports" classes "nav-btn"
                button "Settings" classes "nav-btn"
            end slot
        end fragment
        
        fragment ContentArea
            slot "sidebar":
                title "Quick Actions" classes "sidebar-title"
                button "New Report" classes "action-btn"
                button "View Data" classes "action-btn"
            end slot
            slot "main":
                title "Dashboard Overview" classes "page-title"
                text "Welcome to your dashboard" classes "welcome-text"
            end slot
        end fragment
        
        fragment PageFooter
            slot "copyright":
                text "© 2025 My Application" classes "copyright-text"
            end slot
        end fragment
    end screen
end module
```

This generates semantic, accessible HTML with proper CSS class and inline style support, providing maximum flexibility for responsive design across web and mobile platforms.