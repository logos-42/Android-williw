// Williw 前端入口
// 1. boot：拉 app info / settings / status / models
// 2. 屏切换：data-go / data-back
// 3. 初始化各屏逻辑

import { API_BASE, getStatus, listModels } from './api';
import { state } from './state';
import { $, $$, isTauri, tauri, showToast } from './ui';
import type { AppInfo, AppSettings, EngineStatus, ModelInfo } from './types';
import { initHomeScreen } from './screens/home';
import { initChatScreen, refreshChatStatus } from './screens/chat';
import { initModelsScreen, refreshModelsList } from './screens/models';
import { initConnectScreen, renderConnectPage } from './screens/connect';

// 暴露 go() 到 window 方便 data-go 处理器 / dev console 调用
declare global {
  interface Window {
    __williwGo?: (name: string) => void;
  }
}

console.log('[williw] main.ts loaded, API_BASE=', API_BASE);

// ====== 屏切换 ======
function go(name: string): void {
  console.log('[williw] go(', name, ')');
  $$('.screen').forEach((s) => s.classList.toggle('active', s.dataset.screen === name));
  $$('.tab').forEach((t) => t.classList.toggle('active', t.dataset.go === name));
  if (name === 'chat') void refreshChatStatus();
  if (name === 'connect') renderConnectPage();
  if (name === 'models') void refreshModelsList();
}
window.__williwGo = go;

// ====== Boot ======
async function boot(): Promise<void> {
  // 1) 先把 API_BASE 写回 state
  state.apiBase = API_BASE;

  // 5) 初始化各屏（先 init，屏切换不依赖 boot 结果）
  initHomeScreen();
  initChatScreen();
  initModelsScreen();
  initConnectScreen();

  // 2) 拉 Tauri cmd 信息（如果在 Tauri 容器里）
  try {
    if (isTauri()) {
      const info = (await tauri<AppInfo>('cmd_app_info')) ?? null;
      if (info) {
        state.info = info;
        state.apiPort = info.api_port || 8081;
        state.apiKey = info.api_key ?? null;
      }
      const settings = (await tauri<AppSettings>('cmd_settings_get')) ?? null;
      if (settings) state.settings = settings;
    } else {
      state.apiPort = 8081;
    }
  } catch (e) {
    console.warn('[boot] tauri cmd failed', e);
  }

  // 3) 拉模型列表 + 状态
  try {
    const [modelsR, status] = await Promise.all([
      listModels(),
      getStatus(),
    ]);
    state.models = modelsR.data;
    if (state.models.length > 0 && !state.selectedModel) {
      state.selectedModel = state.models[0]!;
    }
    state.status = status;
  } catch (e) {
    console.warn('[boot] initial fetch failed', e);
  }

  // 4) 如果 cmd_app_info 已经把 selectedModel 通过 settings 推过来
  if (!state.selectedModel && state.settings?.default_model) {
    const m: ModelInfo = {
      id: state.settings.default_model,
      object: 'model',
      created: 0,
      owned_by: 'williw',
    };
    state.selectedModel = m;
  }

  // 6) 绑定全局屏切换（顶层 bindNav 已绑，这里只额外加轮询）

  // 7) 状态轮询（每 4s 拉一次 /v1/status，仅在主页/聊天可见时拉）
  setInterval(() => {
    const homeVisible = document.querySelector<HTMLElement>('.screen[data-screen="home"]')?.classList.contains('active');
    const chatVisible = document.querySelector<HTMLElement>('.screen[data-screen="chat"]')?.classList.contains('active');
    if (!homeVisible && !chatVisible) return;
    void getStatus().then((s: EngineStatus) => {
      state.status = s;
      const applyPill = (state as unknown as { _applyPowerUI?: () => void })._applyPowerUI;
      if (applyPill) applyPill();
    }).catch(() => undefined);
  }, 4000);

  // 8) 错误兜底：qrcode 库可能没加载
  void $;
  void showToast;
}

// ====== 顶层同步：data-go / data-back 绑定（不依赖 boot） ======
function bindNav(): void {
  const goEls = $$('[data-go]');
  const backEls = $$('[data-back]');
  console.log('[williw] bindNav: go=', goEls.length, 'back=', backEls.length);
  goEls.forEach((el) => el.addEventListener('click', (e) => {
    e.preventDefault();
    e.stopPropagation();
    go(el.dataset.go!);
  }));
  backEls.forEach((el) => el.addEventListener('click', (e) => {
    e.preventDefault();
    e.stopPropagation();
    go(el.dataset.back!);
  }));
  // 兜底：document 级委托
  document.addEventListener('click', (e) => {
    const target = (e.target as HTMLElement | null)?.closest('[data-go], [data-back]') as HTMLElement | null;
    if (!target) return;
    const name = (target.dataset.go ?? target.dataset.back);
    if (!name) return;
    e.preventDefault();
    go(name);
  });
}

// 立即同步执行（不 await boot）
bindNav();

// 启动
void boot();
