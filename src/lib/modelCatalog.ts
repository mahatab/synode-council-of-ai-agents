import { PROVIDERS, inferWebSearch } from '../types';
import type { ModelInfo, Provider, ProviderInfo } from '../types';

/** A chat model as a provider's "list models" endpoint reports it. */
export interface RemoteModel {
  id: string;
  name: string;
  created: number | null;
  webSearch: boolean | null;
}

const UPPERCASE_WORDS = new Set(['gpt', 'glm', 'oss', 'ai']);

/** Turn a bare model id such as "gpt-6.2-astra" into a readable name. */
export function prettifyModelId(id: string): string {
  const base = id.includes('/') ? id.slice(id.lastIndexOf('/') + 1) : id;
  const parts = base.split(/[-_]/).filter(Boolean);
  // "mistral-large-latest" reads fine as "Mistral Large", but "chat-latest"
  // would be left as just "Chat", so a one-word name keeps its "Latest".
  const kept = parts.filter((word) => word !== 'latest');
  const words = (kept.length > 1 ? kept : parts).map((word) =>
    UPPERCASE_WORDS.has(word.toLowerCase())
      ? word.toUpperCase()
      : word.charAt(0).toUpperCase() + word.slice(1),
  );
  // Match the built-in names: "GPT-6.2 Sol", not "GPT 6.2 Sol".
  return words.join(' ').replace(/^GPT (?=\d)/, 'GPT-') || id;
}

/**
 * Combine the built-in models for a provider with the ones the provider reports.
 * Built-in models come first, in their curated order and with their own names.
 * Anything else the provider offers follows, in the provider's order (newest first).
 */
export function mergeModels(
  provider: Provider,
  builtIn: ModelInfo[],
  live: RemoteModel[] | undefined,
): ModelInfo[] {
  if (!live || live.length === 0) return builtIn;

  const known = new Set(builtIn.map((m) => m.id));
  const extras: ModelInfo[] = live
    .filter((m) => !known.has(m.id))
    .map((m) => ({
      id: m.id,
      name: m.name && m.name !== m.id ? m.name : prettifyModelId(m.id),
      webSearch: m.webSearch ?? inferWebSearch(provider, m.id),
    }));

  return [...builtIn, ...extras];
}

/** The provider list with each provider's live models merged in. */
export function mergeProviders(
  live: Partial<Record<Provider, RemoteModel[]>>,
): ProviderInfo[] {
  return PROVIDERS.map((p) => ({ ...p, models: mergeModels(p.id, p.models, live[p.id]) }));
}
