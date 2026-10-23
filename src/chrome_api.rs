use gloo_timers::future::TimeoutFuture;
use js_sys::{Function, Object, Promise, Reflect};
use serde::{Deserialize, Serialize, de::DeserializeOwned};
use wasm_bindgen::prelude::*;
use wasm_bindgen_futures::JsFuture;

const TAB_NAVIGATION_TIMEOUT_MS: f64 = 30_000.0;
const TAB_POLL_INTERVAL_MS: u32 = 100;

#[wasm_bindgen]
extern "C" {
    #[wasm_bindgen(catch, js_namespace = ["chrome", "storage", "local"], js_name = get)]
    fn storage_get(keys: &JsValue) -> Result<Promise, JsValue>;

    #[wasm_bindgen(catch, js_namespace = ["chrome", "storage", "local"], js_name = set)]
    fn storage_set(items: &JsValue) -> Result<Promise, JsValue>;

    #[wasm_bindgen(catch, js_namespace = ["chrome", "tabs"], js_name = sendMessage)]
    fn tabs_send_message(tab_id: i32, message: &JsValue) -> Result<Promise, JsValue>;

    #[wasm_bindgen(catch, js_namespace = ["chrome", "tabs"], js_name = captureVisibleTab)]
    fn tabs_capture_visible_tab(window_id: i32, options: &JsValue) -> Result<Promise, JsValue>;

    #[wasm_bindgen(catch, js_namespace = ["chrome", "tabs"], js_name = query)]
    fn tabs_query(query_info: &JsValue) -> Result<Promise, JsValue>;

    #[wasm_bindgen(catch, js_namespace = ["chrome", "tabs"], js_name = update)]
    fn tabs_update(tab_id: i32, update_properties: &JsValue) -> Result<Promise, JsValue>;

    #[wasm_bindgen(catch, js_namespace = ["chrome", "tabs"], js_name = get)]
    fn tabs_get(tab_id: i32) -> Result<Promise, JsValue>;

    #[wasm_bindgen(catch, js_namespace = ["chrome", "windows"], js_name = create)]
    fn windows_create(create_data: &JsValue) -> Result<Promise, JsValue>;

    #[wasm_bindgen(catch, js_namespace = ["chrome", "runtime"], js_name = sendMessage)]
    fn runtime_send_message(message: &JsValue) -> Result<Promise, JsValue>;

    #[wasm_bindgen(
        js_namespace = ["chrome", "runtime", "onMessage"],
        js_name = addListener
    )]
    fn runtime_add_message_listener(listener: &Function);
}

#[derive(Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct TabContext {
    pub window_id: i32,
    #[serde(default)]
    pub url: String,
}

#[derive(Deserialize)]
struct MessageSender {
    tab: Option<TabContext>,
}

#[derive(Serialize)]
struct CaptureOptions {
    format: &'static str,
}

#[derive(Serialize)]
struct CreateWindowOptions {
    url: &'static str,
    #[serde(rename = "type")]
    window_type: &'static str,
    focused: bool,
}

#[derive(Deserialize)]
struct WindowInfo {
    id: i32,
}

#[derive(Serialize)]
struct TabsQueryOptions {
    #[serde(skip_serializing_if = "Option::is_none")]
    url: Option<&'static str>,
    #[serde(rename = "windowId")]
    #[serde(skip_serializing_if = "Option::is_none")]
    window_id: Option<i32>,
}

#[derive(Deserialize)]
#[serde(rename_all = "camelCase")]
struct TabInfo {
    id: i32,
}

#[derive(Deserialize)]
#[serde(rename_all = "camelCase")]
struct TabState {
    #[serde(default)]
    url: Option<String>,
    #[serde(default)]
    status: Option<String>,
}

#[derive(Serialize)]
struct TabUpdateOptions {
    url: &'static str,
}

fn js_error(error: JsValue) -> String {
    format!("{error:?}")
}

async fn resolve(promise: Result<Promise, JsValue>) -> Result<JsValue, String> {
    JsFuture::from(promise.map_err(js_error)?)
        .await
        .map_err(js_error)
}

fn to_js_value<T: Serialize + ?Sized>(value: &T) -> Result<JsValue, String> {
    serde_wasm_bindgen::to_value(value).map_err(|error| error.to_string())
}

fn from_js_value<T: DeserializeOwned>(value: JsValue) -> Result<T, String> {
    serde_wasm_bindgen::from_value(value).map_err(|error| error.to_string())
}

pub async fn get_storage(key: &str) -> Result<Option<JsValue>, String> {
    let keys = JsValue::from_str(key);
    let value = resolve(storage_get(&keys)).await?;
    let value = Reflect::get(&value, &JsValue::from_str(key)).map_err(js_error)?;
    if value.is_null() || value.is_undefined() {
        Ok(None)
    } else {
        Ok(Some(value))
    }
}

pub async fn set_storage(key: &str, value: &JsValue) -> Result<(), String> {
    let items = Object::new();
    Reflect::set(&items, &JsValue::from_str(key), value).map_err(js_error)?;
    resolve(storage_set(&items.into())).await?;
    Ok(())
}

pub fn tab_context(sender: &JsValue) -> Result<TabContext, String> {
    let sender: MessageSender = from_js_value(sender.clone())?;
    sender
        .tab
        .ok_or_else(|| "Message did not come from a tab".to_string())
}

pub async fn send_tab_message<T: Serialize + ?Sized, R: DeserializeOwned>(
    tab_id: i32,
    message: &T,
) -> Result<R, String> {
    let message = to_js_value(message)?;
    let response = resolve(tabs_send_message(tab_id, &message)).await?;
    from_js_value(response)
}

pub async fn capture_visible_tab(window_id: i32) -> Result<String, String> {
    let options = to_js_value(&CaptureOptions { format: "png" })?;
    resolve(tabs_capture_visible_tab(window_id, &options))
        .await?
        .as_string()
        .ok_or_else(|| "Screenshot capture returned no data".to_string())
}

async fn create_chat_tab(url: &'static str, error_message: &str) -> Result<i32, String> {
    let options = to_js_value(&CreateWindowOptions {
        url,
        window_type: "normal",
        focused: false,
    })?;
    let window: WindowInfo = from_js_value(resolve(windows_create(&options)).await?)?;

    let query = to_js_value(&TabsQueryOptions {
        url: None,
        window_id: Some(window.id),
    })?;
    let tabs: Vec<TabInfo> = from_js_value(resolve(tabs_query(&query)).await?)?;
    tabs.into_iter()
        .next()
        .map(|tab| tab.id)
        .ok_or_else(|| error_message.to_string())
}

async fn find_chat_tab(url_pattern: &'static str) -> Result<Option<TabInfo>, String> {
    let query = to_js_value(&TabsQueryOptions {
        url: Some(url_pattern),
        window_id: None,
    })?;
    let tabs: Vec<TabInfo> = from_js_value(resolve(tabs_query(&query)).await?)?;
    Ok(tabs.into_iter().next())
}

async fn wait_for_tab_navigation(tab_id: i32, expected_url: &str) -> Result<(), String> {
    let deadline = js_sys::Date::now() + TAB_NAVIGATION_TIMEOUT_MS;

    while js_sys::Date::now() < deadline {
        let tab: TabState = from_js_value(resolve(tabs_get(tab_id)).await?)?;
        let url_matches = tab.url.as_deref().is_some_and(|url| {
            url.trim_end_matches('/') == expected_url.trim_end_matches('/')
        });

        if url_matches && tab.status.as_deref() == Some("complete") {
            return Ok(());
        }

        TimeoutFuture::new(TAB_POLL_INTERVAL_MS).await;
    }

    Err(format!("Chat page did not finish loading at {expected_url}"))
}

async fn open_chat_tab(
    url: &'static str,
    url_pattern: &'static str,
    error_message: &str,
) -> Result<i32, String> {
    if let Some(tab) = find_chat_tab(url_pattern).await? {
        let update = to_js_value(&TabUpdateOptions { url })?;
        resolve(tabs_update(tab.id, &update)).await?;

        wait_for_tab_navigation(tab.id, url).await?;
        return Ok(tab.id);
    }

    create_chat_tab(url, error_message).await
}

pub async fn create_gemini_tab() -> Result<i32, String> {
    open_chat_tab(
        "https://gemini.google.com/app",
        "https://gemini.google.com/*",
        "Could not open a Gemini tab",
    )
    .await
}

pub async fn create_claude_tab() -> Result<i32, String> {
    open_chat_tab(
        "https://claude.ai/new",
        "https://claude.ai/*",
        "Could not open a Claude tab",
    )
    .await
}

pub async fn send_runtime_message<T: Serialize + ?Sized>(message: &T) -> Result<(), String> {
    let message = to_js_value(message)?;
    let response = resolve(runtime_send_message(&message)).await?;
    let response: crate::messages::MessageResponse = from_js_value(response)?;
    if response.ok {
        Ok(())
    } else {
        Err(response
            .error
            .unwrap_or_else(|| "Background did not accept the captured page".to_string()))
    }
}

pub fn add_runtime_message_listener(listener: &Function) {
    runtime_add_message_listener(listener);
}
