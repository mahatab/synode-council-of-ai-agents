# Adding a New AI Provider

This guide walks through adding a new AI provider to the council.

## Step 1: Add the Rust Provider

Create `src-tauri/src/providers/your_provider.rs`:

```rust
use anyhow::{anyhow, Result};
use reqwest::Client;
use serde_json::{json, Value};

use crate::models::config::ChatMessage;
use super::{parse_sse_stream, StreamEvent, TokenStream, UsageData};

pub struct YourProvider {
    client: Client,
}

impl YourProvider {
    pub fn new() -> Self {
        Self { client: Client::new() }
    }

    pub async fn stream_chat(
        &self,
        api_key: &str,
        model: &str,
        messages: &[ChatMessage],
        system_prompt: Option<&str>,
    ) -> Result<TokenStream> {
        let mut api_messages: Vec<Value> = Vec::new();

        // Add system prompt (most providers use a system role message)
        if let Some(system) = system_prompt {
            api_messages.push(json!({
                "role": "system",
                "content": system
            }));
        }

        for m in messages {
            api_messages.push(json!({
                "role": m.role,
                "content": m.content
            }));
        }

        let body = json!({
            "model": model,
            "messages": api_messages,
            "stream": true
        });

        let response = self
            .client
            .post("https://api.your-provider.com/v1/chat/completions")
            .header("Authorization", format!("Bearer {}", api_key))
            .header("Content-Type", "application/json")
            .json(&body)
            .send()
            .await?;

        if !response.status().is_success() {
            let status = response.status();
            let error_body = response.text().await.unwrap_or_default();
            return Err(anyhow!("YourProvider API error ({}): {}", status, error_body));
        }

        let byte_stream = response.bytes_stream();

        // Use parse_sse_stream with a closure that extracts tokens and usage
        // from your provider's JSON format
        Ok(parse_sse_stream(byte_stream, |event| {
            let mut events = Vec::new();

            // Extract streaming text token
            if let Some(content) = event["choices"][0]["delta"]["content"].as_str() {
                events.push(StreamEvent::Token(content.to_string()));
            }

            // Extract usage from the final chunk
            if let Some(usage) = event.get("usage") {
                if let (Some(input), Some(output)) = (
                    usage["prompt_tokens"].as_u64(),
                    usage["completion_tokens"].as_u64(),
                ) {
                    events.push(StreamEvent::Usage(UsageData {
                        input_tokens: input as u32,
                        output_tokens: output as u32,
                    }));
                }
            }

            events
        }))
    }
}
```

The `parse_sse_stream()` utility in `src-tauri/src/providers/mod.rs` handles all SSE line buffering, `[DONE]` sentinels, and JSON parsing. You only need to provide the closure that maps your provider's JSON to `StreamEvent` variants.

## Step 2: Register the Provider

1. Add to `src-tauri/src/providers/mod.rs`:
   ```rust
   pub mod your_provider;
   ```

2. Add to the `Provider` enum in `src-tauri/src/models/config.rs`:
   ```rust
   pub enum Provider {
       Anthropic,
       OpenAI,
       Google,
       XAI,
       DeepSeek,
       Mistral,
       Together,
       Cohere,
       YourProvider,
   }
   ```

   Also add match arms to `display_name()` and `keychain_service()` in the `impl Provider` block.

3. Add the match arm in `src-tauri/src/commands/api_calls.rs`:
   ```rust
   Provider::YourProvider => {
       let p = YourProvider::new();
       p.stream_chat(&api_key, &model, &messages, system_ref).await
   }
   ```

   Add the import at the top:
   ```rust
   use crate::providers::your_provider::YourProvider;
   ```

## Step 3: Update CSP

Add your provider's API domain to the `connect-src` directive in `src-tauri/tauri.conf.json`:

```json
"security": {
  "csp": "default-src 'self'; connect-src 'self' ... https://api.your-provider.com; ..."
}
```

## Step 4: Add Frontend Configuration

In `src/types/index.ts`, add your provider to the `Provider` type and `PROVIDERS` array:

```typescript
// Add to the Provider type union
export type Provider = 'anthropic' | ... | 'your_provider';

// Add to the PROVIDERS array
{
  id: 'your_provider',
  name: 'Your Provider',
  keychainService: 'com.council-of-ai-agents.your-provider',
  models: [
    { id: 'model-id', name: 'Model Name' },
  ],
  apiKeyUrl: 'https://your-provider.com/api-keys',
  apiKeySteps: [
    'Go to your-provider.com',
    'Sign in or create an account',
    'Navigate to API settings',
    'Create and copy your API key',
  ],
}
```

Also add a color mapping in `getProviderColor()`:
```typescript
case 'your_provider':
  return '#HEX_COLOR';
```

## Step 5: Add Keychain Migration Entry

In `src-tauri/src/commands/keychain.rs`, add your provider to the `LEGACY_SERVICES` array:

```rust
const LEGACY_SERVICES: &[(&str, &str)] = &[
    // ...existing entries...
    ("your_provider", "com.council-of-ai-agents.your-provider"),
];
```

## Step 6: Test

1. Run `cargo check` in `src-tauri/` to verify Rust code compiles
2. Run `npx tsc --noEmit` to verify TypeScript
3. Run `cargo tauri dev` and add your provider in settings
4. Verify streaming works, usage data is reported, and API keys are saved/loaded correctly
