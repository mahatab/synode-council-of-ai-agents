import { useMemo } from 'react';
import { create } from 'zustand';
import { PROVIDERS, registerLiveWebSearch } from '../types';
import type { Provider, ProviderInfo } from '../types';
import { mergeModels, mergeProviders, type RemoteModel } from '../lib/modelCatalog';
import * as tauri from '../lib/tauri';

const STORAGE_KEY = 'synode.liveModels.v1';

type LiveModels = Partial<Record<Provider, RemoteModel[]>>;

interface StoredCatalog {
  live: LiveModels;
  fetchedAt: number | null;
}

interface ModelCatalogState extends StoredCatalog {
  refreshing: boolean;
  /** Providers whose last refresh failed, with the reason. */
  errors: Partial<Record<Provider, string>>;

  /** Ask one provider for its current models. Does nothing without an API key. */
  refreshProvider: (provider: Provider) => Promise<void>;
  /** Ask every cloud provider that has an API key. */
  refreshAll: () => Promise<void>;
  /** Forget a provider's live models, for example after its key is removed. */
  clearProvider: (provider: Provider) => void;
}

const CLOUD_PROVIDERS = PROVIDERS.filter((p) => p.id !== 'lmstudio');

function loadStored(): StoredCatalog {
  try {
    const raw = localStorage.getItem(STORAGE_KEY);
    if (raw) {
      const parsed = JSON.parse(raw) as StoredCatalog;
      if (parsed && typeof parsed.live === 'object' && parsed.live !== null) {
        return { live: parsed.live, fetchedAt: parsed.fetchedAt ?? null };
      }
    }
  } catch {
    // A missing or unreadable cache just means starting from the built-in list.
  }
  return { live: {}, fetchedAt: null };
}

function persist(catalog: StoredCatalog) {
  try {
    localStorage.setItem(STORAGE_KEY, JSON.stringify(catalog));
  } catch {
    // The cache is a convenience; the next refresh rebuilds it.
  }
}

/** Tell the web-search check about the extra models a provider reported. */
function registerExtras(provider: Provider, live: RemoteModel[] | undefined) {
  const info = PROVIDERS.find((p) => p.id === provider);
  if (!info) return;
  const known = new Set(info.models.map((m) => m.id));
  registerLiveWebSearch(
    provider,
    mergeModels(provider, info.models, live).filter((m) => !known.has(m.id)),
  );
}

const initial = loadStored();
for (const provider of Object.keys(initial.live) as Provider[]) {
  registerExtras(provider, initial.live[provider]);
}

async function fetchLive(info: ProviderInfo): Promise<RemoteModel[] | null> {
  const apiKey = await tauri.getApiKey(info.keychainService);
  if (!apiKey) return null;
  return tauri.fetchProviderModels(info.id, apiKey);
}

export const useModelCatalogStore = create<ModelCatalogState>((set, get) => ({
  ...initial,
  refreshing: false,
  errors: {},

  refreshProvider: async (provider) => {
    const info = CLOUD_PROVIDERS.find((p) => p.id === provider);
    if (!info) return;
    try {
      const models = await fetchLive(info);
      if (models === null) return;
      registerExtras(provider, models);
      const live = { ...get().live, [provider]: models };
      const errors = { ...get().errors };
      delete errors[provider];
      const fetchedAt = Date.now();
      set({ live, errors, fetchedAt });
      persist({ live, fetchedAt });
    } catch (e) {
      // Keep whatever list was there before; a failed refresh should not hide models.
      set({ errors: { ...get().errors, [provider]: String(e) } });
    }
  },

  refreshAll: async () => {
    if (get().refreshing) return;
    set({ refreshing: true });
    try {
      await Promise.all(CLOUD_PROVIDERS.map((p) => get().refreshProvider(p.id)));
    } finally {
      set({ refreshing: false });
    }
  },

  clearProvider: (provider) => {
    const live = { ...get().live };
    delete live[provider];
    const errors = { ...get().errors };
    delete errors[provider];
    registerExtras(provider, undefined);
    set({ live, errors });
    persist({ live, fetchedAt: get().fetchedAt });
  },
}));

/** Every provider, with the models its API currently reports merged in. */
export function useCatalogProviders(): ProviderInfo[] {
  const live = useModelCatalogStore((s) => s.live);
  return useMemo(() => mergeProviders(live), [live]);
}
