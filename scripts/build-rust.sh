#!/bin/bash

set -e
source "$(dirname "$0")/common.sh"

log_info "Building Rust shared library"

# Set ONNX Runtime environment variables for system installation
export ORT_STRATEGY=system
export ORT_LIB_LOCATION=/opt/homebrew

# Setup Android ONNX Runtime if needed
setup_onnx_android

log_step "Building Android targets"

# Build with proper library paths for each architecture
export RUSTFLAGS="-L $ANDROID_LIBS_DIR/onnxruntime/arm64-v8a"
cargo ndk --target aarch64-linux-android --platform 24 build --release

export RUSTFLAGS="-L $ANDROID_LIBS_DIR/onnxruntime/armeabi-v7a"
cargo ndk --target armv7-linux-androideabi --platform 24 build --release

export RUSTFLAGS="-L $ANDROID_LIBS_DIR/onnxruntime/x86_64"
cargo ndk --target x86_64-linux-android --platform 24 build --release

log_success "Rust library build complete"