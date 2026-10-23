use std::future::Future;

use js_sys::Function;
use wasm_bindgen::{closure::Closure, prelude::*, JsCast};
use wasm_bindgen_futures::spawn_local;
use web_sys::Document;

use crate::{
    chrome_api,
    config,
    messages::{ExtensionMessage, MessageResponse},
};

fn respond(send_response: &Function, value: &JsValue) {
    let _ = send_response.call1(&JsValue::UNDEFINED, value);
}

fn respond_with_result<F>(send_response: Function, result: F)
where
    F: Future<Output = Result<(), String>> + 'static,
{
    spawn_local(async move {
        let response = match result.await {
            Ok(()) => MessageResponse {
                ok: true,
                error: None,
            },
            Err(error) => MessageResponse {
                ok: false,
                error: Some(error),
            },
        };
        if let Ok(value) = serde_wasm_bindgen::to_value(&response) {
            respond(&send_response, &value);
        }
    });
}

fn handle_message(document: &Document) -> Closure<dyn FnMut(JsValue, JsValue, Function) -> bool> {
    let document = document.clone();
    Closure::wrap(Box::new(
        move |message: JsValue, _sender: JsValue, send_response: Function| -> bool {
            let Ok(message) = serde_wasm_bindgen::from_value::<ExtensionMessage>(message) else {
                return false;
            };
            match message {
                ExtensionMessage::GeminiSubmit { prompt } => {
                    respond_with_result(
                        send_response,
                        crate::assistant::submit_gemini_prompt(prompt),
                    );
                    true
                }
                ExtensionMessage::ClaudeSubmit { prompt } => {
                    respond_with_result(
                        send_response,
                        crate::assistant::submit_claude_prompt(prompt),
                    );
                    true
                }
                ExtensionMessage::CapturePage => {
                    respond_with_result(send_response, capture_page(document.clone()));
                    true
                }
                ExtensionMessage::PageCaptured { .. } => false,
            }
        },
    ))
}

async fn capture_page(document: Document) -> Result<(), String> {
    let settings = config::load().await?;
    let element = document
        .query_selector(&settings.html_selector)
        .map_err(|error| format!("invalid element selector: {error:?}"))?
        .ok_or_else(|| {
            format!(
                "no element matches selector \"{}\"",
                settings.html_selector
            )
        })?;
    let message = ExtensionMessage::PageCaptured {
        html: element.outer_html(),
    };
    chrome_api::send_runtime_message(&message).await
}

pub fn start() {
    console_error_panic_hook::set_once();
    let Some(document) = web_sys::window().and_then(|window| window.document()) else {
        web_sys::console::error_1(&"hidden-remote-browser: document is unavailable".into());
        return;
    };

    let message_listener = handle_message(&document);
    chrome_api::add_runtime_message_listener(message_listener.as_ref().unchecked_ref());
    message_listener.forget();
}
