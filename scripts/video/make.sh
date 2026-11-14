#!/usr/bin/env bash
# Rebuild demo/surety-demo.mp4: build the board, record scenes with Playwright, TTS voice, compose with ffmpeg.
# Needs: node + playwright (chromium), python3, ffmpeg with libass, pip install edge-tts.
set -euo pipefail
HERE="$(cd "$(dirname "$0")" && pwd)"; ROOT="$HERE/../.."
W="$HERE/.work"; rm -rf "$W"; mkdir -p "$W"
node "$ROOT/scripts/build-worker.js"
cp "$ROOT/web/dist/index.html" "$W/board.html"; cp "$ROOT/demo/txs.json" "$W/txs.json"
cp "$HERE/tx.html" "$HERE/record.js" "$HERE/compose.py" "$HERE/voice.py" "$W/"
(cd "$W" && python3 -m http.server 8766 >/dev/null 2>&1 & echo $! > "$W/http.pid")
trap 'kill $(cat "$W/http.pid") 2>/dev/null || true' EXIT
sleep 1
(cd "$W" && node record.js && python3 voice.py && python3 compose.py)
cp "$W/surety-demo.mp4" "$ROOT/demo/surety-demo.mp4"
echo "wrote demo/surety-demo.mp4"
