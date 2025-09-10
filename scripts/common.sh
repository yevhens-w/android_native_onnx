#!/bin/bash

# Common utilities for build scripts
# Source this file in other scripts: source "$(dirname "$0")/common.sh"

# Configuration
export ONNX_VERSION="1.22.0"
export PROJECT_ROOT="$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)"
export ANDROID_LIBS_DIR="$PROJECT_ROOT/android-libs"
export APP_DATA_DIR="$PROJECT_ROOT/app_data"

# Colors and logging
RED='\033[0;31m'
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
BLUE='\033[0;34m'
NC='\033[0m' # No Color

log_info() {
    echo -e "${BLUE}ℹ️  $1${NC}"
}

log_success() {
    echo -e "${GREEN}✅ $1${NC}"
}

log_warning() {
    echo -e "${YELLOW}⚠️  $1${NC}"
}

log_error() {
    echo -e "${RED}❌ $1${NC}"
    exit 1
}

log_step() {
    echo -e "${BLUE}🔄 $1...${NC}"
}

# Platform detection
is_macos() {
    [[ "$OSTYPE" == "darwin"* ]]
}

is_linux() {
    [[ "$OSTYPE" == "linux-gnu"* ]]
}

# Command checks
check_command() {
    if ! command -v "$1" &> /dev/null; then
        log_error "$1 is not installed or not in PATH"
    fi
}

check_android_sdk() {
    if [ -z "$ANDROID_HOME" ]; then
        log_error "ANDROID_HOME environment variable is not set"
    fi
}

check_xcode() {
    if ! is_macos; then
        log_error "Xcode builds require macOS"
    fi
    check_command "xcodebuild"
}

# File operations
ensure_dir() {
    mkdir -p "$1"
}

file_exists() {
    [ -f "$1" ]
}

dir_exists() {
    [ -d "$1" ]
}

file_not_empty() {
    [ -s "$1" ]
}

# Download with retry
download_file() {
    local url="$1"
    local output="$2"
    local max_attempts=3
    local attempt=1
    
    while [ $attempt -le $max_attempts ]; do
        log_info "Download attempt $attempt/$max_attempts: $(basename "$output")"
        if curl -L --fail -o "$output" "$url"; then
            if file_not_empty "$output"; then
                log_success "Downloaded $(basename "$output") ($(du -h "$output" | cut -f1))"
                return 0
            else
                log_warning "Downloaded file is empty"
                rm -f "$output"
            fi
        fi
        ((attempt++))
        [ $attempt -le $max_attempts ] && sleep 2
    done
    
    return 1
}

# ONNX Runtime utilities
setup_onnx_android() {
    log_step "Setting up ONNX Runtime for Android"
    
    ensure_dir "$ANDROID_LIBS_DIR/onnxruntime/{arm64-v8a,armeabi-v7a,x86_64}"
    
    local aar_file="$ANDROID_LIBS_DIR/onnxruntime-android-${ONNX_VERSION}.aar"
    
    # Download if not exists or empty
    if ! file_not_empty "$aar_file"; then
        rm -f "$aar_file"
        
        # Try multiple sources
        local urls=(
            "https://github.com/microsoft/onnxruntime/releases/download/v${ONNX_VERSION}/onnxruntime-android-${ONNX_VERSION}.aar"
            "https://repo1.maven.org/maven2/com/microsoft/onnxruntime/onnxruntime-android/${ONNX_VERSION}/onnxruntime-android-${ONNX_VERSION}.aar"
        )
        
        local downloaded=false
        for url in "${urls[@]}"; do
            if download_file "$url" "$aar_file"; then
                downloaded=true
                break
            fi
        done
        
        if ! $downloaded; then
            log_error "Failed to download ONNX Runtime AAR. Please download manually from Maven Central."
        fi
    fi
    
    # Extract libraries if not already done
    if [ ! -f "$ANDROID_LIBS_DIR/onnxruntime/arm64-v8a/libonnxruntime.so" ]; then
        log_step "Extracting ONNX Runtime libraries"
        
        cd "$ANDROID_LIBS_DIR"
        rm -rf temp_aar
        
        if ! unzip -q "$(basename "$aar_file")" -d temp_aar; then
            log_error "Failed to extract AAR file"
        fi
        
        if [ ! -d "temp_aar/jni" ]; then
            log_error "Invalid AAR structure - no jni directory found"
        fi
        
        # Copy libraries for each architecture
        local lib_copied=false
        for arch in arm64-v8a armeabi-v7a x86_64; do
            if [ -f "temp_aar/jni/$arch/libonnxruntime.so" ]; then
                cp "temp_aar/jni/$arch/libonnxruntime.so" "onnxruntime/$arch/"
                log_success "Copied $arch library"
                lib_copied=true
            else
                log_warning "$arch library not found in AAR"
            fi
        done
        
        if ! $lib_copied; then
            log_error "No ONNX Runtime libraries were extracted"
        fi
        
        # Copy headers if available
        [ -d "temp_aar/headers" ] && cp -r temp_aar/headers onnxruntime/
        
        rm -rf temp_aar
        cd "$PROJECT_ROOT"
    fi
    
    log_success "ONNX Runtime Android setup complete"
}

# Build utilities
copy_rust_libs_android() {
    log_step "Copying Rust libraries to Android project"
    
    local jni_dir="platforms/android/app/src/main/jniLibs"
    ensure_dir "$jni_dir/{arm64-v8a,armeabi-v7a,x86_64}"
    
    local targets=("aarch64-linux-android:arm64-v8a" "armv7-linux-androideabi:armeabi-v7a" "x86_64-linux-android:x86_64")
    
    for target_mapping in "${targets[@]}"; do
        local rust_target="${target_mapping%:*}"
        local android_arch="${target_mapping#*:}"
        local lib_path="target/$rust_target/release/libonnx_inference.so"
        
        if file_exists "$lib_path"; then
            cp "$lib_path" "$jni_dir/$android_arch/"
            log_success "Copied $android_arch library ($(du -h "$lib_path" | cut -f1))"
        else
            log_warning "$android_arch library not found at $lib_path"
        fi
    done
}

copy_model_to_android() {
    log_step "Copying model and labels to Android assets"
    
    local assets_dir="platforms/android/app/src/main/assets"
    ensure_dir "$assets_dir"
    
    # Copy ONNX model
    local model_file="$APP_DATA_DIR/resnet50.onnx"
    if file_exists "$model_file"; then
        cp "$model_file" "$assets_dir/"
        log_success "Copied ONNX model to assets"
    else
        log_warning "No ONNX model found at $model_file"
    fi
    
    # Copy ImageNet labels
    local labels_file="$APP_DATA_DIR/imagenet_labels.txt"
    if file_exists "$labels_file"; then
        cp "$labels_file" "$assets_dir/"
        log_success "Copied ImageNet labels to assets ($(wc -l < "$labels_file") labels)"
    else
        log_warning "No ImageNet labels found at $labels_file"
    fi
}

# Clean utilities
clean_rust() {
    log_step "Cleaning Rust artifacts"
    cargo clean
}

clean_android() {
    if dir_exists "platforms/android"; then
        log_step "Cleaning Android artifacts"
        cd platforms/android
        [ -f "gradlew" ] && ./gradlew clean
        cd "$PROJECT_ROOT"
        rm -rf platforms/android/app/src/main/jniLibs
    fi
}

clean_ios() {
    if is_macos && dir_exists "platforms/ios/SharedCoreApp"; then
        log_step "Cleaning iOS artifacts"
        cd platforms/ios/SharedCoreApp
        command -v xcodebuild &> /dev/null && xcodebuild clean -project SharedCoreApp.xcodeproj
        cd "$PROJECT_ROOT"
    fi
}