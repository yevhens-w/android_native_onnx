# Rust Mobile Cross-Platform Project

A complete project skeleton for building cross-platform mobile applications with:
- **Rust** shared core library with hardware acceleration support
- **Android** app in Kotlin  
- **iOS** app in Swift
- **UniFFI** for seamless Rust-to-mobile bindings
- **Dev Container** for consistent development environment

## 🏗️ Architecture

```
project/
├── .devcontainer/          # Development container setup
├── src/                    # Rust shared library
├── platforms/
│   ├── android/           # Android Kotlin app
│   └── ios/               # iOS Swift app
└── scripts/               # Build automation scripts
```

## 🚀 How to Run This Project

### **Option 1: Dev Container (Recommended)**

1. **Prerequisites:**
   - VS Code with Dev Containers extension
   - Docker Desktop

2. **Steps:**
   ```bash
   # Open project in VS Code
   code .
   
   # Click "Reopen in Container" when prompted
   # OR press Ctrl/Cmd+Shift+P and select "Dev Containers: Reopen in Container"
   
   # Wait for container setup (first time takes ~5-10 minutes)
   
   # Build Rust library
   ./scripts/build-rust.sh
   
   # Generate bindings
   ./scripts/generate-bindings.sh
   
   # Test the build
   cargo test
   ```

### **Option 2: Local Development**

1. **Install Prerequisites:**
   ```bash
   # Install Rust
   curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh
   source ~/.cargo/env
   
   # Add mobile targets
   rustup target add aarch64-linux-android armv7-linux-androideabi x86_64-linux-android
   rustup target add aarch64-apple-ios x86_64-apple-ios aarch64-apple-ios-sim
   
   # Install tools
   cargo install cargo-ndk uniffi_bindgen
   ```

2. **Android Setup:**
   ```bash
   # Download Android Studio or Command Line Tools
   # Set ANDROID_HOME environment variable
   export ANDROID_HOME=/path/to/android/sdk
   export PATH=$PATH:$ANDROID_HOME/cmdline-tools/latest/bin:$ANDROID_HOME/platform-tools
   
   # Install NDK
   sdkmanager "ndk;25.2.9519653"
   ```

3. **Build and Run:**
   ```bash
   # Build Rust library
   ./scripts/build-rust.sh
   
   # Generate bindings
   ./scripts/generate-bindings.sh
   
   # Run tests
   ./scripts/run-tests.sh
   ```

## 📱 Running Mobile Apps

### **Android App**

```bash
# Build APK
./scripts/build-android.sh

# Install and run (with device/emulator connected)
cd platforms/android
./gradlew installDebug
adb shell am start -n com.example.sharedcoreapp/.MainActivity
```

**Or in Android Studio:**
1. Open `platforms/android/` in Android Studio
2. Sync Gradle files
3. Run the app (green play button)

### **iOS App (macOS only)**

```bash
# Build for simulator
./scripts/build-ios.sh

# Or open in Xcode
open platforms/ios/SharedCoreApp/SharedCoreApp.xcodeproj
```

**In Xcode:**
1. Select target device/simulator
2. Click Run (▶️) button

## ⚡ Quick Commands

```bash
# Build everything
./scripts/build-rust.sh && ./scripts/generate-bindings.sh

# Clean everything
./scripts/clean.sh

# Run all tests
./scripts/run-tests.sh

# Build specific platform
./scripts/build-android.sh  # Android
./scripts/build-ios.sh      # iOS (macOS only)
```

## 🎯 What You Should See

### **After successful build:**
- `target/` directory with compiled Rust libraries
- Generated bindings in `platforms/android/app/src/main/java/uniffi/`
- Generated bindings in `platforms/ios/generated/`

### **Mobile apps show:**
- "✓ Rust library loaded successfully" status
- Working greeting function
- Async data processing
- Compute backend information

### **Expected output:**
```
🦀 Building Rust shared library...
Building for host platform...
   Compiling shared-core v0.1.0
    Finished dev [unoptimized + debuginfo] target(s) in 2.31s

Building for Android targets...
   Compiling shared-core v0.1.0
    Finished release [optimized] target(s) in 1.45s

✅ Rust library build complete!
```

## 📱 Platform Details

### Rust Core Library

**Location:** `src/`

**Features:**
- UniFFI bindings for seamless mobile integration
- Hardware acceleration support (GPU + CPU fallback)
- Async/await support
- Comprehensive error handling
- Cross-platform compute backend

**Key Files:**
- `src/lib.rs` - Main library interface
- `src/shared_core.udl` - UniFFI interface definition
- `src/compute.rs` - Hardware acceleration layer
- `Cargo.toml` - Dependencies and features

### Android App

**Location:** `platforms/android/`

**Features:**
- Kotlin with Coroutines
- Material Design 3
- ViewBinding
- JNA for UniFFI integration

**Key Files:**
- `app/src/main/java/com/example/sharedcoreapp/MainActivity.kt`
- `build.gradle.kts` - Gradle configuration
- `app/src/main/AndroidManifest.xml`

### iOS App

**Location:** `platforms/ios/`

**Features:**
- SwiftUI interface
- Async/await integration
- Native iOS styling

**Key Files:**
- `SharedCoreApp/ContentView.swift` - Main UI
- `SharedCoreApp.xcodeproj/` - Xcode project

## 📄 License

This project template is provided as-is for educational and development purposes.

## 📚 Resources

- [UniFFI Guide](https://mozilla.github.io/uniffi-rs/)
- [Rust Mobile Development](https://github.com/rust-mobile)
- [Android NDK](https://developer.android.com/ndk)
- [Cargo NDK](https://github.com/bbqsrc/cargo-ndk)

---

**Happy coding! 🦀📱**