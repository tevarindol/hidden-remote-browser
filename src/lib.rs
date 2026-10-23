mod app;
mod assistant;
mod background;
mod chrome_api;
mod config;
mod content;
mod messages;
mod telegram;

use wasm_bindgen::prelude::*;

#[wasm_bindgen(js_name = mountOptions)]
pub fn mount_options() {
    console_error_panic_hook::set_once();
    leptos::mount::mount_to_body(app::App);
}

#[wasm_bindgen(js_name = handleBackgroundMessage)]
pub fn handle_background_message(
    message: JsValue,
    sender: JsValue,
    send_response: js_sys::Function,
) -> bool {
    console_error_panic_hook::set_once();
    background::handle_message(message, sender, send_response)
}

#[wasm_bindgen(js_name = startContent)]
pub fn start_content() {
    console_error_panic_hook::set_once();
    content::start();
}
