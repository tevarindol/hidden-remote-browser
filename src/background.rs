use js_sys::Function;
use wasm_bindgen::prelude::*;
use wasm_bindgen_futures::spawn_local;

use crate::{
    chrome_api,
    config,
    messages::{ExtensionMessage, MessageResponse},
};

async fn handle_captured_page(html: String, sender: JsValue) -> Result<(), String> {
    let settings = config::load().await?;
    let tab = chrome_api::tab_context(&sender)?;

    let telegram_configured = !settings.telegram_token.is_empty() && !settings.chat_id.is_empty();
    let screenshot = if telegram_configured && settings.send_photo {
        match chrome_api::capture_visible_tab(tab.window_id).await {
            Ok(screenshot) => Some(screenshot),
            Err(error) => {
                log_error("screenshot capture failed", &error);
                None
            }
        }
    } else {
        None
    };

    let ai_provider = settings.ai_provider.clone();
    let ai_html = html.clone();
    let ai_url = tab.url;
    let prompt_template = settings.prompt_template.clone();
    let assistant_future = async move {
        if ai_provider == config::AI_PROVIDER_CLAUDE {
            crate::assistant::send_to_claude(ai_html, ai_url, prompt_template).await
        } else {
            crate::assistant::send_to_gemini(ai_html, ai_url, prompt_template).await
        }
    };
    let telegram_settings = settings.clone();
    let telegram_html = html;
    let telegram_future = async move {
        if telegram_configured {
            crate::telegram::send(
                &telegram_settings,
                &telegram_html,
                screenshot.as_deref(),
            )
            .await
        } else {
            Ok(())
        }
    };
    let (assistant_result, telegram_result) = futures::join!(assistant_future, telegram_future);

    if let Err(error) = assistant_result {
        log_error("AI service submission failed", &error);
    }
    if let Err(error) = telegram_result {
        log_error("Telegram submission failed", &error);
    }
    Ok(())
}

fn log_error(context: &str, error: &str) {
    web_sys::console::error_1(&format!("hidden-remote-browser: {context}: {error}").into());
}

fn respond(send_response: &Function, response: &MessageResponse) {
    if let Ok(value) = serde_wasm_bindgen::to_value(response) {
        let _ = send_response.call1(&JsValue::UNDEFINED, &value);
    }
}

pub fn handle_message(message: JsValue, sender: JsValue, send_response: Function) -> bool {
    let Ok(event) = serde_wasm_bindgen::from_value::<ExtensionMessage>(message) else {
        return false;
    };
    let ExtensionMessage::PageCaptured { html } = event else {
        return false;
    };

    spawn_local(async move {
        match handle_captured_page(html, sender).await {
            Ok(()) => respond(
                &send_response,
                &MessageResponse {
                    ok: true,
                    error: None,
                },
            ),
            Err(error) => {
                log_error("background handler failed", &error);
                respond(
                    &send_response,
                    &MessageResponse {
                        ok: false,
                        error: Some(error),
                    },
                );
            }
        }
    });
    true
}
