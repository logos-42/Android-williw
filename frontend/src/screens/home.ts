// 主页：开关 + 状态
import { state } from '../state';
import { $, $$, showToast, tauri } from '../ui';

export function initHomeScreen(): void {
  const powerBtn = $('power-btn');
  const powerLabel = $('power-label');
  const ringFg = $('ring-fg');
  const stateText = $('state-text');
  const addrText = $('addr-text');
  const heroModel = $('hero-model');
  const modelPill = $('model-pill');
  const modelPillText = $('model-pill-text');
  const tipCard = $('tip-card') ?? document.querySelector<HTMLElement>('.tip');

  function applyPowerUI(): void {
    if (state.apiOn) {
      powerBtn.classList.add('on');
      powerLabel.textContent = '关闭';
      ringFg.classList.add('on');
      stateText.textContent =
        state.status && state.status.state === 'ready' ? '算力服务运行中' : '正在启动…';
      addrText.textContent = state.apiBase;
    } else {
      powerBtn.classList.remove('on');
      powerLabel.textContent = '开启';
      ringFg.classList.remove('on');
      stateText.textContent = '已关闭';
      addrText.textContent = '—';
    }
  }

  function applyModelPill(): void {
    const m = state.selectedModel;
    if (!m || !m.id) {
      modelPill.dataset.state = 'offline';
      modelPillText.textContent = '未加载模型';
      heroModel.textContent = '—';
      return;
    }
    const loaded = state.status?.state === 'ready';
    modelPill.dataset.state = loaded ? 'ready' : (state.status?.state ?? 'offline');
    modelPillText.textContent = loaded ? m.id : (m.id + ' · 未就绪');
    heroModel.textContent = m.id;
  }

  powerBtn.addEventListener('click', async () => {
    if (state.apiOn) {
      await tauri('cmd_api_set_enabled', { enabled: false });
      state.apiOn = false;
      applyPowerUI();
      showToast('已关闭算力服务');
    } else {
      if (!state.selectedModel || !state.selectedModel.id) {
        showToast('未加载模型，API 将以 mock 兜底返回', 2400);
      }
      if (!state.apiKey) {
        const k = await import('../ui').then((m) => m.generateKey());
        await tauri('cmd_settings_set_api_key', { apiKey: k });
        state.apiKey = k;
      }
      await tauri('cmd_api_set_enabled', { enabled: true });
      state.apiOn = true;
      applyPowerUI();
      showToast('算力服务已开启');
    }
  });

  // 把 apply 函数挂到 state 方便其他屏调
  (state as unknown as { _applyPowerUI?: () => void })._applyPowerUI = applyPowerUI;
  (state as unknown as { _applyModelPill?: () => void })._applyModelPill = applyModelPill;

  // tip card（如果不存在就不处理）
  void tipCard;

  // 初始渲染
  applyPowerUI();
  applyModelPill();

  // tab/back 按钮的 data-go / data-back 在 main.ts 里统一绑
  void $$;
}
