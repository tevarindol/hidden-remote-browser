use js_sys::{Function, Object, Promise, Reflect};
use serde::{Deserialize, Serialize, de::DeserializeOwned};
use wasm_bindgen::prelude::*;
use wasm_bindgen_futures::JsFuture;

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
    #[serde(rename = "windowId")]
    window_id: i32,
}

#[derive(Deserialize)]
struct TabInfo {
    id: i32,
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
        focused: true,
    })?;
    let window: WindowInfo = from_js_value(resolve(windows_create(&options)).await?)?;

    let query = to_js_value(&TabsQueryOptions {
        window_id: window.id,
    })?;
    let tabs: Vec<TabInfo> = from_js_value(resolve(tabs_query(&query)).await?)?;
    tabs.into_iter()
        .next()
        .map(|tab| tab.id)
        .ok_or_else(|| error_message.to_string())
}

pub async fn create_gemini_tab() -> Result<i32, String> {
    create_chat_tab("https://gemini.google.com/app", "Could not open a Gemini tab").await
}

pub async fn create_claude_tab() -> Result<i32, String> {
    create_chat_tab("https://claude.ai/new", "Could not open a Claude tab").await
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
