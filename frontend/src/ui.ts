// 通用 UI 工具

export class WilliwApiError extends Error {
  constructor(public readonly status: number, public readonly kind: string, message: string) {
    super(message);
    this.name = 'WilliwApiError';
  }
}

/** 严格版：找不到就 throw。** 慎用** — 元素可能动态生成。
 *  业务屏内建议用 try$() 或 getElementById。 */
export function $<T extends HTMLElement = HTMLElement>(id: string): T {
  const el = document.getElementById(id);
  if (!el) throw new Error(`element #${id} not found`);
  return el as T;
}

/** 安全版：找不到返回 null。** 推荐在屏 init 里用这个。 */
export function try$<T extends Element = Element>(id: string): T | null {
  return document.getElementById(id) as T | null;
}

export function $$(selector: string, root: ParentNode = document): HTMLElement[] {
  return Array.from(root.querySelectorAll<HTMLElement>(selector));
}

export function createEl<K extends keyof HTMLElementTagNameMap>(
  tag: K,
  className?: string,
  text?: string,
): HTMLElementTagNameMap[K] {
  const el = document.createElement(tag);
  if (className) el.className = className;
  if (text !== undefined) el.textContent = text;
  return el;
}

let toastTimer: number | null = null;
export function showToast(msg: string, duration = 1800): void {
  let toast = document.getElementById('__toast') as HTMLDivElement | null;
  if (!toast) {
    toast = document.createElement('div');
    toast.id = '__toast';
    toast.style.cssText = `
      position: fixed; left: 50%; bottom: 80px; transform: translateX(-50%);
      background: rgba(8, 28, 22, 0.92); color: #E6F5EE;
      padding: 10px 18px; border-radius: 999px; font-size: 13px;
      border: 1px solid rgba(143, 220, 188, 0.25);
      box-shadow: 0 4px 16px rgba(0,0,0,0.4);
      z-index: 9999; opacity: 0; transition: opacity 0.2s;
      pointer-events: none; max-width: 80%;
    `;
    document.body.appendChild(toast);
  }
  toast.textContent = msg;
  requestAnimationFrame(() => { if (toast) toast.style.opacity = '1'; });
  if (toastTimer !== null) window.clearTimeout(toastTimer);
  toastTimer = window.setTimeout(() => {
    if (toast) toast.style.opacity = '0';
  }, duration);
}

export function generateKey(): string {
  const arr = new Uint8Array(24);
  (window.crypto ?? (window as unknown as { msCrypto: Crypto }).msCrypto).getRandomValues(arr);
  return Array.from(arr, (b) => b.toString(16).padStart(2, '0')).join('');
}

export function isTauri(): boolean {
  return !!(window as unknown as { __TAURI__?: unknown; __TAURI_INTERNALS__?: unknown }).__TAURI__
      || !!(window as unknown as { __TAURI_INTERNALS__?: unknown }).__TAURI_INTERNALS__;
}

export async function tauri<T>(cmd: string, args?: Record<string, unknown>): Promise<T | null> {
  const w = window as unknown as {
    __TAURI_INTERNALS__?: { invoke: (cmd: string, args?: Record<string, unknown>) => Promise<unknown> };
  };
  if (isTauri() && w.__TAURI_INTERNALS__?.invoke) {
    return (await w.__TAURI_INTERNALS__.invoke(cmd, args ?? {})) as T;
  }
  return null;
}
