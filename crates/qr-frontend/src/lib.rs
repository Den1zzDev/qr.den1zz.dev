//! WebAssembly frontend for the QR studio.
//!
//! The crate exposes [`App`] for server rendering and pre-rendering, and the
//! `start_hydrate` and `start_csr` entry points for the browser bundle.

mod app;
mod browser;
mod components;
mod studio_state;

pub use app::App;

/// Hydrates the server-rendered application in the browser.
#[cfg(feature = "hydrate")]
#[wasm_bindgen::prelude::wasm_bindgen(start)]
pub fn start_hydrate() {
    console_error_panic_hook::set_once();
    leptos::mount::hydrate_body(App);
}

/// Mounts the application in the browser without server-rendered markup.
#[cfg(feature = "csr")]
#[wasm_bindgen::prelude::wasm_bindgen(start)]
pub fn start_csr() {
    console_error_panic_hook::set_once();
    leptos::mount::mount_to_body(App);
}
