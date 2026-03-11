use anyhow::Result;
use futures::StreamExt;

use crate::models::config::{ChatMessage, Provider};
use crate::providers::UsageData;
use crate::providers::{
    anthropic::AnthropicProvider, cohere::CohereProvider, deepseek::DeepSeekProvider,
    google::GoogleProvider, mistral::MistralProvider, openai::OpenAIProvider,
    together::TogetherProvider, xai::XAIProvider, StreamEvent, TokenStream,
};

/// Result of a completed chat call.
#[derive(Debug, Clone)]
pub struct ChatResult {
    pub content: String,
    pub usage: Option<UsageData>,
}

/// Call a model and collect the full response (non-streaming).
pub async fn call_model(
    provider: &Provider,
    model: &str,
    messages: &[ChatMessage],
    system_prompt: Option<&str>,
    api_key: &str,
) -> Result<ChatResult> {
    let stream = create_stream(provider, model, messages, system_prompt, api_key).await?;
    collect_stream(stream).await
}

/// Call a model with a callback for each token (for progressive updates).
pub async fn call_model_streaming<F>(
    provider: &Provider,
    model: &str,
    messages: &[ChatMessage],
    system_prompt: Option<&str>,
    api_key: &str,
    mut on_token: F,
) -> Result<ChatResult>
where
    F: FnMut(&str),
{
    let stream = create_stream(provider, model, messages, system_prompt, api_key).await?;
    let mut content = String::new();
    let mut usage = UsageData {
        input_tokens: 0,
        output_tokens: 0,
    };

    futures::pin_mut!(stream);
    while let Some(event_result) = stream.next().await {
        match event_result? {
            StreamEvent::Token(token) => {
                on_token(&token);
                content.push_str(&token);
            }
            StreamEvent::Usage(u) => {
                usage.input_tokens = usage.input_tokens.max(u.input_tokens);
                usage.output_tokens = usage.output_tokens.max(u.output_tokens);
            }
        }
    }

    let final_usage = if usage.input_tokens > 0 || usage.output_tokens > 0 {
        Some(usage)
    } else {
        None
    };

    Ok(ChatResult {
        content,
        usage: final_usage,
    })
}

async fn create_stream(
    provider: &Provider,
    model: &str,
    messages: &[ChatMessage],
    system_prompt: Option<&str>,
    api_key: &str,
) -> Result<TokenStream> {
    match provider {
        Provider::Anthropic => {
            AnthropicProvider::new()
                .stream_chat(api_key, model, messages, system_prompt)
                .await
        }
        Provider::OpenAI => {
            OpenAIProvider::new()
                .stream_chat(api_key, model, messages, system_prompt)
                .await
        }
        Provider::Google => {
            GoogleProvider::new()
                .stream_chat(api_key, model, messages, system_prompt)
                .await
        }
        Provider::XAI => {
            XAIProvider::new()
                .stream_chat(api_key, model, messages, system_prompt)
                .await
        }
        Provider::DeepSeek => {
            DeepSeekProvider::new()
                .stream_chat(api_key, model, messages, system_prompt)
                .await
        }
        Provider::Mistral => {
            MistralProvider::new()
                .stream_chat(api_key, model, messages, system_prompt)
                .await
        }
        Provider::Together => {
            TogetherProvider::new()
                .stream_chat(api_key, model, messages, system_prompt)
                .await
        }
        Provider::Cohere => {
            CohereProvider::new()
                .stream_chat(api_key, model, messages, system_prompt)
                .await
        }
    }
}

async fn collect_stream(stream: TokenStream) -> Result<ChatResult> {
    let mut content = String::new();
    let mut usage = UsageData {
        input_tokens: 0,
        output_tokens: 0,
    };

    futures::pin_mut!(stream);
    while let Some(event_result) = stream.next().await {
        match event_result? {
            StreamEvent::Token(token) => content.push_str(&token),
            StreamEvent::Usage(u) => {
                usage.input_tokens = usage.input_tokens.max(u.input_tokens);
                usage.output_tokens = usage.output_tokens.max(u.output_tokens);
            }
        }
    }

    let final_usage = if usage.input_tokens > 0 || usage.output_tokens > 0 {
        Some(usage)
    } else {
        None
    };

    Ok(ChatResult {
        content,
        usage: final_usage,
    })
}
