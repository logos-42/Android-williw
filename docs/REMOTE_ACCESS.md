# Williw · 任意网络访问

目标：在任意网络（不只是局域网）的设备上，调用本机 williw 算力服务。

## 架构

```
[任意网络的设备]                                       [你的桌面]
+-----------------------+                      +-----------------------+
| 手机 / 笔记本 / 服务器 |   --- 公网隧道 --->  | williw.exe (Tauri)    |
|                       |                      |  ├── Tauri Webview UI  |
| curl $URL/v1/chat/... |                      |  └── Axum :8081       |
+-----------------------+                      |       └─ candle + GGUF
                                               |          (Qwen2.5-0.5B)
                                               +-----------------------+
```

## 方案 A：ngrok（推荐，零配置）

### 1. 注册 + 安装

- https://dashboard.ngrok.com/signup 注册
- https://dashboard.ngrok.com/get-started/your-authtoken 拿 authtoken
- 安装：`choco install ngrok` 或 `scoop install ngrok`

### 2. 启动 williw 桌面壳

```bash
cd williw
cargo run -p williw-tauri --release
# 或直接跑 ./target/release/williw.exe
# settings.json 里 auto_start=true 时，API 自动绑 0.0.0.0:8081
```

### 3. 起 ngrok 隧道

```bash
# PowerShell
$env:NGROK_AUTHTOKEN = "2abc...你的 token..."
.\scripts\start-ngrok.ps1

# 或 bash
NGROK_AUTHTOKEN=2abc...xyz ./scripts/start-ngrok.sh
```

ngrok 会打印：
```
Session Status    online
Forwarding        https://xxxx-xxx-xxx-xxx-xxx.ngrok-free.app -> http://127.0.0.1:8081
```

**这个 `https://xxxx.ngrok-free.app` 就是公网 base URL。**

### 4. 任意设备调用

```bash
# 你 settings.json 里的 api_key
KEY=9d6bce3d84744bf73642f95489acb11de0afa88d73f033e7
URL=https://xxxx-xxx-xxx-xxx-xxx.ngrok-free.app

curl -X POST "$URL/v1/chat/completions" \
  -H "content-type: application/json" \
  -H "Authorization: Bearer $KEY" \
  -d '{
    "model": "williw-local",
    "messages": [{"role":"user","content":"用一句话介绍上海。"}],
    "max_tokens": 60,
    "temperature": 0.3
  }'
```

返回：
```json
{
  "choices":[{"message":{"role":"assistant","content":"上海是中国的直辖市，位于..."}}],
  "usage":{"prompt_tokens":13,"completion_tokens":22,"total_tokens":35}
}
```

### 5. 配套客户端

任何兼容 OpenAI Chat Completions API 的客户端都能直接用：
- **Open WebUI** (https://github.com/open-webui/open-webui)
- **LobeChat**
- **NextChat**
- **ChatBox**
- **VSCode 插件**: Cline / Continue
- **手机**: ChatBox / OpenCat / 等任何支持自定义 base URL 的

把 base URL 设成 ngrok 给的地址、api key 设成 williw 的 api_key、模型名 `williw-local` 即可。

## 方案 B：Cloudflare Tunnel（无流量限制，需要域名）

如果你有自己的域名托管在 Cloudflare：

```bash
cloudflared tunnel login
cloudflared tunnel create williw
cloudflared tunnel route dns williw ai.example.com
cloudflared tunnel run --url http://127.0.0.1:8081 williw
```

## 方案 C：Tailscale Funnel（设备间 Tailscale 网格 + 公网）

```bash
# 你的机器
sudo tailscale up
sudo tailscale funnel 8081 on
# 任意设备（要装 Tailscale）
curl -H "Authorization: Bearer $KEY" \
  https://你的机器名.tail-xxxx.ts.net/v1/chat/completions -d '...'
```

## 方案 D：本地 0.0.0.0 + 路由器端口映射

最朴素。需要：
1. 路由器有公网 IP（向运营商要）
2. 路由器后台 → 虚拟服务器 / DMZ → 把外网 8081 映射到局域网 192.168.x.x:8081
3. 防火墙放行 8081

然后公网访问 `http://你的公网IP:8081/v1/chat/completions` 即可。

⚠️ **风险**：直接暴露 8081 到公网 + Bearer 鉴权 ≈ 弱保护。
**强烈建议**：用 ngrok / Cloudflare / Tailscale 这类带 HTTPS + 一定访问控制的方案。

## 性能

- 0.5B Q4_K_M 在笔记本 CPU 上：约 2-5 tokens/秒
- 公网延迟 = 本地推理时间 + ngrok 隧道往返（通常 < 50ms）
- 长 prompt 时会更慢（candle 0.8.4 的 quantized_qwen2 不带 KV cache，每次 forward 重算 prompt 段）

## 故障排查

| 现象 | 排查 |
|---|---|
| ngrok 起来后 curl 超时 | 检查 williw API 是否在 127.0.0.1:8081 监听：`curl http://127.0.0.1:8081/v1/status` |
| 401 invalid api key | 请求里加 `-H "Authorization: Bearer <settings.json 里的 api_key>"` |
| 502 from ngrok | williw 进程挂了 / 端口没起 |
| 中文乱码（`åĮĹäº¬...`） | 用了**未启用 candle feature** 的旧 williw 二进制。重编：`cargo build --release -p williw-tauri --features williw-core/candle` |
| 模型 not found | `williw/models/qwen2.5-0.5b-instruct-q4_k_m.gguf` 不存在；从 https://huggingface.co/Qwen/Qwen2.5-0.5B-Instruct-GGUF 下载 |
