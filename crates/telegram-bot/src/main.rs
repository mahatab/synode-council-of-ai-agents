#[tokio::main]
async fn main() {
    env_logger::init();
    log::info!("Starting Council of AI Agents Telegram Bot...");

    let token = std::env::var("TELOXIDE_TOKEN")
        .expect("TELOXIDE_TOKEN environment variable not set");

    // Create a shutdown channel; for standalone mode, we never send it
    // (the process exits on Ctrl+C naturally).
    let (_shutdown_tx, shutdown_rx) = tokio::sync::oneshot::channel::<()>();

    if let Err(e) = council_telegram_bot::start_bot(token, shutdown_rx).await {
        log::error!("Bot error: {}", e);
        std::process::exit(1);
    }
}
