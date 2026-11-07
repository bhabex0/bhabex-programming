#!/bin/bash
# build-all.sh
# Build all installers for Bhabex

set -e

VERSION="0.1.0"

echo "======================================"
echo "Building all Bhabex $VERSION installers"
echo "======================================"

# Build core project first
echo "Building core project..."
cargo build --release

# Function to run a build script
run_build() {
    local script=$1
    local name=$2
    
    if [ -f "$script" ]; then
        echo ""
        echo "======================================"
        echo "Building $name..."
        echo "======================================"
        chmod +x "$script"
        if "$script"; then
            echo "✓ $name build successful"
        else
            echo "✗ $name build failed"
            return 1
        fi
    else
        echo "Script $script not found, skipping $name"
    fi
}

# Build for current platform
case "$(uname -s)" in
    Linux*)
        echo "Linux detected - building Linux packages"
        run_build "./build-appimage.sh" "AppImage"
        run_build "./build-deb.sh" "Debian package"
        run_build "./build-rpm.sh" "RPM package"
        run_build "./build-termux.sh" "Termux package"
        ;;
    Darwin*)
        echo "macOS detected - building macOS installer"
        run_build "./build-macos-installer.sh" "macOS package"
        ;;
    CYGWIN*|MINGW*|MSYS*)
        echo "Windows detected - building Windows installer"
        # NSIS would need to be installed separately
        echo "To build Windows installer, run: makensis installer.nsi"
        ;;
    *)
        echo "Unknown platform: $(uname -s)"
        ;;
esac

echo ""
echo "======================================"
echo "All builds complete!"
echo "======================================"
ls -la *.AppImage *.deb *.rpm *.pkg *.AppImage 2>/dev/null || true