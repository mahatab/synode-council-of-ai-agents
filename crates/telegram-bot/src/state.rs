use std::collections::HashMap;
use std::sync::Arc;
use tokio::sync::RwLock;

use council_core::keychain::ApiKeyCache;
use council_core::models::config::{AppSettings, ModelConfig, Provider};
use council_core::models::session::{DirectChatAgent, DirectChatMessage, Session};

/// Per-chat conversation state tracked by the bot.
#[derive(Debug, Clone)]
pub enum ChatMode {
    /// No active conversation.
    Idle,
    /// Council discussion in progress (cannot accept new commands).
    CouncilActive,
    /// First model asked a clarifying question; waiting for user's reply.
    CouncilWaitingClarification {
        /// Partial session accumulated so far.
        session: Session,
        /// Discussion entries collected so far.
        discussion_json: String,
        /// The clarifying question content.
        clarifying_content: String,
        /// Model that asked the question.
        model: ModelConfig,
        /// System prompt used for the model.
        system_prompt: String,
        /// Messages sent to the model before its response.
        messages_json: String,
    },
    /// Direct chat session active; replies continue the conversation.
    DirectChat {
        session: Session,
        agent: DirectChatAgent,
        messages: Vec<DirectChatMessage>,
    },
}

/// Shared application state across all Telegram handlers.
#[derive(Clone)]
pub struct AppState {
    pub keychain: Arc<ApiKeyCache>,
    pub settings: Arc<RwLock<AppSettings>>,
    pub chats: Arc<RwLock<HashMap<i64, ChatMode>>>,
}

impl AppState {
    pub fn new() -> Self {
        let settings = council_core::settings::load_settings().unwrap_or_default();

        Self {
            keychain: Arc::new(ApiKeyCache::default()),
            settings: Arc::new(RwLock::new(settings)),
            chats: Arc::new(RwLock::new(HashMap::new())),
        }
    }

    /// Get an API key for the given provider name (e.g., "anthropic").
    pub fn get_api_key(&self, provider: &str) -> Option<String> {
        let service = format!("com.council-of-ai-agents.{}", provider);
        council_core::keychain::get_api_key(&self.keychain, &service)
            .ok()
            .flatten()
    }

    /// Get an API key for a Provider enum value.
    pub fn get_api_key_for_provider(&self, provider: &Provider) -> Option<String> {
        let name = provider_to_string(provider);
        self.get_api_key(&name)
    }

    /// Set the chat mode for a given chat_id.
    pub async fn set_chat_mode(&self, chat_id: i64, mode: ChatMode) {
        let mut chats = self.chats.write().await;
        chats.insert(chat_id, mode);
    }

    /// Get the current chat mode (defaults to Idle).
    pub async fn get_chat_mode(&self, chat_id: i64) -> ChatMode {
        let chats = self.chats.read().await;
        chats.get(&chat_id).cloned().unwrap_or(ChatMode::Idle)
    }

    /// Clear chat mode back to Idle.
    pub async fn clear_chat_mode(&self, chat_id: i64) {
        let mut chats = self.chats.write().await;
        chats.remove(&chat_id);
    }
}

/// Convert a Provider enum to its lowercase string name.
pub fn provider_to_string(provider: &Provider) -> String {
    match provider {
        Provider::Anthropic => "anthropic",
        Provider::OpenAI => "openai",
        Provider::Google => "google",
        Provider::XAI => "xai",
        Provider::DeepSeek => "deepseek",
        Provider::Mistral => "mistral",
        Provider::Together => "together",
        Provider::Cohere => "cohere",
    }
    .to_string()
}

/// Find a model by a fuzzy name match.
pub fn find_model_by_name<'a>(models: &'a [ModelConfig], name: &str) -> Option<&'a ModelConfig> {
    let lower = name.to_lowercase();
    // Try exact model ID match first
    models
        .iter()
        .find(|m| m.model.to_lowercase() == lower)
        // Then try display name contains
        .or_else(|| {
            models
                .iter()
                .find(|m| m.display_name.to_lowercase().contains(&lower))
        })
        // Then try provider name
        .or_else(|| {
            models
                .iter()
                .find(|m| provider_to_string(&m.provider) == lower)
        })
}
