// 聊天页：直接 fetch /v1/chat/completions 测试本地模型
import { state } from '../state';
import { API_BASE, chatCompletions, getStatus } from '../api';
import type { ChatMessage, EngineStatus } from '../types';
import { $, showToast, WilliwApiError } from '../ui';

const chatMsgs = $('chat-messages');
const chatEmpty = $('chat-empty');
const chatInput = $('chat-input') as HTMLTextAreaElement;
const chatSend = $('chat-send') as HTMLButtonElement;
const chatStatusPill = $('chat-status-pill');
const chatStatusText = $('chat-status-text');

const chatHistory: ChatMessage[] = [];
const MAX_HISTORY = 12;

let busy = false;

function appendMessage(role: 'user' | 'assistant' | 'system', text: string, meta?: string): HTMLDivElement {
  if (chatEmpty && !chatEmpty.hidden) chatEmpty.hidden = true;
  const wrap = document.createElement('div');
  wrap.className = 'msg ' + role;
  const roleEl = document.createElement('div');
  roleEl.className = 'msg-role';
  roleEl.textContent = role === 'user' ? '你' : role === 'assistant' ? '本机模型' : '系统';
  const bubble = document.createElement('div');
  bubble.className = 'msg-bubble';
  bubble.textContent = text;
  wrap.appendChild(roleEl);
  wrap.appendChild(bubble);
  if (meta) {
    const metaEl = document.createElement('div');
    metaEl.className = 'msg-meta';
    metaEl.textContent = meta;
    wrap.appendChild(metaEl);
  }
  chatMsgs.appendChild(wrap);
  chatMsgs.scrollTop = chatMsgs.scrollHeight;
  return bubble;
}

function updateChatStatusPill(): void {
  const st: EngineStatus['state'] | undefined = state.status?.state;
  if (state.apiOn && st === 'ready') {
    chatStatusPill.dataset.state = 'ready';
    chatStatusText.textContent = '运行中';
  } else if (state.apiOn && st === 'loading') {
    chatStatusPill.dataset.state = 'loading';
    chatStatusText.textContent = '加载中';
  } else if (state.apiOn && st === 'error') {
    chatStatusPill.dataset.state = 'error';
    chatStatusText.textContent = '错误';
  } else {
    chatStatusPill.dataset.state = 'offline';
    chatStatusText.textContent = '未开启';
  }
}

export async function refreshChatStatus(): Promise<void> {
  try {
    const j = await getStatus();
    state.status = j;
    updateChatStatusPill();
  } catch { /* 网络错误，保留上次状态 */ }
}

function autoSizeChatInput(): void {
  chatInput.style.height = 'auto';
  chatInput.style.height = Math.min(chatInput.scrollHeight, 120) + 'px';
}

async function sendChatMessage(): Promise<void> {
  if (busy) return;
  const text = chatInput.value.trim();
  if (!text) return;
  if (!state.apiOn) {
    appendMessage('system', '算力服务未开启。请先在主页点 "开启" 按钮。');
    chatInput.value = '';
    autoSizeChatInput();
    return;
  }

  chatInput.value = '';
  autoSizeChatInput();
  appendMessage('user', text);
  chatHistory.push({ role: 'user', content: text });
  if (chatHistory.length > MAX_HISTORY) chatHistory.splice(0, chatHistory.length - MAX_HISTORY);

  const bubble = appendMessage('assistant', '', '生成中…');
  const wrap = bubble.parentElement as HTMLDivElement;
  wrap.classList.add('streaming');
  bubble.textContent = '';
  busy = true;
  chatSend.disabled = true;
  chatSend.classList.add('busy');
  chatInput.disabled = true;

  const t0 = performance.now();
  const sysMsg: ChatMessage = {
    role: 'system',
    content: '你是 Williw，一个运行在用户设备上的本地 AI 助手。回答简洁。',
  };
  const body = {
    model: 'williw-local',
    messages: [sysMsg, ...chatHistory],
    temperature: 0.5,
    max_tokens: 200,
    stream: false,
  };

  try {
    const j = await chatCompletions(body, state.apiKey);
    const answer = j.choices?.[0]?.message?.content ?? '(空响应)';
    const dt = ((performance.now() - t0) / 1000).toFixed(2);
    const usage = j.usage ?? { prompt_tokens: 0, completion_tokens: 0, total_tokens: 0 };
    const meta =
      (usage.completion_tokens ? usage.completion_tokens + ' tokens · ' : '') +
      dt + 's · ' +
      (usage.total_tokens ? usage.total_tokens + ' tok total' : '');
    bubble.textContent = answer;
    wrap.classList.remove('streaming');
    const metaEl = wrap.querySelector<HTMLDivElement>('.msg-meta');
    if (metaEl) metaEl.textContent = meta;
    chatHistory.push({ role: 'assistant', content: answer });
    if (chatHistory.length > MAX_HISTORY) chatHistory.splice(0, chatHistory.length - MAX_HISTORY);
  } catch (e) {
    const msg = e instanceof WilliwApiError
      ? `请求失败 (${e.status}): ${e.message}`
      : '网络错误: ' + (e instanceof Error ? e.message : String(e));
    bubble.textContent = msg;
    wrap.classList.remove('streaming');
    const metaEl = wrap.querySelector<HTMLDivElement>('.msg-meta');
    if (metaEl) metaEl.textContent = '错误';
  } finally {
    busy = false;
    chatSend.disabled = false;
    chatSend.classList.remove('busy');
    chatInput.disabled = false;
    chatInput.focus();
  }
}

export function initChatScreen(): void {
  // 暴露给 window 方便 chat 页 cta 按钮调用
  void API_BASE;
  void showToast;
  chatSend.addEventListener('click', sendChatMessage);
  chatInput.addEventListener('keydown', (e: KeyboardEvent) => {
    if (e.key === 'Enter' && !e.shiftKey) {
      e.preventDefault();
      void sendChatMessage();
    }
  });
  chatInput.addEventListener('input', autoSizeChatInput);

  // 切到 chat 屏时拉一次状态
  document.querySelectorAll<HTMLElement>('[data-go="chat"]').forEach((b) => {
    b.addEventListener('click', () => {
      void refreshChatStatus();
    });
  });
}
