import { useEffect } from 'react';
import MainLayout from './components/layout/MainLayout';
import SetupWizard from './components/setup/SetupWizard';
import { useSettingsStore } from './stores/settingsStore';
import { useModelCatalogStore } from './stores/modelCatalogStore';

export default function App() {
  const { settings, loaded, loadSettings } = useSettingsStore();

  useEffect(() => {
    loadSettings();
  }, [loadSettings]);

  // Once keys can exist, ask each provider for its current models. This also runs
  // right after the setup wizard saves the first keys.
  const refreshAll = useModelCatalogStore((s) => s.refreshAll);
  useEffect(() => {
    if (loaded && settings.setupCompleted) {
      void refreshAll();
    }
  }, [loaded, settings.setupCompleted, refreshAll]);

  const showApp = loaded && settings.setupCompleted;

  if (!loaded) {
    return (
      <div className="flex items-center justify-center h-screen bg-[var(--color-bg-primary)]">
        <div className="flex gap-1.5">
          {[0, 1, 2].map((i) => (
            <div
              key={i}
              className="w-2.5 h-2.5 rounded-full bg-[var(--color-accent)] thinking-dot"
            />
          ))}
        </div>
      </div>
    );
  }

  if (!showApp && !settings.setupCompleted) {
    return <SetupWizard />;
  }

  return <MainLayout />;
}
