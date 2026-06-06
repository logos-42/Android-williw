// 连接页：QR + 局域网地址 + API key
import { state } from '../state';
import { API_BASE } from '../api';
import { $ } from '../ui';

function drawQRToCanvas(canvas: HTMLCanvasElement, text: string): void {
  const ctx = canvas.getContext('2d');
  if (!ctx) return;
  ctx.fillStyle = '#fff';
  ctx.fillRect(0, 0, canvas.width, canvas.height);
  const qr = (window as unknown as { qrcode?: (ec: number, mode: string) => { addData: (t: string) => void; make: () => void; getModuleCount: () => number; isDark: (r: number, c: number) => boolean } }).qrcode;
  if (!qr) {
    ctx.fillStyle = '#04110C';
    ctx.font = '14px sans-serif';
    ctx.textAlign = 'center';
    ctx.fillText('QR 库未加载', canvas.width / 2, canvas.height / 2);
    return;
  }
  try {
    const obj = qr(0, 'M');
    obj.addData(text);
    obj.make();
    const count = obj.getModuleCount();
    const tile = Math.floor((canvas.width - 16) / count);
    const offset = (canvas.width - tile * count) / 2;
    ctx.fillStyle = '#fff';
    ctx.fillRect(0, 0, canvas.width, canvas.height);
    ctx.fillStyle = '#04110C';
    for (let r = 0; r < count; r++) {
      for (let c = 0; c < count; c++) {
        if (obj.isDark(r, c)) {
          ctx.fillRect(offset + c * tile, offset + r * tile, Math.ceil(tile), Math.ceil(tile));
        }
      }
    }
  } catch {
    ctx.fillStyle = '#04110C';
    ctx.font = '14px sans-serif';
    ctx.textAlign = 'center';
    ctx.fillText('生成失败', canvas.width / 2, canvas.height / 2);
  }
}

function collectLanAddrs(port: number): string[] {
  // 这个 API 在浏览器里没法直接枚举网卡；由后端通过 cmd_app_info 或 connect 子命令提供。
  // 简化：返回 127.0.0.1 + 桌面 IP 由 settings 推断（如果有）。
  return [`http://127.0.0.1:${port}`];
}

export function renderConnectPage(): void {
  const ciAddr = $('ci-addr');
  const ciKey = $('ci-key');
  const ciModel = $('ci-model');
  const curlSnippet = $('curl-snippet');
  const lanList = $('lan-list');
  const qrCanvas = document.getElementById('qr-canvas') as HTMLCanvasElement | null;
  const connectSub = $('connect-sub');

  if (!state.apiOn) {
    connectSub.textContent = '服务未开启';
    ciAddr.textContent = '—';
    ciKey.textContent = '—';
    ciModel.textContent = '—';
    curlSnippet.textContent = '开启服务后显示调用示例';
    lanList.innerHTML = '';
    return;
  }

  const primary = API_BASE;
  connectSub.textContent = '扫码即可调用本机算力';
  ciAddr.textContent = primary;
  ciKey.textContent = state.apiKey ?? '（未设置）';
  ciModel.textContent = state.selectedModel?.id ?? '—';

  const curlText =
    `curl -X POST ${primary}/v1/chat/completions \\\n` +
    `  -H "content-type: application/json" \\\n` +
    `  -H "Authorization: Bearer ${state.apiKey ?? '<key>'}" \\\n` +
    `  -d '{\n    "model": "${state.selectedModel?.id ?? 'williw-local'}",\n    "messages": [{"role":"user","content":"hi"}]\n  }'`;
  curlSnippet.textContent = curlText;

  // LAN 列表（简版：只显示 127.0.0.1，后端可通过 cmd_get_lan_addrs 提供真实地址）
  const addrs = collectLanAddrs(state.apiPort);
  lanList.innerHTML = '';
  for (const addr of addrs) {
    const row = document.createElement('div');
    row.className = 'lan-row';
    row.innerHTML = `<span class="lan-addr">${addr}</span> <button class="lan-copy">复制</button>`;
    row.querySelector<HTMLButtonElement>('.lan-copy')?.addEventListener('click', () => {
      void navigator.clipboard.writeText(addr);
    });
    lanList.appendChild(row);
  }

  if (qrCanvas) {
    const payload = JSON.stringify({
      base: primary,
      key: state.apiKey ?? '',
      model: state.selectedModel?.id ?? 'williw-local',
    });
    drawQRToCanvas(qrCanvas, payload);
  }
}

export function initConnectScreen(): void {
  // 切到 connect 屏时刷新
  document.querySelectorAll<HTMLElement>('[data-go="connect"]').forEach((b) => {
    b.addEventListener('click', () => renderConnectPage());
  });

  // 复制按钮
  const copyCurl = document.getElementById('copy-curl');
  copyCurl?.addEventListener('click', () => {
    const curlSnippet = $('curl-snippet');
    void navigator.clipboard.writeText(curlSnippet.textContent ?? '');
  });
  const refreshQr = document.getElementById('refresh-qr');
  refreshQr?.addEventListener('click', () => {
    renderConnectPage();
  });
}
