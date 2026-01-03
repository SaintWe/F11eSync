#!/usr/bin/env bash
set -euo pipefail

APP_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)"
REPO_ROOT="$(cd "$APP_DIR/.." && pwd)"

DIST_DIR="$REPO_ROOT/dist"
TARGET_DIR="$DIST_DIR/rust-target"

need_cmd() {
  if ! command -v "$1" >/dev/null 2>&1; then
    echo "$1 not found" >&2
    exit 1
  fi
}

need_cmd cargo
need_cmd zip

mkdir -p "$DIST_DIR"

build_x64() {
  echo "Building Rust CLI (linux-x64)..."
  (
    cd "$APP_DIR"
    CARGO_TARGET_DIR="$TARGET_DIR" cargo build --release --no-default-features
  )

  cp "$TARGET_DIR/release/f11esync" "$DIST_DIR/f11esync-rust-linux-x64"
  (
    cd "$DIST_DIR"
    rm -f "f11esync-rust-linux-x64.zip"
    zip -9 "f11esync-rust-linux-x64.zip" "f11esync-rust-linux-x64"
    rm -f "f11esync-rust-linux-x64"
  )
  echo "OK: $DIST_DIR/f11esync-rust-linux-x64.zip"
}

build_arm64_native() {
  echo "Building Rust CLI (linux-arm64 native)..."
  (
    cd "$APP_DIR"
    CARGO_TARGET_DIR="$TARGET_DIR" cargo build --release --no-default-features
  )

  cp "$TARGET_DIR/release/f11esync" "$DIST_DIR/f11esync-rust-linux-arm64"
  (
    cd "$DIST_DIR"
    rm -f "f11esync-rust-linux-arm64.zip"
    zip -9 "f11esync-rust-linux-arm64.zip" "f11esync-rust-linux-arm64"
    rm -f "f11esync-rust-linux-arm64"
  )
  echo "OK: $DIST_DIR/f11esync-rust-linux-arm64.zip"
}

build_arm64_cross() {
  need_cmd cross
  echo "Building Rust CLI (linux-arm64 via cross)..."
  (
    cd "$APP_DIR"
    CARGO_TARGET_DIR="$TARGET_DIR" cross build --release --no-default-features --target aarch64-unknown-linux-gnu
  )

  cp "$TARGET_DIR/aarch64-unknown-linux-gnu/release/f11esync" "$DIST_DIR/f11esync-rust-linux-arm64"
  (
    cd "$DIST_DIR"
    rm -f "f11esync-rust-linux-arm64.zip"
    zip -9 "f11esync-rust-linux-arm64.zip" "f11esync-rust-linux-arm64"
    rm -f "f11esync-rust-linux-arm64"
  )
  echo "OK: $DIST_DIR/f11esync-rust-linux-arm64.zip"
}

arch="$(uname -m || true)"
case "$arch" in
  x86_64|amd64)
    build_x64
    if command -v cross >/dev/null 2>&1; then
      build_arm64_cross
    else
      echo "cross not found; skip linux-arm64 (install cross to enable)"
    fi
    ;;
  aarch64|arm64)
    build_arm64_native
    echo "Skip linux-x64 on arm64 host (use a x86_64 Linux host or CI for linux-x64)"
    ;;
  *)
    echo "Unsupported host arch: $arch" >&2
    exit 2
    ;;
esac

