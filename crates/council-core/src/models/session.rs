use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use uuid::Uuid;

use super::config::{MasterModelConfig, ModelConfig, SystemPromptMode, UsageData};

#[derive(Debug, Clone, Default, Serialize, Deserialize, PartialEq)]
#[serde(rename_all = "snake_case")]
pub enum SessionType {
    #[default]
    Council,
    DirectChat,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct DirectChatMessage {
    pub role: String,
    pub content: String,
    pub timestamp: DateTime<Utc>,
    #[serde(default)]
    #[serde(skip_serializing_if = "Option::is_none")]
    pub usage: Option<UsageData>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct DirectChatAgent {
    pub provider: String,
    pub model: String,
    pub display_name: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ClarifyingExchange {
    pub question: String,
    pub answer: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all_fields = "camelCase")]
#[serde(tag = "role")]
pub enum DiscussionEntry {
    #[serde(rename = "user")]
    User { content: String },
    #[serde(rename = "model")]
    Model {
        provider: String,
        model: String,
        display_name: String,
        #[serde(default)]
        system_prompt: Option<String>,
        content: String,
        #[serde(default)]
        #[serde(skip_serializing_if = "Option::is_none")]
        clarifying_exchange: Option<Vec<ClarifyingExchange>>,
        #[serde(default)]
        #[serde(skip_serializing_if = "Option::is_none")]
        usage: Option<UsageData>,
    },
    #[serde(rename = "master_verdict")]
    MasterVerdict {
        provider: String,
        model: String,
        content: String,
        #[serde(default)]
        #[serde(skip_serializing_if = "Option::is_none")]
        usage: Option<UsageData>,
    },
    #[serde(rename = "follow_up_question")]
    FollowUpQuestion {
        content: String,
        target_provider: String,
        target_model: String,
        target_display_name: String,
    },
    #[serde(rename = "follow_up_answer")]
    FollowUpAnswer {
        provider: String,
        model: String,
        display_name: String,
        content: String,
        #[serde(default)]
        #[serde(skip_serializing_if = "Option::is_none")]
        usage: Option<UsageData>,
    },
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct CouncilConfig {
    pub models: Vec<ModelConfig>,
    pub master_model: MasterModelConfig,
    pub system_prompt_mode: SystemPromptMode,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Session {
    pub id: String,
    pub title: String,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
    pub user_question: String,
    pub council_config: CouncilConfig,
    pub discussion: Vec<DiscussionEntry>,
    #[serde(default)]
    pub session_type: SessionType,
    #[serde(default)]
    #[serde(skip_serializing_if = "Option::is_none")]
    pub direct_chat_agent: Option<DirectChatAgent>,
    #[serde(default)]
    #[serde(skip_serializing_if = "Option::is_none")]
    pub direct_chat_messages: Option<Vec<DirectChatMessage>>,
}

#[allow(dead_code)]
impl Session {
    pub fn new(
        user_question: String,
        council_config: CouncilConfig,
    ) -> Self {
        let now = Utc::now();
        Self {
            id: Uuid::new_v4().to_string(),
            title: truncate_title(&user_question),
            created_at: now,
            updated_at: now,
            user_question,
            council_config,
            discussion: vec![],
            session_type: SessionType::Council,
            direct_chat_agent: None,
            direct_chat_messages: None,
        }
    }
}

#[allow(dead_code)]
fn truncate_title(question: &str) -> String {
    let trimmed = question.trim();
    if trimmed.len() <= 60 {
        trimmed.to_string()
    } else {
        format!("{}...", &trimmed[..57])
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct SessionSummary {
    pub id: String,
    pub title: String,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
    #[serde(default)]
    pub session_type: SessionType,
}
