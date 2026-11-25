import { useState, useEffect } from 'react';
import { Info, Eye, EyeOff, Check, Trash2 } from 'lucide-react';
import Button from '../common/Button';
import ApiKeyInfoPopover from './ApiKeyInfoPopover';
import { PROVIDERS } from '../../types';
import type { ProviderInfo } from '../../types';
import * as tauri from '../../lib/tauri';
import { useModelCatalogStore } from '../../stores/modelCatalogStore';

export default function ApiKeyManager() {
  const [keyStates, setKeyStates] = useState<
    Record<string, { hasKey: boolean; visible: boolean; value: string; editing: boolean }>
  >({});
  const [openInfo, setOpenInfo] = useState<string | null>(null);

  useEffect(() => {
    let cancelled = false;
    async function loadKeyStates() {
      const states: Record<string, { hasKey: boolean; visible: boolean; value: string; editing: boolean }> = {};
      for (const provider of PROVIDERS.filter(p => p.id !== 'lmstudio')) {
        const has = await tauri.hasApiKey(provider.keychainService);
        states[provider.id] = { hasKey: has, visible: false, value: '', editing: false };
      }
      if (!cancelled) {
        setKeyStates(states);
      }
    }
    loadKeyStates();
    return () => { cancelled = true; };
  }, []);

  const handleSaveKey = async (provider: ProviderInfo) => {
    const state = keyStates[provider.id];
    if (!state?.value.trim()) return;

    await tauri.saveApiKey(provider.keychainService, state.value.trim());
    void useModelCatalogStore.getState().refreshProvider(provider.id);
    setKeyStates((prev) => ({
      ...prev,
      [provider.id]: { ...prev[provider.id], hasKey: true, editing: false, value: '' },
    }));
  };

  const handleDeleteKey = async (provider: ProviderInfo) => {
    await tauri.deleteApiKey(provider.keychainService);
    useModelCatalogStore.getState().clearProvider(provider.id);
    setKeyStates((prev) => ({
      ...prev,
      [provider.id]: { hasKey: false, visible: false, value: '', editing: false },
    }));
  };

  const handleRevealKey = async (provider: ProviderInfo) => {
    const state = keyStates[provider.id];
    if (state?.visible) {
      setKeyStates((prev) => ({
        ...prev,
        [provider.id]: { ...prev[provider.id], visible: false, value: '' },
      }));
    } else {
      const key = await tauri.getApiKey(provider.keychainService);
      setKeyStates((prev) => ({
        ...prev,
        [provider.id]: { ...prev[provider.id], visible: true, value: key || '' },
      }));
    }
  };

  return (
    <div className="space-y-4">
      <h3 className="text-sm font-semibold text-[var(--color-text-primary)] mb-3">
        API Keys
      </h3>
      <p className="text-xs text-[var(--color-text-tertiary)] mb-4">
        Keys are stored securely in macOS Keychain
      </p>

      {PROVIDERS.filter(p => p.id !== 'lmstudio').map((provider) => {
        const state = keyStates[provider.id];

        return (
          <div
            key={provider.id}
            className="p-3 rounded-[var(--radius-md)] border border-[var(--color-border-primary)] bg-[var(--color-bg-secondary)]"
          >
            <div className="flex items-center justify-between mb-2">
              <div className="flex items-center gap-2">
                <span className="text-sm font-medium text-[var(--color-text-primary)]">
                  {provider.name}
                </span>
                {state?.hasKey && (
                  <span className="flex items-center gap-1 text-xs text-[var(--color-success)]">
                    <Check size={12} /> Configured
                  </span>
                )}
              </div>
              <div className="relative">
                <button
                  onClick={() =>
                    setOpenInfo(openInfo === provider.id ? null : provider.id)
                  }
                  className="p-1.5 rounded-[var(--radius-sm)] text-[var(--color-text-tertiary)] hover:text-[var(--color-accent)] hover:bg-[var(--color-bg-hover)] transition-colors"
                  title="How to get API key"
                >
                  <Info size={14} />
                </button>
                <ApiKeyInfoPopover
                  provider={provider}
                  isOpen={openInfo === provider.id}
                  onClose={() => setOpenInfo(null)}
                />
              </div>
            </div>

            {state?.hasKey && !state.editing ? (
              <div className="flex items-center gap-2">
                <div className="flex-1 px-3 py-1.5 text-sm text-[var(--color-text-tertiary)] bg-[var(--color-bg-input)] border border-[var(--color-border-primary)] rounded-[var(--radius-sm)] font-mono">
                  {state.visible ? state.value : '\u2022\u2022\u2022\u2022\u2022\u2022\u2022\u2022\u2022\u2022\u2022\u2022'}
                </div>
                <button
                  onClick={() => handleRevealKey(provider)}
                  className="p-1.5 text-[var(--color-text-tertiary)] hover:text-[var(--color-text-primary)]"
                >
                  {state.visible ? <EyeOff size={14} /> : <Eye size={14} />}
                </button>
                <button
                  onClick={() =>
                    setKeyStates((prev) => ({
                      ...prev,
                      [provider.id]: { ...prev[provider.id], editing: true, value: '' },
                    }))
                  }
                  className="text-xs text-[var(--color-accent)] hover:underline"
                >
                  Change
                </button>
                <button
                  onClick={() => handleDeleteKey(provider)}
                  className="p-1.5 text-[var(--color-text-tertiary)] hover:text-[var(--color-error)]"
                >
                  <Trash2 size={14} />
                </button>
              </div>
            ) : (
              <div className="flex items-center gap-2">
                <input
                  type="password"
                  value={state?.value || ''}
                  onChange={(e) =>
                    setKeyStates((prev) => ({
                      ...prev,
                      [provider.id]: { ...prev[provider.id], value: e.target.value },
                    }))
                  }
                  placeholder={`Enter ${provider.name} API key`}
                  className="flex-1 px-3 py-1.5 text-sm bg-[var(--color-bg-input)] border border-[var(--color-border-primary)] rounded-[var(--radius-sm)] text-[var(--color-text-primary)] placeholder:text-[var(--color-text-tertiary)] focus:outline-none focus:ring-1 focus:ring-[var(--color-border-focus)] font-mono"
                />
                <Button
                  size="sm"
                  onClick={() => handleSaveKey(provider)}
                  disabled={!state?.value?.trim()}
                >
                  Save
                </Button>
                {state?.editing && (
                  <Button
                    size="sm"
                    variant="ghost"
                    onClick={() =>
                      setKeyStates((prev) => ({
                        ...prev,
                        [provider.id]: { ...prev[provider.id], editing: false, value: '' },
                      }))
                    }
                  >
                    Cancel
                  </Button>
                )}
              </div>
            )}
          </div>
        );
      })}
    </div>
  );
}
