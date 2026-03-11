import { useState, useEffect } from 'react';
import { useSettingsStore } from '../../stores/settingsStore';
import * as tauri from '../../lib/tauri';

const TELEGRAM_TOKEN_SERVICE = 'com.council-of-ai-agents.telegram-bot-token';

const COMMANDS = [
  { command: '/council <question>', description: 'Start a council discussion' },
  { command: '/chat <model> <msg>', description: 'Direct chat with a model' },
  { command: '/models', description: 'List configured models' },
  { command: '/sessions', description: 'List recent sessions' },
  { command: '/settings', description: 'Show current settings' },
  { command: '/stop', description: 'Cancel ongoing discussion' },
  { command: '/help', description: 'Show help' },
];

export default function TelegramSettings() {
  const { settings, updateSettings } = useSettingsStore();
  const [token, setToken] = useState('');
  const [tokenSaved, setTokenSaved] = useState(false);
  const [running, setRunning] = useState(false);
  const [loading, setLoading] = useState(false);
  const [showToken, setShowToken] = useState(false);

  useEffect(() => {
    // Load saved token and bot status on mount
    const init = async () => {
      try {
        const saved = await tauri.getApiKey(TELEGRAM_TOKEN_SERVICE);
        if (saved) {
          setToken(saved);
          setTokenSaved(true);
        }
        const status = await tauri.getTelegramStatus();
        setRunning(status.running);
      } catch {
        // Ignore errors on init
      }
    };
    init();
  }, []);

  const handleSaveToken = async () => {
    if (!token.trim()) return;
    await tauri.saveApiKey(TELEGRAM_TOKEN_SERVICE, token.trim());
    setTokenSaved(true);
  };

  const handleRemoveToken = async () => {
    await tauri.deleteApiKey(TELEGRAM_TOKEN_SERVICE);
    setToken('');
    setTokenSaved(false);
    if (running) {
      await handleStop();
    }
  };

  const handleStart = async () => {
    if (!tokenSaved) {
      await handleSaveToken();
    }
    setLoading(true);
    try {
      await tauri.startTelegramBot(token.trim());
      setRunning(true);
      await updateSettings({ telegramEnabled: true });
    } catch (e) {
      console.error('Failed to start bot:', e);
    }
    setLoading(false);
  };

  const handleStop = async () => {
    setLoading(true);
    try {
      await tauri.stopTelegramBot();
      setRunning(false);
      await updateSettings({ telegramEnabled: false });
    } catch (e) {
      console.error('Failed to stop bot:', e);
    }
    setLoading(false);
  };

  const handleToggle = () => {
    if (running) {
      handleStop();
    } else {
      handleStart();
    }
  };

  return (
    <div>
      <h3 className="text-sm font-semibold text-[var(--color-text-primary)] mb-1">
        Telegram Bot
      </h3>
      <p className="text-xs text-[var(--color-text-tertiary)] mb-4">
        Chat with Synode through Telegram. Create a bot with{' '}
        <span className="text-[var(--color-text-secondary)] font-medium">@BotFather</span>{' '}
        on Telegram and paste the token below.
      </p>

      {/* Token input */}
      <div className="mb-4">
        <label className="block text-xs text-[var(--color-text-secondary)] mb-1.5">
          Bot Token
        </label>
        <div className="flex gap-2">
          <div className="relative flex-1">
            <input
              type={showToken ? 'text' : 'password'}
              value={token}
              onChange={(e) => {
                setToken(e.target.value);
                setTokenSaved(false);
              }}
              placeholder="123456:ABC-DEF..."
              className="w-full px-3 py-2 pr-16 text-sm rounded-[var(--radius-md)] border border-[var(--color-border-primary)] bg-[var(--color-bg-primary)] text-[var(--color-text-primary)] placeholder:text-[var(--color-text-tertiary)] focus:outline-none focus:border-[var(--color-accent)]"
            />
            <button
              onClick={() => setShowToken(!showToken)}
              className="absolute right-2 top-1/2 -translate-y-1/2 text-xs text-[var(--color-text-tertiary)] hover:text-[var(--color-text-secondary)]"
            >
              {showToken ? 'Hide' : 'Show'}
            </button>
          </div>
          {tokenSaved ? (
            <button
              onClick={handleRemoveToken}
              className="px-3 py-2 text-xs rounded-[var(--radius-md)] border border-red-300 text-red-500 hover:bg-red-50 dark:border-red-800 dark:hover:bg-red-900/20 transition-colors"
            >
              Remove
            </button>
          ) : (
            <button
              onClick={handleSaveToken}
              disabled={!token.trim()}
              className="px-3 py-2 text-xs rounded-[var(--radius-md)] bg-[var(--color-accent)] text-white hover:opacity-90 disabled:opacity-40 transition-colors"
            >
              Save
            </button>
          )}
        </div>
      </div>

      {/* Enable/Disable toggle */}
      <div className="flex items-center justify-between p-3 mb-4 rounded-[var(--radius-md)] border border-[var(--color-border-primary)] bg-[var(--color-bg-secondary)]">
        <div className="flex items-center gap-2.5">
          <div
            className={`w-2 h-2 rounded-full ${running ? 'bg-green-500' : 'bg-[var(--color-text-tertiary)]'}`}
          />
          <span className="text-sm text-[var(--color-text-primary)]">
            {running ? 'Bot is running' : 'Bot is stopped'}
          </span>
        </div>
        <button
          onClick={handleToggle}
          disabled={loading || (!tokenSaved && !token.trim())}
          className={`px-4 py-1.5 text-xs font-medium rounded-[var(--radius-md)] transition-colors disabled:opacity-40 ${
            running
              ? 'bg-[var(--color-bg-primary)] border border-[var(--color-border-primary)] text-[var(--color-text-secondary)] hover:text-[var(--color-text-primary)]'
              : 'bg-[var(--color-accent)] text-white hover:opacity-90'
          }`}
        >
          {loading ? '...' : running ? 'Stop' : 'Start'}
        </button>
      </div>

      {settings.telegramEnabled && (
        <p className="text-xs text-[var(--color-text-tertiary)] mb-4">
          Bot will auto-start when Synode launches.
        </p>
      )}

      {/* Command reference */}
      <h3 className="text-sm font-semibold text-[var(--color-text-primary)] mb-1 mt-6">
        Available Commands
      </h3>
      <p className="text-xs text-[var(--color-text-tertiary)] mb-3">
        Use these commands in your Telegram chat with the bot.
      </p>

      <div className="space-y-1">
        {COMMANDS.map(({ command, description }) => (
          <div
            key={command}
            className="flex items-baseline gap-3 py-1.5 px-2 rounded text-xs"
          >
            <code className="text-[var(--color-accent)] font-mono whitespace-nowrap">
              {command}
            </code>
            <span className="text-[var(--color-text-tertiary)]">{description}</span>
          </div>
        ))}
      </div>
    </div>
  );
}
