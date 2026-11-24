//! Asks a cloud provider which chat models it currently offers.
//!
//! Each provider has its own "list models" endpoint and response shape, and most of
//! them mix chat models with embedding, image and audio models. The `parse_*`
//! functions turn one provider's response into a plain list of chat models; they take
//! JSON and do no I/O, so they can be tested without a network.

use anyhow::{anyhow, Result};
use reqwest::{Client, RequestBuilder};
use serde::{Deserialize, Serialize};
use serde_json::Value;
use std::collections::{HashMap, HashSet};
use std::time::Duration;

use crate::models::config::Provider;

const REQUEST_TIMEOUT: Duration = Duration::from_secs(20);

/// A chat model reported by a provider.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct RemoteModel {
    pub id: String,
    /// The provider's display name, or the id when it gives none.
    pub name: String,
    /// When the model was published, in Unix seconds, when the provider says.
    pub created: Option<i64>,
    /// Whether the model can use the provider's web search tool, when the provider
    /// reports it. `None` means the list does not say.
    pub web_search: Option<bool>,
}

impl RemoteModel {
    fn new(id: &str, name: Option<&str>, created: Option<i64>) -> Self {
        Self {
            id: id.to_string(),
            name: name
                .filter(|n| !n.trim().is_empty())
                .unwrap_or(id)
                .to_string(),
            created,
            web_search: None,
        }
    }
}

/// Fetch the chat models `provider` offers to the owner of `api_key`, newest first
/// where the provider reports dates.
pub async fn list_models(provider: &Provider, api_key: &str) -> Result<Vec<RemoteModel>> {
    let client = Client::builder().timeout(REQUEST_TIMEOUT).build()?;
    let bearer = format!("Bearer {}", api_key);

    let (request, parse): (RequestBuilder, fn(&Value) -> Vec<RemoteModel>) = match provider {
        Provider::Anthropic => (
            client
                .get("https://api.anthropic.com/v1/models?limit=1000")
                .header("x-api-key", api_key)
                .header("anthropic-version", "2023-06-01"),
            parse_anthropic,
        ),
        Provider::OpenAI => (
            client
                .get("https://api.openai.com/v1/models")
                .header("Authorization", bearer),
            parse_openai,
        ),
        Provider::Google => (
            client
                .get("https://generativelanguage.googleapis.com/v1beta/models")
                .query(&[("pageSize", "1000"), ("key", api_key)]),
            parse_google,
        ),
        Provider::XAI => (
            client
                .get("https://api.x.ai/v1/models")
                .header("Authorization", bearer),
            parse_xai,
        ),
        Provider::DeepSeek => (
            client
                .get("https://api.deepseek.com/models")
                .header("Authorization", bearer),
            parse_deepseek,
        ),
        Provider::Mistral => (
            client
                .get("https://api.mistral.ai/v1/models")
                .header("Authorization", bearer),
            parse_mistral,
        ),
        Provider::Together => (
            client
                .get("https://api.together.xyz/v1/models")
                .header("Authorization", bearer),
            parse_together,
        ),
        Provider::Cohere => (
            client
                .get("https://api.cohere.com/v1/models?endpoint=chat&page_size=1000")
                .header("Authorization", bearer),
            parse_cohere,
        ),
        Provider::LMStudio => {
            return Err(anyhow!(
                "LM Studio models are listed through the local server, not this function"
            ));
        }
    };

    let name = provider.display_name();
    // The Google request carries the key in its URL, so keep URLs out of errors.
    let response = request
        .send()
        .await
        .map_err(|e| anyhow!("{} model list request failed: {}", name, e.without_url()))?;

    if !response.status().is_success() {
        let status = response.status();
        let body = response.text().await.unwrap_or_default();
        let detail: String = body.chars().take(300).collect();
        return Err(anyhow!(
            "{} model list request failed ({}): {}",
            name,
            status,
            detail
        ));
    }

    let body: Value = response.json().await.map_err(|e| {
        anyhow!(
            "{} sent a model list that could not be read: {}",
            name,
            e.without_url()
        )
    })?;

    let mut models = dedupe(parse(&body));
    // A stable sort, so models without a date keep the provider's own order.
    models.sort_by_key(|m| std::cmp::Reverse(m.created.unwrap_or(0)));
    Ok(models)
}

fn dedupe(models: Vec<RemoteModel>) -> Vec<RemoteModel> {
    let mut seen = HashSet::new();
    models
        .into_iter()
        .filter(|m| seen.insert(m.id.clone()))
        .collect()
}

fn contains_any(id: &str, parts: &[&str]) -> bool {
    parts.iter().any(|p| id.contains(p))
}

fn all_digits(s: &str, len: usize) -> bool {
    s.len() == len && s.bytes().all(|b| b.is_ascii_digit())
}

/// True for ids that end in a calendar date, such as `gpt-4o-2024-08-06`. Providers
/// list these dated snapshots next to the plain alias, which is the one to show.
fn ends_with_date(id: &str) -> bool {
    let parts: Vec<&str> = id.rsplitn(4, '-').collect();
    parts.len() == 4
        && all_digits(parts[0], 2)
        && all_digits(parts[1], 2)
        && all_digits(parts[2], 4)
}

/// True for `prefix` followed by a digit, such as `gpt-6` or `o3`.
fn starts_with_then_digit(id: &str, prefix: &str) -> bool {
    id.strip_prefix(prefix)
        .is_some_and(|rest| rest.starts_with(|c: char| c.is_ascii_digit()))
}

/// True when `value` is a date (`YYYY-MM-DD`, with or without a time after it) that
/// is already in the past.
fn date_has_passed(value: &Value) -> bool {
    value
        .as_str()
        .and_then(|s| s.get(..10))
        .and_then(|d| chrono::NaiveDate::parse_from_str(d, "%Y-%m-%d").ok())
        .is_some_and(|d| d < chrono::Utc::now().date_naive())
}

/// Entries of the array at `key`, or of `body` itself when it is a bare array.
fn entries<'a>(body: &'a Value, key: &str) -> &'a [Value] {
    body.get(key)
        .and_then(Value::as_array)
        .or_else(|| body.as_array())
        .map(Vec::as_slice)
        .unwrap_or(&[])
}

/// Anthropic lists chat models only. Skip the ones that are no longer active and the
/// limited-availability Mythos line, which an ordinary key cannot call.
pub fn parse_anthropic(body: &Value) -> Vec<RemoteModel> {
    entries(body, "data")
        .iter()
        .filter_map(|m| {
            let id = m["id"].as_str()?;
            let inactive = m["lifecycle"].as_str().is_some_and(|l| l != "active");
            if inactive || m["line"].as_str() == Some("mythos") || id.contains("mythos") {
                return None;
            }
            let created = m["created_at"]
                .as_str()
                .and_then(|s| chrono::DateTime::parse_from_rfc3339(s).ok())
                .map(|d| d.timestamp());
            let mut model = RemoteModel::new(id, m["display_name"].as_str(), created);
            model.web_search =
                m["capabilities"]["server_tools"]["web_search"]["supported"].as_bool();
            Some(model)
        })
        .collect()
}

/// OpenAI lists every model the account can call, with no field that says what a
/// model is for. Keep the chat families, then drop the special-purpose ones, the
/// dated snapshots, and anything already shut down.
pub fn parse_openai(body: &Value) -> Vec<RemoteModel> {
    const NOT_CHAT: &[&str] = &[
        "embedding",
        "audio",
        "realtime",
        "live",
        "tts",
        "transcribe",
        "whisper",
        "image",
        "dall-e",
        "moderation",
        "instruct",
        "search",
        "codex",
        "cyber",
        "oss",
        "daybreak",
        "computer-use",
        "deep-research",
    ];
    entries(body, "data")
        .iter()
        .filter_map(|m| {
            let id = m["id"].as_str()?;
            let chat_family = starts_with_then_digit(id, "gpt-")
                || starts_with_then_digit(id, "o")
                || id.starts_with("chatgpt-")
                || id == "chat-latest";
            if !chat_family
                || contains_any(id, NOT_CHAT)
                || ends_with_date(id)
                || date_has_passed(&m["shutdown_date"])
            {
                return None;
            }
            Some(RemoteModel::new(id, None, m["created"].as_i64()))
        })
        .collect()
}

/// Google marks what each model can do, but image, speech and live models also
/// generate content, so they are dropped by name.
pub fn parse_google(body: &Value) -> Vec<RemoteModel> {
    const NOT_CHAT: &[&str] = &[
        "embedding",
        "image",
        "nano-banana",
        "tts",
        "audio",
        "live",
        "transcribe",
        "translate",
        "omni",
        "robotics",
        "computer-use",
        "deep-research",
    ];
    entries(body, "models")
        .iter()
        .filter_map(|m| {
            let id = m["name"].as_str()?.trim_start_matches("models/");
            let generates_content = m["supportedGenerationMethods"]
                .as_array()
                .is_some_and(|methods| methods.iter().any(|x| x == "generateContent"));
            if !generates_content || !id.starts_with("gemini-") || contains_any(id, NOT_CHAT) {
                return None;
            }
            Some(RemoteModel::new(id, m["displayName"].as_str(), None))
        })
        .collect()
}

pub fn parse_xai(body: &Value) -> Vec<RemoteModel> {
    const NOT_CHAT: &[&str] = &["image", "imagine", "video", "voice"];
    entries(body, "data")
        .iter()
        .filter_map(|m| {
            let id = m["id"].as_str()?;
            if !starts_with_then_digit(id, "grok-") || contains_any(id, NOT_CHAT) {
                return None;
            }
            Some(RemoteModel::new(id, None, m["created"].as_i64()))
        })
        .collect()
}

pub fn parse_deepseek(body: &Value) -> Vec<RemoteModel> {
    entries(body, "data")
        .iter()
        .filter_map(|m| {
            let id = m["id"].as_str()?;
            // Older responses have no modality field; newer ones say what a model emits.
            let no_text = m["output_modalities"]
                .as_array()
                .is_some_and(|out| !out.iter().any(|x| x == "text"));
            if no_text {
                return None;
            }
            Some(RemoteModel::new(id, m["name"].as_str(), None))
        })
        .collect()
}

/// Mistral lists one model several times: under its versioned id and under each of
/// its aliases. Keep the chat models, and show each one once under its `-latest`
/// alias when it has one, or its shortest id when it does not.
pub fn parse_mistral(body: &Value) -> Vec<RemoteModel> {
    const NOT_CHAT_CAPABILITIES: &[&str] = &[
        "ocr",
        "moderation",
        "classification",
        "audio_transcription",
        "audio_speech",
    ];

    fn preferred<'a>(a: &'a str, b: &'a str) -> &'a str {
        let rank = |id: &str| (!id.ends_with("-latest"), id.len());
        if rank(b) < rank(a) {
            b
        } else {
            a
        }
    }

    let mut order: Vec<String> = Vec::new();
    let mut best: HashMap<String, RemoteModel> = HashMap::new();

    for m in entries(body, "data") {
        let Some(id) = m["id"].as_str() else { continue };
        let caps = &m["capabilities"];
        let chat = caps["completion_chat"].as_bool().unwrap_or(false);
        let special = NOT_CHAT_CAPABILITIES
            .iter()
            .any(|c| caps[*c].as_bool().unwrap_or(false));
        let fine_tuned = m["type"].as_str().is_some_and(|t| t != "base");
        if !chat || special || fine_tuned || date_has_passed(&m["deprecation"]) {
            continue;
        }

        // Every name this entry answers to. The smallest one identifies the model, so
        // the entries for one model land in the same group whichever name they use.
        let mut names: Vec<&str> = vec![id];
        if let Some(aliases) = m["aliases"].as_array() {
            names.extend(aliases.iter().filter_map(Value::as_str));
        }
        let key = names.iter().min().copied().unwrap_or(id).to_string();
        let shown = names.iter().copied().reduce(preferred).unwrap_or(id);
        let candidate = RemoteModel::new(shown, None, m["created"].as_i64());

        match best.get_mut(&key) {
            Some(current) => {
                if preferred(&current.id, &candidate.id) == candidate.id {
                    *current = candidate;
                }
            }
            None => {
                order.push(key.clone());
                best.insert(key, candidate);
            }
        }
    }

    order
        .into_iter()
        .filter_map(|key| best.remove(&key))
        .collect()
}

pub fn parse_together(body: &Value) -> Vec<RemoteModel> {
    entries(body, "data")
        .iter()
        .filter_map(|m| {
            let id = m["id"].as_str()?;
            if m["type"].as_str() != Some("chat") {
                return None;
            }
            Some(RemoteModel::new(
                id,
                m["display_name"].as_str(),
                m["created"].as_i64().filter(|c| *c > 0),
            ))
        })
        .collect()
}

pub fn parse_cohere(body: &Value) -> Vec<RemoteModel> {
    entries(body, "models")
        .iter()
        .filter_map(|m| {
            let id = m["name"].as_str()?;
            let chat = m["endpoints"]
                .as_array()
                .is_some_and(|e| e.iter().any(|x| x == "chat"));
            let retired = m["is_deprecated"].as_bool().unwrap_or(false);
            let fine_tuned = m["finetuned"].as_bool().unwrap_or(false);
            if !chat || retired || fine_tuned {
                return None;
            }
            Some(RemoteModel::new(id, None, None))
        })
        .collect()
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde_json::json;

    fn ids(models: &[RemoteModel]) -> Vec<&str> {
        models.iter().map(|m| m.id.as_str()).collect()
    }

    #[test]
    fn anthropic_reads_name_date_and_web_search() {
        let body = json!({
            "data": [
                { "type": "model", "id": "claude-sonnet-4-6", "display_name": "Claude Sonnet 4.6",
                  "created_at": "2026-02-17T00:00:00Z", "lifecycle": "active", "line": "sonnet",
                  "capabilities": { "server_tools": { "web_search": { "supported": true } } } },
                { "type": "model", "id": "claude-haiku-4-5-20251001", "display_name": "",
                  "created_at": "not a date" }
            ],
            "has_more": false
        });
        let models = parse_anthropic(&body);
        assert_eq!(models[0].name, "Claude Sonnet 4.6");
        assert_eq!(models[0].created, Some(1771286400));
        assert_eq!(models[0].web_search, Some(true));
        // An empty display name falls back to the id, and a bad date is dropped.
        assert_eq!(models[1].name, "claude-haiku-4-5-20251001");
        assert_eq!(models[1].created, None);
        assert_eq!(models[1].web_search, None);
    }

    #[test]
    fn anthropic_skips_inactive_and_mythos_models() {
        let body = json!({
            "data": [
                { "id": "claude-opus-5-5", "lifecycle": "active", "line": "opus" },
                { "id": "claude-sonnet-4-5", "lifecycle": "deprecated", "line": "sonnet" },
                { "id": "claude-mythos-5-2", "lifecycle": "active", "line": "mythos" }
            ]
        });
        assert_eq!(ids(&parse_anthropic(&body)), vec!["claude-opus-5-5"]);
    }

    #[test]
    fn openai_keeps_chat_aliases_only() {
        let body = json!({
            "object": "list",
            "data": [
                { "id": "gpt-6.2-astra", "created": 1790000000 },
                { "id": "gpt-4.1", "created": 1744316542, "shutdown_date": null },
                { "id": "gpt-4o-2024-08-06", "created": 1722814719 },
                { "id": "gpt-4o-mini-tts", "created": 1 },
                { "id": "gpt-4o-realtime-preview", "created": 1 },
                { "id": "gpt-realtime-2", "created": 1 },
                { "id": "gpt-live-1", "created": 1 },
                { "id": "gpt-image-1", "created": 1 },
                { "id": "gpt-oss-120b", "created": 1 },
                { "id": "gpt-transcribe", "created": 1 },
                { "id": "text-embedding-3-large", "created": 1 },
                { "id": "omni-moderation-latest", "created": 1 },
                { "id": "o4-mini", "created": 1744225351, "shutdown_date": "2026-10-23" },
                { "id": "o3", "created": 1744225300, "shutdown_date": "2999-12-11" },
                { "id": "o3-deep-research", "created": 1 },
                { "id": "chatgpt-4o-latest", "created": 1723515131 },
                { "id": "whisper-1", "created": 1 },
                { "id": "gpt-3.5-turbo-instruct", "created": 1 }
            ]
        });
        assert_eq!(
            ids(&parse_openai(&body)),
            vec!["gpt-6.2-astra", "gpt-4.1", "o3", "chatgpt-4o-latest"]
        );
    }

    #[test]
    fn google_needs_generate_content_and_strips_the_prefix() {
        let body = json!({
            "models": [
                { "name": "models/gemini-2.5-pro", "displayName": "Gemini 2.5 Pro",
                  "supportedGenerationMethods": ["generateContent", "countTokens"] },
                { "name": "models/gemini-embedding-001", "displayName": "Gemini Embedding",
                  "supportedGenerationMethods": ["embedContent"] },
                { "name": "models/gemini-2.5-flash-image", "displayName": "Image",
                  "supportedGenerationMethods": ["generateContent"] },
                { "name": "models/gemini-2.5-flash-preview-tts", "displayName": "TTS",
                  "supportedGenerationMethods": ["generateContent"] },
                { "name": "models/gemini-omni-1", "displayName": "Omni",
                  "supportedGenerationMethods": ["generateContent"] },
                { "name": "models/gemini-nano-banana-2", "displayName": "Nano Banana",
                  "supportedGenerationMethods": ["generateContent"] },
                { "name": "models/gemma-3-27b-it", "displayName": "Gemma",
                  "supportedGenerationMethods": ["generateContent"] },
                { "name": "models/imagen-4.0-generate-001", "displayName": "Imagen",
                  "supportedGenerationMethods": ["predict"] }
            ]
        });
        let models = parse_google(&body);
        assert_eq!(ids(&models), vec!["gemini-2.5-pro"]);
        assert_eq!(models[0].name, "Gemini 2.5 Pro");
    }

    #[test]
    fn xai_drops_image_and_voice_models() {
        let body = json!({
            "data": [
                { "id": "grok-4.7", "created": 1790000000 },
                { "id": "grok-2-image-1212", "created": 1 },
                { "id": "grok-imagine-video", "created": 1 },
                { "id": "grok-voice-1", "created": 1 }
            ]
        });
        assert_eq!(ids(&parse_xai(&body)), vec!["grok-4.7"]);
    }

    #[test]
    fn deepseek_reads_names_and_skips_non_text_models() {
        let body = json!({
            "object": "list",
            "data": [
                { "id": "deepseek-v4-pro", "name": "DeepSeek-V4-Pro",
                  "output_modalities": ["text"] },
                { "id": "deepseek-chat", "object": "model", "owned_by": "deepseek" },
                { "id": "deepseek-voice", "output_modalities": ["audio"] }
            ]
        });
        let models = parse_deepseek(&body);
        assert_eq!(ids(&models), vec!["deepseek-v4-pro", "deepseek-chat"]);
        assert_eq!(models[0].name, "DeepSeek-V4-Pro");
        assert_eq!(models[1].name, "deepseek-chat");
    }

    #[test]
    fn mistral_shows_each_chat_model_once() {
        let chat = json!({ "completion_chat": true });
        let body = json!({
            "object": "list",
            "data": [
                { "id": "mistral-large-4-0", "created": 5, "type": "base", "capabilities": chat,
                  "aliases": ["mistral-large-4", "mistral-large-latest"] },
                { "id": "mistral-large-4", "created": 5, "type": "base", "capabilities": chat,
                  "aliases": ["mistral-large-4-0", "mistral-large-latest"] },
                { "id": "mistral-large-latest", "created": 5, "type": "base", "capabilities": chat,
                  "aliases": ["mistral-large-4-0", "mistral-large-4"] },
                { "id": "ministral-8b-2512", "created": 4, "type": "base", "capabilities": chat,
                  "aliases": [] },
                { "id": "mistral-embed", "created": 5,
                  "capabilities": { "completion_chat": false } },
                { "id": "mistral-ocr-latest", "created": 5,
                  "capabilities": { "completion_chat": true, "ocr": true } },
                { "id": "ft:mistral-small:abc", "created": 5, "type": "fine-tuned",
                  "capabilities": chat },
                { "id": "mistral-medium-old", "created": 5, "capabilities": chat,
                  "deprecation": "2020-01-01T00:00:00Z" }
            ]
        });
        assert_eq!(
            ids(&parse_mistral(&body)),
            vec!["mistral-large-latest", "ministral-8b-2512"]
        );
    }

    #[test]
    fn together_accepts_a_bare_array_and_keeps_chat_type() {
        let body = json!([
            { "id": "moonshotai/Kimi-K3.1-Pro", "type": "chat",
              "display_name": "Kimi K3.1 Pro", "created": 0 },
            { "id": "black-forest-labs/FLUX.1-schnell", "type": "image",
              "display_name": "FLUX.1 Schnell", "created": 0 },
            { "id": "BAAI/bge-large-en-v1.5", "type": "embedding", "created": 0 }
        ]);
        let models = parse_together(&body);
        assert_eq!(ids(&models), vec!["moonshotai/Kimi-K3.1-Pro"]);
        assert_eq!(models[0].name, "Kimi K3.1 Pro");
        assert_eq!(models[0].created, None);

        let wrapped = json!({ "data": body });
        assert_eq!(
            ids(&parse_together(&wrapped)),
            vec!["moonshotai/Kimi-K3.1-Pro"]
        );
    }

    #[test]
    fn cohere_keeps_live_chat_models() {
        let body = json!({
            "models": [
                { "name": "command-a-03-2025", "endpoints": ["chat"], "is_deprecated": false },
                { "name": "command-r", "endpoints": ["chat"], "is_deprecated": true },
                { "name": "my-tuned-command", "endpoints": ["chat"], "finetuned": true },
                { "name": "embed-v4.0", "endpoints": ["embed"] },
                { "name": "rerank-v3.5", "endpoints": ["rerank"] }
            ]
        });
        assert_eq!(ids(&parse_cohere(&body)), vec!["command-a-03-2025"]);
    }

    #[test]
    fn id_helpers_recognise_dates_and_families() {
        assert!(ends_with_date("gpt-4o-2024-08-06"));
        assert!(!ends_with_date("gpt-4.1-mini"));
        assert!(!ends_with_date("o4-mini"));
        assert!(starts_with_then_digit("o3", "o"));
        assert!(!starts_with_then_digit("omni-moderation-latest", "o"));
        assert!(!starts_with_then_digit("gpt-image-1", "gpt-"));
        assert!(date_has_passed(&json!("2020-01-01")));
        assert!(!date_has_passed(&json!("2999-01-01T00:00:00Z")));
        assert!(!date_has_passed(&json!(null)));
        assert!(!date_has_passed(&json!("soon")));
    }

    #[test]
    fn dedupe_keeps_the_first_of_each_id() {
        let models = vec![
            RemoteModel::new("a", Some("First"), None),
            RemoteModel::new("b", None, None),
            RemoteModel::new("a", Some("Second"), None),
        ];
        let out = dedupe(models);
        assert_eq!(ids(&out), vec!["a", "b"]);
        assert_eq!(out[0].name, "First");
    }

    #[test]
    fn an_unexpected_shape_gives_an_empty_list() {
        let body = json!({ "error": { "message": "nope" } });
        assert!(parse_anthropic(&body).is_empty());
        assert!(parse_openai(&body).is_empty());
        assert!(parse_google(&body).is_empty());
        assert!(parse_mistral(&body).is_empty());
        assert!(parse_together(&body).is_empty());
        assert!(parse_cohere(&body).is_empty());
    }
}
