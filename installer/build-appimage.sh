#!/bin/bash
# build-appimage.sh
# Build Linux AppImage for Bhabex

set -e

VERSION="0.1.0"
APPDIR="Bhabex.AppDir"

echo "Building Bhabex $VERSION AppImage..."

# Build the project first
cargo build --release

# Create AppDir structure
mkdir -p "$APPDIR/usr/bin"
mkdir -p "$APPDIR/usr/share/bhabex/stdlib"
mkdir -p "$APPDIR/usr/share/applications"
mkdir -p "$APPDIR/usr/share/icons/hicolor/256x256/apps"
mkdir -p "$APPDIR/usr/share/mime/packages"

# Copy files
cp target/release/bx "$APPDIR/usr/bin/"
cp target/release/bxpkg "$APPDIR/usr/bin/"
cp -r stdlib/* "$APPDIR/usr/share/bhabex/stdlib/"

# Create desktop entry
cat > "$APPDIR/usr/share/applications/bhabex.desktop" <<EOF
[Desktop Entry]
Type=Application
Name=Bhabex
Comment=Bhabex Compiler
Exec=bx
Icon=bhabex
Terminal=false
Categories=Development;IDE;
MimeType=text/x-bhabex;
EOF

# Copy icon (create a simple one if not exists)
if [ ! -f assets/bhabex.png ]; then
    # Create a simple PNG using ImageMagick if available, or skip
    echo "Warning: assets/bhabex.png not found, skipping icon"
else
    cp assets/bhabex.png "$APPDIR/usr/share/icons/hicolor/256x256/apps/bhabex.png"
    cp assets/bhabex.png "$APPDIR/bhabex.png"
fi

# Create MIME type definition
cat > "$APPDIR/usr/share/mime/packages/bhabex.xml" <<EOF
<?xml version="1.0" encoding="UTF-8"?>
<mime-info xmlns="http://www.freedesktop.org/standards/shared-mime-info">
  <mime-type type="text/x-bhabex">
    <comment>Bhabex Source File</comment>
    <glob pattern="*.bx"/>
    <magic priority="50">
      <match type="string" value="fn main()" offset="0:256"/>
    </magic>
  </mime-type>
</mime-info>
EOF

# Create AppRun
cat > "$APPDIR/AppRun" <<'EOF'
#!/bin/bash
HERE="$(dirname "$(readlink -f "${BASH_SOURCE[0]}")")"
export PATH="$HERE/usr/bin:$PATH"
export BHABEX_STDLIB="$HERE/usr/share/bhabex/stdlib"
export XDG_DATA_DIRS="$HERE/usr/share:$XDG_DATA_DIRS"
exec "$HERE/usr/bin/bx" "$@"
EOF
chmod +x "$APPDIR/AppRun"

# Download appimagetool if needed
if ! command -v appimagetool &> /dev/null; then
    echo "Downloading appimagetool..."
    wget -q "https://github.com/AppImage/AppImageKit/releases/download/continuous/appimagetool-x86_64.AppImage" -O appimagetool
    chmod +x appimagetool
    APPIMAGETOOL="./appimagetool"
else
    APPIMAGETOOL="appimagetool"
fi

# Build AppImage
ARCH=x86_64 $APPIMAGETOOL "$APPDIR" "Bhabex-$VERSION-x86_64.AppImage"

# Cleanup
rm -rf "$APPDIR"
[ -f "./appimagetool" ] && rm -f ./appimagetool

echo "AppImage created: Bhabex-$VERSION-x86_64.AppImage"