//! Shared prompt-template expansion for AI providers.

pub fn build(template: &str, html: &str, page_url: &str) -> String {
    let template = if template.trim().is_empty() {
        crate::config::DEFAULT_PROMPT_TEMPLATE
    } else {
        template
    };
    template.replace("{url}", page_url).replace("{html}", html)
}
