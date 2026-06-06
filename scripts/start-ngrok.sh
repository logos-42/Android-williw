#!/usr/bin/env bash
# ----------------------------------------------------------------------------
# Williw · ngrok 公网分享脚本
#
# 目的：把本地 williw API (0.0.0.0:8081) 通过 ngrok 隧道暴露到公网，
#       让任意网络的设备都能用 OpenAI 兼容协议调用本机 AI 算力。
#
# 前置：
#   1. 已注册 ngrok 账号：https://dashboard.ngrok.com/signup
#   2. 拿到 authtoken：https://dashboard.ngrok.com/get-started/your-authtoken
#   3. 安装 ngrok：https://ngrok.com/download  （或 `choco install ngrok`）
#
# 用法：
#   1. 第一次：export NGROK_AUTHTOKEN=2abc...xyz && ./scripts/start-ngrok.sh
#   2. 之后：直接 ./scripts/start-ngrok.sh
#
# 脚本启动后会打印一个 https://xxxx.ngrok-free.app 的公网地址，
# 那个就是公网可用的 OpenAI 兼容 API base。
# 任何能上网的设备用这个地址 + 你 settings.json 里的 api_key 就能调本机算力。
# ----------------------------------------------------------------------------
set -euo pipefail

PORT="${WILLIW_API_PORT:-8081}"

if ! command -v ngrok >/dev/null 2>&1; then
  echo "❌ ngrok 未安装。"
  echo "   下载：https://ngrok.com/download"
  echo "   或者：choco install ngrok   (Windows / scoop install ngrok)"
  exit 1
fi

if [[ -n "${NGROK_AUTHTOKEN:-}" ]]; then
  echo "→ 设置 ngrok authtoken ..."
  ngrok config add-authtoken "$NGROK_AUTHTOKEN"
fi

# 健康检查：8081 是否在跑
if ! curl -s --max-time 2 "http://127.0.0.1:${PORT}/health" >/dev/null 2>&1; then
  echo "⚠️  williw API 在 ${PORT} 端口没有响应。"
  echo "   先启动 williw 桌面壳，或单独跑 williw-api.exe。"
  echo "   （Tauri 默认 auto_start=true 时 API 会自动起来。）"
  exit 1
fi

echo "🚀 启动 ngrok 隧道：127.0.0.1:${PORT} -> 公网"
echo "   任何设备都可以用 ngrok 给的 https://*.ngrok-free.app 地址 + api_key 调用"
echo "   Ctrl+C 退出"
echo
ngrok http "${PORT}"
