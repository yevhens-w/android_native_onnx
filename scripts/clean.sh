#!/bin/bash

set -e
source "$(dirname "$0")/common.sh"

log_info "Cleaning build artifacts"

# Clean all build artifacts
clean_rust
clean_android
clean_ios

# Clean generated bindings
log_step "Cleaning generated bindings"
rm -rf platforms/android/app/src/main/java/uniffi
rm -rf platforms/ios/generated

log_success "Clean complete"