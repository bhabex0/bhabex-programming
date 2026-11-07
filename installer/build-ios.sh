#!/bin/bash
# build-ios.sh
# Build iOS framework for Bhabex

# Note: iOS apps must be built on macOS with Xcode
# This creates a framework that can be embedded in iOS apps

set -e

VERSION="0.1.0"

echo "Building Bhabex $VERSION iOS framework..."

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
mkdir -p Bhabex.xcframework/ios-arm64/Bhabex.framework/Headers
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

# Create test app project structure
mkdir -p BhabexTestApp/BhabexTestApp
cat > BhabexTestApp/BhabexTestApp/ViewController.swift <<EOF
import UIKit
import Bhabex

class ViewController: UIViewController {
    override func viewDidLoad() {
        super.viewDidLoad()
        
        // Run Bhabex code
        let code = """
        fn main() {
            print("Hello from Bhabex on iOS!")
        }
        """
        
        let result = bhabex_run(code)
        print("Bhabex result: \\(result)")
    }
}
EOF

cat > BhabexTestApp/BhabexTestApp/AppDelegate.swift <<EOF
import UIKit

@UIApplicationMain
class AppDelegate: UIResponder, UIApplicationDelegate {
    func application(_ application: UIApplication, didFinishLaunchingWithOptions launchOptions: [UIApplication.LaunchOptionsKey: Any]?) -> Bool {
        return true
    }
}
EOF

cat > BhabexTestApp/BhabexTestApp/Info.plist <<EOF
<?xml version="1.0" encoding="UTF-8"?>
<!DOCTYPE plist PUBLIC "-//Apple//DTD PLIST 1.0//EN" "http://www.apple.com/DTDs/PropertyList-1.0.dtd">
<plist version="1.0">
<dict>
    <key>CFBundleIdentifier</key>
    <string>dev.bxLabs.BhabexTest</string>
    <key>CFBundleName</key>
    <string>Bhabex Test</string>
    <key>CFBundleVersion</key>
    <string>$VERSION</string>
    <key>CFBundleShortVersionString</key>
    <string>$VERSION</string>
    <key>UILaunchStoryboardName</key>
    <string>LaunchScreen</string>
    <key>UISupportedInterfaceOrientations</key>
    <array>
        <string>UIInterfaceOrientationPortrait</string>
    </array>
</dict>
</plist>
EOF

echo "iOS framework created at Bhabex.xcframework"
echo "Test app structure created at BhabexTestApp/"
echo "To build and run on iOS:"
echo "  1. Open BhabexTestApp.xcodeproj in Xcode"
echo "  2. Add Bhabex.xcframework to Frameworks"
echo "  3. Build and run on device/simulator"