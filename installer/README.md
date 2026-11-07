# Bhabex Installer

Copyright BHABEX 2026 | Securely designed & developed by bxLabs | Protected by bxSecurity | Powered by BHABEX®

## Platform-Specific Installers

### Windows (NSIS Script - `installer.nsi`)

```nsis
; Bhabex Installer for Windows
; Copyright BHABEX 2026

!include "MUI2.nsh"

Name "Bhabex"
OutFile "bhabex-installer.exe"
InstallDir "$LOCALAPPDATA\Bhabex"
RequestExecutionLevel user

!define MUI_ICON "bhabex.ico"
!define MUI_UNICON "bhabex.ico"

!insertmacro MUI_PAGE_WELCOME
!insertmacro MUI_PAGE_LICENSE "LICENSE.txt"
!insertmacro MUI_PAGE_DIRECTORY
!insertmacro MUI_PAGE_INSTFILES
!insertmacro MUI_PAGE_FINISH

!insertmacro MUI_UNPAGE_WELCOME
!insertmacro MUI_UNPAGE_CONFIRM
!insertmacro MUI_UNPAGE_INSTFILES
!insertmacro MUI_UNPAGE_FINISH

Function .onInit
    ; Check if already installed
    ReadRegStr $0 HKCU "Software\Bhabex" "InstallPath"
    StrCmp $0 "" 0 already_installed
    Goto done
    
already_installed:
    MessageBox MB_YESNO "Bhabex is already installed at $0.\nReinstall?" IDYES done IDNO cancel
    
cancel:
    Abort
    
done:
FunctionEnd

Section "Main"
    SetOutPath $INSTDIR
    File /r "bin\*"
    File /r "stdlib\*"
    File "bx.exe"
    File "bxpkg.exe"
    File "LICENSE.txt"
    
    ; Add to PATH
    ReadEnvStr $0 PATH
    StrStr $1 $0 "$INSTDIR\bin"
    StrCmp $1 "" add_to_path skip_path
    
add_to_path:
    System::Call 'kernel32::SetEnvironmentVariable(t "PATH", t "$0;$INSTDIR\bin")'
    WriteRegStr HKCU "Environment" "PATH" "$0;$INSTDIR\bin"
    
skip_path:
    ; Register .bx file association
    WriteRegStr HKCR ".bx" "" "BhabexFile"
    WriteRegStr HKCR "BhabexFile" "" "Bhabex Source File"
    WriteRegStr HKCR "BhabexFile\DefaultIcon" "" "$INSTDIR\bin\bhabex.ico"
    WriteRegStr HKCR "BhabexFile\shell\open\command" "" '"$INSTDIR\bin\bx.exe" "%1"'
    
    ; Write uninstaller
    WriteUninstaller "$INSTDIR\uninstall.exe"
    WriteRegStr HKCU "Software\Bhabex" "InstallPath" "$INSTDIR"
    
    ; Install VS Code extension
    ExecWait '"$INSTDIR\bin\bx" vscode install'
SectionEnd

Section "Uninstall"
    ; Remove from PATH
    ReadEnvStr $0 PATH
    StrReplace $0 $0 "$INSTDIR\bin;" ""
    System::Call 'kernel32::SetEnvironmentVariable(t "PATH", t "$0")'
    WriteRegStr HKCU "Environment" "PATH" "$0"
    
    ; Remove file association
    DeleteRegKey HKCR ".bx"
    DeleteRegKey HKCR "BhabexFile"
    
    ; Remove registry
    DeleteRegKey HKCU "Software\Bhabex"
    
    ; Uninstall VS Code extension
    ExecWait '"$INSTDIR\bin\bx" vscode uninstall'
SectionEnd
```

### macOS (pkgbuild)

```bash
#!/bin/bash
# build-macos-installer.sh

set -e

VERSION="0.1.0"
IDENTIFIER="dev.bxLabs.bhabex"
INSTALL_LOCATION="/usr/local/bhabex"

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

# Create component package
pkgbuild \
    --root pkgroot \
    --identifier "$IDENTIFIER" \
    --version "$VERSION" \
    --install-location "$INSTALL_LOCATION" \
    --scripts scripts \
    bhabex.pkg

# Create distribution package
productbuild \
    --distribution distribution.xml \
    --package-path . \
    --resources resources \
    "Bhabex-$VERSION-macos.pkg"

# Cleanup
rm -rf pkgroot bhabex.pkg
```

Distribution XML:
```xml
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
```

### Linux (AppImage)

```bash
#!/bin/bash
# build-appimage.sh

set -e

VERSION="0.1.0"
APPDIR="Bhabex.AppDir"

# Build
cargo build --release

# Create AppDir structure
mkdir -p "$APPDIR/usr/bin"
mkdir -p "$APPDIR/usr/share/bhabex/stdlib"
mkdir -p "$APPDIR/usr/share/applications"
mkdir -p "$APPDIR/usr/share/icons/hicolor/256x256/apps"

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
EOF

# Copy icon
cp assets/bhabex.png "$APPDIR/usr/share/icons/hicolor/256x256/apps/bhabex.png"
cp assets/bhabex.png "$APPDIR/bhabex.png"

# Create AppRun
cat > "$APPDIR/AppRun" <<'EOF'
#!/bin/bash
HERE="$(dirname "$(readlink -f "${BASH_SOURCE[0]}")")"
export PATH="$HERE/usr/bin:$PATH"
export BHABEX_STDLIB="$HERE/usr/share/bhabex/stdlib"
exec "$HERE/usr/bin/bx" "$@"
EOF
chmod +x "$APPDIR/AppRun"

# Download appimagetool if needed
if ! command -v appimagetool &> /dev/null; then
    wget -q "https://github.com/AppImage/AppImageKit/releases/download/continuous/appimagetool-x86_64.AppImage" -O appimagetool
    chmod +x appimagetool
fi

# Build AppImage
ARCH=x86_64 ./appimagetool "$APPDIR" "Bhabex-$VERSION-x86_64.AppImage"

# Cleanup
rm -rf "$APPDIR"
```

### Linux (Debian Package)

```bash
#!/bin/bash
# build-deb.sh

set -e

VERSION="0.1.0"
PACKAGE="bhabex"
ARCH="amd64"

# Build
cargo build --release

# Create package structure
mkdir -p "deb/DEBIAN"
mkdir -p "deb/usr/bin"
mkdir -p "deb/usr/share/bhabex/stdlib"
mkdir -p "deb/usr/share/doc/bhabex"
mkdir -p "deb/usr/share/man/man1"

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
update-mime-database /usr/share/mime
EOF
chmod +x "deb/DEBIAN/postinst"

# Create prerm
cat > "deb/DEBIAN/prerm" <<'EOF'
#!/bin/bash
rm -f /etc/profile.d/bhabex.sh
EOF
chmod +x "deb/DEBIAN/prerm"

# Build package
dpkg-deb --build deb "bhabex_${VERSION}_${ARCH}.deb"

# Cleanup
rm -rf deb
```

### Linux (RPM Package)

```bash
#!/bin/bash
# build-rpm.sh

set -e

VERSION="0.1.0"
RELEASE="1"

# Build
cargo build --release

# Create RPM build structure
mkdir -p ~/rpmbuild/{BUILD,RPMS,SOURCES,SPECS,SRPMS}

# Create spec file
cat > ~/rpmbuild/SPECS/bhabex.spec <<EOF
Name:           bhabex
Version:        $VERSION
Release:        $RELEASE%{?dist}
Summary:        Bhabex Compiler - Fast, safe, powerful programming language
License:        MIT
URL:            https://bhabex.dev
Source0:        %{name}-%{version}.tar.gz

BuildRequires:  cargo, rust
Requires:       glibc

%description
Bhabex is a modern, high-performance programming language
with a bytecode VM and native compilation support.

%prep
%autosetup

%build
cargo build --release

%install
rm -rf \$RPM_BUILD_ROOT
mkdir -p \$RPM_BUILD_ROOT/usr/bin
mkdir -p \$RPM_BUILD_ROOT/usr/share/bhabex/stdlib
mkdir -p \$RPM_BUILD_ROOT/usr/share/doc/bhabex

cp target/release/bx \$RPM_BUILD_ROOT/usr/bin/
cp target/release/bxpkg \$RPM_BUILD_ROOT/usr/bin/
cp -r stdlib/* \$RPM_BUILD_ROOT/usr/share/bhabex/stdlib/
cp LICENSE.txt \$RPM_BUILD_ROOT/usr/share/doc/bhabex/

%files
/usr/bin/bx
/usr/bin/bxpkg
/usr/share/bhabex/stdlib/*
/usr/share/doc/bhabex/LICENSE.txt

%changelog
* $(date "+%a %b %d %Y") bxLabs <contact@bhabex.dev> - $VERSION-$RELEASE
- Initial release
EOF

# Create source tarball
tar czf ~/rpmbuild/SOURCES/bhabex-$VERSION.tar.gz .

# Build RPM
rpmbuild -ba ~/rpmbuild/SPECS/bhabex.spec

# Copy result
cp ~/rpmbuild/RPMS/x86_64/bhabex-$VERSION-$RELEASE.x86_64.rpm .
```

### Android (Termux Package)

```bash
#!/bin/bash
# build-termux.sh

set -e

VERSION="0.1.0"

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
Depends: libc++
EOF

# Build package
dpkg-deb --build termux-package "bhabex_${VERSION}_aarch64.deb"

# Cleanup
rm -rf termux-package
```

### iOS (via TestFlight)

```bash
#!/bin/bash
# build-ios.sh

# Note: iOS apps must be built on macOS with Xcode
# This creates a framework that can be embedded in iOS apps

set -e

VERSION="0.1.0"

# Build for iOS
rustup target add aarch64-apple-ios
cargo build --release --target aarch64-apple-ios

# Create XCFramework
mkdir -p Bhabex.xcframework/ios-arm64/Bhabex.framework
cp target/aarch64-apple-ios/release/libbhabex.a Bhabex.xcframework/ios-arm64/Bhabex.framework/Bhabex

# Create Info.plist
cat > Bhabex.xcframework/Info.plist <<EOF
<?xml version="1.0" encoding="UTF-8"?>
<!DOCTYPE plist PUBLIC "-//Apple//DTD PLIST 1.0//EN" "http://www.apple.com/DTDs/PropertyList-1.0.dtd">
<plist version="1.0">
<dict>
    <key>CFBundleIdentifier</key>
    <string>dev.bxLabs.Bhabex</string>
    <key>CFBundleName</key>
    <string>Bhabex</string>
    <key>CFBundleVersion</key>
    <string>$VERSION</string>
    <key>CFBundleShortVersionString</key>
    <string>$VERSION</string>
</dict>
</dict>
</plist>
EOF

# Create module map
mkdir -p Bhabex.xcframework/ios-arm64/Bhabex.framework/Modules
cat > Bhabex.xcframework/ios-arm64/Bhabex.framework/Modules/module.modulemap <<EOF
framework module Bhabex {
    umbrella header "Bhabex.h"
    export *
    module * { export * }
}
EOF

# Create header
cat > Bhabex.xcframework/ios-arm64/Bhabex.framework/Headers/Bhabex.h <<EOF
#ifndef BHABEX_H
#define BHABEX_H

#ifdef __cplusplus
extern "C" {
#endif

// Bhabex C API for embedding
int bhabex_run(const char* source);
int bhabex_compile(const char* source, char** output);

#ifdef __cplusplus
}
#endif

#endif
EOF

echo "XCFramework created at Bhabex.xcframework"
```

---

Copyright BHABEX 2026 | Securely designed & developed by bxLabs | Protected by bxSecurity | Powered by BHABEX®