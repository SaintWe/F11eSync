#!/usr/bin/env bash
set -euo pipefail

ROOT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)"
CRATE_DIR="$ROOT_DIR"
REPO_ROOT="$(cd "$ROOT_DIR/.." && pwd)"
DIST_DIR="$REPO_ROOT/dist"
TARGET_DIR="$DIST_DIR/rust-target"
LICENSES_FILE="$DIST_DIR/THIRD_PARTY_LICENSES.txt"

need_cmd() {
  if ! command -v "$1" >/dev/null 2>&1; then
    echo "$1 not found" >&2
    exit 1
  fi
}

need_cmd cargo
need_cmd rustup
need_cmd ditto
need_cmd python3

if ! command -v cargo-bundle >/dev/null 2>&1; then
  echo "cargo-bundle not installed; installing..." >&2
  cargo install cargo-bundle
fi

ASSETS_DIR="$CRATE_DIR/assets"
ICON_ICNS="$ASSETS_DIR/icon.icns"
ICON_PNG="$ASSETS_DIR/icon.png"
TRAY_1X="$ASSETS_DIR/trayTemplate.png"
TRAY_2X="$ASSETS_DIR/trayTemplate@2x.png"

missing_assets=0
for f in "$ICON_ICNS" "$ICON_PNG" "$TRAY_1X" "$TRAY_2X"; do
  if [[ ! -f "$f" ]]; then
    missing_assets=1
  fi
done
if [[ "$missing_assets" == "1" ]]; then
  echo "Icon assets missing in $ASSETS_DIR (expected icon.icns/icon.png/trayTemplate*.png)" >&2
  exit 3
fi

mkdir -p "$DIST_DIR"
python3 "$REPO_ROOT/scripts/generate-third-party-licenses.py" >/dev/null
if [[ ! -f "$LICENSES_FILE" ]]; then
  echo "THIRD_PARTY_LICENSES.txt not found at: $LICENSES_FILE" >&2
  exit 4
fi

echo "Ensuring Rust targets..."
rustup target add aarch64-apple-darwin x86_64-apple-darwin >/dev/null

echo "Building aarch64-apple-darwin..."
(
  cd "$ROOT_DIR"
  CARGO_TARGET_DIR="$TARGET_DIR" cargo build --release --target aarch64-apple-darwin
)

echo "Building x86_64-apple-darwin..."
(
  cd "$ROOT_DIR"
  CARGO_TARGET_DIR="$TARGET_DIR" cargo build --release --target x86_64-apple-darwin
)

AARCH_BIN="$TARGET_DIR/aarch64-apple-darwin/release/f11esync"
X64_BIN="$TARGET_DIR/x86_64-apple-darwin/release/f11esync"

echo "Bundling .app (base)..."
(
  cd "$CRATE_DIR"
  CARGO_TARGET_DIR="$TARGET_DIR" cargo bundle --release
)

BASE_APP="$TARGET_DIR/release/bundle/osx/F11eSync.app"
if [[ ! -d "$BASE_APP" ]]; then
  echo "Bundle output not found at: $BASE_APP" >&2
  exit 2
fi

OUT_ROOT="$TARGET_DIR/macos-arches"
mkdir -p "$OUT_ROOT"

zip_with_ditto() {
  local stage_dir="$1"
  local zip_path="$2"
  (
    rm -f "$zip_path"
    DITTO_ARGS=( -c -k --sequesterRsrc )
    if ditto --help 2>&1 | grep -q "zlibCompressionLevel"; then
      DITTO_ARGS+=( --zlibCompressionLevel "${F11ESYNC_ZIP_LEVEL:-9}" )
    fi
    ditto "${DITTO_ARGS[@]}" "$stage_dir" "$zip_path"
  )
}

make_app() {
  local arch="$1"
  local bin_path="$2"
  local zip_name="$3"

  local out_dir="$OUT_ROOT/$arch"
  local out_app="$out_dir/F11eSync.app"
  local out_zip="$DIST_DIR/$zip_name"
  local stage_dir="$out_dir/.zip-stage"

  rm -rf "$out_dir"
  mkdir -p "$out_dir"
  cp -R "$BASE_APP" "$out_app"

  cp "$bin_path" "$out_app/Contents/MacOS/f11esync"
  chmod +x "$out_app/Contents/MacOS/f11esync"

  if [[ "${F11ESYNC_STRIP_BIN:-1}" == "1" ]] && command -v strip >/dev/null 2>&1; then
    strip -S -x "$out_app/Contents/MacOS/f11esync" || true
  fi

  if command -v lipo >/dev/null 2>&1; then
    lipo -info "$out_app/Contents/MacOS/f11esync" || true
  fi

  echo "Zipping: $out_zip"
  rm -rf "$stage_dir"
  mkdir -p "$stage_dir"
  cp -R "$out_app" "$stage_dir/F11eSync.app"
  cp "$LICENSES_FILE" "$stage_dir/THIRD_PARTY_LICENSES.txt"
  zip_with_ditto "$stage_dir" "$out_zip"
  rm -rf "$stage_dir"

  echo "OK: $out_app"
  echo "OK: $out_zip"
}

make_app "arm64" "$AARCH_BIN" "f11esync-gui-darwin-arm64.zip"
make_app "x64" "$X64_BIN" "f11esync-gui-darwin-x64.zip"
