#!/usr/bin/env bash
# Serve web/ + Wisp proxy (libcurl.js) in one process via wisp-python.
# Usage: ./scripts/run-static.sh [port]
# Then open http://127.0.0.1:PORT/ and set Settings → Wisp URL to ws://127.0.0.1:PORT/
set -euo pipefail
ROOT="$(cd "$(dirname "$0")/.." && pwd)"
PORT="${1:-8080}"
VENV="${ELECTIONIZER_WISP_VENV:-$ROOT/.venv-wisp}"
WEB="$ROOT/web"

if [[ ! -f "$WEB/pkg/electionizer_wasm.js" ]]; then
  echo "web/pkg missing — building WASM first…"
  "$ROOT/scripts/build-wasm.sh"
fi

if [[ ! -x "$VENV/bin/python" ]]; then
  echo "Creating venv at $VENV …"
  python3 -m venv "$VENV"
  "$VENV/bin/pip" install -q -U pip
  "$VENV/bin/pip" install -q 'wisp-python>=0.9'
fi

# Ensure wisp-python present (venv may pre-exist without it)
if ! "$VENV/bin/python" -c "import wisp.server" 2>/dev/null; then
  "$VENV/bin/pip" install -q 'wisp-python>=0.9'
fi

echo "Serving $WEB on http://127.0.0.1:${PORT}/"
echo "Wisp WS:  ws://127.0.0.1:${PORT}/"
echo "Settings → Wisp URL → ws://127.0.0.1:${PORT}/  (or click “Use this origin”)"
echo ""

exec "$VENV/bin/python" -m wisp.server \
  --host 127.0.0.1 \
  --port "$PORT" \
  --static "$WEB" \
  --log-level info
