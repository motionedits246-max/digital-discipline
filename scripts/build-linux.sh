#!/usr/bin/env bash
set -euo pipefail
cd "$(dirname "${BASH_SOURCE[0]}")/.."
root="$PWD"
export CARGO_TARGET_DIR="$root/target"
export CARGO_PROFILE_RELEASE_STRIP=symbols

cargo test --locked -p digital-discipline-agent
cargo build --locked --release -p digital-discipline-agent
node --test tests/*.test.mjs
npm ci --prefix apps/desktop
(cd apps/desktop && npm run tauri:build -- --bundles deb)

version=$(node -p 'require("./apps/desktop/src-tauri/tauri.conf.json").version')
architecture=$(dpkg --print-architecture)
mapfile -t packages < <(find target/release/bundle/deb -maxdepth 1 -name "*_${version}_${architecture}.deb" -print)
if [[ "${#packages[@]}" != 1 ]]; then
  echo "Expected one Debian package for ${version}/${architecture}." >&2
  exit 1
fi
mkdir -p release-output
package="Digital-Discipline-${version}-linux-${architecture}.deb"
cp "${packages[0]}" "release-output/$package"
dpkg-deb --fsys-tarfile "release-output/$package" | tar -tvf - > target/linux-package-files.txt
awk '$0 ~ /\/bin\/digital-discipline-agent$/ && substr($1,4,1) == "x" { found=1 } END { exit !found }' target/linux-package-files.txt
(cd release-output && sha256sum "$package" > SHA256SUMS-linux.txt)
