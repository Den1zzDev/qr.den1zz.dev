//! Controls for the optional centre logo.

use leptos::prelude::*;
use wasm_bindgen::closure::Closure;
use wasm_bindgen::JsCast;
use web_sys::DragEvent;
use web_sys::Event;
use web_sys::File;
use web_sys::FileReader;
use web_sys::HtmlInputElement;
use web_sys::ProgressEvent;

use crate::components::icons::Close;
use crate::components::icons::Upload;
use crate::components::icons::Warning;
use crate::studio_state::StudioState;

/// Largest logo the SVG renderer accepts, in bytes.
const MAX_LOGO_BYTES: f64 = 1_000_000.0;

/// Returns whether `file` is an image the renderer can embed.
fn is_supported(file: &File) -> bool {
    matches!(
        file.type_().as_str(),
        "image/png" | "image/jpeg" | "image/svg+xml"
    )
}

/// Reads `file` as a data URL and stores it as the centre logo.
fn read_logo(state: StudioState, error: RwSignal<Option<String>>, file: File) {
    if !is_supported(&file) {
        error.set(Some("Use a PNG, JPEG, or SVG file.".to_string()));
        return;
    }
    if file.size() > MAX_LOGO_BYTES {
        error.set(Some("That file is larger than 1 MB.".to_string()));
        return;
    }

    let reader = match FileReader::new() {
        Ok(reader) => reader,
        Err(_) => {
            error.set(Some("The browser could not read that file.".to_string()));
            return;
        }
    };

    let callback_reader = reader.clone();
    let onload =
        Closure::wrap(Box::new(
            move |_event: ProgressEvent| match callback_reader.result() {
                Ok(result) => match result.as_string() {
                    Some(data_url) => {
                        state.logo_data_url.set(Some(data_url));
                        error.set(None);
                    }
                    None => error.set(Some("The browser returned an empty file.".to_string())),
                },
                Err(_) => error.set(Some("The browser could not read that file.".to_string())),
            },
        ) as Box<dyn FnMut(ProgressEvent)>);

    let callback_error = error;
    let onerror = Closure::wrap(Box::new(move |_event: ProgressEvent| {
        callback_error.set(Some("The browser could not read that file.".to_string()));
    }) as Box<dyn FnMut(ProgressEvent)>);

    reader.set_onload(Some(onload.as_ref().unchecked_ref()));
    reader.set_onerror(Some(onerror.as_ref().unchecked_ref()));
    onload.forget();
    onerror.forget();

    if reader.read_as_data_url(&file).is_err() {
        error.set(Some("The browser could not read that file.".to_string()));
    }
}

/// Reads the first file from an input change event.
fn handle_input(state: StudioState, error: RwSignal<Option<String>>, event: Event) {
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

    read_logo(state, error, file);
    // Allow re-selecting the same file after an error.
    input.set_value("");
}

/// Reads the first file from a drop event.
fn handle_drop(
    state: StudioState,
    error: RwSignal<Option<String>>,
    set_dragging: RwSignal<bool>,
    event: DragEvent,
) {
    event.prevent_default();
    set_dragging.set(false);

    let Some(transfer) = event.data_transfer() else {
        return;
    };
    let Some(files) = transfer.files() else {
        return;
    };
    let Some(file) = files.get(0) else {
        return;
    };

    read_logo(state, error, file);
}

/// Upload and removal controls for the centre logo.
#[component]
pub(crate) fn LogoPanel(state: StudioState) -> impl IntoView {
    let is_dragging = RwSignal::new(false);
    let error = RwSignal::new(Option::<String>::None);

    view! {
        <div class="panel">
            <div class="panel__head">
                <h2 class="panel__title">"Centre logo"</h2>
                {move || if state.logo_data_url.get().is_some() {
                    view! {
                        <button
                            type="button"
                            class="link-button"
                            on:click=move |_| {
                                state.logo_data_url.set(None);
                                error.set(None);
                            }
                        >
                            <Close size=12 />
                            "Remove"
                        </button>
                    }.into_any()
                } else {
                    view! { <span /> }.into_any()
                }}
            </div>
            <div class="panel__body space-y-2">
                <label
                    class=move || if is_dragging.get() {
                        "dropzone is-dragging"
                    } else {
                        "dropzone"
                    }
                    on:dragover=move |event| {
                        event.prevent_default();
                        is_dragging.set(true);
                    }
                    on:dragleave=move |_| is_dragging.set(false)
                    on:drop=move |event| handle_drop(state, error, is_dragging, event)
                >
                    <span class="dropzone__thumb" aria-hidden="true">
                        {move || match state.logo_data_url.get() {
                            Some(data_url) => view! {
                                <img src=data_url alt="" />
                            }.into_any(),
                            None => view! { <Upload size=18 /> }.into_any(),
                        }}
                    </span>
                    <span class="dropzone__body">
                        <span class="dropzone__title">
                            {move || if state.logo_data_url.get().is_some() {
                                "Logo loaded. Drop or click to replace."
                            } else {
                                "Upload a logo"
                            }}
                        </span>
                        <span class="dropzone__hint">
                            "PNG, JPEG, or SVG up to 1 MB. A square, transparent PNG works best."
                        </span>
                    </span>
                    <input
                        type="file"
                        class="sr-only"
                        accept="image/png,image/jpeg,image/svg+xml"
                        aria-label="Centre logo file"
                        on:change=move |event| handle_input(state, error, event)
                    />
                </label>

                {move || state.logo_data_url.get().is_some().then(|| view! {
                    <p class="field-hint">"Error correction is raised to H while a logo is present."</p>
                })}

                {move || error.get().map(|message| view! {
                    <p class="dropzone__error" role="alert">
                        <Warning size=12 />
                        {message}
                    </p>
                })}
            </div>
        </div>
    }
}
