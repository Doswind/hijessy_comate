#!/usr/bin/env bash
# 用法：./scripts/bundle-macos.sh
# 先运行 cargo build --release，再执行本脚本。
set -euo pipefail

BINARY=target/release/hijessy
APP=Hijessy.app

if [[ ! -f "$BINARY" ]]; then
  echo "错误：$BINARY 不存在，请先运行 cargo build --release"
  exit 1
fi

rm -rf "$APP"
mkdir -p "$APP/Contents/MacOS"
mkdir -p "$APP/Contents/Resources"

cp "$BINARY" "$APP/Contents/MacOS/hijessy"

cat > "$APP/Contents/Info.plist" << 'PLIST'
<?xml version="1.0" encoding="UTF-8"?>
<!DOCTYPE plist PUBLIC "-//Apple//DTD PLIST 1.0//EN"
  "http://www.apple.com/DTDs/PropertyList-1.0.dtd">
<plist version="1.0">
<dict>
    <key>CFBundleExecutable</key>
    <string>hijessy</string>
    <key>CFBundleIdentifier</key>
    <string>com.doswind.hijessy</string>
    <key>CFBundleName</key>
    <string>Hijessy</string>
    <key>CFBundleDisplayName</key>
    <string>Hijessy</string>
    <key>CFBundleVersion</key>
    <string>0.1.0</string>
    <key>CFBundleShortVersionString</key>
    <string>0.1.0</string>
    <key>CFBundlePackageType</key>
    <string>APPL</string>
    <key>NSHighResolutionCapable</key>
    <true/>
    <key>LSUIElement</key>
    <true/>
    <key>NSScreenCaptureUsageDescription</key>
    <string>Hijessy 需要屏幕录制权限才能截图。</string>
</dict>
</plist>
PLIST

echo "打包完成：$APP"
echo "将 $APP 拖入 /Applications 即可使用"
