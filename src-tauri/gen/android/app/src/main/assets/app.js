/* =============================================================
 *  Williw 移动端 · 核心控制
 *  v0.1.1
 *
 *  设计目标：
 *   - 一个开关：决定手机是否对外提供 AI 算力
 *   - 主页：状态 + 开关 + 关键信息
 *   - 模型页：选择 / 下载本地模型
 *   - 连接页：开启后展示 QR + 地址 + 密钥 + 调用示例
 * ============================================================= */

(function () {
  'use strict';

  // ====== 基础 ======
  const $ = (id) => document.getElementById(id);
  const API_BASE = (window.__WILLIW_API_BASE__ || 'http://127.0.0.1:8081').replace(/\/+$/, '');
  const isTauri = !!(window.__TAURI__ || window.__TAURI_INTERNALS__);

  // Tauri 命令调用（v2 通过 __TAURI_INTERNALS__.invoke）
  async function tauri(cmd, args) {
    if (isTauri && window.__TAURI_INTERNALS__ && window.__TAURI_INTERNALS__.invoke) {
      return await window.__TAURI_INTERNALS__.invoke(cmd, args || {});
    }
    return null;
  }

  // ====== 状态 ======
  const state = {
    apiOn: false,
    apiPort: 8081,
    apiKey: null,
    info: null,
    status: null,
    model: null,
    models: [],
  };

  // ====== 工具 ======
  function fmtBytes(n) {
    if (n == null) return '—';
    if (n < 1024) return n + ' B';
    if (n < 1024 * 1024) return (n / 1024).toFixed(1) + ' KB';
    if (n < 1024 * 1024 * 1024) return (n / 1024 / 1024).toFixed(1) + ' MB';
    return (n / 1024 / 1024 / 1024).toFixed(2) + ' GB';
  }

  function showToast(msg, ms) {
    const t = $('toast');
    t.textContent = msg;
    t.hidden = false;
    requestAnimationFrame(() => t.classList.add('show'));
    clearTimeout(showToast._t);
    showToast._t = setTimeout(() => {
      t.classList.remove('show');
      setTimeout(() => (t.hidden = true), 240);
    }, ms || 2200);
  }

  async function copyText(text) {
    try {
      await navigator.clipboard.writeText(text);
      showToast('已复制');
    } catch (e) {
      showToast('复制失败');
    }
  }

  // ====== 路由（屏幕切换） ======
  function go(name) {
    document.querySelectorAll('.screen').forEach((s) => {
      s.classList.toggle('active', s.dataset.screen === name);
    });
    document.querySelectorAll('.tab').forEach((t) => {
      t.classList.toggle('active', t.dataset.go === name);
    });
    if (name === 'models') refreshModels();
    if (name === 'connect') refreshConnect();
  }

  document.querySelectorAll('[data-go]').forEach((el) =>
    el.addEventListener('click', () => go(el.dataset.go))
  );
  document.querySelectorAll('[data-back]').forEach((el) =>
    el.addEventListener('click', () => go(el.dataset.back))
  );

  // ====== 主页：开关 ======
  const powerBtn = $('power-btn');
  const powerLabel = $('power-label');
  const ringFg = $('ring-fg');
  const stateText = $('state-text');
  const addrText = $('addr-text');
  const heroModel = $('hero-model');
  const modelPill = $('model-pill');
  const modelPillText = $('model-pill-text');
  const tipCard = $('tip-card') || document.querySelector('.tip');

  function applyPowerUI() {
    if (state.apiOn) {
      powerBtn.classList.add('on');
      powerLabel.textContent = '关闭';
      ringFg.classList.add('on');
      stateText.textContent = state.status && state.status.state === 'ready'
        ? '算力服务运行中' : '正在启动…';
      addrText.textContent = API_BASE;
    } else {
      powerBtn.classList.remove('on');
      powerLabel.textContent = '开启';
      ringFg.classList.remove('on');
      stateText.textContent = '已关闭';
      addrText.textContent = '—';
    }
  }

  function applyModelPill() {
    const m = state.model;
    if (!m || !m.id) {
      modelPill.dataset.state = 'offline';
      modelPillText.textContent = '未加载模型';
      heroModel.textContent = '—';
      return;
    }
    const loaded = m.state === 'ready' || m.state === 'loaded';
    modelPill.dataset.state = loaded ? 'ready' : (m.state || 'offline');
    modelPillText.textContent = loaded ? m.name : (m.name + ' · 未就绪');
    heroModel.textContent = m.name;
  }

  powerBtn.addEventListener('click', async () => {
    if (state.apiOn) {
      // 关闭
      await tauri('cmd_api_set_enabled', { enabled: false });
      state.apiOn = false;
      applyPowerUI();
      showToast('已关闭算力服务');
    } else {
      // 开启：必须先有模型
      if (!state.model || !state.model.id) {
        showToast('请先在「模型」页选择或下载模型', 2400);
        go('models');
        return;
      }
      // 首次开启时如果没有 apiKey，自动生成
      if (!state.apiKey) {
        const k = generateKey();
        await tauri('cmd_settings_set_api_key', { apiKey: k });
        state.apiKey = k;
      }
      await tauri('cmd_api_set_enabled', { enabled: true });
      state.apiOn = true;
      applyPowerUI();
      showToast('算力服务已开启');
    }
  });

  // ====== 启动拉取：app info / settings / status / models ======
  async function boot() {
    if (isTauri) {
      const info = await tauri('cmd_app_info');
      if (info) {
        state.info = info;
        state.apiPort = info.api_port || 8081;
      }
      const s = await tauri('cmd_settings_get');
      if (s) {
        state.apiKey = s.api_key || null;
        state.apiPort = s.api_port || state.apiPort;
      }
      const status = await tauri('cmd_api_status');
      if (status) {
        state.apiOn = !!status.enabled;
        state.model = status.model || null;
      }
    }
    applyPowerUI();
    applyModelPill();
    pollStatus();
  }

  // ====== 状态轮询 ======
  async function pollStatus() {
    try {
      // Tauri 命令优先，回退到 HTTP
      let st = null;
      if (isTauri) st = await tauri('cmd_status');
      if (!st) {
        const r = await fetch(API_BASE + '/v1/status');
        if (r.ok) st = await r.json();
      }
      if (st) {
        state.status = st;
        // 如果有 model_id，更新 state.model
        if (st.model_id) {
          state.model = {
            id: st.model_id,
            name: st.model_id,
            state: st.state,
            path: st.model_path,
          };
        } else if (st.state === 'idle' || st.state === 'offline') {
          // keep last known model
        }
        applyModelPill();
        // 当服务开着时，更新 ring/state 文本
        if (state.apiOn) {
          stateText.textContent = st.state === 'ready'
            ? '算力服务运行中' : ('状态: ' + (st.state || '…'));
        }
      }
    } catch (e) { /* 静默 */ }
  }

  setInterval(pollStatus, 3500);

  // ====== 模型页 ======
  const modelList = $('model-list');
  const curName = $('cur-name');
  const curSub = $('cur-sub');
  const curState = $('cur-state');

  // 推荐模型（前端硬编码，后端可扩展）
  const RECOMMENDED = [
    { id: 'qwen2.5-0.5b-instruct-q4_k_m',
      name: 'Qwen 2.5 0.5B (q4)',
      size: '约 400 MB',
      url: 'https://huggingface.co/Qwen/Qwen2.5-0.5B-Instruct-GGUF/resolve/main/qwen2.5-0.5b-instruct-q4_k_m.gguf',
      filename: 'qwen2.5-0.5b-instruct-q4_k_m.gguf',
      desc: '超小 · 速度快 · 中文友好',
    },
    { id: 'qwen2.5-1.5b-instruct-q4_k_m',
      name: 'Qwen 2.5 1.5B (q4)',
      size: '约 1.1 GB',
      url: 'https://huggingface.co/Qwen/Qwen2.5-1.5B-Instruct-GGUF/resolve/main/qwen2.5-1.5b-instruct-q4_k_m.gguf',
      filename: 'qwen2.5-1.5b-instruct-q4_k_m.gguf',
      desc: '均衡 · 质量与速度兼顾',
    },
    { id: 'qwen2.5-3b-instruct-q4_k_m',
      name: 'Qwen 2.5 3B (q4)',
      size: '约 2.0 GB',
      url: 'https://huggingface.co/Qwen/Qwen2.5-3B-Instruct-GGUF/resolve/main/qwen2.5-3b-instruct-q4_k_m.gguf',
      filename: 'qwen2.5-3b-instruct-q4_k_m.gguf',
      desc: '更强 · 需要 4GB+ 内存',
    },
  ];

  function renderCurrentModel() {
    const m = state.model;
    if (!m || !m.id) {
      curName.textContent = '未加载';
      curSub.textContent = '请在下方选择或下载模型';
      curState.textContent = '—';
      curState.className = 'cm-state';
      return;
    }
    curName.textContent = m.name;
    curSub.textContent = m.path || (m.id + ' · 已就绪');
    const s = m.state || 'idle';
    curState.textContent = s;
    curState.className = 'cm-state ' + s;
  }

  async function refreshModels() {
    renderCurrentModel();
    // 已下载列表
    let installed = [];
    if (isTauri) {
      installed = (await tauri('cmd_models_list')) || [];
    }
    state.models = installed;

    // 合并：推荐 + 已下载（去重）
    const installedIds = new Set(installed.map((m) => m.id));
    const items = [];

    installed.forEach((m) => {
      items.push({
        kind: 'installed',
        id: m.id,
        name: m.name,
        size: fmtBytes(m.size_bytes),
        state: m.state,
        path: m.path,
        isDefault: m.is_default,
      });
    });
    RECOMMENDED.forEach((r) => {
      if (!installedIds.has(r.id)) {
        items.push({
          kind: 'recommend',
          id: r.id,
          name: r.name,
          size: r.size,
          desc: r.desc,
          url: r.url,
          filename: r.filename,
        });
      }
    });

    modelList.innerHTML = '';
    if (!items.length) {
      modelList.innerHTML = '<div class="empty-list">暂无模型</div>';
      return;
    }
    items.forEach((it) => {
      const li = document.createElement('li');
      li.className = 'model-item' + (it.kind === 'installed' && (it.state === 'ready' || it.state === 'loaded') ? ' is-loaded' : '');

      if (it.kind === 'installed') {
        const stateLabel = it.state || '未加载';
        li.innerHTML = `
          <div class="mi-main">
            <div class="mi-name"></div>
            <div class="mi-meta">
              <span>${it.size}</span>
              <span class="dot-sep"></span>
              <span>${stateLabel}</span>
            </div>
          </div>
          <button type="button" class="mi-action use" data-act="use">使用</button>
        `;
        li.querySelector('.mi-name').textContent = it.name;
        li.querySelector('[data-act="use"]').addEventListener('click', () => useModel(it));
      } else {
        li.innerHTML = `
          <div class="mi-main">
            <div class="mi-name"></div>
            <div class="mi-meta">
              <span>${it.size}</span>
              <span class="dot-sep"></span>
              <span>${it.desc}</span>
            </div>
          </div>
          <button type="button" class="mi-action dl" data-act="dl">下载</button>
        `;
        li.querySelector('.mi-name').textContent = it.name;
        li.querySelector('[data-act="dl"]').addEventListener('click', () => downloadRecommended(it));
      }
      modelList.appendChild(li);
    });
  }

  async function useModel(m) {
    if (!m.path) { showToast('该模型缺少路径'); return; }
    showToast('正在加载 ' + m.name);
    const r = await tauri('cmd_reload', { req: { model_path: m.path } });
    if (r && r.model_id) {
      state.model = {
        id: r.model_id,
        name: r.model_id,
        state: r.state,
        path: r.model_path,
      };
      applyModelPill();
      renderCurrentModel();
      showToast('已加载');
    } else if (r) {
      showToast('加载完成');
      state.model = { id: m.id, name: m.name, state: 'ready', path: m.path };
      applyModelPill();
      renderCurrentModel();
    } else {
      showToast('加载失败');
    }
  }

  function downloadRecommended(it) {
    $('dl-url').value = it.url;
    $('dl-name').value = it.filename;
    startDownload();
  }

  // ====== 下载 ======
  const sheet = $('sheet');
  const sheetTitle = $('sheet-title');
  const sheetSub = $('sheet-sub');
  const progressBar = $('progress-bar');
  let downloadCancel = false;

  $('sheet-cancel').addEventListener('click', () => {
    downloadCancel = true;
    closeSheet();
    showToast('已取消');
  });
  document.querySelector('.sheet-mask').addEventListener('click', () => {
    // 不允许通过遮罩取消
  });

  function openSheet(title, sub) {
    sheetTitle.textContent = title;
    sheetSub.textContent = sub;
    progressBar.style.width = '0%';
    sheet.hidden = false;
  }
  function closeSheet() { sheet.hidden = true; }

  $('btn-dl').addEventListener('click', startDownload);

  async function startDownload() {
    const url = $('dl-url').value.trim();
    const filename = $('dl-name').value.trim() || (url.split('/').pop() || 'model.gguf');
    if (!url) { showToast('请填写 URL'); return; }
    downloadCancel = false;
    openSheet('下载模型', filename);
    // 调用 Tauri 命令执行下载（支持进度）
    await tauri('cmd_download_stream', { req: { url, filename } });
    // 进度由后端通过 event 推过来
  }

  // 监听后端下载进度（如果可用）
  function listenDownloadEvents() {
    if (!isTauri) return;
    const tryListen = () => {
      if (window.__TAURI__ && window.__TAURI__.event && window.__TAURI__.event.listen) {
        window.__TAURI__.event.listen('download://progress', (ev) => {
          const p = ev.payload || {};
          if (typeof p.percent === 'number') {
            progressBar.style.width = Math.min(100, Math.max(0, p.percent)) + '%';
          }
          if (p.label) sheetSub.textContent = p.label;
        });
        window.__TAURI__.event.listen('download://done', (ev) => {
          const p = ev.payload || {};
          progressBar.style.width = '100%';
          sheetSub.textContent = '下载完成';
          setTimeout(() => {
            closeSheet();
            showToast('下载完成');
            refreshModels();
          }, 600);
        });
        window.__TAURI__.event.listen('download://error', (ev) => {
          const p = ev.payload || {};
          closeSheet();
          showToast('下载失败：' + (p.message || '未知错误'), 3200);
        });
      } else {
        setTimeout(tryListen, 200);
      }
    };
    tryListen();
  }
  listenDownloadEvents();

  // ====== 连接页 ======
  const ciAddr = $('ci-addr');
  const ciKey = $('ci-key');
  const ciModel = $('ci-model');
  const qrCanvas = $('qr');
  const curlSnippet = $('curl-snippet');
  const connectSub = $('connect-sub');

  function refreshConnect() {
    if (!state.apiOn) {
      connectSub.textContent = '服务未开启';
      ciAddr.textContent = '—';
      ciKey.textContent = '—';
      ciModel.textContent = '—';
      drawPlaceholderQR();
      curlSnippet.textContent = '开启服务后显示调用示例';
      return;
    }
    connectSub.textContent = '扫码即可调用本机算力';
    ciAddr.textContent = API_BASE;
    ciKey.textContent = state.apiKey || '（未设置）';
    ciModel.textContent = (state.model && state.model.name) || '—';

    // QR 内容：URL + 端口 + 密钥（其他设备扫码后用此调用）
    const payload = JSON.stringify({
      v: 1,
      url: API_BASE + '/v1/chat/completions',
      port: state.apiPort,
      key: state.apiKey || '',
      model: (state.model && state.model.id) || 'williw-local',
    });
    drawQRToCanvas(qrCanvas, payload);
    curlSnippet.textContent =
      "curl -X POST " + API_BASE + "/v1/chat/completions \\\n" +
      "  -H 'Content-Type: application/json' \\\n" +
      "  -H 'Authorization: Bearer " + (state.apiKey || '<KEY>') + "' \\\n" +
      "  -d '{\n" +
      "    \"model\": \"" + ((state.model && state.model.id) || 'williw-local') + "\",\n" +
      "    \"messages\": [{\"role\": \"user\", \"content\": \"你好\"}]\n" +
      "  }'";
  }

  $('copy-curl').addEventListener('click', () => copyText(curlSnippet.textContent));
  $('refresh-qr').addEventListener('click', () => { refreshConnect(); showToast('已刷新'); });

  // ====== 简单 QR 绘制（无外部依赖；使用 qrcode-generator 的轻量子集算法） ======
  // 这里用一个更简单可靠的方案：调一个最小的 QR 库
  // 实际生成通过动态注入脚本
  function drawPlaceholderQR() {
    const ctx = qrCanvas.getContext('2d');
    ctx.fillStyle = '#fff';
    ctx.fillRect(0, 0, qrCanvas.width, qrCanvas.height);
    ctx.fillStyle = '#04110C';
    ctx.font = '12px sans-serif';
    ctx.textAlign = 'center';
    ctx.fillText('开启服务后显示', qrCanvas.width / 2, qrCanvas.height / 2);
  }

  // 加载 QR 库（极简实现）
  loadQRLib().then((ok) => {
    if (ok) refreshConnect();
  });

  function loadQRLib() {
    return new Promise((resolve) => {
      if (window.qrcode) return resolve(true);
      const s = document.createElement('script');
      s.src = './qrcode.min.js';
      s.onload = () => resolve(!!window.qrcode);
      s.onerror = () => resolve(false);
      document.head.appendChild(s);
    });
  }

  function drawQRToCanvas(canvas, text) {
    const ctx = canvas.getContext('2d');
    ctx.fillStyle = '#fff';
    ctx.fillRect(0, 0, canvas.width, canvas.height);
    if (!window.qrcode) {
      drawPlaceholderQR();
      return;
    }
    try {
      const qr = window.qrcode(0, 'M');
      qr.addData(text);
      qr.make();
      const count = qr.getModuleCount();
      const size = canvas.width - 12; // 边距
      const tile = size / count;
      const offset = 6;
      // 浅绿背景
      ctx.fillStyle = '#fff';
      ctx.fillRect(0, 0, canvas.width, canvas.height);
      ctx.fillStyle = '#04110C';
      for (let r = 0; r < count; r++) {
        for (let c = 0; c < count; c++) {
          if (qr.isDark(r, c)) {
            ctx.fillRect(offset + c * tile, offset + r * tile, Math.ceil(tile), Math.ceil(tile));
          }
        }
      }
    } catch (e) {
      drawPlaceholderQR();
    }
  }

  // ====== 工具：生成 API key ======
  function generateKey() {
    const arr = new Uint8Array(24);
    (window.crypto || window.msCrypto).getRandomValues(arr);
    return Array.from(arr, (b) => b.toString(16).padStart(2, '0')).join('');
  }

  // ====== Boot ======
  boot();
})();
