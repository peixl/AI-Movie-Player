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

package_root="dist/AI-Movie-Player-${version}-linux-x86_64"
archive_path="dist/AI-Movie-Player-${version}-linux-x86_64.tar.gz"
checksum_path="${archive_path}.sha256"

rm -rf "$package_root" "$archive_path" "$checksum_path"
mkdir -p "$package_root"

if [[ "$skip_build" != "true" ]]; then
  cargo build --release --locked
fi

cp target/release/ai-movie-player "$package_root/AI-Movie-Player"
chmod +x "$package_root/AI-Movie-Player"
cp README.md readme-cn.md LICENSE "$package_root/"

tar -czf "$archive_path" -C dist "$(basename "$package_root")"
sha256sum "$archive_path" | awk '{print $1"  "FILENAME}' FILENAME="$(basename "$archive_path")" > "$checksum_path"

echo "Linux package created: $archive_path"
echo "Linux checksum created: $checksum_path"