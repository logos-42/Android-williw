# ----------------------------------------------------------------------------
# Williw · ngrok 公网分享脚本 (Windows PowerShell)
#
# 目的：把本地 williw API (0.0.0.0:8081) 通过 ngrok 暴露到公网，
#       让任意网络的设备都能用 OpenAI 兼容协议调用本机算力。
#
# 前置：
#   1. 注册 ngrok 账号：https://dashboard.ngrok.com/signup
#   2. 拿 authtoken：https://dashboard.ngrok.com/get-started/your-authtoken
#   3. 安装 ngrok：choco install ngrok   或   scoop install ngrok
#
# 用法：
#   $env:NGROK_AUTHTOKEN = "2abc...xyz"
#   .\scripts\start-ngrok.ps1
#
# 打印的 https://xxxx.ngrok-free.app 即为公网可用的 OpenAI 兼容 API base。
# 客户端用 base + api_key 调本机算力。
# ----------------------------------------------------------------------------
$ErrorActionPreference = "Stop"

$port = if ($env:WILLIW_API_PORT) { $env:WILLIW_API_PORT } else { 8081 }

$ngrok = Get-Command ngrok -ErrorAction SilentlyContinue
if (-not $ngrok) {
  Write-Host "❌ ngrok 未安装。" -ForegroundColor Red
  Write-Host "   安装: choco install ngrok  或  scoop install ngrok"
  exit 1
}

if ($env:NGROK_AUTHTOKEN) {
  Write-Host "→ 设置 ngrok authtoken ..."
  & ngrok config add-authtoken $env:NGROK_AUTHTOKEN
}

# 健康检查
try {
  $null = Invoke-RestMethod -Uri "http://127.0.0.1:${port}/health" -TimeoutSec 2
} catch {
  Write-Host "⚠️  williw API 在 ${port} 端口没有响应。" -ForegroundColor Yellow
  Write-Host "   先启动 williw 桌面壳，或单独跑 williw-api.exe。"
  exit 1
}

Write-Host "🚀 启动 ngrok 隧道：127.0.0.1:${port} -> 公网" -ForegroundColor Green
Write-Host "   任意设备可用 ngrok 给的 https://*.ngrok-free.app 地址 + api_key 调用"
Write-Host "   Ctrl+C 退出"
Write-Host ""
& ngrok http $port
