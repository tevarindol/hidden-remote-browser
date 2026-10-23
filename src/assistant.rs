mod claude;
mod gemini;
mod prompt;

pub(crate) use claude::{send_to_claude, submit_claude_prompt};
pub(crate) use gemini::{send_to_gemini, submit_gemini_prompt};
