#!/usr/bin/env bash
# Fetch FL DOS Candidate Tracking extracts (STA+MUL+LOC) and merge into web/data/fl-dos-{cycle}.tsv
# Usage: ./scripts/refresh-fl-dos.sh [cycle] [elec_id]
#   cycle   default 2026
#   elec_id default 20261103-GEN (general election id on DOS site)
set -euo pipefail
ROOT="$(cd "$(dirname "$0")/.." && pwd)"
CYCLE="${1:-2026}"
ELEC_ID="${2:-${CYCLE}1103-GEN}"
OUT_DIR="$ROOT/web/data"
OUT="$OUT_DIR/fl-dos-${CYCLE}.tsv"
TMP="${TMPDIR:-/tmp}/electionizer-fl-dos-$$"
URL="https://dos.elections.myflorida.com/candidates/extractCanList.asp"

mkdir -p "$OUT_DIR" "$TMP"
cleanup() { rm -rf "$TMP"; }
trap cleanup EXIT

echo "FL DOS cycle=$CYCLE elec_id=$ELEC_ID → $OUT"

for t in STA MUL LOC; do
  echo "  fetching cantype=$t …"
  curl -sS -f -X POST "$URL" \
    -H 'Content-Type: application/x-www-form-urlencoded' \
    -H 'Referer: https://dos.elections.myflorida.com/candidates/' \
    --data "elecID=${ELEC_ID}&office=All&status=All&cantype=${t}&FormSubmit=Download+Candidate+List" \
    -o "$TMP/fl-${t}.tsv"
  lines=$(wc -l < "$TMP/fl-${t}.tsv" | tr -d ' ')
  if [[ "$lines" -lt 2 ]]; then
    echo "error: fl-${t}.tsv looks empty ($lines lines). Check elec_id on DOS site." >&2
    exit 1
  fi
  echo "    $lines lines"
done

{
  head -1 "$TMP/fl-STA.tsv"
  tail -n +2 -q "$TMP/fl-STA.tsv" "$TMP/fl-MUL.tsv" "$TMP/fl-LOC.tsv"
} > "$OUT"

total=$(wc -l < "$OUT" | tr -d ' ')
bytes=$(wc -c < "$OUT" | tr -d ' ')
echo "OK → $OUT ($total lines, $bytes bytes)"
echo "Static app loads this automatically for FL ZIPs when present."
