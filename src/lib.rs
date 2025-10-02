#[cfg(feature = "ssr")]
pub mod api;
#[cfg(feature = "ssr")]
pub mod storage;

pub mod webui;

#[cfg(feature = "hydrate")]
#[wasm_bindgen::prelude::wasm_bindgen]
pub fn hydrate() {
    console_error_panic_hook::set_once();
    leptos::mount::hydrate_body(webui::App);
}
