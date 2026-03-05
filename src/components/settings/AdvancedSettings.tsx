import { useSettingsStore } from '../../stores/settingsStore';
import type { SystemPromptMode, DiscussionDepth, DiscussionStyle } from '../../types';

export default function AdvancedSettings() {
  const { settings, updateSettings } = useSettingsStore();

  const modes: { id: SystemPromptMode; label: string; description: string }[] = [
    {
      id: 'upfront',
      label: 'Generate All Upfront',
      description:
        'Master model generates system prompts for all council models before the discussion starts. One API call, faster overall.',
    },
    {
      id: 'dynamic',
      label: 'Generate Dynamically Per Turn',
      description:
        'Master model generates each system prompt right before that model responds, incorporating context from previous responses. More adaptive but uses more API calls.',
    },
  ];

  const styles: { id: DiscussionStyle; label: string; description: string }[] = [
    {
      id: 'sequential',
      label: 'Sequential',
      description:
        'Each model sees previous responses and can build on, challenge, or refine them. Best for deep, iterative analysis.',
    },
    {
      id: 'independent',
      label: 'Independent',
      description:
        'Each model receives only the original question. Prevents groupthink and gives completely unbiased perspectives. The master model still sees all responses.',
    },
  ];

  const depths: { id: DiscussionDepth; label: string; description: string }[] = [
    {
      id: 'thorough',
      label: 'Thorough',
      description:
        'Models provide detailed analysis with comprehensive explanations. Best for complex decisions (uses more tokens).',
    },
    {
      id: 'concise',
      label: 'Concise',
      description:
        'Models give brief, focused responses with 2-3 key points only. Great for quick insights (saves tokens).',
    },
  ];

  return (
    <div>
      <h3 className="text-sm font-semibold text-[var(--color-text-primary)] mb-1">
        Discussion Style
      </h3>
      <p className="text-xs text-[var(--color-text-tertiary)] mb-3">
        How council models interact with each other during discussion
      </p>

      <div className="space-y-2 mb-6">
        {styles.map((style) => (
          <button
            key={style.id}
            onClick={() => updateSettings({ discussionStyle: style.id })}
            className={`w-full text-left p-3 rounded-[var(--radius-md)] border transition-all ${
              settings.discussionStyle === style.id
                ? 'border-[var(--color-accent)] bg-[var(--color-accent-light)]'
                : 'border-[var(--color-border-primary)] hover:border-[var(--color-border-secondary)] bg-[var(--color-bg-secondary)]'
            }`}
          >
            <span
              className={`text-sm font-medium ${
                settings.discussionStyle === style.id
                  ? 'text-[var(--color-accent)]'
                  : 'text-[var(--color-text-primary)]'
              }`}
            >
              {style.label}
            </span>
            <p className="text-xs text-[var(--color-text-tertiary)] mt-1">
              {style.description}
            </p>
          </button>
        ))}
      </div>

      <h3 className="text-sm font-semibold text-[var(--color-text-primary)] mb-1">
        Discussion Depth
      </h3>
      <p className="text-xs text-[var(--color-text-tertiary)] mb-3">
        Control how detailed the council's responses should be
      </p>

      <div className="space-y-2 mb-6">
        {depths.map((depth) => (
          <button
            key={depth.id}
            onClick={() => updateSettings({ discussionDepth: depth.id })}
            className={`w-full text-left p-3 rounded-[var(--radius-md)] border transition-all ${
              settings.discussionDepth === depth.id
                ? 'border-[var(--color-accent)] bg-[var(--color-accent-light)]'
                : 'border-[var(--color-border-primary)] hover:border-[var(--color-border-secondary)] bg-[var(--color-bg-secondary)]'
            }`}
          >
            <span
              className={`text-sm font-medium ${
                settings.discussionDepth === depth.id
                  ? 'text-[var(--color-accent)]'
                  : 'text-[var(--color-text-primary)]'
              }`}
            >
              {depth.label}
            </span>
            <p className="text-xs text-[var(--color-text-tertiary)] mt-1">
              {depth.description}
            </p>
          </button>
        ))}
      </div>

      <h3 className="text-sm font-semibold text-[var(--color-text-primary)] mb-1">
        System Prompt Generation
      </h3>
      <p className="text-xs text-[var(--color-text-tertiary)] mb-3">
        How the master model creates system prompts for council members
      </p>

      <div className="space-y-2">
        {modes.map((mode) => (
          <button
            key={mode.id}
            onClick={() => updateSettings({ systemPromptMode: mode.id })}
            className={`w-full text-left p-3 rounded-[var(--radius-md)] border transition-all ${
              settings.systemPromptMode === mode.id
                ? 'border-[var(--color-accent)] bg-[var(--color-accent-light)]'
                : 'border-[var(--color-border-primary)] hover:border-[var(--color-border-secondary)] bg-[var(--color-bg-secondary)]'
            }`}
          >
            <span
              className={`text-sm font-medium ${
                settings.systemPromptMode === mode.id
                  ? 'text-[var(--color-accent)]'
                  : 'text-[var(--color-text-primary)]'
              }`}
            >
              {mode.label}
            </span>
            <p className="text-xs text-[var(--color-text-tertiary)] mt-1">
              {mode.description}
            </p>
          </button>
        ))}
      </div>
    </div>
  );
}
