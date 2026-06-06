// Williw 前端类型定义
// 跟 williw/api-server/src/main.rs + williw/src-tauri/src/lib.rs 的 schema 保持一致

export interface AppSettings {
  api_port: number;
  api_key: string | null;
  model_dir: string | null;
  default_model: string | null;
  temperature: number;
  max_tokens: number;
  top_p: number;
  system_prompt: string | null;
  allow_external_access: boolean;
  theme: string;
  auto_start: boolean;
  [key: string]: unknown;
}

export interface AppInfo {
  version: string;
  name: string;
  api_port: number;
  api_base: string;
  platform: string;
  started_at_ms: number;
  api_key: string | null;
}

export type EngineState = 'idle' | 'loading' | 'ready' | 'error' | 'generating';

export interface EngineStatus {
  state: EngineState;
  model_id: string | null;
  model_path: string | null;
  context_len: number;
  error: string | null;
  last_prompt_tokens: number | null;
  last_completion_tokens: number | null;
  last_total_ms: number | null;
}

export interface ModelInfo {
  id: string;
  object: 'model';
  created: number;
  owned_by: string;
}

export interface ModelsResponse {
  object: 'list';
  data: ModelInfo[];
}

export type Role = 'system' | 'user' | 'assistant';

export interface ChatMessage {
  role: Role;
  content: string;
}

export interface ChatCompletionsRequest {
  model: string;
  messages: ChatMessage[];
  temperature?: number;
  top_p?: number;
  max_tokens?: number;
  stop?: string[];
  stream?: boolean;
}

export interface ChatCompletionsResponse {
  id: string;
  object: 'chat.completion';
  created: number;
  model: string;
  choices: Array<{
    index: number;
    message: ChatMessage;
    finish_reason: string;
  }>;
  usage: {
    prompt_tokens: number;
    completion_tokens: number;
    total_tokens: number;
  };
}

export interface ApiError {
  error: { message: string; type: string; code?: string | null };
}

export type ScreenName = 'home' | 'chat' | 'models' | 'connect';

// 运行时全局状态
export interface GlobalState {
  info: AppInfo | null;
  settings: AppSettings | null;
  status: EngineStatus | null;
  models: ModelInfo[];
  selectedModel: ModelInfo | null;
  apiKey: string | null;
  apiOn: boolean;
  apiPort: number;
  apiBase: string;
}
