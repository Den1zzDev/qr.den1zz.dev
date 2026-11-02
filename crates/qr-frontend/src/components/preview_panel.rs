//! QR preview, scannability indicators, and export actions.

use std::rc::Rc;

use leptos::prelude::*;
use qr_core::contrast::MIN_GRAPHIC_CONTRAST;
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

    view! {
        <div class="panel preview">
            <p class="sr-only" role="status" aria-live="polite">
                {move || match result.get() {
                    PreviewState::Empty => "Waiting for content.",
                    PreviewState::Error(_) => "The payload could not be encoded.",
                    PreviewState::Ready(_) => "",
                }}
            </p>

            <div class="preview__top">
                <h2 class="panel__title">"Preview"</h2>
                {move || {
                    let ratios = contrast.get();
                    let transparent = state.is_transparent_background.get();
                    match result.get() {
                        PreviewState::Empty => view! {
                            <span class="status">
                                <span class="ratio-chip__dot" style="background: var(--text-faint)"></span>
                                "Waiting for content"
                            </span>
                        }.into_any(),
                        PreviewState::Error(_) => view! {
                            <span class="status status--warn">
                                <Warning size=12 />
                                "Not generated"
                            </span>
                        }.into_any(),
                        PreviewState::Ready(_) if ratios.is_acceptable() => view! {
                            <span
                                class="status status--pass"
                                title=if transparent {
                                    "The export is transparent. Ratios assume a white surface."
                                } else {
                                    "Every coloured region meets the 3:1 minimum against the background."
                                }
                            >
                                <Check size=12 />
                                {if transparent { "Contrast vs white" } else { "Contrast passes" }}
                            </span>
                        }.into_any(),
                        PreviewState::Ready(_) => view! {
                            <span class="status status--warn">
                                <Warning size=12 />
                                "Low contrast"
                            </span>
                        }.into_any(),
                    }
                }}
            </div>

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
                    let bytes = data.cleaned_bytes;
                    let RenderedCode { matrix, svg } = rendered;
                    let all_acceptable = ratios.is_acceptable();
                    let svg = Rc::new(svg);
                    let svg_for_download = Rc::clone(&svg);
                    let svg_for_copy = Rc::clone(&svg);
                    let svg_for_svg_download = Rc::clone(&svg);
                    let svg_for_svg_copy = Rc::clone(&svg);
                    let has_logo = state.logo_data_url.get().is_some();

                    view! {
                        <figure class="space-y-4">
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
                            >
                                <div class="qr-stage__canvas" inner_html=svg.as_str().to_string()></div>
                            </div>

                            <div class="preview__actions">
                                <p class=if all_acceptable {
                                    "preview__caveat"
                                } else {
                                    "preview__caveat preview__caveat--warn"
                                }>
                                    {if all_acceptable {
                                        "Contrast checked. Test-scan before printing."
                                    } else {
                                        "Contrast is below 3:1. Adjust the palette before exporting."
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
                                    let notification = state.notification.get();
                                    let notice_class = match &notification {
                                        Some(notification) if notification.is_error => {
                                            "notice is-visible notice--error"
                                        }
                                        Some(_) => "notice is-visible notice--success",
                                        None => "notice",
                                    };
                                    view! {
                                        <p class=notice_class aria-hidden="true">
                                            {notification.map(|notification| view! {
                                                {if notification.is_error {
                                                    view! { <Warning size=12 /> }.into_any()
                                                } else {
                                                    view! { <Check size=12 /> }.into_any()
                                                }}
                                                <span>{notification.message}</span>
                                            })}
                                        </p>
                                    }
                                }}
                            </div>

                            <div class="preview__divider"></div>

                            <dl class="spec-grid">
                                <div class="spec">
                                    <dt class="spec__label">"Version"</dt>
                                    <dd class="spec__value">{format!("V{}", matrix.version)}</dd>
                                </div>
                                <div class="spec">
                                    <dt class="spec__label">"Matrix"</dt>
                                    <dd class="spec__value">
                                        {format!("{}×{}", matrix.size, matrix.size)}
                                    </dd>
                                </div>
                                <div class="spec">
                                    <dt class="spec__label">"Error level"</dt>
                                    <dd class="spec__value">
                                        {format!(
                                            "{}{}",
                                            matrix.ec_level.label(),
                                            if has_logo { " · logo" } else { "" },
                                        )}
                                    </dd>
                                </div>
                                <div class="spec">
                                    <dt class="spec__label">"Payload"</dt>
                                    <dd class="spec__value">{format!("{bytes} B")}</dd>
                                </div>
                            </dl>

                            <div class="ratio-strip" role="group" aria-label="Contrast against the background">
                                {[
                                    ("Modules", ratios.modules),
                                    ("Eyes", ratios.eye_frame),
                                    ("Dots", ratios.eye_dot),
                                ].into_iter().map(|(label, ratio)| {
                                    let passes = ratio >= MIN_GRAPHIC_CONTRAST;
                                    view! {
                                        <span
                                            class=if passes {
                                                "ratio-chip ratio-chip--pass"
                                            } else {
                                                "ratio-chip ratio-chip--fail"
                                            }
                                            title=format!(
                                                "{label} contrast {ratio:.1} to 1, {}",
                                                if passes {
                                                    "meets the 3 to 1 minimum"
                                                } else {
                                                    "below the 3 to 1 minimum"
                                                },
                                            )
                                        >
                                            <span class="ratio-chip__dot" aria-hidden="true"></span>
                                            <span>{label}</span>
                                            <span class="tabular">{format!("{ratio:.1}:1")}</span>
                                        </span>
                                    }
                                }).collect_view()}
                            </div>

                            {move || transparent.get().then(|| view! {
                                <p class="field-hint">
                                    "The export is transparent. Ratios assume a white surface."
                                </p>
                            })}
                        </figure>
                    }.into_any()
                }
            }}
        </div>
    }
}
