mod app;
mod config;
mod telegram;

use wasm_bindgen::prelude::*;

#[wasm_bindgen(js_name = mountOptions)]
pub fn mount_options() {
    console_error_panic_hook::set_once();
    leptos::mount::mount_to_body(app::App);
}

#[wasm_bindgen(js_name = sendToTelegram)]
pub async fn send_to_telegram(
    html: String,
    settings: JsValue,
    screenshot: Option<String>,
) -> Result<String, String> {
    console_error_panic_hook::set_once();
    let settings: config::Settings =
        serde_wasm_bindgen::from_value(settings).map_err(|e| e.to_string())?;
    if settings.telegram_token.is_empty() || settings.chat_id.is_empty() {
        return Err("telegram bot token and chat id are required".to_string());
    }
    telegram::send(&settings, &html, screenshot.as_deref()).await?;
    Ok("sent".to_string())
}
