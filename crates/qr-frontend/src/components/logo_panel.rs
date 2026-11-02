//! Controls for the optional centre logo.

use leptos::prelude::*;
use wasm_bindgen::closure::Closure;
use wasm_bindgen::JsCast;
use web_sys::Event;
use web_sys::FileReader;
use web_sys::HtmlInputElement;
use web_sys::ProgressEvent;

use crate::studio_state::StudioState;

/// Loads an image file and stores it as a data URL.
fn load_logo(state: StudioState, event: Event) {
    let Some(target) = event.target() else {
        return;
    };
    let input = target.unchecked_into::<HtmlInputElement>();
    let Some(files) = input.files() else {
        return;
    };
    let Some(file) = files.get(0) else {
        return;
    };

    let reader = FileReader::new().expect("FileReader is available in browsers");
    let callback_reader = reader.clone();
    let onload = Closure::wrap(Box::new(move |_event: ProgressEvent| {
        if let Ok(result) = callback_reader.result() {
            if let Some(data_url) = result.as_string() {
                state.logo_data_url.set(Some(data_url));
            }
        }
    }) as Box<dyn FnMut(ProgressEvent)>);

    reader.set_onload(Some(onload.as_ref().unchecked_ref()));
    onload.forget();
    let _ = reader.read_as_data_url(&file);
}

/// Upload and removal controls for the centre logo.
#[component]
pub(crate) fn LogoPanel(state: StudioState) -> impl IntoView {
    view! {
        <div class="glass-panel p-4 sm:p-5 space-y-3">
            <div class="flex items-center justify-between">
                <h3 class="text-xs font-semibold uppercase tracking-wider text-[#a0a0a0]">
                    "Center Logo (Optional)"
                </h3>
                {move || if state.logo_data_url.get().is_some() {
                    view! {
                        <button
                            class="text-xs text-rose-400 hover:text-rose-300"
                            on:click=move |_| state.logo_data_url.set(None)
                        >
                            "Remove Logo"
                        </button>
                    }.into_any()
                } else {
                    view! { <span /> }.into_any()
                }}
            </div>
            <div class="flex items-center gap-3">
                <label class="button button-secondary text-xs cursor-pointer py-1.5 px-3">
                    <span>"Upload PNG/SVG"</span>
                    <input
                        type="file"
                        accept="image/png,image/jpeg,image/svg+xml"
                        class="hidden"
                        on:change=move |event| load_logo(state, event)
                    />
                </label>
                <span class="text-xs text-[#555555]">
                    {move || if state.logo_data_url.get().is_some() {
                        "Logo loaded (Ecc auto-boosted to Level H)"
                    } else {
                        "No logo selected"
                    }}
                </span>
            </div>
        </div>
    }
}
