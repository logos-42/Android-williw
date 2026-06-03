#!/usr/bin/env bash
# test-e2e.sh
# 端到端冒烟测试：启动 api-server、curl 所有端点、断言响应、停服。
# 不会拉起 Tauri 窗口（避免需要图形环境）；只测 HTTP API。

set -euo pipefail
cd "$(dirname "$0")/.."

PORT="${WILLIW_API_PORT:-18083}"
LOG="/tmp/williw-e2e-$$.log"
PID=""

cleanup() {
  if [[ -n "$PID" ]] && kill -0 "$PID" 2>/dev/null; then
    kill "$PID" 2>/dev/null || true
    wait "$PID" 2>/dev/null || true
  fi
  rm -f "$LOG"
}
trap cleanup EXIT

echo "==> start api-server on :$PORT"
WILLIW_API_PORT="$PORT" ./target/release/williw-api > "$LOG" 2>&1 &
PID=$!
sleep 3

if ! curl -fsS -m 3 "http://127.0.0.1:$PORT/health" > /dev/null; then
  echo "✗ health endpoint not responding"
  cat "$LOG"
  exit 1
fi
echo "  ✓ /health"

MODELS=$(curl -fsS -m 3 "http://127.0.0.1:$PORT/v1/models")
if ! echo "$MODELS" | grep -q '"object":"list"'; then
  echo "✗ /v1/models returned: $MODELS"; exit 1
fi
echo "  ✓ /v1/models"

STATUS=$(curl -fsS -m 3 "http://127.0.0.1:$PORT/v1/status")
if ! echo "$STATUS" | grep -q '"state":"ready"'; then
  echo "✗ /v1/status not ready: $STATUS"; exit 1
fi
echo "  ✓ /v1/status (state=ready)"

CHAT=$(curl -fsS -m 10 -X POST "http://127.0.0.1:$PORT/v1/chat/completions" \
  -H "content-type: application/json" \
  -d '{"model":"williw-local","messages":[{"role":"user","content":"ping"}],"max_tokens":32}')
if ! echo "$CHAT" | grep -q '"finish_reason":"stop"'; then
  echo "✗ /v1/chat/completions bad response: $CHAT"; exit 1
fi
echo "  ✓ /v1/chat/completions"

echo ""
echo "✅ All e2e checks passed."
