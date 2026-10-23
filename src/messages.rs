use serde::{Deserialize, Serialize};

#[derive(Deserialize, Serialize)]
#[serde(tag = "type", rename_all_fields = "camelCase")]
pub enum ExtensionMessage {
    #[serde(rename = "hrb-page-captured")]
    PageCaptured { html: String },
    #[serde(rename = "hrb-capture-page")]
    CapturePage,
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
