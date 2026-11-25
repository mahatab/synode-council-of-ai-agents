export type Provider = 'anthropic' | 'openai' | 'google' | 'xai' | 'deepseek' | 'mistral' | 'together' | 'cohere' | 'lmstudio';

export interface ModelConfig {
  provider: Provider;
  model: string;
  displayName: string;
  order: number;
}

export interface MasterModelConfig {
  provider: Provider;
  model: string;
}

export type SystemPromptMode = 'upfront' | 'dynamic';

export type DiscussionDepth = 'thorough' | 'concise';

export type DiscussionStyle = 'sequential' | 'independent';

export type ThemeMode = 'light' | 'dark' | 'system';

export type CursorStyle = 'ripple' | 'breathing' | 'orbit' | 'multi';

export interface AppSettings {
  councilModels: ModelConfig[];
  masterModel: MasterModelConfig;
  systemPromptMode: SystemPromptMode;
  discussionDepth: DiscussionDepth;
  discussionStyle: DiscussionStyle;
  theme: ThemeMode;
  cursorStyle: CursorStyle;
  sessionSavePath: string | null;
  setupCompleted: boolean;
  telegramEnabled: boolean;
  internetAccessEnabled: boolean;
  lmStudioBaseUrl?: string;
}

export interface ChatMessage {
  role: string;
  content: string;
}

export interface UsageData {
  inputTokens: number;
  outputTokens: number;
}

export interface ClarifyingExchange {
  question: string;
  answer: string;
}

export interface DiscussionEntryUser {
  role: 'user';
  content: string;
}

export interface DiscussionEntryModel {
  role: 'model';
  provider: string;
  model: string;
  displayName: string;
  systemPrompt?: string;
  content: string;
  clarifyingExchange?: ClarifyingExchange[];
  usage?: UsageData;
}

export interface DiscussionEntryMasterVerdict {
  role: 'master_verdict';
  provider: string;
  model: string;
  content: string;
  usage?: UsageData;
}

export interface DiscussionEntryFollowUpQuestion {
  role: 'follow_up_question';
  content: string;
  targetProvider: string;
  targetModel: string;
  targetDisplayName: string;
}

export interface DiscussionEntryFollowUpAnswer {
  role: 'follow_up_answer';
  provider: string;
  model: string;
  displayName: string;
  content: string;
  usage?: UsageData;
}

export type DiscussionEntry =
  | DiscussionEntryUser
  | DiscussionEntryModel
  | DiscussionEntryMasterVerdict
  | DiscussionEntryFollowUpQuestion
  | DiscussionEntryFollowUpAnswer;

export type SessionType = 'council' | 'direct_chat';

export type AppMode = 'council' | 'direct_chat';

export type DirectChatState = 'idle' | 'streaming' | 'error';

export interface DirectChatMessage {
  role: 'user' | 'assistant';
  content: string;
  timestamp: string;
  usage?: UsageData;
}

export interface DirectChatAgent {
  provider: string;
  model: string;
  displayName: string;
}

export interface CouncilConfig {
  models: ModelConfig[];
  masterModel: MasterModelConfig;
  systemPromptMode: SystemPromptMode;
}

export interface Session {
  id: string;
  title: string;
  createdAt: string;
  updatedAt: string;
  userQuestion: string;
  councilConfig: CouncilConfig;
  discussion: DiscussionEntry[];
  sessionType?: SessionType;
  directChatAgent?: DirectChatAgent;
  directChatMessages?: DirectChatMessage[];
}

export interface SessionSummary {
  id: string;
  title: string;
  createdAt: string;
  updatedAt: string;
  sessionType?: SessionType;
}

export interface StreamToken {
  streamId: string;
  token: string;
  done: boolean;
  error?: string;
  usage?: UsageData;
}

export interface StreamChatResult {
  content: string;
  usage?: UsageData;
}

export type CouncilState =
  | 'idle'
  | 'user_input'
  | 'generating_system_prompts'
  | 'model_turn'
  | 'clarifying_qa'
  | 'master_verdict'
  | 'complete'
  | 'follow_up'
  | 'error';

export interface ModelInfo {
  id: string;
  name: string;
  webSearch?: boolean;
}

export interface ProviderInfo {
  id: Provider;
  name: string;
  keychainService: string;
  models: ModelInfo[];
  apiKeyUrl: string;
  apiKeySteps: string[];
}

export const WEB_SEARCH_PROVIDERS: Provider[] = ['anthropic', 'openai', 'google', 'xai'];

/**
 * Best guess at web search support for a model that is not in the built-in list,
 * from what each provider's current model families support.
 */
export function inferWebSearch(provider: string, modelId: string): boolean {
  const majorVersion = (prefix: string) => {
    const match = modelId.startsWith(prefix) ? /^\d+/.exec(modelId.slice(prefix.length)) : null;
    return match ? Number(match[0]) : 0;
  };
  switch (provider) {
    case 'anthropic':
    case 'google':
      return true;
    case 'openai':
      return majorVersion('gpt-') >= 5;
    case 'xai':
      return majorVersion('grok-') >= 4;
    default:
      return false;
  }
}

// Web search support for models that came from a provider's live model list, keyed
// by "provider:model". The model catalog store fills this in.
const liveWebSearch = new Map<string, boolean>();

export function registerLiveWebSearch(provider: string, models: ModelInfo[]): void {
  for (const key of [...liveWebSearch.keys()]) {
    if (key.startsWith(`${provider}:`)) liveWebSearch.delete(key);
  }
  for (const model of models) {
    liveWebSearch.set(`${provider}:${model.id}`, model.webSearch === true);
  }
}

/** Check if a specific model supports web search */
export function modelSupportsWebSearch(provider: string, modelId: string): boolean {
  const providerInfo = PROVIDERS.find((p) => p.id === provider);
  if (!providerInfo) return false;
  const model = providerInfo.models.find((m) => m.id === modelId);
  if (model) return model.webSearch === true;
  const live = liveWebSearch.get(`${provider}:${modelId}`);
  if (live !== undefined) return live;
  return WEB_SEARCH_PROVIDERS.includes(provider as Provider) && inferWebSearch(provider, modelId);
}

export const PROVIDERS: ProviderInfo[] = [
  {
    id: 'anthropic',
    name: 'Anthropic',
    keychainService: 'com.council-of-ai-agents.anthropic',
    models: [
      { id: 'claude-fable-5-2', name: 'Claude Fable 5.2', webSearch: true },
      { id: 'claude-opus-5-5', name: 'Claude Opus 5.5', webSearch: true },
      { id: 'claude-sonnet-5-5', name: 'Claude Sonnet 5.5', webSearch: true },
      { id: 'claude-haiku-5-5', name: 'Claude Haiku 5.5', webSearch: true },
    ],
    apiKeyUrl: 'https://console.anthropic.com/settings/keys',
    apiKeySteps: [
      'Go to console.anthropic.com',
      'Sign in or create an account',
      'Navigate to Settings > API Keys',
      'Click "Create Key" and give it a name',
      'Copy the key (it starts with "sk-ant-")',
    ],
  },
  {
    id: 'openai',
    name: 'OpenAI',
    keychainService: 'com.council-of-ai-agents.openai',
    models: [
      { id: 'gpt-6.2-astra', name: 'GPT-6.2 Astra', webSearch: true },
      { id: 'gpt-6.2-sol', name: 'GPT-6.2 Sol', webSearch: true },
      { id: 'gpt-6.1-terra', name: 'GPT-6.1 Terra', webSearch: true },
      { id: 'gpt-6-luna', name: 'GPT-6 Luna', webSearch: true },
      { id: 'gpt-5.6-sol', name: 'GPT-5.6 Sol' },
      { id: 'gpt-5.2', name: 'GPT-5.2', webSearch: true },
    ],
    apiKeyUrl: 'https://platform.openai.com/api-keys',
    apiKeySteps: [
      'Go to platform.openai.com',
      'Sign in or create an account',
      'Navigate to API Keys in the sidebar',
      'Click "Create new secret key"',
      'Copy the key (it starts with "sk-")',
    ],
  },
  {
    id: 'google',
    name: 'Google',
    keychainService: 'com.council-of-ai-agents.google',
    models: [
      { id: 'gemini-4-carbon', name: 'Gemini 4 Carbon', webSearch: true },
      { id: 'gemini-4-neon', name: 'Gemini 4 Neon', webSearch: true },
      { id: 'gemini-4-hydrogen', name: 'Gemini 4 Hydrogen', webSearch: true },
      { id: 'gemini-3.8-flash', name: 'Gemini 3.8 Flash', webSearch: true },
      { id: 'gemini-3.5-flash-lite', name: 'Gemini 3.5 Flash-Lite', webSearch: true },
    ],
    apiKeyUrl: 'https://aistudio.google.com/apikey',
    apiKeySteps: [
      'Go to aistudio.google.com',
      'Sign in with your Google account',
      'Click "Get API Key" in the top bar',
      'Click "Create API Key"',
      'Select or create a Google Cloud project',
      'Copy the generated API key',
    ],
  },
  {
    id: 'xai',
    name: 'xAI',
    keychainService: 'com.council-of-ai-agents.xai',
    models: [
      { id: 'grok-4.7', name: 'Grok 4.7', webSearch: true },
      { id: 'grok-4.6', name: 'Grok 4.6', webSearch: true },
      { id: 'grok-4.3', name: 'Grok 4.3', webSearch: true },
    ],
    apiKeyUrl: 'https://console.x.ai',
    apiKeySteps: [
      'Go to console.x.ai',
      'Sign in with your X (Twitter) account',
      'Navigate to API Keys section',
      'Click "Create API Key"',
      'Copy the generated key',
    ],
  },
  {
    id: 'deepseek',
    name: 'DeepSeek',
    keychainService: 'com.council-of-ai-agents.deepseek',
    models: [
      { id: 'deepseek-v4-pro', name: 'DeepSeek V4 Pro' },
      { id: 'deepseek-flash', name: 'DeepSeek V4.1 Flash' },
    ],
    apiKeyUrl: 'https://platform.deepseek.com/api_keys',
    apiKeySteps: [
      'Go to platform.deepseek.com',
      'Sign in or create an account',
      'Navigate to API Keys section',
      'Click "Create new API key"',
      'Copy the generated key (it starts with "sk-")',
    ],
  },
  {
    id: 'mistral',
    name: 'Mistral',
    keychainService: 'com.council-of-ai-agents.mistral',
    models: [
      { id: 'mistral-large-latest', name: 'Mistral Large' },
      { id: 'mistral-small-latest', name: 'Mistral Small' },
      { id: 'codestral-latest', name: 'Codestral' },
    ],
    apiKeyUrl: 'https://console.mistral.ai/api-keys',
    apiKeySteps: [
      'Go to console.mistral.ai',
      'Sign in or create an account',
      'Navigate to API Keys section',
      'Click "Create new key"',
      'Copy the generated API key',
    ],
  },
  {
    id: 'together',
    name: 'Together AI',
    keychainService: 'com.council-of-ai-agents.together',
    models: [
      { id: 'moonshotai/Kimi-K3.1-Pro', name: 'Kimi K3.1 Pro' },
      { id: 'zai-org/GLM-5.3', name: 'GLM-5.3' },
      { id: 'Qwen/Qwen4-Max', name: 'Qwen4 Max' },
      { id: 'deepseek-ai/DeepSeek-V4.1-Flash-1017', name: 'DeepSeek V4.1 Flash' },
      { id: 'openai/gpt-oss-120b', name: 'GPT-OSS 120B' },
    ],
    apiKeyUrl: 'https://api.together.xyz/settings/api-keys',
    apiKeySteps: [
      'Go to api.together.xyz',
      'Sign in or create an account',
      'Navigate to Settings > API Keys',
      'Copy your API key',
    ],
  },
  {
    id: 'cohere',
    name: 'Cohere',
    keychainService: 'com.council-of-ai-agents.cohere',
    models: [
      { id: 'command-a-plus-05-2026', name: 'Command A+' },
      { id: 'command-a-03-2025', name: 'Command A' },
      { id: 'command-r-plus-08-2024', name: 'Command R+' },
    ],
    apiKeyUrl: 'https://dashboard.cohere.com/api-keys',
    apiKeySteps: [
      'Go to dashboard.cohere.com',
      'Sign in or create an account',
      'Navigate to API Keys section',
      'Click "Create Trial Key" or "Create Production Key"',
      'Copy the generated API key',
    ],
  },
  {
    id: 'lmstudio',
    name: 'LM Studio',
    keychainService: 'com.council-of-ai-agents.lmstudio',
    models: [],
    apiKeyUrl: 'https://lmstudio.ai',
    apiKeySteps: [
      'Download and install LM Studio from lmstudio.ai',
      'Open LM Studio and download a model',
      'Load the model and start the local server (default port: 1234)',
      'No API key is required',
    ],
  },
];

export function getProviderInfo(providerId: Provider): ProviderInfo {
  return PROVIDERS.find((p) => p.id === providerId)!;
}

export function getProviderColor(provider: Provider): string {
  switch (provider) {
    case 'anthropic':
      return '#D97757';
    case 'openai':
      return '#10A37F';
    case 'google':
      return '#4285F4';
    case 'xai':
      return '#1DA1F2';
    case 'deepseek':
      return '#536AF6';
    case 'mistral':
      return '#FF7000';
    case 'together':
      return '#6366F1';
    case 'cohere':
      return '#39594D';
    case 'lmstudio':
      return '#00A67E';
  }
}
