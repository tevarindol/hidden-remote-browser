use serde::{Deserialize, Serialize};

use crate::chrome_api;

pub const STORAGE_KEY: &str = "settings";
pub const AI_PROVIDER_GEMINI: &str = "gemini";
pub const AI_PROVIDER_CLAUDE: &str = "claude";
pub const DEFAULT_PROMPT_TEMPLATE: &str = "Базоий прмопт";

#[derive(Clone, Debug, Serialize, Deserialize)]
#[serde(default)]
#[serde(rename_all = "camelCase")]
pub struct Settings {
    pub telegram_token: String,
    pub chat_id: String,
    pub html_selector: String,
    pub ai_provider: String,
    pub send_photo: bool,
    #[serde(alias = "geminiPrompt")]
    pub prompt_template: String,
}

impl Default for Settings {
    fn default() -> Self {
        Settings {
            telegram_token: String::new(),
            chat_id: String::new(),
            html_selector: "main".to_string(),
            ai_provider: AI_PROVIDER_GEMINI.to_string(),
            send_photo: false,
            prompt_template: DEFAULT_PROMPT_TEMPLATE.to_string(),
        }
    }
}

impl Settings {
    pub fn normalize(&mut self) {
        self.telegram_token = self.telegram_token.trim().to_string();
        self.chat_id = self.chat_id.trim().to_string();
        self.html_selector = self.html_selector.trim().to_string();
        self.ai_provider = self.ai_provider.trim().to_ascii_lowercase();
        if self.ai_provider != AI_PROVIDER_GEMINI && self.ai_provider != AI_PROVIDER_CLAUDE {
            self.ai_provider = AI_PROVIDER_GEMINI.to_string();
        }
        self.prompt_template = self.prompt_template.trim().to_string();
        if self.prompt_template.is_empty() {
            self.prompt_template = DEFAULT_PROMPT_TEMPLATE.to_string();
        }
    }
}

pub async fn load() -> Result<Settings, String> {
    let Some(raw) = chrome_api::get_storage(STORAGE_KEY)
        .await
        .map_err(|error| format!("storage.get failed: {error}"))?
    else {
        return Ok(Settings::default());
    };
    let mut settings: Settings = if let Some(json) = raw.as_string() {
        // older builds stored the settings as a JSON string
        serde_json::from_str(&json).unwrap_or_default()
    } else {
        serde_wasm_bindgen::from_value(raw)
            .map_err(|error| format!("invalid stored settings: {error}"))?
    };
    settings.normalize();
    Ok(settings)
}

pub async fn save(settings: &Settings) -> Result<(), String> {
    let mut settings = settings.clone();
    settings.normalize();
    let value = serde_wasm_bindgen::to_value(&settings).map_err(|e| e.to_string())?;
    chrome_api::set_storage(STORAGE_KEY, &value)
        .await
        .map_err(|error| format!("storage.set failed: {error}"))?;
    Ok(())
}
