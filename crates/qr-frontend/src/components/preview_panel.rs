//! QR preview, scannability indicators, and export actions.

use std::rc::Rc;
use std::time::Duration;

use leptos::prelude::*;
use qr_core::payload::SanitizedPayload;

use crate::browser;
use crate::components::icons::Check;
use crate::components::icons::Copy;
use crate::components::icons::Download;
use crate::components::icons::QrFrame;
use crate::components::icons::Warning;
use crate::studio_state::ContrastRatios;
use crate::studio_state::PreviewState;
use crate::studio_state::RenderedCode;
use crate::studio_state::StudioState;
use crate::studio_state::DEFAULT_URL;

/// Returns the first 60 characters of `content`, with an ellipsis.
fn summarize(content: &str) -> String {
    let mut summary = content.chars().take(60).collect::<String>();
    if content.chars().count() > 60 {
        summary.push('…');
    }
    summary
}

/// Returns a human-readable encoding failure.
fn friendly_error(message: &str) -> String {
    if message.to_lowercase().contains("too long") {
        "This payload is larger than a QR code can hold.".to_string()
    } else {
        message.to_string()
    }
}

/// Formats the physical size of a square export at 300 dpi.
fn print_size(pixels: u32) -> String {
    let centimetres = f64::from(pixels) / 300.0 * 2.54;
    format!("{pixels} px ≈ {centimetres:.1} cm at 300 dpi")
}

/// Live preview of the generated code with export controls.
#[component]
pub(crate) fn PreviewPanel(
    state: StudioState,
    result: Memo<PreviewState>,
    contrast: Memo<ContrastRatios>,
    sanitized: Memo<SanitizedPayload>,
) -> impl IntoView {
    let transparent = Signal::derive(move || state.is_transparent_background.get());

    // One decorative scan pass over the code once the page takes over. The span
    // is removed from the DOM afterwards, so the preview itself never animates
    // while the user types or tunes a colour.
    let (show_scan, set_show_scan) = signal(false);
    Effect::new(move |_| {
        set_show_scan.set(true);
        set_timeout(
            move || set_show_scan.set(false),
            Duration::from_millis(1400),
        );
    });

    view! {
        <div class="panel preview">
            <p class="sr-only" role="status" aria-live="polite">
                {move || match result.get() {
                    PreviewState::Empty => "Waiting for content.",
                    PreviewState::Error(_) => "The payload could not be encoded.",
                    PreviewState::Ready(_) if contrast.get().is_acceptable() => {
                        "Code ready. Contrast passes."
                    }
                    PreviewState::Ready(_) => "Code ready. Low contrast.",
                }}
            </p>

            <h2 class="sr-only">"Preview"</h2>

            {move || match result.get() {
                PreviewState::Empty => view! {
                    <div class="qr-stage">
                        <div class="qr-stage__empty">
                            <span class="qr-stage__empty-icon" aria-hidden="true">
                                <QrFrame size=22 />
                            </span>
                            <p class="qr-stage__empty-title">"Nothing to encode yet"</p>
                            <p class="qr-stage__empty-copy">
                                "Fill in the payload fields and the code appears here as you type."
                            </p>
                            <button
                                type="button"
                                class="btn btn--secondary btn--sm"
                                on:click=move |_| {
                                    state.active_tab.set(crate::components::glide_select::PayloadTab::Url);
                                    state.url_input.set(DEFAULT_URL.to_string());
                                }
                            >
                                "Use the sample link"
                            </button>
                        </div>
                    </div>
                }.into_any(),

                PreviewState::Error(message) => {
                    let bytes = sanitized.get().cleaned_bytes;
                    view! {
                        <div class="qr-stage">
                            <div class="preview__error">
                                <p class="preview__error-title">
                                    <Warning size=14 />
                                    "Could not encode this payload"
                                </p>
                                <p class="preview__error-copy">{friendly_error(&message)}</p>
                                <p class="preview__error-copy">
                                    {if bytes > 0 {
                                        format!("The current payload is {bytes} B. Shorten it or remove the centre logo, then try again.")
                                    } else {
                                        "Shorten the content or remove the centre logo, then try again.".to_string()
                                    }}
                                </p>
                            </div>
                        </div>
                    }.into_any()
                }

                PreviewState::Ready(rendered) => {
                    let ratios = contrast.get();
                    let data = sanitized.get();
                    let summary = summarize(&data.content);
                    let RenderedCode { matrix, svg } = rendered;
                    let all_acceptable = ratios.is_acceptable();
                    let svg = Rc::new(svg);
                    let svg_for_download = Rc::clone(&svg);
                    let svg_for_copy = Rc::clone(&svg);
                    let svg_for_svg_download = Rc::clone(&svg);
                    let svg_for_svg_copy = Rc::clone(&svg);

                    view! {
                        <figure class="preview__figure">
                            <div class="status-pill">
                                <span
                                    class=if all_acceptable {
                                        "status-pill__led"
                                    } else {
                                        "status-pill__led is-fail"
                                    }
                                    aria-hidden="true"
                                ></span>
                                <span class="status-pill__state">
                                    {if all_acceptable {
                                        "Contrast passes"
                                    } else {
                                        "Low contrast"
                                    }}
                                </span>
                                <span class="status-pill__readouts">
                                    <span class="status-pill__sep" aria-hidden="true">"·"</span>
                                    <span
                                        class="status-pill__ratio"
                                        role="img"
                                        aria-label=format!("Module contrast {:.1} to 1", ratios.modules)
                                        title="Module contrast"
                                    >
                                        {format!("M:{:.1}:1", ratios.modules)}
                                    </span>
                                    <span class="status-pill__sep" aria-hidden="true">"·"</span>
                                    <span
                                        class="status-pill__ratio"
                                        role="img"
                                        aria-label=format!("Eye frame contrast {:.1} to 1", ratios.eye_frame)
                                        title="Eye frame contrast"
                                    >
                                        {format!("E:{:.1}:1", ratios.eye_frame)}
                                    </span>
                                    <span class="status-pill__sep" aria-hidden="true">"·"</span>
                                    <span
                                        class="status-pill__ratio"
                                        role="img"
                                        aria-label=format!("Eye dot contrast {:.1} to 1", ratios.eye_dot)
                                        title="Eye dot contrast"
                                    >
                                        {format!("D:{:.1}:1", ratios.eye_dot)}
                                    </span>
                                </span>
                                <span class="status-pill__sep" aria-hidden="true">"·"</span>
                                <span class="status-pill__meta">
                                    {format!(
                                        "v{} · {}×{} · ECC {}",
                                        matrix.version,
                                        matrix.size,
                                        matrix.size,
                                        matrix.ec_level.label(),
                                    )}
                                </span>
                            </div>

                            <div
                                class=move || {
                                    if transparent.get() {
                                        "qr-stage qr-stage--transparent"
                                    } else {
                                        "qr-stage"
                                    }
                                }
                                role="img"
                                aria-label=format!("QR code encoding {summary}")
                                title="The white margin is the quiet zone that keeps the code scannable."
                            >
                                {move || show_scan.get().then(|| view! {
                                    <span class="qr-stage__scan" aria-hidden="true"></span>
                                })}
                                <div class="qr-stage__canvas" inner_html=svg.as_str().to_string()></div>
                            </div>
                            <p class="sr-only">
                                "The light margin around the code is its quiet zone."
                            </p>

                            <div class="preview__actions">
                                <p class=move || if contrast.get().is_acceptable() {
                                    "preview__caveat"
                                } else {
                                    "preview__caveat preview__caveat--warn"
                                }>
                                    {move || {
                                        if !contrast.get().is_acceptable() {
                                            if transparent.get() {
                                                "Contrast is below 3:1 against white. Adjust the palette before exporting."
                                            } else {
                                                "Contrast is below 3:1. Adjust the palette before exporting."
                                            }
                                        } else if transparent.get() {
                                            "Contrast checked against white. Test-scan before printing."
                                        } else {
                                            "Contrast checked. Test-scan before printing."
                                        }
                                    }}
                                </p>

                                <div class="space-y-2">
                                    <div class="flex flex-wrap items-center justify-between gap-3">
                                        <span
                                            id="export-size-label"
                                            class="field-label !mb-0 whitespace-nowrap"
                                        >
                                            "Export size"
                                        </span>
                                        <div
                                            class="segmented shrink-0"
                                            role="radiogroup"
                                            aria-labelledby="export-size-label"
                                        >
                                            {[512_u32, 1024, 2048].into_iter().map(|resolution| {
                                                view! {
                                                    <label class="seg">
                                                        <input
                                                            type="radio"
                                                            name="export-resolution"
                                                            value=resolution.to_string()
                                                            prop:checked=move || {
                                                                state.target_resolution.get() == resolution
                                                            }
                                                            on:change=move |_| {
                                                                state.target_resolution.set(resolution)
                                                            }
                                                        />
                                                        <span>{resolution}</span>
                                                    </label>
                                                }
                                            }).collect_view()}
                                        </div>
                                    </div>
                                    <p class="preview__print-size">
                                        {move || print_size(state.target_resolution.get())}
                                    </p>

                                    <div class="export-grid">
                                        <button
                                            type="button"
                                            class="btn btn--primary"
                                            on:click=move |_| {
                                                let resolution = state.target_resolution.get();
                                                let outcome = browser::download_png(
                                                    &svg_for_download,
                                                    resolution,
                                                    move || state.notify("PNG downloaded", false),
                                                    move |error| state.notify(&error, true),
                                                );
                                                if let Err(error) = outcome {
                                                    state.notify(&error, true);
                                                }
                                            }
                                        >
                                            <Download size=15 />
                                            "Download PNG"
                                        </button>

                                        <button
                                            type="button"
                                            class="btn btn--secondary"
                                            on:click=move |_| {
                                                let outcome = browser::download_svg(
                                                    &svg_for_svg_download,
                                                    move || state.notify("SVG downloaded", false),
                                                );
                                                if let Err(error) = outcome {
                                                    state.notify(&error, true);
                                                }
                                            }
                                        >
                                            <Download size=15 />
                                            "Download SVG"
                                        </button>

                                        <button
                                            type="button"
                                            class="btn btn--ghost"
                                            on:click=move |_| {
                                                let resolution = state.target_resolution.get();
                                                let outcome = browser::copy_png(
                                                    &svg_for_copy,
                                                    resolution,
                                                    move || state.notify("PNG copied", false),
                                                    move |error| state.notify(&error, true),
                                                );
                                                if let Err(error) = outcome {
                                                    state.notify(&error, true);
                                                }
                                            }
                                        >
                                            <Copy size=15 />
                                            "Copy PNG"
                                        </button>

                                        <button
                                            type="button"
                                            class="btn btn--ghost"
                                            on:click=move |_| {
                                                let outcome = browser::copy_svg(
                                                    &svg_for_svg_copy,
                                                    move || state.notify("SVG copied", false),
                                                    move |error| state.notify(&error, true),
                                                );
                                                if let Err(error) = outcome {
                                                    state.notify(&error, true);
                                                }
                                            }
                                        >
                                            <Copy size=15 />
                                            "Copy SVG"
                                        </button>
                                    </div>
                                </div>

                                {move || {
                                    state.notification.get().map(|notification| {
                                        let notice_class = if notification.is_error {
                                            "notice is-visible notice--error"
                                        } else {
                                            "notice is-visible notice--success"
                                        };
                                        let icon = if notification.is_error {
                                            view! { <Warning size=12 /> }.into_any()
                                        } else {
                                            view! { <Check size=12 /> }.into_any()
                                        };
                                        view! {
                                            <p class=notice_class aria-hidden="true">
                                                {icon}
                                                <span>{notification.message}</span>
                                            </p>
                                        }
                                    })
                                }}
                            </div>
                        </figure>
                    }.into_any()
                }
            }}
        </div>
    }
}
