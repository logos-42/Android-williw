// Williw API 客户端
// 通过 window.__WILLIW_API_BASE__（Tauri 注入）或 fallback 127.0.0.1:8081
// 兼容 OpenAI Chat Completions + 自定义 /v1/status

import type {
  EngineStatus,
  ModelsResponse,
  ChatCompletionsRequest,
  ChatCompletionsResponse,
  ApiError,
} from './types';

declare global {
  interface Window {
    __WILLIW_API_BASE__?: string;
    __WILLIW_API_PORT__?: number;
  }
}

export const API_BASE: string = (
  (typeof window !== 'undefined' && window.__WILLIW_API_BASE__) ||
  `${location.protocol}//${location.hostname}:${window.__WILLIW_API_PORT__ ?? 8081}`
).replace(/\/+$/, '');

export class WilliwApiError extends Error {
  constructor(public status: number, public kind: string, message: string) {
    super(message);
  }
}

async function request<T>(path: string, init: RequestInit = {}): Promise<T> {
  const r = await fetch(API_BASE + path, {
    ...init,
    headers: { 'content-type': 'application/json', ...(init.headers ?? {}) },
  });
  if (!r.ok) {
    let kind = 'http_error';
    let msg = r.statusText;
    try {
      const body = (await r.json()) as ApiError;
      kind = body.error?.type ?? kind;
      msg = body.error?.message ?? msg;
    } catch {
      msg = (await r.text().catch(() => r.statusText)) || msg;
    }
    throw new WilliwApiError(r.status, kind, msg);
  }
  return (await r.json()) as T;
}

export function getStatus(): Promise<EngineStatus> {
  return request<EngineStatus>('/v1/status', { cache: 'no-store' });
}

export function listModels(): Promise<ModelsResponse> {
  return request<ModelsResponse>('/v1/models');
}

export function chatCompletions(
  body: ChatCompletionsRequest,
  apiKey: string | null,
): Promise<ChatCompletionsResponse> {
  const headers: Record<string, string> = {};
  if (apiKey) headers['Authorization'] = 'Bearer ' + apiKey;
  return request<ChatCompletionsResponse>('/v1/chat/completions', {
    method: 'POST',
    headers,
    body: JSON.stringify(body),
  });
}
