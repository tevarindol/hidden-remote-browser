mod app;

use app::App;
use wasm_bindgen::prelude::*;

#[wasm_bindgen]
extern "C" {
    #[wasm_bindgen(js_namespace = console)]
    fn log(s: &str);
}

#[wasm_bindgen(start)]
fn start() {
    console_error_panic_hook::set_once();
    leptos::mount::mount_to_body(App);
}
