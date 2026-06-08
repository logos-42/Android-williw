// 模型页：列表 + 下载 + 选中
import { state } from '../state';
import { listModels } from '../api';
import { showToast, tauri } from '../ui';
import type { ModelInfo } from '../types';

export async function refreshModelsList(): Promise<void> {
  try {
    const r = await listModels();
    state.models = r.data;
    if (!state.selectedModel && state.models.length > 0) {
      state.selectedModel = state.models[0]!;
    }
    renderModels();
  } catch (e) {
    showToast('加载模型列表失败: ' + (e instanceof Error ? e.message : String(e)));
  }
}

function renderModels(): void {
  const list = document.getElementById('model-list');
  if (!list) return;
  list.innerHTML = '';
  if (state.models.length === 0) {
    list.innerHTML = '<li class="empty-list">暂无模型，请下载</li>';
    return;
  }
  for (const m of state.models) {
    const li = document.createElement('li');
    li.className = 'mi';
    li.innerHTML = `
      <div class="mi-name">${escapeHtml(m.id)}</div>
      <div class="mi-meta">${escapeHtml(m.owned_by)}</div>
      <div class="mi-actions">
        <button class="mi-btn" data-act="use">选用</button>
      </div>
    `;
    li.querySelector<HTMLButtonElement>('[data-act="use"]')?.addEventListener('click', () => {
      void useModel(m);
    });
    list.appendChild(li);
  }
}

async function useModel(m: ModelInfo): Promise<void> {
  state.selectedModel = m;
  showToast('已选用模型：' + m.id);
  // 通知后端（如果有 cmd_models_select）
  await tauri('cmd_models_select', { id: m.id });
}

function escapeHtml(s: string): string {
  return s.replace(/[&<>"']/g, (c) => ({ '&': '&amp;', '<': '&lt;', '>': '&gt;', '"': '&quot;', "'": '&#39;' }[c]!));
}

export function initModelsScreen(): void {
  // 切到 models 屏时拉一次列表
  document.querySelectorAll<HTMLElement>('[data-go="models"]').forEach((b) => {
    b.addEventListener('click', () => { void refreshModelsList(); });
  });
}
