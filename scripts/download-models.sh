#!/bin/bash

set -e
source "$(dirname "$0")/common.sh"

log_info "Downloading ONNX models"

# Ensure app_data directory exists
ensure_dir "$APP_DATA_DIR"
ensure_dir "platforms/android/app/src/main/assets"

# Download ResNet50 model from ONNX Model Zoo
RESNET50_URL="https://github.com/onnx/models/raw/main/vision/classification/resnet/model/resnet50-v2-7.onnx"
RESNET50_FILE="$APP_DATA_DIR/resnet50.onnx"

if ! file_not_empty "$RESNET50_FILE"; then
    log_step "Downloading ResNet50 model"
    if download_file "$RESNET50_URL" "$RESNET50_FILE"; then
        log_success "Downloaded ResNet50 model ($(du -h "$RESNET50_FILE" | cut -f1))"
    else
        log_error "Failed to download ResNet50 from ONNX Model Zoo. You can manually download from: $RESNET50_URL"
    fi
else
    log_success "ResNet50 model already exists ($(du -h "$RESNET50_FILE" | cut -f1))"
fi

# Ensure ImageNet labels exist
LABELS_FILE="$APP_DATA_DIR/imagenet_labels.txt"
if ! file_not_empty "$LABELS_FILE"; then
    log_step "Downloading ImageNet labels"
    if download_file "https://raw.githubusercontent.com/pytorch/hub/master/imagenet_classes.txt" "$LABELS_FILE"; then
        log_success "Downloaded ImageNet labels ($(wc -l < "$LABELS_FILE") labels)"
    else
        log_error "Failed to download ImageNet labels"
    fi
else
    log_success "ImageNet labels already exist ($(wc -l < "$LABELS_FILE") labels)"
fi

# Copy to Android assets
cp "$RESNET50_FILE" platforms/android/app/src/main/assets/
cp "$LABELS_FILE" platforms/android/app/src/main/assets/
log_success "Copied model and labels to Android assets"

echo ""
log_info "Available files in app_data:"
find "$APP_DATA_DIR" \( -name "*.onnx" -o -name "*.txt" \) -exec ls -lh {} \; | sed 's/^/   /'
echo ""
log_success "Models and labels ready for Android app"