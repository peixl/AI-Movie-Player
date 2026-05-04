#!/usr/bin/env bash
set -euo pipefail

repo_root="$(cd "$(dirname "$0")/.." && pwd)"
cd "$repo_root"

version=""
skip_build="false"

while [[ $# -gt 0 ]]; do
  case "$1" in
    --skip-build)
      skip_build="true"
      shift
      ;;
    *)
      if [[ -z "$version" ]]; then
        version="$1"
        shift
      else
        echo "Unexpected argument: $1" >&2
        exit 1
      fi
      ;;
  esac
done

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
checksum_path="${archive_path}.sha256"

rm -rf "$package_root" "$archive_path" "$checksum_path"
mkdir -p "$app_root/Contents/MacOS" "$app_root/Contents/Resources"

if [[ "$skip_build" != "true" ]]; then
  cargo build --release --locked
fi

cp target/release/ai-movie-player "$app_root/Contents/MacOS/AI-Movie-Player"
chmod +x "$app_root/Contents/MacOS/AI-Movie-Player"

cat > "$app_root/Contents/Info.plist" <<EOF
<?xml version="1.0" encoding="UTF-8"?>
<!DOCTYPE plist PUBLIC "-//Apple//DTD PLIST 1.0//EN" "http://www.apple.com/DTDs/PropertyList-1.0.dtd">
<plist version="1.0">
  <dict>
    <key>CFBundleDisplayName</key>
    <string>AI Movie Player</string>
    <key>CFBundleExecutable</key>
    <string>AI-Movie-Player</string>
    <key>CFBundleIdentifier</key>
    <string>ai.ifq.AIMoviePlayer</string>
    <key>CFBundleName</key>
    <string>AI Movie Player</string>
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
shasum -a 256 "$archive_path" | awk '{print $1"  "FILENAME}' FILENAME="$(basename "$archive_path")" > "$checksum_path"

echo "macOS package created: $archive_path"
echo "macOS checksum created: $checksum_path"