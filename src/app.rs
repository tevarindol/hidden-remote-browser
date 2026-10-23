use crate::config::{self, Settings};
use leptos::prelude::*;
use leptos::task::spawn_local;

#[component]
pub fn App() -> impl IntoView {
    let telegram_token = RwSignal::new(String::new());
    let chat_id = RwSignal::new(String::new());
    let html_selector = RwSignal::new("main".to_string());
    let ai_provider = RwSignal::new(crate::config::AI_PROVIDER_GEMINI.to_string());
    let send_photo = RwSignal::new(false);
    let prompt_template = RwSignal::new(Settings::default().prompt_template);
    let status = RwSignal::new(String::new());

    Effect::new(move |_| {
        spawn_local(async move {
            match config::load().await {
                Ok(settings) => {
                    telegram_token.set(settings.telegram_token);
                    chat_id.set(settings.chat_id);
                    html_selector.set(settings.html_selector);
                    ai_provider.set(settings.ai_provider);
                    send_photo.set(settings.send_photo);
                    prompt_template.set(settings.prompt_template);
                }
                Err(error) => status.set(format!("failed to load settings: {error}")),
            }
        });
    });

    let on_save = move |_| {
        let settings = Settings {
            telegram_token: telegram_token.get(),
            chat_id: chat_id.get(),
            html_selector: html_selector.get(),
            ai_provider: ai_provider.get(),
            send_photo: send_photo.get(),
            prompt_template: prompt_template.get(),
        };
        spawn_local(async move {
            status.set(match config::save(&settings).await {
                Ok(()) => "saved".to_string(),
                Err(error) => format!("failed to save: {error}"),
            });
        });
    };

    view! {
        <main style="max-width: 32rem; margin: 2rem auto; font-family: sans-serif; display: flex; flex-direction: column; gap: 0.75rem">
            <h1>"Hidden Remote Browser"</h1>
            <label>
                "Telegram bot token"
                <input r#type="text" style="width: 100%" bind:value=telegram_token />
            </label>
            <label>
                "Chat id"
                <input r#type="text" style="width: 100%" bind:value=chat_id />
            </label>
            <label>
                "Element selector (CSS)"
                <input r#type="text" placeholder="main" style="width: 100%" bind:value=html_selector />
            </label>
            <label>
                "AI service"
                <select style="width: 100%" bind:value=ai_provider>
                    <option value="gemini">"Gemini"</option>
                    <option value="claude">"Claude"</option>
                </select>
            </label>
            <label>
                "AI prompt template"
                <textarea rows="8" style="width: 100%" bind:value=prompt_template />
                <small>"Use {url} for the page URL and {html} for the captured element."</small>
            </label>
            <p>"Keyboard shortcut: Ctrl+Shift+Y by default. Change it in chrome://extensions/shortcuts."</p>
            <div style="display: flex; gap: 1rem; align-items: center; flex-wrap: wrap">
                <label>
                    "Send screenshot"
                    <input r#type="checkbox" bind:checked=send_photo />
                </label>
            </div>
            <button on:click=on_save style="width: max-content">"Save"</button>
            <p aria-live="polite" style="min-height: 1rem">{move || status.get()}</p>
        </main>
    }
}
