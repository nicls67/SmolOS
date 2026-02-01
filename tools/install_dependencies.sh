#!/usr/bin/env bash

# This script installs the dependencies for the SmolOS project.
# It targets STM32F769I development on Ubuntu/Debian.

set -e

echo "Installing system dependencies..."
if command -v apt-get &> /dev/null; then
    sudo apt-get update
    sudo apt-get install -y \
        cmake \
        ninja-build \
        gcc-arm-none-eabi \
        libnewlib-arm-none-eabi \
        python3 \
        python3-pip \
        python3-yaml \
        pkg-config \
        libusb-1.0-0-dev \
        tar
else
    echo "Warning: apt-get not found. Please install cmake, ninja, gcc-arm-none-eabi, libnewlib-arm-none-eabi, python3, pip, python3-yaml, pkg-config, libusb-1.0-0-dev, and tar manually."
fi

echo "Installing Python dependencies..."
# Prefer system package for yaml, but fallback to pip if needed
python3 -c "import yaml" 2>/dev/null || pip3 install PyYAML

echo "Installing Rust target..."
rustup target add thumbv7em-none-eabihf

echo "Installing optional embedded tools..."
if ! command -v probe-rs &> /dev/null; then
    echo "Installing probe-rs..."
    cargo install probe-rs --features=cli
fi

echo "Dependencies installation complete!"
