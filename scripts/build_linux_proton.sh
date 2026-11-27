#!/usr/bin/env bash
set -euo pipefail

# Build Kelvaroq for Linux (native Proton launcher) + Windows x64 (kelvaroq.exe)
# Requires: rustup target add x86_64-pc-windows-gnu (or cargo-xwin for x86_64-pc-windows-msvc)

ROOT="$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)"
OUT="${1:-$ROOT/dist/linux-proton}"
mkdir -p "$OUT"

echo "==> Building Linux native kelvaroq..."
cargo build --manifest-path "$ROOT/Cargo.toml" --release
cp "$ROOT/target/release/kelvaroq" "$OUT/kelvaroq"

if rustup target list --installed | grep -q "x86_64-pc-windows-gnu"; then
    echo "==> Cross-compiling Windows x64 (x86_64-pc-windows-gnu)..."
    cargo build --manifest-path "$ROOT/Cargo.toml" --release --target x86_64-pc-windows-gnu
    cp "$ROOT/target/x86_64-pc-windows-gnu/release/kelvaroq.exe" "$OUT/kelvaroq.exe"
    cp "$ROOT/target/x86_64-pc-windows-gnu/release/kelvaroq_simd.dll" "$OUT/kelvaroq_simd.dll"
elif command -v cargo-xwin >/dev/null 2>&1; then
    echo "==> Cross-compiling Windows x64 (cargo-xwin x86_64-pc-windows-msvc)..."
    cargo xwin build --manifest-path "$ROOT/Cargo.toml" --release --target x86_64-pc-windows-msvc
    cp "$ROOT/target/x86_64-pc-windows-msvc/release/kelvaroq.exe" "$OUT/kelvaroq.exe"
    cp "$ROOT/target/x86_64-pc-windows-msvc/release/kelvaroq_simd.dll" "$OUT/kelvaroq_simd.dll"
else
    echo "WARNING: Neither x86_64-pc-windows-gnu nor cargo-xwin is installed."
    echo "Install with: rustup target add x86_64-pc-windows-gnu"
    echo "Or copy kelvaroq.exe and native/ from a Windows build into $OUT/"
fi

echo "==> Done. Run with: $OUT/kelvaroq play <path/to/game>"
