#![allow(
    clippy::too_many_arguments,
    clippy::manual_strip,
    clippy::if_same_then_else,
    clippy::needless_range_loop,
    clippy::manual_find,
    clippy::collapsible_if
)]

pub mod council;
pub mod direct_chat;
pub mod formatting;
pub mod handlers;
pub mod state;

use handlers::Command;
use state::AppState;
use teloxide::prelude::*;

/// Start the Telegram bot with the given token.
///
/// The bot runs until `shutdown_rx` is signalled or the dispatcher stops.
/// Pass a `oneshot::Receiver` to control shutdown from the caller.
pub async fn start_bot(
    token: String,
    shutdown_rx: tokio::sync::oneshot::Receiver<()>,
) -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
    let bot = Bot::new(token);
    let app_state = AppState::new();

    log::info!("Bot initialized. Listening for commands...");

    let handler = Update::filter_message()
        .branch(
            dptree::entry()
                .filter_command::<Command>()
                .endpoint(handlers::handle_command),
        )
        .branch(dptree::entry().endpoint(handlers::handle_message));

    let mut dispatcher = Dispatcher::builder(bot, handler)
        .dependencies(dptree::deps![app_state])
        .build();

    let shutdown_token = dispatcher.shutdown_token();

    // Spawn a task that listens for the external shutdown signal
    tokio::spawn(async move {
        let _ = shutdown_rx.await;
        shutdown_token
            .shutdown()
            .expect("failed to shutdown dispatcher")
            .await;
    });

    dispatcher.dispatch().await;

    Ok(())
}
