use js_sys::{Array, Object, Reflect};
use serde::{Deserialize, Serialize};
use wasm_bindgen::prelude::*;
use wasm_bindgen_futures::JsFuture;

pub const STORAGE_KEY: &str = "settings";

#[derive(Clone, Debug, Serialize, Deserialize)]
#[serde(default)]
#[serde(rename_all = "camelCase")]
pub struct Settings {
    pub telegram_token: String,
    pub chat_id: String,
    pub html_selector: String,
    pub hotkey_code: String,
    pub hotkey_ctrl: bool,
    pub hotkey_alt: bool,
    pub hotkey_shift: bool,
    pub send_photo: bool,
}

impl Default for Settings {
    fn default() -> Self {
        Settings {
            telegram_token: String::new(),
            chat_id: String::new(),
            html_selector: "main".to_string(),
            hotkey_code: "NumpadSubtract".to_string(),
            hotkey_ctrl: false,
            hotkey_alt: false,
            hotkey_shift: false,
            send_photo: false,
        }
    }
}

impl Settings {
    pub fn normalize(&mut self) {
        self.telegram_token = self.telegram_token.trim().to_string();
        self.chat_id = self.chat_id.trim().to_string();
        self.html_selector = self.html_selector.trim().to_string();
        self.hotkey_code = self.hotkey_code.trim().to_string();
    }
}

#[wasm_bindgen]
extern "C" {
    #[wasm_bindgen(js_namespace = ["chrome", "storage", "local"], js_name = get)]
    fn storage_get(keys: &JsValue) -> js_sys::Promise;

    #[wasm_bindgen(js_namespace = ["chrome", "storage", "local"], js_name = set)]
    fn storage_set(items: &JsValue) -> js_sys::Promise;
}

pub async fn load() -> Result<Settings, String> {
    let keys = Array::new();
    keys.push(&JsValue::from_str(STORAGE_KEY));
    let value = JsFuture::from(storage_get(&keys.into()))
        .await
        .map_err(|e| format!("storage.get failed: {e:?}"))?;
    let raw = Reflect::get(&value, &JsValue::from_str(STORAGE_KEY))
        .map_err(|e| format!("storage read failed: {e:?}"))?;
    if raw.is_undefined() || raw.is_null() {
        return Ok(Settings::default());
    }
    if let Some(json) = raw.as_string() {
        // older builds stored the settings as a JSON string
        return Ok(serde_json::from_str(&json).unwrap_or_default());
    }
    serde_wasm_bindgen::from_value(raw.clone()).map_err(|e| format!("invalid stored settings: {e}"))
}

pub async fn save(settings: &Settings) -> Result<(), String> {
    let mut settings = settings.clone();
    settings.normalize();
    let value = serde_wasm_bindgen::to_value(&settings).map_err(|e| e.to_string())?;
    let items = Object::new();
    Reflect::set(&items, &JsValue::from_str(STORAGE_KEY), &value)
        .map_err(|e| format!("storage write failed: {e:?}"))?;
    JsFuture::from(storage_set(&items))
        .await
        .map_err(|e| format!("storage.set failed: {e:?}"))?;
    Ok(())
}
