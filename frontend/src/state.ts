// 全局 state（单例）
import type { GlobalState } from './types';

export const state: GlobalState = {
  info: null,
  settings: null,
  status: null,
  models: [],
  selectedModel: null,
  apiKey: null,
  apiOn: false,
  apiPort: 8081,
  apiBase: '',
};
