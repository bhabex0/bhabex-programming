#!/bin/bash
# build-deb.sh
# Build Debian package for Bhabex

set -e

VERSION="0.1.0"
PACKAGE="bhabex"
ARCH="amd64"

echo "Building Bhabex $VERSION Debian package..."

# Build the project first
cargo build --release

# Create package structure
mkdir -p "deb/DEBIAN"
mkdir -p "deb/usr/bin"
mkdir -p "deb/usr/share/bhabex/stdlib"
mkdir -p "deb/usr/share/doc/bhabex"
mkdir -p "deb/usr/share/man/man1"
mkdir -p "deb/usr/share/mime/packages"
mkdir -p "deb/usr/share/icons/hicolor/256x256/apps"

# Copy files
cp target/release/bx "deb/usr/bin/"
cp target/release/bxpkg "deb/usr/bin/"
cp -r stdlib/* "deb/usr/share/bhabex/stdlib/"
cp LICENSE.txt "deb/usr/share/doc/bhabex/copyright"
cp README.md "deb/usr/share/doc/bhabex/README.md"

# Create control file
cat > "deb/DEBIAN/control" <<EOF
Package: $PACKAGE
Version: $VERSION
Section: devel
Priority: optional
Architecture: $ARCH
Maintainer: bxLabs <contact@bhabex.dev>
Description: Bhabex Compiler - Fast, safe, powerful programming language
 Bhabex is a modern, high-performance programming language
 with a bytecode VM and native compilation support.
Homepage: https://bhabex.dev
EOF

# Create postinst for PATH setup
cat > "deb/DEBIAN/postinst" <<'EOF'
#!/bin/bash
set -e
echo "export PATH=\$PATH:/usr/bin" > /etc/profile.d/bhabex.sh
chmod +x /etc/profile.d/bhabex.sh

# Register .bx MIME type
if [ -f /usr/share/mime/packages/bhabex.xml ]; then
    update-mime-database /usr/share/mime
fi

# Update desktop database
if command -v update-desktop-database &> /dev/null; then
    update-desktop-database /usr/share/applications
fi

# Update icon cache
if command -v gtk-update-icon-cache &> /dev/null; then
    gtk-update-icon-cache -f /usr/share/icons/hicolor
fi
EOF
chmod +x "deb/DEBIAN/postinst"

# Create prerm
cat > "deb/DEBIAN/prerm" <<'EOF'
#!/bin/bash
set -e
rm -f /etc/profile.d/bhabex.sh
EOF
chmod +x "deb/DEBIAN/prerm"

# Create postrm
cat > "deb/DEBIAN/postrm" <<'EOF'
#!/bin/bash
set -e
# Update MIME database
update-mime-database /usr/share/mime

# Update desktop database
if command -v update-desktop-database &> /dev/null; then
    update-desktop-database /usr/share/applications
fi

# Update icon cache
if command -v gtk-update-icon-cache &> /dev/null; then
    gtk-update-icon-cache -f /usr/share/icons/hicolor
fi
EOF
chmod +x "deb/DEBIAN/postrm"

# Create MIME type definition
cat > "deb/usr/share/mime/packages/bhabex.xml" <<EOF
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

# Create desktop entry
cat > "deb/usr/share/applications/bhabex.desktop" <<EOF
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

# Copy icon if exists
if [ -f assets/bhabex.png ]; then
    cp assets/bhabex.png "deb/usr/share/icons/hicolor/256x256/apps/bhabex.png"
fi

# Build package
dpkg-deb --build deb "bhabex_${VERSION}_${ARCH}.deb"

# Cleanup
rm -rf deb

echo "Debian package created: bhabex_${VERSION}_${ARCH}.deb"