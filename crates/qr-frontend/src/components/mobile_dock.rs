//! Compact download bar shown below 1024 pixels.

use std::mem;

use leptos::prelude::*;
use qr_core::payload::SanitizedPayload;
use wasm_bindgen::closure::Closure;
use wasm_bindgen::JsCast;

use crate::browser;
use crate::components::icons::Download;
use crate::components::icons::Warning;
use crate::studio_state::PreviewState;
use crate::studio_state::StudioState;

/// Fixed quick-action bar for phones and tablets.
///
/// The full preview sits above the styling panels on one-column layouts, so
/// this bar keeps the current code and the primary export one tap away.
#[component]
pub(crate) fn MobileDock(
    state: StudioState,
    result: Memo<PreviewState>,
    sanitized: Memo<SanitizedPayload>,
) -> impl IntoView {
    // Hide the bar while the preview tray is on screen so it never covers the
    // code. Once the tray scrolls away, the bar returns. The wrapper is used
    // because the stage itself is replaced whenever the code changes.
    let (qr_visible, set_qr_visible) = signal(true);

    Effect::new(move |_| {
        let Some(window) = web_sys::window() else {
            return;
        };
        let Some(document) = window.document() else {
            return;
        };
        let Ok(Some(target)) = document.query_selector(".preview") else {
            return;
        };

        let callback = Closure::wrap(Box::new(move |entries: js_sys::Array| {
            let Some(entry) = entries
                .get(0)
                .dyn_into::<web_sys::IntersectionObserverEntry>()
                .ok()
            else {
                return;
            };
            let visible = entry.is_intersecting();
            if qr_visible.get_untracked() != visible {
                set_qr_visible.set(visible);
            }
        }) as Box<dyn FnMut(js_sys::Array)>);

        let Ok(observer) = web_sys::IntersectionObserver::new(callback.as_ref().unchecked_ref())
        else {
            return;
        };
        observer.observe(&target);

        // The component lives for the whole page, so both allocations stay
        // valid for the session. Keeping them alive is the point.
        callback.forget();
        mem::forget(observer);
    });

    view! {
        <div
            class=move || if qr_visible.get() { "mobile-dock is-hidden" } else { "mobile-dock" }
            role="region"
            aria-label="Quick export"
        >
            {move || match result.get() {
                PreviewState::Ready(rendered) => {
                    let bytes = sanitized.get().cleaned_bytes;
                    let svg = rendered.svg.clone();
                    let thumb_svg = rendered.svg;
                    let title = format!(
                        "v{} · {}×{}",
                        rendered.matrix.version,
                        rendered.matrix.size,
                        rendered.matrix.size,
                    );

                    view! {
                        // The thumbnail is only mounted while the bar is
                        // visible. Until then the full SVG stays out of the
                        // initial document, which avoids shipping the same
                        // path data twice.
                        {move || (!qr_visible.get()).then(|| view! {
                            <div class="dock-qr" aria-hidden="true" inner_html=thumb_svg.clone()></div>
                        })}
                        <div class="dock-info">
                            <p class="dock-title">{title}</p>
                            <p
                                class=move || match state.notification.get() {
                                    Some(notification) if notification.is_error => {
                                        "dock-sub dock-warn"
                                    }
                                    Some(_) => "dock-sub dock-ok",
                                    None => "dock-sub",
                                }
                            >
                                {move || match state.notification.get() {
                                    Some(notification) => notification.message,
                                    None => {
                                        format!(
                                            "{} B · {} px",
                                            bytes,
                                            state.target_resolution.get(),
                                        )
                                    }
                                }}
                            </p>
                        </div>
                        <button
                            type="button"
                            class="btn btn--primary dock-btn"
                            aria-label=move || {
                                format!(
                                    "Download PNG at {} pixels",
                                    state.target_resolution.get(),
                                )
                            }
                            on:click=move |_| {
                                let resolution = state.target_resolution.get();
                                let outcome = browser::download_png(
                                    &svg,
                                    resolution,
                                    move || state.notify("PNG downloaded", false),
                                    move |error| state.notify(&error, true),
                                );
                                if let Err(error) = outcome {
                                    state.notify(&error, true);
                                }
                            }
                        >
                            <Download size=14 />
                            "PNG"
                        </button>
                    }.into_any()
                }
                PreviewState::Empty => view! {
                    <div class="dock-info">
                        <p class="dock-title">"Waiting for content"</p>
                        <p class="dock-sub">"Fill in the payload to generate a code."</p>
                    </div>
                    <button type="button" class="btn btn--ghost dock-btn" disabled=true>
                        <Download size=14 />
                        "PNG"
                    </button>
                }.into_any(),
                PreviewState::Error(_) => view! {
                    <div class="dock-info">
                        <p class="dock-title dock-warn">
                            <Warning size=12 />
                            "Not generated"
                        </p>
                        <p class="dock-sub">"Check the payload and try again."</p>
                    </div>
                    <button type="button" class="btn btn--ghost dock-btn" disabled=true>
                        <Download size=14 />
                        "PNG"
                    </button>
                }.into_any(),
            }}
        </div>
    }
}
