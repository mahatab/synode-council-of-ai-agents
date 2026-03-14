import { useEffect, useState } from 'react';
import { getVersion } from '@tauri-apps/api/app';
import { openUrl } from '@tauri-apps/plugin-opener';

export default function AboutWindow() {
  const [version, setVersion] = useState('');

  useEffect(() => {
    getVersion().then(setVersion).catch(() => setVersion('0.4.3'));

    // Apply theme from system preference
    const prefersDark = window.matchMedia('(prefers-color-scheme: dark)').matches;
    if (prefersDark) document.documentElement.classList.add('dark');
  }, []);

  const handleLink = (url: string) => {
    openUrl(url).catch(console.error);
  };

  return (
    <div className="h-screen flex flex-col items-center select-none bg-[var(--color-bg-primary)] text-[var(--color-text-primary)] font-[var(--font-sans)] px-8 py-6">
      {/* Main content - centered */}
      <div className="flex-1 flex flex-col items-center justify-center">
        {/* Logo */}
        <img
          src="/synod-icon.png"
          alt="Synode"
          className="w-20 h-20 rounded-[var(--radius-xl)] mb-5 shadow-lg"
        />

        {/* App name & version */}
        <h1 className="text-2xl font-bold tracking-tight mb-1">Synode</h1>
        <p className="text-xs text-[var(--color-text-tertiary)] mb-4 font-[var(--font-mono)]">
          v{version}
        </p>

        {/* Description */}
        <p className="text-sm text-[var(--color-text-secondary)] text-center leading-relaxed max-w-[280px]">
          A council of AI models that collaboratively help you make informed, fact-based decisions.
        </p>
      </div>

      {/* Links - pinned to bottom */}
      <div className="flex gap-4 pt-4 border-t border-[var(--color-border-primary)] w-full justify-center">
        <button
          onClick={() => handleLink('https://github.com/mahatab/Council-of-AI-Agents')}
          className="text-xs text-[var(--color-accent)] hover:text-[var(--color-accent-hover)] transition-colors cursor-pointer"
        >
          GitHub
        </button>
        <span className="text-[var(--color-text-tertiary)]">·</span>
        <button
          onClick={() => handleLink('https://github.com/mahatab/Council-of-AI-Agents/blob/main/LICENSE')}
          className="text-xs text-[var(--color-accent)] hover:text-[var(--color-accent-hover)] transition-colors cursor-pointer"
        >
          MIT License
        </button>
        <span className="text-[var(--color-text-tertiary)]">·</span>
        <button
          onClick={() => handleLink('https://github.com/mahatab/Council-of-AI-Agents/issues')}
          className="text-xs text-[var(--color-accent)] hover:text-[var(--color-accent-hover)] transition-colors cursor-pointer"
        >
          Report Issue
        </button>
      </div>
    </div>
  );
}
