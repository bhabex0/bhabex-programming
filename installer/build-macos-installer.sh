#!/bin/bash
# build-macos-installer.sh
# Build macOS installer package for Bhabex

set -e

VERSION="0.1.0"
IDENTIFIER="dev.bxLabs.bhabex"
INSTALL_LOCATION="/usr/local/bhabex"

echo "Building Bhabex $VERSION for macOS..."

# Build the project first
cargo build --release

# Create package structure
mkdir -p pkgroot/usr/local/bhabex/bin
mkdir -p pkgroot/usr/local/bhabex/stdlib
mkdir -p pkgroot/usr/local/bhabex/lib

# Copy binaries
cp target/release/bx pkgroot/usr/local/bhabex/bin/
cp target/release/bxpkg pkgroot/usr/local/bhabex/bin/

# Copy stdlib
cp -r stdlib/* pkgroot/usr/local/bhabex/stdlib/

# Create scripts directory
mkdir -p scripts

# Create postinstall script
cat > scripts/postinstall <<'EOF'
#!/bin/bash
# Add to PATH
echo 'export PATH=$PATH:/usr/local/bhabex/bin' > /etc/paths.d/bhabex

# Register .bx file association
echo '{"uti":"dev.bxLabs.bhabex","ext":"bx","role":"editor","name":"Bhabex Source File"}' > /tmp/bhabex.uti
/System/Library/Frameworks/CoreServices.framework/Frameworks/LaunchServices.framework/Support/lsregister -u /tmp/bhabex.uti
rm /tmp/bhabex.uti

# Install VS Code extension
/usr/local/bhabex/bin/bx vscode install
EOF
chmod +x scripts/postinstall

# Create preuninstall script
cat > scripts/preuninstall <<'EOF'
#!/bin/bash
# Remove from PATH
rm -f /etc/paths.d/bhabex

# Uninstall VS Code extension
/usr/local/bhabex/bin/bx vscode uninstall
EOF
chmod +x scripts/preuninstall

# Create component package
pkgbuild \
    --root pkgroot \
    --identifier "$IDENTIFIER" \
    --version "$VERSION" \
    --install-location "$INSTALL_LOCATION" \
    --scripts scripts \
    bhabex.pkg

# Create distribution package
cat > distribution.xml <<EOF
<?xml version="1.0" encoding="utf-8"?>
<installer-gui-script minSpecVersion="2">
    <title>Bhabex Installer</title>
    <organization>bxLabs</organization>
    <domains enable_anywhere="true" enable_currentUserHome="true" enable_localSystem="true"/>
    <options customize="never" require-scripts="false" rootVolumeOnly="false"/>
    <welcome file="welcome.html" mime-type="text/html"/>
    <license file="LICENSE.txt" mime-type="text/plain"/>
    <conclusion file="conclusion.html" mime-type="text/html"/>
    <pkg-ref id="dev.bxLabs.bhabex">bhabex.pkg</pkg-ref>
    <choices-outline>
        <line choice="default">
            <line choice="bhabex"/>
        </line>
    </choices-outline>
    <choice id="default"/>
    <choice id="bhabex" visible="true" title="Bhabex" description="Bhabex compiler and tools">
        <pkg-ref id="dev.bxLabs.bhabex"/>
    </choice>
</installer-gui-script>
EOF

# Create welcome and conclusion HTML
cat > welcome.html <<'EOF'
<!DOCTYPE html>
<html>
<head><title>Welcome to Bhabex</title></head>
<body style="font-family: -apple-system; padding: 20px;">
<h1>Welcome to Bhabex Installer</h1>
<p>This installer will install Bhabex v0.1.0 on your system.</p>
<p>Bhabex is a modern, high-performance programming language with a bytecode VM and native compilation support.</p>
</body>
</html>
EOF

cat > conclusion.html <<'EOF'
<!DOCTYPE html>
<html>
<head><title>Installation Complete</title></head>
<body style="font-family: -apple-system; padding: 20px;">
<h1>Installation Complete</h1>
<p>Bhabex has been successfully installed!</p>
<p>You can now run <code>bx</code> from Terminal.</p>
<p>Try: <code>bx run examples/hello.bx</code></p>
</body>
</html>
EOF

productbuild \
    --distribution distribution.xml \
    --package-path . \
    --resources . \
    "Bhabex-$VERSION-macos.pkg"

# Cleanup
rm -rf pkgroot bhabex.pkg scripts distribution.xml welcome.html conclusion.html

echo "macOS installer created: Bhabex-$VERSION-macos.pkg"