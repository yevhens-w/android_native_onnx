#!/bin/bash

echo "Setting up development environment..."

# Update Rust to latest stable
rustup update stable

# Verify Android setup
echo "Android SDK: $ANDROID_HOME"
echo "Android NDK: $ANDROID_NDK_HOME"

# Install additional tools if needed
echo "Installing additional development tools..."

# Set up git if not already configured
if [ -z "$(git config --global user.name)" ]; then
    echo "Please configure git with your name and email:"
    echo "git config --global user.name 'Your Name'"
    echo "git config --global user.email 'your.email@example.com'"
fi

echo "Development environment setup complete!"
echo ""
echo "Available commands:"
echo "  - cargo build                    : Build Rust library"
echo "  - ./scripts/build-android.sh     : Build Android bindings"
echo "  - ./scripts/build-ios.sh         : Build iOS bindings (requires macOS)"
echo "  - ./scripts/run-tests.sh         : Run all tests"