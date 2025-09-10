#!/bin/bash

set -e
source "$(dirname "$0")/common.sh"

log_info "Building iOS app"

# Check prerequisites
check_xcode

# Build Rust library
log_step "Building Rust library for iOS"
"$PROJECT_ROOT/scripts/build-rust.sh"

# Build iOS app
log_step "Building iOS app"
cd platforms/ios/SharedCoreApp

xcodebuild \
    -project SharedCoreApp.xcodeproj \
    -scheme SharedCoreApp \
    -configuration Debug \
    -destination 'platform=iOS Simulator,name=iPhone 15,OS=latest' \
    build

cd "$PROJECT_ROOT"

log_success "iOS build complete"
echo ""
echo "To run in simulator:"
echo "1. Open platforms/ios/SharedCoreApp/SharedCoreApp.xcodeproj in Xcode"
echo "2. Select your target device/simulator"
echo "3. Click Run (▶️)"