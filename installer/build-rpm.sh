#!/bin/bash
# build-rpm.sh
# Build RPM package for Bhabex

set -e

VERSION="0.1.0"
RELEASE="1"

echo "Building Bhabex $VERSION RPM package..."

# Build the project first
cargo build --release

# Create RPM build structure
mkdir -p ~/rpmbuild/{BUILD,RPMS,SOURCES,SPECS,SRPMS}

# Create source tarball
tar czf ~/rpmbuild/SOURCES/bhabex-$VERSION.tar.gz .

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
mkdir -p \$RPM_BUILD_ROOT/usr/share/applications
mkdir -p \$RPM_BUILD_ROOT/usr/share/mime/packages
mkdir -p \$RPM_BUILD_ROOT/usr/share/icons/hicolor/256x256/apps

cp target/release/bx \$RPM_BUILD_ROOT/usr/bin/
cp target/release/bxpkg \$RPM_BUILD_ROOT/usr/bin/
cp -r stdlib/* \$RPM_BUILD_ROOT/usr/share/bhabex/stdlib/
cp LICENSE.txt \$RPM_BUILD_ROOT/usr/share/doc/bhabex/

# Desktop entry
cat > \$RPM_BUILD_ROOT/usr/share/applications/bhabex.desktop <<DESKTOPEOF
[Desktop Entry]
Type=Application
Name=Bhabex
Comment=Bhabex Compiler
Exec=bx
Icon=bhabex
Terminal=false
Categories=Development;IDE;
MimeType=text/x-bhabex;
DESKTOPEOF

# MIME type
cat > \$RPM_BUILD_ROOT/usr/share/mime/packages/bhabex.xml <<MIMEEOF
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
MIMEEOF

# Icon (if available)
[ -f assets/bhabex.png ] && cp assets/bhabex.png \$RPM_BUILD_ROOT/usr/share/icons/hicolor/256x256/apps/bhabex.png

%post
# Update MIME database
update-mime-database /usr/share/mime &> /dev/null || true

# Update desktop database
update-desktop-database /usr/share/applications &> /dev/null || true

# Update icon cache
gtk-update-icon-cache -f /usr/share/icons/hicolor &> /dev/null || true

# Add to PATH
echo 'export PATH=\$PATH:/usr/bin' > /etc/profile.d/bhabex.sh
chmod +x /etc/profile.d/bhabex.sh

%preun
# Remove from PATH
rm -f /etc/profile.d/bhabex.sh

%postun
# Update MIME database
update-mime-database /usr/share/mime &> /dev/null || true

# Update desktop database
update-desktop-database /usr/share/applications &> /dev/null || true

# Update icon cache
gtk-update-icon-cache -f /usr/share/icons/hicolor &> /dev/null || true

%files
/usr/bin/bx
/usr/bin/bxpkg
/usr/share/bhabex/stdlib/*
/usr/share/doc/bhabex/LICENSE.txt
/usr/share/applications/bhabex.desktop
/usr/share/mime/packages/bhabex.xml
%{_datadir}/icons/hicolor/256x256/apps/bhabex.png

%changelog
* $(date "+%a %b %d %Y") bxLabs <contact@bhabex.dev> - $VERSION-$RELEASE
- Initial release
EOF

# Build RPM
rpmbuild -ba ~/rpmbuild/SPECS/bhabex.spec

# Copy result
cp ~/rpmbuild/RPMS/x86_64/bhabex-$VERSION-$RELEASE.x86_64.rpm .

echo "RPM package created: bhabex-$VERSION-$RELEASE.x86_64.rpm"