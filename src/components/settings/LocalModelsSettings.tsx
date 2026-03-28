import { useState, useEffect } from 'react';
import { Monitor, RefreshCw, Plus, Trash2, ExternalLink, CheckCircle2, Circle, Server } from 'lucide-react';
import Button from '../common/Button';
import { useSettingsStore } from '../../stores/settingsStore';
import { getProviderColor } from '../../types';
import type { ModelConfig } from '../../types';
import * as tauri from '../../lib/tauri';

type ServerStatus = 'checking' | 'online' | 'offline';

const SETUP_STEPS = [
  {
    title: 'Install LM Studio',
    description: 'Download and install LM Studio from the official website.',
    link: 'https://lmstudio.ai',
    linkText: 'lmstudio.ai',
  },
  {
    title: 'Download a model',
    description: 'Open LM Studio, go to the Model Search tab, and download a model (e.g., Llama, Mistral, DeepSeek, Qwen).',
  },
  {
    title: 'Load the model',
    description: 'Select and load the downloaded model in LM Studio so it is ready for inference.',
  },
  {
    title: 'Start the local server',
    description: 'Go to the Developer tab in LM Studio and turn on the Status toggle to start the server.',
  },
];

export default function LocalModelsSettings() {
  const { settings, updateSettings } = useSettingsStore();
  const [serverStatus, setServerStatus] = useState<ServerStatus>('checking');
  const [availableModels, setAvailableModels] = useState<{ id: string }[]>([]);
  const [loading, setLoading] = useState(false);
  const [error, setError] = useState<string | null>(null);
  const [showAddModel, setShowAddModel] = useState(false);
  const [selectedModel, setSelectedModel] = useState('');
  const [baseUrl, setBaseUrl] = useState(settings.lmStudioBaseUrl || '');

  const color = getProviderColor('lmstudio');

  const localModels = settings.councilModels.filter(m => m.provider === 'lmstudio');

  const checkServerAndFetchModels = async () => {
    setLoading(true);
    setError(null);
    try {
      const url = baseUrl.trim() || undefined;
      const models = await tauri.fetchLMStudioModels(url);
      setAvailableModels(models);
      setServerStatus('online');
    } catch {
      setAvailableModels([]);
      setServerStatus('offline');
      setError('Could not connect to LM Studio server.');
    } finally {
      setLoading(false);
    }
  };

  useEffect(() => {
    checkServerAndFetchModels();
  }, []);

  const handleAddModel = () => {
    if (!selectedModel) return;

    const exists = settings.councilModels.some(
      (m) => m.provider === 'lmstudio' && m.model === selectedModel,
    );
    if (exists) return;

    const newModel: ModelConfig = {
      provider: 'lmstudio',
      model: selectedModel,
      displayName: selectedModel,
      order: settings.councilModels.length + 1,
    };

    updateSettings({ councilModels: [...settings.councilModels, newModel] });
    setShowAddModel(false);
    setSelectedModel('');
  };

  const handleRemoveModel = (modelId: string) => {
    const updated = settings.councilModels
      .filter((m) => !(m.provider === 'lmstudio' && m.model === modelId))
      .map((m, i) => ({ ...m, order: i + 1 }));
    updateSettings({ councilModels: updated });
  };

  const handleSaveBaseUrl = () => {
    const url = baseUrl.trim() || undefined;
    updateSettings({ lmStudioBaseUrl: url });
    checkServerAndFetchModels();
  };

  // Filter out embedding models
  const chatModels = availableModels.filter(m => !m.id.includes('embed'));

  return (
    <div className="space-y-6">
      {/* Header */}
      <div>
        <div className="flex items-center gap-2 mb-1">
          <Monitor size={16} style={{ color }} />
          <h3 className="text-sm font-semibold text-[var(--color-text-primary)]">
            Local Models
          </h3>
          <span className="text-xs px-2 py-0.5 rounded-full font-medium bg-emerald-100 text-emerald-700 dark:bg-emerald-900/30 dark:text-emerald-400">
            LM Studio
          </span>
        </div>
        <p className="text-xs text-[var(--color-text-tertiary)]">
          Run AI models locally on your machine using LM Studio. Free, private, and offline.
        </p>
      </div>

      {/* Server Status */}
      <div className="p-3 rounded-[var(--radius-md)] border border-[var(--color-border-primary)] bg-[var(--color-bg-secondary)]">
        <div className="flex items-center justify-between">
          <div className="flex items-center gap-2">
            <Server size={14} className="text-[var(--color-text-tertiary)]" />
            <span className="text-sm font-medium text-[var(--color-text-primary)]">Server Status</span>
            {serverStatus === 'checking' ? (
              <span className="text-xs text-[var(--color-text-tertiary)]">Checking...</span>
            ) : serverStatus === 'online' ? (
              <span className="flex items-center gap-1 text-xs text-emerald-600 dark:text-emerald-400">
                <span className="w-1.5 h-1.5 rounded-full bg-emerald-500 inline-block" />
                Online
              </span>
            ) : (
              <span className="flex items-center gap-1 text-xs text-[var(--color-error)]">
                <span className="w-1.5 h-1.5 rounded-full bg-[var(--color-error)] inline-block" />
                Offline
              </span>
            )}
          </div>
          <button
            onClick={checkServerAndFetchModels}
            disabled={loading}
            className="p-1.5 text-[var(--color-text-tertiary)] hover:text-[var(--color-accent)] transition-colors"
            title="Check connection"
          >
            <RefreshCw size={14} className={loading ? 'animate-spin' : ''} />
          </button>
        </div>
        {/* Base URL config */}
        <div className="mt-2 flex gap-2">
          <input
            type="text"
            value={baseUrl}
            onChange={(e) => setBaseUrl(e.target.value)}
            placeholder="http://localhost:1234/v1"
            className="flex-1 px-3 py-1.5 text-xs font-mono bg-[var(--color-bg-input)] border border-[var(--color-border-primary)] rounded-[var(--radius-sm)] text-[var(--color-text-primary)] placeholder:text-[var(--color-text-tertiary)] focus:outline-none focus:ring-1 focus:ring-[var(--color-border-focus)]"
          />
          <Button size="sm" variant="secondary" onClick={handleSaveBaseUrl}>
            Apply
          </Button>
        </div>
        {error && (
          <p className="text-xs text-[var(--color-error)] mt-2">{error}</p>
        )}
      </div>

      {/* Setup Guide */}
      <div>
        <h4 className="text-sm font-semibold text-[var(--color-text-primary)] mb-3">
          Setup Guide
        </h4>
        <div className="space-y-3">
          {SETUP_STEPS.map((step, i) => (
            <div key={i} className="flex gap-3">
              <div className="flex-shrink-0 mt-0.5">
                {serverStatus === 'online' ? (
                  <CheckCircle2 size={16} className="text-emerald-500" />
                ) : (
                  <Circle size={16} className="text-[var(--color-text-tertiary)]" />
                )}
              </div>
              <div className="min-w-0">
                <p className="text-sm font-medium text-[var(--color-text-primary)]">
                  {i + 1}. {step.title}
                </p>
                <p className="text-xs text-[var(--color-text-tertiary)] mt-0.5">
                  {step.description}
                </p>
                {step.link && (
                  <a
                    href={step.link}
                    target="_blank"
                    rel="noopener noreferrer"
                    className="inline-flex items-center gap-1 text-xs text-[var(--color-accent)] hover:underline mt-1"
                  >
                    {step.linkText} <ExternalLink size={10} />
                  </a>
                )}
              </div>
            </div>
          ))}
        </div>
      </div>

      {/* Added Local Models */}
      {localModels.length > 0 && (
        <div>
          <h4 className="text-sm font-semibold text-[var(--color-text-primary)] mb-2">
            Active Local Models
          </h4>
          <p className="text-xs text-[var(--color-text-tertiary)] mb-3">
            These models are included in your council and direct chat.
          </p>
          <div className="space-y-2">
            {localModels.map((model) => (
              <div
                key={model.model}
                className="flex items-center gap-3 p-3 rounded-[var(--radius-md)] border border-[var(--color-border-primary)] bg-[var(--color-bg-card)]"
              >
                <div
                  className="w-2 h-2 rounded-full flex-shrink-0"
                  style={{ backgroundColor: color }}
                />
                <div className="flex-1 min-w-0">
                  <span className="text-sm font-medium text-[var(--color-text-primary)] truncate block">
                    {model.displayName}
                  </span>
                  <span className="text-xs text-emerald-600 dark:text-emerald-400">Local</span>
                </div>
                <button
                  onClick={() => handleRemoveModel(model.model)}
                  className="p-1.5 text-[var(--color-text-tertiary)] hover:text-[var(--color-error)] transition-colors"
                >
                  <Trash2 size={14} />
                </button>
              </div>
            ))}
          </div>
        </div>
      )}

      {/* Add Model */}
      <div>
        <h4 className="text-sm font-semibold text-[var(--color-text-primary)] mb-2">
          Add Local Model
        </h4>
        {serverStatus === 'online' ? (
          showAddModel ? (
            <div className="p-3 rounded-[var(--radius-md)] border border-dashed border-[var(--color-border-secondary)] bg-[var(--color-bg-secondary)] space-y-2">
              <div className="flex gap-2">
                <select
                  value={selectedModel}
                  onChange={(e) => setSelectedModel(e.target.value)}
                  className="flex-1 px-3 py-1.5 text-sm bg-[var(--color-bg-input)] border border-[var(--color-border-primary)] rounded-[var(--radius-sm)] text-[var(--color-text-primary)]"
                >
                  <option value="">Select model...</option>
                  {chatModels.map((m) => (
                    <option key={m.id} value={m.id}>
                      {m.id}
                    </option>
                  ))}
                </select>
                <button
                  onClick={checkServerAndFetchModels}
                  disabled={loading}
                  className="p-1.5 text-[var(--color-text-tertiary)] hover:text-[var(--color-accent)] transition-colors"
                  title="Refresh models"
                >
                  <RefreshCw size={14} className={loading ? 'animate-spin' : ''} />
                </button>
              </div>
              <div className="flex gap-2">
                <Button size="sm" onClick={handleAddModel} disabled={!selectedModel}>
                  Add to Council
                </Button>
                <Button size="sm" variant="ghost" onClick={() => setShowAddModel(false)}>
                  Cancel
                </Button>
              </div>
            </div>
          ) : (
            <Button size="sm" variant="secondary" onClick={() => setShowAddModel(true)}>
              <Plus size={14} /> Add Local Model
            </Button>
          )
        ) : (
          <p className="text-xs text-[var(--color-text-tertiary)] p-3 rounded-[var(--radius-md)] border border-dashed border-[var(--color-border-secondary)] bg-[var(--color-bg-secondary)]">
            Start the LM Studio server to add local models.
          </p>
        )}
      </div>
    </div>
  );
}
