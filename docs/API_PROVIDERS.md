# API Provider Integrations

## Overview

All API calls are made from the Rust backend. API keys are stored in the macOS Keychain and never exposed to the frontend JavaScript context. Responses are streamed via Tauri's event system.

All 8 providers share a common `parse_sse_stream()` utility (`src-tauri/src/providers/mod.rs`) that handles SSE line buffering across TCP chunk boundaries. Each provider supplies a closure to extract `StreamEvent::Token` and `StreamEvent::Usage` events from its JSON format.

Usage data is accumulated using a MAX strategy in `stream_chat` — this uniformly handles Anthropic's split usage events, Google's cumulative totals, and single-event providers like OpenAI.

## Anthropic (Claude)

- **Endpoint**: `POST https://api.anthropic.com/v1/messages`
- **Auth**: `x-api-key` header + `anthropic-version` header
- **Streaming**: Server-Sent Events (SSE)
- **Token extraction**: `content_block_delta` events → `delta.text`
- **Usage extraction**: Split across `message_start` (input) and `message_delta` (output) events
- **System prompt**: Top-level `system` field in request body
- **Implementation**: `src-tauri/src/providers/anthropic.rs`

## OpenAI (GPT)

- **Endpoint**: `POST https://api.openai.com/v1/chat/completions`
- **Auth**: `Authorization: Bearer {key}` header
- **Streaming**: Server-Sent Events (SSE)
- **Token extraction**: `choices[0].delta.content`
- **Usage extraction**: Final chunk with `usage.prompt_tokens` / `usage.completion_tokens` (requires `stream_options.include_usage: true`)
- **System prompt**: `system` role message in messages array
- **Implementation**: `src-tauri/src/providers/openai.rs`

## Google (Gemini)

- **Endpoint**: `POST https://generativelanguage.googleapis.com/v1beta/models/{model}:streamGenerateContent`
- **Auth**: `key` query parameter
- **Streaming**: SSE with `alt=sse` query parameter
- **Token extraction**: `candidates[0].content.parts[].text`
- **Usage extraction**: Cumulative `usageMetadata` in every chunk (promptTokenCount / candidatesTokenCount)
- **System prompt**: `systemInstruction` field in request body
- **Note**: Uses `model` role instead of `assistant`
- **Implementation**: `src-tauri/src/providers/google.rs`

## xAI (Grok)

- **Endpoint**: `POST https://api.x.ai/v1/chat/completions`
- **Auth**: `Authorization: Bearer {key}` header
- **Streaming**: OpenAI-compatible SSE format
- **Token extraction**: `choices[0].delta.content`
- **Usage extraction**: Final chunk `usage.prompt_tokens` / `usage.completion_tokens`
- **System prompt**: `system` role message in messages array
- **Implementation**: `src-tauri/src/providers/xai.rs`

## DeepSeek

- **Endpoint**: `POST https://api.deepseek.com/chat/completions`
- **Auth**: `Authorization: Bearer {key}` header
- **Streaming**: OpenAI-compatible SSE format
- **Token extraction**: `choices[0].delta.content`
- **Usage extraction**: Final chunk `usage.prompt_tokens` / `usage.completion_tokens`
- **System prompt**: `system` role message in messages array
- **Implementation**: `src-tauri/src/providers/deepseek.rs`

## Mistral

- **Endpoint**: `POST https://api.mistral.ai/v1/chat/completions`
- **Auth**: `Authorization: Bearer {key}` header
- **Streaming**: OpenAI-compatible SSE format
- **Token extraction**: `choices[0].delta.content`
- **Usage extraction**: Final streaming chunk includes `usage.prompt_tokens` / `usage.completion_tokens` by default
- **System prompt**: `system` role message in messages array
- **Implementation**: `src-tauri/src/providers/mistral.rs`

## Together AI (Llama)

- **Endpoint**: `POST https://api.together.xyz/v1/chat/completions`
- **Auth**: `Authorization: Bearer {key}` header
- **Streaming**: OpenAI-compatible SSE format
- **Token extraction**: `choices[0].delta.content`
- **Usage extraction**: Final chunk `usage.prompt_tokens` / `usage.completion_tokens`
- **System prompt**: `system` role message in messages array
- **Implementation**: `src-tauri/src/providers/together.rs`

## Cohere (Command)

- **Endpoint**: `POST https://api.cohere.com/v2/chat`
- **Auth**: `Authorization: Bearer {key}` header
- **Streaming**: SSE with custom event types
- **Token extraction**: `content-delta` events → `delta.message.content.text`
- **Usage extraction**: `message-end` events → `delta.usage.tokens.input_tokens` / `delta.usage.tokens.output_tokens`
- **System prompt**: `system` role message in messages array
- **Implementation**: `src-tauri/src/providers/cohere.rs`
