//! QR preview, scannability indicators, and export actions.

use std::time::Duration;

use leptos::prelude::*;
use qr_core::matrix::EncodeError;

use crate::browser;
use crate::studio_state::ContrastRatios;
use crate::studio_state::RenderedCode;
use crate::studio_state::StudioState;

/// How long a copy confirmation stays visible.
const TOAST_DURATION_MILLIS: u64 = 2500;

/// Shows `message` and schedules its removal.
fn notify(state: StudioState, message: &str) {
    state.notification.set(Some(message.to_string()));
    let notification = state.notification;
    set_timeout(
        move || notification.set(None),
        Duration::from_millis(TOAST_DURATION_MILLIS),
    );
}

/// Live preview of the generated code with export controls.
#[component]
pub(crate) fn PreviewPanel(
    state: StudioState,
    result: Memo<Result<RenderedCode, EncodeError>>,
    contrast: Memo<ContrastRatios>,
) -> impl IntoView {
    view! {
        <div class="glass-panel p-4 sm:p-6 flex flex-col items-center justify-center relative space-y-4 sm:space-y-5">
            {move || match result.get() {
                Ok(rendered) => {
                    let ratios = contrast.get();
                    let all_acceptable = ratios.is_acceptable();
                    let svg_for_png = rendered.svg.clone();
                    let svg_for_copy_png = rendered.svg.clone();
                    let svg_for_download = rendered.svg.clone();
                    let svg_for_copy = rendered.svg.clone();

                    view! {
                        <div class="w-full flex flex-col items-center justify-center space-y-4">
                            <div class="flex items-center justify-center gap-1 sm:gap-2 text-[10px] sm:text-xs px-2.5 sm:px-3 py-1 rounded-full bg-[#050505] border border-[rgba(255,255,255,0.12)] whitespace-nowrap max-w-full overflow-x-auto [scrollbar-width:none] [&::-webkit-scrollbar]:hidden">
                                <span class=if all_acceptable {
                                    "w-2 h-2 shrink-0 rounded-full bg-emerald-400 shadow-[0_0_8px_#34d399]"
                                } else {
                                    "w-2 h-2 shrink-0 rounded-full bg-amber-400 shadow-[0_0_8px_#fbbf24]"
                                }></span>
                                <span class="text-[#ffffff] shrink-0 whitespace-nowrap">
                                    {if all_acceptable { "Ready to Scan" } else { "Low Contrast" }}
                                </span>
                                <span class="hidden sm:inline text-[#555555] shrink-0">"•"</span>
                                <span class="hidden sm:inline text-[#ffffff] shrink-0" title="Module Contrast">
                                    {format!("M:{:.1}:1", ratios.modules)}
                                </span>
                                <span class="hidden sm:inline text-[#555555] shrink-0">"•"</span>
                                <span class="hidden sm:inline text-[#ffffff] shrink-0" title="Eye Frame Contrast">
                                    {format!("E:{:.1}:1", ratios.eye_frame)}
                                </span>
                                <span class="hidden sm:inline text-[#555555] shrink-0">"•"</span>
                                <span class="hidden sm:inline text-[#ffffff] shrink-0" title="Eye Dot Contrast">
                                    {format!("D:{:.1}:1", ratios.eye_dot)}
                                </span>
                                <span class="hidden sm:inline text-[#555555] shrink-0">"•"</span>
                                <span class="hidden sm:inline text-[#a0a0a0] font-mono shrink-0">
                                    {format!(
                                        "v{} ({}×{})",
                                        rendered.matrix.version,
                                        rendered.matrix.size,
                                        rendered.matrix.size,
                                    )}
                                </span>
                            </div>

                            <div class="viewfinder-frame">
                                <div class="qr-svg-wrapper" inner_html=rendered.svg></div>
                            </div>

                            <div class="w-full max-w-[480px] space-y-3 pt-2 border-t border-[rgba(255,255,255,0.08)]">
                                <div class="flex items-center justify-between text-xs">
                                    <span class="text-[#a0a0a0] font-semibold uppercase text-[11px]">
                                        "Resolution"
                                    </span>
                                    <div class="flex gap-1">
                                        {[512u32, 1024, 2048].into_iter().map(|resolution| {
                                            let is_active =
                                                move || state.target_resolution.get() == resolution;
                                            view! {
                                                <button
                                                    class=move || {
                                                        if is_active() {
                                                            "px-2.5 py-0.5 rounded text-[11px] font-bold bg-[#00f0ff] text-[#000000]"
                                                        } else {
                                                            "px-2.5 py-0.5 rounded text-[11px] text-[#a0a0a0] hover:text-[#ffffff] hover:bg-[#151515]"
                                                        }
                                                    }
                                                    on:click=move |_| state.target_resolution.set(resolution)
                                                >
                                                    {format!("{resolution}px")}
                                                </button>
                                            }
                                        }).collect_view()}
                                    </div>
                                </div>

                                <div class="grid grid-cols-2 sm:grid-cols-4 gap-2">
                                    <button
                                        class="button button-primary w-full py-2 text-center text-xs font-bold"
                                        on:click=move |_| {
                                            let resolution = state.target_resolution.get();
                                            if let Err(error) =
                                                browser::download_png(&svg_for_png, resolution)
                                            {
                                                notify(state, &error);
                                            }
                                        }
                                    >
                                        {move || format!("↓ PNG ({}px)", state.target_resolution.get())}
                                    </button>

                                    <button
                                        class="button button-white w-full py-2 text-center text-xs font-semibold"
                                        on:click=move |_| {
                                            let resolution = state.target_resolution.get();
                                            let outcome = browser::copy_png(
                                                &svg_for_copy_png,
                                                resolution,
                                                move || notify(state, "COPIED PNG TO CLIPBOARD"),
                                                move || notify(state, "CLIPBOARD PERMISSION DENIED"),
                                            );
                                            if let Err(error) = outcome {
                                                notify(state, &error);
                                            }
                                        }
                                    >
                                        "Copy PNG"
                                    </button>

                                    <button
                                        class="button button-secondary w-full py-2 px-3 text-xs font-semibold text-center"
                                        on:click=move |_| {
                                            if let Err(error) = browser::download_svg(&svg_for_download) {
                                                notify(state, &error);
                                            }
                                        }
                                    >
                                        "↓ SVG"
                                    </button>

                                    <button
                                        class="button button-secondary w-full py-2 px-3 text-xs font-semibold text-center"
                                        on:click=move |_| {
                                            let outcome = browser::copy_svg(
                                                &svg_for_copy,
                                                move || notify(state, "COPIED SVG TO CLIPBOARD"),
                                                move || notify(state, "CLIPBOARD PERMISSION DENIED"),
                                            );
                                            if let Err(error) = outcome {
                                                notify(state, &error);
                                            }
                                        }
                                    >
                                        "Copy SVG"
                                    </button>
                                </div>
                            </div>
                        </div>
                    }.into_any()
                }
                Err(error) => view! {
                    <div class="p-6 text-center text-rose-400 text-sm">
                        {format!("Generation error: {error}")}
                    </div>
                }.into_any(),
            }}

            {move || state.notification.get().map(|message| {
                view! {
                    <div class="absolute bottom-4 px-4 py-1.5 rounded-full bg-[#050505] border border-[#00f0ff] text-[#00f0ff] text-xs font-bold shadow-[0_0_15px_rgba(0,240,255,0.35)] z-20">
                        {message}
                    </div>
                }
            })}
        </div>
    }
}
