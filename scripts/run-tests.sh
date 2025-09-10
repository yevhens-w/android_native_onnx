#!/bin/bash

set -e
source "$(dirname "$0")/common.sh"

log_info "Running tests"

# Run Rust tests
log_step "Running Rust tests"
cargo test

log_step "Running Rust tests with GPU features"
cargo test --features gpu

# Run Android tests (if Android SDK is available)
if [ -n "$ANDROID_HOME" ]; then
    log_step "Running Android tests"
    cd platforms/android
    ./gradlew test
    cd "$PROJECT_ROOT"
else
    log_warning "Skipping Android tests - ANDROID_HOME not set"
fi

# Run iOS tests (if on macOS)
if is_macos && command -v xcodebuild &> /dev/null; then
    log_step "Running iOS tests"
    cd platforms/ios/SharedCoreApp
    xcodebuild \
        -project SharedCoreApp.xcodeproj \
        -scheme SharedCoreApp \
        -destination 'platform=iOS Simulator,name=iPhone 15,OS=latest' \
        test
    cd "$PROJECT_ROOT"
else
    log_warning "Skipping iOS tests - not on macOS or Xcode not available"
fi

log_success "All tests completed"