// 模型页：列表 + 下载 + 选中
import { state } from '../state';
import { listModels } from '../api';
import { $, showToast, tauri } from '../ui';
import type { ModelInfo } from '../types';

let installedModels: Array<{ id: string; name: string; path: string; size_bytes?: number; state: string }> = [];

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
  for (const it of installedModels) {
    const li = document.createElement('div');
    li.className = 'mi';
    li.innerHTML = `
      <div class="mi-name">${escapeHtml(it.name || it.id)}</div>
      <div class="mi-meta">${formatSize(it.size_bytes)} · ${escapeHtml(it.path || '')}</div>
      <div class="mi-actions">
        <button class="mi-btn" data-act="use">选用</button>
        <button class="mi-btn mi-btn-del" data-act="del">删除</button>
      </div>
    `;
    li.querySelector<HTMLButtonElement>('[data-act="use"]')?.addEventListener('click', () => {
      void useModel(it);
    });
    list.appendChild(li);
  }
}

async function useModel(it: { id: string; name: string; path: string }): Promise<void> {
  state.selectedModel = { id: it.id, object: 'model', created: 0, owned_by: 'williw' };
  showToast('已选用模型：' + it.name);
  // 通知后端（如果有 cmd_models_select）
  await tauri('cmd_models_select', { id: it.id });
}

function formatSize(n: number | undefined): string {
  if (!n) return '—';
  if (n < 1024 * 1024) return (n / 1024).toFixed(1) + ' KB';
  if (n < 1024 * 1024 * 1024) return (n / 1024 / 1024).toFixed(1) + ' MB';
  return (n / 1024 / 1024 / 1024).toFixed(2) + ' GB';
}

function escapeHtml(s: string): string {
  return s.replace(/[&<>"']/g, (c) => ({ '&': '&amp;', '<': '&lt;', '>': '&gt;', '"': '&quot;', "'": '&#39;' }[c]!));
}

export function initModelsScreen(): void {
  void $;
  void refreshModelsList;
}
