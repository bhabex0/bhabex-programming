#!/bin/bash
# build-termux.sh
# Build Termux package for Bhabex (Android)

set -e

VERSION="0.1.0"

echo "Building Bhabex $VERSION Termux package..."

# Build for Android (aarch64)
rustup target add aarch64-linux-android
cargo build --release --target aarch64-linux-android

# Create Termux package structure
mkdir -p termux-package/data/data/com.termux/files/usr/bin
mkdir -p termux-package/data/data/com.termux/files/usr/share/bhabex/stdlib

# Copy binaries
cp target/aarch64-linux-android/release/bx termux-package/data/data/com.termux/files/usr/bin/
cp target/aarch64-linux-android/release/bxpkg termux-package/data/data/com.termux/files/usr/bin/
cp -r stdlib/* termux-package/data/data/com.termux/files/usr/share/bhabex/stdlib/

# Create control file
cat > termux-package/control <<EOF
Package: bhabex
Version: $VERSION
Architecture: aarch64
Maintainer: bxLabs <contact@bhabex.dev>
Description: Bhabex Compiler for Termux
 Bhabex is a modern, high-performance programming language
 with a bytecode VM and native compilation support.
Depends: libc++
EOF

# Create postinst
cat > termux-package/postinst <<'EOF'
#!/bin/bash
# Add to PATH
echo 'export PATH=$PATH:$PREFIX/bin' >> $PREFIX/etc/profile.d/bhabex.sh
chmod +x $PREFIX/etc/profile.d/bhabex.sh
EOF
chmod +x termux-package/postinst

# Create prerm
cat > termux-package/prerm <<'EOF'
#!/bin/bash
rm -f $PREFIX/etc/profile.d/bhabex.sh
EOF
chmod +x termux-package/prerm

# Build package
dpkg-deb --build termux-package "bhabex_${VERSION}_aarch64.deb"

# Cleanup
rm -rf termux-package

echo "Termux package created: bhabex_${VERSION}_aarch64.deb"
echo "Install on Android with: dpkg -i bhabex_${VERSION}_aarch64.deb"