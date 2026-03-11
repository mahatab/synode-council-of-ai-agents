use std::sync::Arc;
use tauri::{command, State};
use tokio::sync::{oneshot, Mutex};

pub struct BotHandleInner {
    pub shutdown_tx: Option<oneshot::Sender<()>>,
    pub task: tauri::async_runtime::JoinHandle<()>,
}

pub struct TelegramBotState {
    handle: Arc<Mutex<Option<BotHandleInner>>>,
}

impl TelegramBotState {
    /// Expose handle for auto-start in setup hook.
    pub fn clone_handle(&self) -> Arc<Mutex<Option<BotHandleInner>>> {
        Arc::clone(&self.handle)
    }
}

impl Default for TelegramBotState {
    fn default() -> Self {
        Self {
            handle: Arc::new(Mutex::new(None)),
        }
    }
}

#[derive(serde::Serialize)]
#[serde(rename_all = "camelCase")]
pub struct TelegramBotStatus {
    pub running: bool,
}

#[command]
pub async fn start_telegram_bot(
    token: String,
    state: State<'_, TelegramBotState>,
) -> Result<(), String> {
    let mut handle = state.handle.lock().await;

    // If already running, stop first
    if let Some(bot) = handle.take() {
        if let Some(tx) = bot.shutdown_tx {
            let _ = tx.send(());
        }
        let _ = bot.task.await;
    }

    let (shutdown_tx, shutdown_rx) = oneshot::channel();

    let task = tauri::async_runtime::spawn(async move {
        if let Err(e) = council_telegram_bot::start_bot(token, shutdown_rx).await {
            log::error!("Telegram bot error: {}", e);
        }
    });

    *handle = Some(BotHandleInner {
        shutdown_tx: Some(shutdown_tx),
        task,
    });

    log::info!("Telegram bot started");
    Ok(())
}

#[command]
pub async fn stop_telegram_bot(state: State<'_, TelegramBotState>) -> Result<(), String> {
    let mut handle = state.handle.lock().await;

    if let Some(bot) = handle.take() {
        if let Some(tx) = bot.shutdown_tx {
            let _ = tx.send(());
        }
        let _ = bot.task.await;
        log::info!("Telegram bot stopped");
    }

    Ok(())
}

#[command]
pub async fn get_telegram_status(
    state: State<'_, TelegramBotState>,
) -> Result<TelegramBotStatus, String> {
    let handle = state.handle.lock().await;
    let running = handle.is_some();
    Ok(TelegramBotStatus { running })
}
