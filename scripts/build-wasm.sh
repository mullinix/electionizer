#!/usr/bin/env bash
# Build electionizer-wasm for the browser into web/pkg/
set -euo pipefail
ROOT="$(cd "$(dirname "$0")/.." && pwd)"
cd "$ROOT"

# Prefer rustup toolchain (Homebrew rust often lacks wasm32 std)
if command -v rustup >/dev/null 2>&1; then
  CARGO="$(rustup which cargo 2>/dev/null || true)"
fi
CARGO="${CARGO:-cargo}"

# CI / Netlify: bootstrap rustup if only system cargo is missing wasm target tooling
if ! command -v rustup >/dev/null 2>&1 && [[ -n "${NETLIFY:-}${CI:-}" ]]; then
  echo "CI: installing rustup…"
  curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh -s -- -y --default-toolchain stable
  # shellcheck disable=SC1091
  source "$HOME/.cargo/env"
fi

if ! command -v wasm-pack >/dev/null 2>&1; then
  if [[ -n "${NETLIFY:-}${CI:-}" ]]; then
    echo "CI: installing wasm-pack…"
    curl https://rustwasm.github.io/wasm-pack/installer/init.sh -sSf | sh
    export PATH="$HOME/.cargo/bin:$PATH"
  else
    echo "wasm-pack not found. Install with:"
    echo "  cargo install wasm-pack"
    echo "  # or: curl https://rustwasm.github.io/wasm-pack/installer/init.sh -sSf | sh"
    exit 1
  fi
fi

rustup target add wasm32-unknown-unknown >/dev/null 2>&1 || true

echo "Building with: $CARGO"
export CARGO_TARGET_DIR="${CARGO_TARGET_DIR:-$ROOT/target}"
# Point wasm-pack at rustup cargo when PATH has Homebrew first
export PATH="$(dirname "$CARGO"):$PATH"

cp "$ROOT/voter-profile-defaults.json" "$ROOT/web/voter-profile-defaults.json"

wasm-pack build crates/electionizer-wasm \
  --target web \
  --out-dir "$ROOT/web/pkg" \
  --release

echo "OK → web/pkg/"
echo "Serve:  python3 -m http.server 8080 --directory web"
echo "Open:   http://127.0.0.1:8080/"
