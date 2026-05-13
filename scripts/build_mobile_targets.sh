#!/usr/bin/env bash
set -euo pipefail

TARGETS=(
  aarch64-apple-ios
  aarch64-apple-ios-sim
  x86_64-apple-ios
  aarch64-linux-android
  armv7-linux-androideabi
  x86_64-linux-android
  i686-linux-android
)

TOOLCHAIN="${RUSTUP_TOOLCHAIN:-stable}"
if command -v rustup >/dev/null 2>&1 && rustup toolchain list | grep -q "^${TOOLCHAIN}"; then
  RUSTC_PATH="$(rustup which --toolchain "$TOOLCHAIN" rustc)"
  CARGO_PATH="$(rustup which --toolchain "$TOOLCHAIN" cargo)"
  CARGO=(env "RUSTC=$RUSTC_PATH" "$CARGO_PATH")
  TARGET_LIST=(rustup target list --toolchain "$TOOLCHAIN" --installed)
else
  CARGO=(cargo)
  TARGET_LIST=()
fi

if ((${#TARGET_LIST[@]})); then
  for target in "${TARGETS[@]}"; do
    if ! "${TARGET_LIST[@]}" | grep -qx "$target"; then
      echo "missing Rust target for toolchain $TOOLCHAIN: $target"
      echo "install it with: rustup target add --toolchain $TOOLCHAIN $target"
      exit 1
    fi
  done
else
  for target in "${TARGETS[@]}"; do
    if ! rustc --print target-libdir --target "$target" >/dev/null 2>&1; then
      echo "missing Rust target: $target"
      echo "install it with rustup, or set RUSTUP_TOOLCHAIN to an installed toolchain"
      exit 1
    fi
  done
fi

"${CARGO[@]}" test --no-default-features --test portable_api
"${CARGO[@]}" test --no-default-features --features ffi --test ffi_smoke

for target in "${TARGETS[@]}"; do
  echo "checking decoder core for $target"
  "${CARGO[@]}" check --no-default-features --target "$target"

  echo "checking FFI surface for $target"
  "${CARGO[@]}" check --no-default-features --features ffi --lib --target "$target"
done
