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

async fn submit_to_claude(prompt: &str) -> Result<(), String> {
    let tab_id = chrome_api::create_claude_tab().await?;
    let request = ExtensionMessage::ClaudeSubmit {
        prompt: prompt.to_string(),
    };
    let deadline = js_sys::Date::now() + MESSAGE_TIMEOUT_MS;
    let mut last_error = String::new();

    while js_sys::Date::now() < deadline {
        match chrome_api::send_tab_message::<_, MessageResponse>(tab_id, &request).await {
            Ok(response) if response.ok => return Ok(()),
            Ok(response) => {
                return Err(response
                    .error
                    .unwrap_or_else(|| "Claude did not accept the message".to_string()));
            }
            Err(error) => {
                last_error = error;
                TimeoutFuture::new(500).await;
            }
        }
    }

    Err(format!("Claude did not become ready in time: {last_error}"))
}

fn find_claude_editor() -> Result<Option<HtmlElement>, String> {
    let document = web_sys::window()
        .and_then(|window| window.document())
        .ok_or_else(|| "Could not access the Claude document".to_string())?;

    document
        .query_selector(r#"[data-testid="chat-input"][contenteditable="true"]"#)
        .map_err(js_err)?
        .map(|element| {
            element
                .dyn_into::<HtmlElement>()
                .map_err(|_| "Claude editor is not an HTML element".to_string())
        })
        .transpose()
}

async fn wait_for_claude_editor(timeout_ms: f64) -> Result<HtmlElement, String> {
    let deadline = js_sys::Date::now() + timeout_ms;
    while js_sys::Date::now() < deadline {
        if let Some(editor) = find_claude_editor()? {
            return Ok(editor);
        }
        TimeoutFuture::new(200).await;
    }
    Err("Claude chat editor did not appear".to_string())
}

fn find_claude_send_button() -> Result<Option<HtmlButtonElement>, String> {
    let document = web_sys::window()
        .and_then(|window| window.document())
        .ok_or_else(|| "Could not access the Claude document".to_string())?;

    document
        .query_selector(r#"button[data-testid="chat-input-send"]"#)
        .map_err(js_err)?
        .map(|button| {
            button
                .dyn_into::<HtmlButtonElement>()
                .map_err(|_| "Claude send control is not a button".to_string())
        })
        .transpose()
}

async fn wait_for_claude_send_button(timeout_ms: f64) -> Result<HtmlButtonElement, String> {
    let deadline = js_sys::Date::now() + timeout_ms;
    while js_sys::Date::now() < deadline {
        if let Some(button) = find_claude_send_button()? {
            let aria_disabled = button.get_attribute("aria-disabled");
            let trigger_disabled = button.has_attribute("data-trigger-disabled");
            if !button.disabled() && aria_disabled.as_deref() != Some("true") && !trigger_disabled {
                return Ok(button);
            }
        }
        TimeoutFuture::new(100).await;
    }
    Err("Claude send button did not become enabled".to_string())
}

async fn submit_prompt_in_claude_page(prompt: &str) -> Result<(), String> {
    let hostname = web_sys::window()
        .ok_or_else(|| "Could not access the Claude window".to_string())?
        .location()
        .hostname()
        .map_err(js_err)?;
    if hostname != "claude.ai" {
        return Err("The message was delivered outside Claude".to_string());
    }

    let editor = wait_for_claude_editor(30_000.0).await?;
    editor.focus().map_err(js_err)?;
    if let Some(paragraph) = editor.query_selector("p").map_err(js_err)? {
        paragraph.set_text_content(Some(prompt));
    } else {
        editor.set_text_content(Some(prompt));
    }

    let input_init = InputEventInit::new();
    input_init.set_bubbles(true);
    input_init.set_input_type("insertText");
    input_init.set_data(Some(prompt));
    let input_event = InputEvent::new_with_event_init_dict("input", &input_init).map_err(js_err)?;
    editor.dispatch_event(&input_event).map_err(js_err)?;

    let send_button = wait_for_claude_send_button(10_000.0).await?;
    send_button.click();
    Ok(())
}

pub(crate) async fn submit_claude_prompt(prompt: String) -> Result<(), String> {
    submit_prompt_in_claude_page(&prompt).await
}

pub(crate) async fn send_to_claude(
    html: String,
    page_url: String,
    prompt_template: String,
) -> Result<(), String> {
    let prompt = prompt::build(&prompt_template, &html, &page_url);
    submit_to_claude(&prompt).await
}
