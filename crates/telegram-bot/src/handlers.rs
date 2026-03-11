use teloxide::prelude::*;
use teloxide::types::Me;
use teloxide::utils::command::BotCommands;

use crate::council;
use crate::direct_chat;
use crate::state::{AppState, ChatMode};

#[derive(BotCommands, Clone)]
#[command(rename_rule = "lowercase", description = "Available commands:")]
pub enum Command {
    #[command(description = "Welcome message and available commands")]
    Start,
    #[command(description = "Start a council discussion — /council <question>")]
    Council(String),
    #[command(description = "Direct chat with a model — /chat <model> <message>")]
    Chat(String),
    #[command(description = "List available models")]
    Models,
    #[command(description = "List recent sessions")]
    Sessions,
    #[command(description = "Show current settings")]
    Settings,
    #[command(description = "Cancel ongoing discussion or chat")]
    Stop,
    #[command(description = "Show help")]
    Help,
}

/// Handle slash commands.
pub async fn handle_command(
    bot: Bot,
    msg: Message,
    cmd: Command,
    app_state: AppState,
) -> ResponseResult<()> {
    let chat_id = msg.chat.id;

    match cmd {
        Command::Start => {
            let text = "Welcome to the Council of AI Agents!\n\n\
                I can help you get opinions from multiple AI models on any question.\n\n\
                Commands:\n\
                /council <question> — Start a council discussion\n\
                /chat <model> <message> — Direct chat with a model\n\
                /models — List available models\n\
                /sessions — List recent sessions\n\
                /settings — Show current settings\n\
                /stop — Cancel ongoing discussion\n\
                /help — Show this help";
            bot.send_message(chat_id, text).await?;
        }

        Command::Council(question) => {
            if question.trim().is_empty() {
                bot.send_message(chat_id, "Usage: /council <your question>")
                    .await?;
                return Ok(());
            }

            let mode = app_state.get_chat_mode(chat_id.0).await;
            if matches!(mode, ChatMode::CouncilActive) {
                bot.send_message(
                    chat_id,
                    "A council discussion is already in progress. Use /stop to cancel it first.",
                )
                .await?;
                return Ok(());
            }

            let bot_clone = bot.clone();
            let state_clone = app_state.clone();
            tokio::spawn(async move {
                if let Err(e) = council::run_council(&bot_clone, chat_id, &question, &state_clone).await {
                    log::error!("Council error: {}", e);
                    let _ = bot_clone
                        .send_message(chat_id, format!("Council discussion failed: {}", e))
                        .await;
                    state_clone.clear_chat_mode(chat_id.0).await;
                }
            });
        }

        Command::Chat(args) => {
            let parts: Vec<&str> = args.splitn(2, ' ').collect();
            if parts.len() < 2 || parts[0].trim().is_empty() || parts[1].trim().is_empty() {
                bot.send_message(
                    chat_id,
                    "Usage: /chat <model> <message>\nExample: /chat claude-sonnet Hello!",
                )
                .await?;
                return Ok(());
            }

            let model_name = parts[0].trim();
            let message = parts[1].trim();

            let mode = app_state.get_chat_mode(chat_id.0).await;
            if matches!(mode, ChatMode::CouncilActive) {
                bot.send_message(
                    chat_id,
                    "A council discussion is in progress. Use /stop first.",
                )
                .await?;
                return Ok(());
            }

            let bot_clone = bot.clone();
            let state_clone = app_state.clone();
            let model_name = model_name.to_string();
            let message = message.to_string();
            tokio::spawn(async move {
                if let Err(e) = direct_chat::start_direct_chat(
                    &bot_clone, chat_id, &model_name, &message, &state_clone,
                )
                .await
                {
                    log::error!("Direct chat error: {}", e);
                    let _ = bot_clone
                        .send_message(chat_id, format!("Direct chat failed: {}", e))
                        .await;
                }
            });
        }

        Command::Models => {
            let settings = app_state.settings.read().await;
            if settings.council_models.is_empty() {
                bot.send_message(chat_id, "No models configured. Set up models in the desktop app.")
                    .await?;
            } else {
                let mut text = String::from("Configured council models:\n\n");
                for (i, m) in settings.council_models.iter().enumerate() {
                    text.push_str(&format!(
                        "{}. {} ({} / {})\n",
                        i + 1,
                        m.display_name,
                        m.provider.display_name(),
                        m.model,
                    ));
                }
                text.push_str(&format!(
                    "\nMaster model: {} ({})",
                    settings.master_model.model,
                    settings.master_model.provider.display_name(),
                ));
                bot.send_message(chat_id, text).await?;
            }
        }

        Command::Sessions => {
            match council_core::sessions::list_sessions(None) {
                Ok(sessions) => {
                    if sessions.is_empty() {
                        bot.send_message(chat_id, "No sessions found.").await?;
                    } else {
                        let mut text = String::from("Recent sessions:\n\n");
                        for (i, s) in sessions.iter().take(10).enumerate() {
                            let type_label = match s.session_type {
                                council_core::models::session::SessionType::Council => "Council",
                                council_core::models::session::SessionType::DirectChat => "Chat",
                            };
                            text.push_str(&format!(
                                "{}. [{}] {} ({})\n",
                                i + 1,
                                type_label,
                                s.title,
                                s.created_at.format("%Y-%m-%d %H:%M"),
                            ));
                        }
                        bot.send_message(chat_id, text).await?;
                    }
                }
                Err(e) => {
                    bot.send_message(chat_id, format!("Failed to list sessions: {}", e))
                        .await?;
                }
            }
        }

        Command::Settings => {
            let settings = app_state.settings.read().await;
            let text = format!(
                "Current settings:\n\n\
                Council models: {}\n\
                Master model: {} ({})\n\
                System prompt mode: {:?}\n\
                Discussion depth: {:?}\n\
                Discussion style: {:?}",
                settings.council_models.len(),
                settings.master_model.model,
                settings.master_model.provider.display_name(),
                settings.system_prompt_mode,
                settings.discussion_depth,
                settings.discussion_style,
            );
            bot.send_message(chat_id, text).await?;
        }

        Command::Stop => {
            let mode = app_state.get_chat_mode(chat_id.0).await;
            match mode {
                ChatMode::Idle => {
                    bot.send_message(chat_id, "Nothing active to stop.")
                        .await?;
                }
                _ => {
                    app_state.clear_chat_mode(chat_id.0).await;
                    bot.send_message(chat_id, "Stopped. Ready for new commands.")
                        .await?;
                }
            }
        }

        Command::Help => {
            let text = Command::descriptions().to_string();
            bot.send_message(chat_id, text).await?;
        }
    }

    Ok(())
}

/// Handle free-text messages (not slash commands).
/// Routes to clarification answers or direct chat continuations.
pub async fn handle_message(
    bot: Bot,
    msg: Message,
    _me: Me,
    app_state: AppState,
) -> ResponseResult<()> {
    let chat_id = msg.chat.id;
    let text = match msg.text() {
        Some(t) => t.to_string(),
        None => return Ok(()),
    };

    let mode = app_state.get_chat_mode(chat_id.0).await;

    match mode {
        ChatMode::CouncilWaitingClarification {
            session,
            discussion_json,
            clarifying_content,
            model,
            system_prompt,
            messages_json,
        } => {
            let bot_clone = bot.clone();
            let state_clone = app_state.clone();
            tokio::spawn(async move {
                if let Err(e) = council::resume_after_clarification(
                    &bot_clone,
                    chat_id,
                    &text,
                    session,
                    &discussion_json,
                    &clarifying_content,
                    &model,
                    &system_prompt,
                    &messages_json,
                    &state_clone,
                )
                .await
                {
                    log::error!("Resume council error: {}", e);
                    let _ = bot_clone
                        .send_message(chat_id, format!("Failed to resume council: {}", e))
                        .await;
                    state_clone.clear_chat_mode(chat_id.0).await;
                }
            });
        }

        ChatMode::DirectChat {
            session,
            agent,
            messages,
        } => {
            let bot_clone = bot.clone();
            let state_clone = app_state.clone();
            tokio::spawn(async move {
                if let Err(e) = direct_chat::continue_direct_chat(
                    &bot_clone, chat_id, &text, session, agent, messages, &state_clone,
                )
                .await
                {
                    log::error!("Direct chat continuation error: {}", e);
                    let _ = bot_clone
                        .send_message(chat_id, format!("Chat error: {}", e))
                        .await;
                }
            });
        }

        ChatMode::CouncilActive => {
            bot.send_message(
                chat_id,
                "Council discussion in progress. Please wait or use /stop to cancel.",
            )
            .await?;
        }

        ChatMode::Idle => {
            bot.send_message(
                chat_id,
                "No active conversation. Use /council or /chat to start one, or /help for commands.",
            )
            .await?;
        }
    }

    Ok(())
}
