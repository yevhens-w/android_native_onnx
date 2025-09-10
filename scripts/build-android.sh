#!/bin/bash

set -e
source "$(dirname "$0")/common.sh"

log_info "Building Android app"

# Check prerequisites
check_android_sdk

# Build Rust library
log_step "Building Rust library for Android"
"$PROJECT_ROOT/scripts/build-rust.sh"

# Copy libraries and assets
copy_rust_libs_android
copy_model_to_android

# Verify copied libraries
echo ""
log_info "Final Android library status:"
find platforms/android/app/src/main/jniLibs -name "*.so" -exec ls -lh {} \; | sed 's/^/   /' || log_warning "No libraries found"

# Build Android app
log_step "Building Android APK"
cd platforms/android
if gradle assembleDebug 2>/dev/null; then
    log_success "Android build complete"
    echo "APK location: platforms/android/app/build/outputs/apk/debug/app-debug.apk"
else
    log_warning "Gradle build failed or wrapper not available. Please build from Android Studio instead."
fi