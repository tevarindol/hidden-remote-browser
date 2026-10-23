use serde::{Deserialize, Serialize};

#[derive(Clone, Copy, Debug, Deserialize, Serialize)]
#[serde(rename_all = "lowercase")]
pub enum AIProvider {
    Gemini,
    Claude,
}

#[derive(Deserialize, Serialize)]
#[serde(tag = "type", rename_all_fields = "camelCase")]
pub enum ExtensionMessage {
    #[serde(rename = "hrb-page-captured")]
    PageCaptured { html: String, provider: AIProvider },
    #[serde(rename = "hrb-capture-page")]
    CapturePage { provider: AIProvider },
    #[serde(rename = "hrb-gemini-submit")]
    GeminiSubmit { prompt: String },
    #[serde(rename = "hrb-claude-submit")]
    ClaudeSubmit { prompt: String },
}

#[derive(Deserialize, Serialize)]
pub struct MessageResponse {
    pub ok: bool,
    pub error: Option<String>,
}
