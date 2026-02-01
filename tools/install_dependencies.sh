#!/usr/bin/env bash

# This script installs the dependencies for the SmolOS project.
# It targets STM32F769I development on Arch Linux.

set -e

echo "Installing system dependencies..."
if command -v pacman &> /dev/null; then
    sudo pacman -Syu --needed \
        cmake \
        ninja \
        arm-none-eabi-gcc \
        arm-none-eabi-newlib \
        python \
        python-pip \
        python-yaml \
        pkgconf \
        libusb \
        tar
else
    echo "Error: pacman not found. This script is intended for Arch Linux."
    exit 1
fi

echo "Installing Python dependencies..."
# Prefer system package for yaml (python-yaml), which was installed via pacman above.
if ! python -c "import yaml" 2>/dev/null; then
    echo "Python yaml module not found, attempting to install via pip..."
    pip install PyYAML || echo "Warning: Could not install PyYAML via pip. You might need to use a virtual environment or --break-system-packages (not recommended)."
fi

echo "Installing Rust target..."
rustup target add thumbv7em-none-eabihf

echo "Installing optional embedded tools..."
if ! command -v probe-rs &> /dev/null; then
    echo "Installing probe-rs..."
    # On Arch Linux, probe-rs can also be installed from AUR if preferred.
    # Here we stick to cargo install for consistency.
    cargo install probe-rs --features=cli
fi

echo "Dependencies installation complete!"
