# Mobile Target Migration Guide

## ✨ New Mobile Target System

The Roelang compiler now includes a comprehensive mobile target that generates complete **Android (Kotlin)** and **iOS (Swift)** projects, replacing the previous basic kotlin/swift targets.

## 🔄 Migration Required

### ❌ Old Syntax (No Longer Works)
```roe
@target kotlin
// or
@target swift
```

### ✅ New Syntax (Recommended)
```roe
// Generate for mobile platforms only
@metadata(platform="mobile")

// Or generate for specific platforms
@targets "android, ios"

// Or generate for all platforms
@targets "web, android, ios"
```

## 📱 What You Get Now

### Before (Old Targets)
- ❌ Single Kotlin/Swift files
- ❌ No project structure
- ❌ No Android/iOS specific features
- ❌ No permission handling
- ❌ No build configuration

### After (New Mobile Target)
- ✅ Complete Android project with Gradle
- ✅ Complete iOS project with Xcode configuration
- ✅ Native mobile components (Camera, Location, etc.)
- ✅ Automatic permission declarations
- ✅ Platform-specific UI generation
- ✅ Jinja2 template-based customization

## 📋 Generated Project Structure

### Android Output
```
MyApp/
├── app/
│   ├── src/main/java/com/example/myapp/
│   │   ├── MainActivity.kt
│   │   └── [Form]Activity.kt
│   ├── res/layout/
│   │   ├── activity_main.xml
│   │   └── [layout].xml
│   ├── AndroidManifest.xml
│   └── build.gradle
└── build.gradle
```

### iOS Output  
```
MyApp/
├── ContentView.swift
├── MyAppApp.swift
├── Views/
│   ├── MainView.swift
│   └── [Form]FormView.swift
├── Models/
│   └── DataModels.swift
├── Info.plist
└── MyApp.xcodeproj/
    └── project.pbxproj
```

## 🚀 Example Migration

### Old DSL
```roe
@target kotlin

show "Hello Mobile"
```

### New DSL
```roe
@metadata(platform="mobile", name="MyApp", package="com.example.myapp")

module myapp
  layout MainScreen
    column class "container"
      title "Hello Mobile" class "app-title"
      
      // Mobile-specific components
      button "Take Photo" type camera action capturePhoto
      button "Get Location" type location action getLocation
      
      button "Say Hello" action sayHello
    end column
  end layout
  
  action capturePhoto
    when device has camera permission then
      show message "Opening camera..."
      run native camera capture
    otherwise
      show alert "Camera permission required"  
    end when
  end action
  
  action getLocation
    when device has location permission then
      show message "Getting location..."
      run native location service
    otherwise
      show alert "Location permission required"
    end when
  end action
  
  action sayHello
    show toast "Hello from mobile!"
  end action
end module
```

## 🎯 Target Selection Guide

| Use Case | Recommended Syntax |
|----------|-------------------|
| Mobile only | `@metadata(platform="mobile")` |
| Web only | `@target html` |  
| Cross-platform | `@targets "web, android, ios"` |
| Android only | `@targets "android"` |
| iOS only | `@targets "ios"` |

## 🔧 Available Mobile Components

The new mobile target supports these components:

- **Standard**: `title`, `input`, `button`, `textarea`, `dropdown`, `toggle`, `checkbox`, `radio`, `image`, `video`, `audio`
- **Mobile-specific**: `Camera`, `Location`, `Notification`, `Storage`, `Sensor`, `Contact`

### Example Mobile Components
```roe
// Camera integration
button "Take Photo" type camera action capturePhoto permissions "camera, storage"

// Location services  
button "Get Location" type location action getLocation permissions "location" accuracy high

// Notifications
button "Send Notification" type notification action sendNotification permissions "notifications"
```

## 📦 Compilation Instructions

```bash
# Compile to mobile platforms
roe compile myapp.roe

# Output will be generated in:
# examples/output/android/  (if android in targets)
# examples/output/ios/      (if ios in targets)
```

## 🆘 Need Help?

If you encounter issues migrating from the old kotlin/swift targets:

1. **Update your metadata** to use the new syntax
2. **Add mobile components** for enhanced functionality  
3. **Test compilation** with the new mobile target
4. **Check generated projects** in the output directories

The new mobile target is much more powerful and generates production-ready mobile applications!