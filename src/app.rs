use crate::config::{self, Settings};
use leptos::prelude::*;
use leptos::task::spawn_local;

#[component]
pub fn App() -> impl IntoView {
    let telegram_token = RwSignal::new(String::new());
    let chat_id = RwSignal::new(String::new());
    let html_selector = RwSignal::new("main".to_string());
    let hotkey_code = RwSignal::new("NumpadSubtract".to_string());
    let hotkey_ctrl = RwSignal::new(false);
    let hotkey_alt = RwSignal::new(false);
    let hotkey_shift = RwSignal::new(false);
    let send_photo = RwSignal::new(false);
    let status = RwSignal::new(String::new());

    Effect::new(move |_| {
        spawn_local(async move {
            match config::load().await {
                Ok(settings) => {
                    telegram_token.set(settings.telegram_token);
                    chat_id.set(settings.chat_id);
                    html_selector.set(settings.html_selector);
                    hotkey_code.set(settings.hotkey_code);
                    hotkey_ctrl.set(settings.hotkey_ctrl);
                    hotkey_alt.set(settings.hotkey_alt);
                    hotkey_shift.set(settings.hotkey_shift);
                    send_photo.set(settings.send_photo);
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
            hotkey_code: hotkey_code.get(),
            hotkey_ctrl: hotkey_ctrl.get(),
            hotkey_alt: hotkey_alt.get(),
            hotkey_shift: hotkey_shift.get(),
            send_photo: send_photo.get(),
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
                "Hotkey (KeyboardEvent.code, e.g. NumpadSubtract)"
                <input r#type="text" placeholder="NumpadSubtract" style="width: 100%" bind:value=hotkey_code />
            </label>
            <div style="display: flex; gap: 1rem; align-items: center; flex-wrap: wrap">
                <label>
                    "Ctrl"
                    <input r#type="checkbox" bind:checked=hotkey_ctrl />
                </label>
                <label>
                    "Alt"
                    <input r#type="checkbox" bind:checked=hotkey_alt />
                </label>
                <label>
                    "Shift"
                    <input r#type="checkbox" bind:checked=hotkey_shift />
                </label>
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
