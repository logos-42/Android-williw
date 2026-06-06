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

// ====== 屏切换 ======
function go(name: string): void {
  $$('.screen').forEach((s) => s.classList.toggle('active', s.dataset.screen === name));
  $$('.tab').forEach((t) => t.classList.toggle('active', t.dataset.go === name));
  if (name === 'chat') void refreshChatStatus();
  if (name === 'connect') renderConnectPage();
  if (name === 'models') void refreshModelsList();
}

// ====== Boot ======
async function boot(): Promise<void> {
  // 1) 先把 API_BASE 写回 state
  state.apiBase = API_BASE;

  // 2) 拉 Tauri cmd 信息（如果在 Tauri 容器里）
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
    // 浏览器直接打开（开发模式）
    state.apiPort = 8081;
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

  // 5) 初始化各屏
  initHomeScreen();
  initChatScreen();
  initModelsScreen();
  initConnectScreen();

  // 6) 绑定全局屏切换
  $$('[data-go]').forEach((el) => el.addEventListener('click', () => go(el.dataset.go!)));
  $$('[data-back]').forEach((el) => el.addEventListener('click', () => go(el.dataset.back!)));

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

// 启动
void boot();
