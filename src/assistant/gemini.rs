use gloo_timers::future::TimeoutFuture;
use wasm_bindgen::{JsCast, prelude::*};
use web_sys::{HtmlButtonElement, HtmlElement, InputEvent, InputEventInit};

use super::prompt;
use crate::{
    chrome_api,
    messages::{ExtensionMessage, MessageResponse},
};

const MESSAGE_TIMEOUT_MS: f64 = 30_000.0;

fn js_err(error: JsValue) -> String {
    format!("{error:?}")
}

async fn submit_to_gemini(prompt: &str) -> Result<(), String> {
    let tab_id = chrome_api::create_gemini_tab().await?;
    let request = ExtensionMessage::GeminiSubmit {
        prompt: prompt.to_string(),
    };
    let deadline = js_sys::Date::now() + MESSAGE_TIMEOUT_MS;
    let mut last_error = String::new();

    while js_sys::Date::now() < deadline {
        match chrome_api::send_tab_message::<_, MessageResponse>(tab_id, &request).await {
            Ok(response) => {
                if response.ok {
                    return Ok(());
                }
                return Err(response
                    .error
                    .unwrap_or_else(|| "Gemini did not accept the message".to_string()));
            }
            Err(error) => {
                last_error = error;
                TimeoutFuture::new(500).await;
            }
        }
    }

    Err(format!("Gemini did not become ready in time: {last_error}"))
}

fn find_gemini_editor() -> Result<Option<HtmlElement>, String> {
    let document = web_sys::window()
        .and_then(|window| window.document())
        .ok_or_else(|| "Could not access the Gemini document".to_string())?;

    for selector in [
        r#"[contenteditable="true"][role="textbox"][aria-label="Введіть запит для Gemini"]"#,
        r#"[contenteditable="true"][role="textbox"]"#,
    ] {
        if let Some(element) = document.query_selector(selector).map_err(js_err)? {
            return element
                .dyn_into::<HtmlElement>()
                .map(Some)
                .map_err(|_| "Gemini editor is not an HTML element".to_string());
        }
    }

    Ok(None)
}

async fn wait_for_gemini_editor(timeout_ms: f64) -> Result<HtmlElement, String> {
    let deadline = js_sys::Date::now() + timeout_ms;
    while js_sys::Date::now() < deadline {
        if let Some(editor) = find_gemini_editor()? {
            return Ok(editor);
        }
        TimeoutFuture::new(200).await;
    }
    Err("Gemini chat editor did not appear".to_string())
}

fn find_gemini_send_button() -> Result<Option<HtmlButtonElement>, String> {
    let document = web_sys::window()
        .and_then(|window| window.document())
        .ok_or_else(|| "Could not access the Gemini document".to_string())?;

    let button = match document
        .query_selector(r#"button[aria-label="Надіслати повідомлення"]"#)
        .map_err(js_err)?
    {
        Some(button) => Some(button),
        None => document
            .query_selector(r#"mat-icon[data-mat-icon-name="arrow_upward"]"#)
            .map_err(js_err)?
            .map(|icon| icon.closest("button").map_err(js_err))
            .transpose()?
            .flatten(),
    };

    button
        .map(|button| {
            button
                .dyn_into::<HtmlButtonElement>()
                .map_err(|_| "Gemini send control is not a button".to_string())
        })
        .transpose()
}

async fn wait_for_gemini_send_button(timeout_ms: f64) -> Result<HtmlButtonElement, String> {
    let deadline = js_sys::Date::now() + timeout_ms;
    while js_sys::Date::now() < deadline {
        if let Some(button) = find_gemini_send_button()? {
            let aria_disabled = button.get_attribute("aria-disabled");
            if !button.disabled() && aria_disabled.as_deref() != Some("true") {
                return Ok(button);
            }
        }
        TimeoutFuture::new(100).await;
    }
    Err("Gemini send button did not become enabled".to_string())
}

async fn submit_prompt_in_gemini_page(prompt: &str) -> Result<(), String> {
    let hostname = web_sys::window()
        .ok_or_else(|| "Could not access the Gemini window".to_string())?
        .location()
        .hostname()
        .map_err(js_err)?;
    if hostname != "gemini.google.com" {
        return Err("The message was delivered outside Gemini".to_string());
    }

    let editor = wait_for_gemini_editor(30_000.0).await?;
    editor.set_text_content(Some(prompt));

    let input_init = InputEventInit::new();
    input_init.set_bubbles(true);
    input_init.set_input_type("insertText");
    input_init.set_data(Some(prompt));
    let input_event = InputEvent::new_with_event_init_dict("input", &input_init).map_err(js_err)?;
    editor.dispatch_event(&input_event).map_err(js_err)?;

    let send_button = wait_for_gemini_send_button(10_000.0).await?;
    send_button.click();
    Ok(())
}

pub(crate) async fn submit_gemini_prompt(prompt: String) -> Result<(), String> {
    submit_prompt_in_gemini_page(&prompt).await
}

pub(crate) async fn send_to_gemini(
    html: String,
    page_url: String,
    prompt_template: String,
) -> Result<(), String> {
    let prompt = prompt::build(&prompt_template, &html, &page_url);
    submit_to_gemini(&prompt).await
}
