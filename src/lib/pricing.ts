import {
  pricingFromUsdPerMillion,
  estimateUsdCost,
  type CostBreakdown,
  type Pricing,
} from 'tokentally';

// Static pricing map: model ID → USD per 1M tokens.
// Keyed by exact model IDs from PROVIDERS in src/types/index.ts.
// Prices are best-effort estimates from the providers' pricing pages (November 2026):
// standard tier, no caching or batch discount. A model that is not listed here, such
// as one that only appeared in a provider's live model list, has no cost estimate.
interface ModelPricingEntry {
  inputUsdPerMillion: number;
  outputUsdPerMillion: number;
}

const PRICING_MAP: Record<string, ModelPricingEntry> = {
  // Anthropic
  'claude-fable-5-2':  { inputUsdPerMillion: 8, outputUsdPerMillion: 40 },
  'claude-opus-5-5':   { inputUsdPerMillion: 4, outputUsdPerMillion: 20 },
  'claude-sonnet-5-5': { inputUsdPerMillion: 2, outputUsdPerMillion: 10 },
  'claude-haiku-5-5':  { inputUsdPerMillion: 0.1, outputUsdPerMillion: 0.5 },

  // OpenAI
  'gpt-6.2-astra': { inputUsdPerMillion: 8, outputUsdPerMillion: 40 },
  'gpt-6.2-sol':   { inputUsdPerMillion: 2, outputUsdPerMillion: 10 },
  'gpt-6.1-terra': { inputUsdPerMillion: 1, outputUsdPerMillion: 5 },
  'gpt-6-luna':    { inputUsdPerMillion: 0.1, outputUsdPerMillion: 0.5 },
  'gpt-5.6-sol':   { inputUsdPerMillion: 4, outputUsdPerMillion: 20 },
  'gpt-5.2':       { inputUsdPerMillion: 1.75, outputUsdPerMillion: 14 },

  // Google
  'gemini-4-carbon':       { inputUsdPerMillion: 2, outputUsdPerMillion: 10 },
  'gemini-4-neon':         { inputUsdPerMillion: 3, outputUsdPerMillion: 15 },
  'gemini-4-hydrogen':     { inputUsdPerMillion: 1.5, outputUsdPerMillion: 7.5 },
  'gemini-3.8-flash':      { inputUsdPerMillion: 0.75, outputUsdPerMillion: 3.75 },
  'gemini-3.5-flash-lite': { inputUsdPerMillion: 0.3, outputUsdPerMillion: 2.5 },

  // xAI
  'grok-4.7': { inputUsdPerMillion: 2, outputUsdPerMillion: 6 },
  'grok-4.6': { inputUsdPerMillion: 2, outputUsdPerMillion: 6 },
  'grok-4.3': { inputUsdPerMillion: 1.25, outputUsdPerMillion: 2.5 },

  // DeepSeek (peak-hours rate)
  'deepseek-v4-pro': { inputUsdPerMillion: 1.32, outputUsdPerMillion: 3.96 },
  'deepseek-flash':  { inputUsdPerMillion: 0.3, outputUsdPerMillion: 1.2 },

  // Mistral
  'mistral-large-latest': { inputUsdPerMillion: 0.7, outputUsdPerMillion: 2.1 },
  'mistral-small-latest': { inputUsdPerMillion: 0.15, outputUsdPerMillion: 0.6 },
  'codestral-latest':     { inputUsdPerMillion: 0.3, outputUsdPerMillion: 0.9 },

  // Together AI
  'moonshotai/Kimi-K3.1-Pro':             { inputUsdPerMillion: 2, outputUsdPerMillion: 10 },
  'zai-org/GLM-5.3':                      { inputUsdPerMillion: 1.4, outputUsdPerMillion: 4.4 },
  'Qwen/Qwen4-Max':                       { inputUsdPerMillion: 1.425, outputUsdPerMillion: 4.275 },
  'deepseek-ai/DeepSeek-V4.1-Flash-1017': { inputUsdPerMillion: 0.3, outputUsdPerMillion: 1.2 },
  'openai/gpt-oss-120b':                  { inputUsdPerMillion: 0.15, outputUsdPerMillion: 0.6 },

  // Cohere
  'command-a-03-2025':      { inputUsdPerMillion: 2.5, outputUsdPerMillion: 10 },
  'command-r-plus-08-2024': { inputUsdPerMillion: 2.5, outputUsdPerMillion: 10 },
};

// Pre-compute Pricing objects for each model
const pricingCache = new Map<string, Pricing>();
function getPricing(modelId: string): Pricing | null {
  const entry = PRICING_MAP[modelId];
  if (!entry) return null;

  let pricing = pricingCache.get(modelId);
  if (!pricing) {
    pricing = pricingFromUsdPerMillion(entry);
    pricingCache.set(modelId, pricing);
  }
  return pricing;
}

/**
 * Calculate USD cost for a given model's token usage.
 * Returns null if the model has no pricing data.
 */
export function calculateModelCost(
  modelId: string,
  inputTokens: number,
  outputTokens: number,
): CostBreakdown | null {
  const pricing = getPricing(modelId);
  if (!pricing) return null;

  return estimateUsdCost({
    usage: { inputTokens, outputTokens },
    pricing,
  });
}

/**
 * Format a USD amount for display.
 * - Zero: "$0.00"
 * - Below $0.01: "<$0.01"
 * - Below $1: 2-4 significant decimal places (e.g., "$0.0042", "$0.15")
 * - $1 and above: 2 decimal places (e.g., "$1.50", "$23.45")
 */
export function formatUsdCost(usd: number): string {
  if (usd === 0) return '$0.00';
  if (usd < 0.01) return '<$0.01';
  if (usd < 1) {
    // Show enough precision to be meaningful
    const formatted = usd.toFixed(4).replace(/0+$/, '');
    const parts = formatted.split('.');
    const decimals = parts[1] || '00';
    return `$${parts[0]}.${decimals.length < 2 ? decimals + '0' : decimals}`;
  }
  return `$${usd.toFixed(2)}`;
}
