#!/usr/bin/env bash
set -euo pipefail

repo_root="$(cd "$(dirname "$0")/.." && pwd)"
cd "$repo_root"

version="${1:-}"
if [[ -z "$version" ]]; then
  version="v$(sed -n 's/^version = "\(.*\)"$/\1/p' Cargo.toml | head -n 1)"
fi

if [[ -z "$version" || "$version" == "v" ]]; then
  echo "Unable to determine version from Cargo.toml" >&2
  exit 1
fi

package_root="dist/AI-Movie-Player-${version}-macOS-$(uname -m)"
app_root="$package_root/AI-Movie-Player.app"
archive_path="dist/AI-Movie-Player-${version}-macOS-$(uname -m).tar.gz"

rm -rf "$package_root" "$archive_path"
mkdir -p "$app_root/Contents/MacOS" "$app_root/Contents/Resources"

cargo build --release

cp target/release/ai-movie-player "$app_root/Contents/MacOS/AI-Movie-Player"
chmod +x "$app_root/Contents/MacOS/AI-Movie-Player"

cat > "$app_root/Contents/Info.plist" <<EOF
<?xml version="1.0" encoding="UTF-8"?>
<!DOCTYPE plist PUBLIC "-//Apple//DTD PLIST 1.0//EN" "http://www.apple.com/DTDs/PropertyList-1.0.dtd">
<plist version="1.0">
  <dict>
    <key>CFBundleDisplayName</key>
    <string>AI-Movie-Player</string>
    <key>CFBundleExecutable</key>
    <string>AI-Movie-Player</string>
    <key>CFBundleIdentifier</key>
    <string>ai.ifq.AIMoviePlayer</string>
    <key>CFBundleName</key>
    <string>AI-Movie-Player</string>
    <key>CFBundlePackageType</key>
    <string>APPL</string>
    <key>CFBundleShortVersionString</key>
    <string>${version}</string>
    <key>CFBundleVersion</key>
    <string>${version}</string>
    <key>LSMinimumSystemVersion</key>
    <string>12.0</string>
    <key>NSHighResolutionCapable</key>
    <true/>
  </dict>
</plist>
EOF

cp README.md readme-cn.md LICENSE "$package_root/"

tar -czf "$archive_path" -C dist "$(basename "$package_root")"

echo "macOS package created: $archive_path"