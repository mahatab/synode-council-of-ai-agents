use tauri::{AppHandle, Manager, WebviewUrl, WebviewWindowBuilder};

#[tauri::command]
pub async fn open_about_window(app: AppHandle) -> Result<(), String> {
    // If the window already exists, just focus it
    if let Some(window) = app.get_webview_window("about") {
        window.set_focus().map_err(|e| e.to_string())?;
        return Ok(());
    }

    WebviewWindowBuilder::new(&app, "about", WebviewUrl::App("about.html".into()))
        .title("About Synode")
        .inner_size(400.0, 480.0)
        .resizable(false)
        .maximizable(false)
        .minimizable(false)
        .center()
        .build()
        .map_err(|e| e.to_string())?;

    Ok(())
}
