//! Palette controls for module, finder, and background colours.

use leptos::prelude::*;
use qr_core::contrast::MIN_GRAPHIC_CONTRAST;

use crate::browser;
use crate::components::icons::Check;
use crate::components::icons::Copy;
use crate::components::icons::Pipette;
use crate::components::icons::Warning;
use crate::studio_state::ContrastRatios;
use crate::studio_state::StudioState;

/// Colours offered for modules and finder patterns.
const CODE_PRESETS: [&str; 4] = ["#000000", "#008b99", "#00f0ff", "#ffffff"];

/// Colours offered for the finder dot.
const DOT_PRESETS: [&str; 4] = ["#008b99", "#00f0ff", "#000000", "#ffffff"];

/// Colours offered for the background.
const BACKGROUND_PRESETS: [&str; 3] = ["#ffffff", "#f2f2f2", "#000000"];

/// Renders a contrast badge for `value`.
fn ratio_badge(value: f32, label: &'static str) -> AnyView {
    let passes = value >= MIN_GRAPHIC_CONTRAST;
    let description = if passes {
        format!("{label} contrast {value:.1} to 1, meets the 3 to 1 minimum")
    } else {
        format!("{label} contrast {value:.1} to 1, below the 3 to 1 minimum")
    };

    view! {
        <span
            class=if passes { "ratio ratio--pass" } else { "ratio ratio--fail" }
            aria-label=description.clone()
            title=description
        >
            {if passes {
                view! { <Check size=10 /> }.into_any()
            } else {
                view! { <Warning size=10 /> }.into_any()
            }}
            <span class="tabular">{format!("{value:.1}:1")}</span>
        </span>
    }
    .into_any()
}

/// Native colour picker styled as a swatch.
#[component]
fn CustomColor(
    value: RwSignal<String>,
    label: &'static str,
    disabled: Signal<bool>,
) -> impl IntoView {
    view! {
        <label
            class=move || if disabled.get() { "color-custom is-disabled" } else { "color-custom" }
            title=move || format!("Pick a custom colour ({})", value.get())
        >
            <span
                class="color-custom__chip"
                style=move || format!("background-color: {}", value.get())
                aria-hidden="true"
            ></span>
            <span class="color-custom__icon" aria-hidden="true">
                <Pipette size=9 />
            </span>
            <input
                type="color"
                class="color-custom__input"
                aria-label=format!("{label} custom colour")
                disabled=move || disabled.get()
                prop:value=move || value.get()
                on:input=move |event| value.set(event_target_value(&event))
            />
        </label>
    }
}

/// Preset swatches for one colour role.
#[component]
fn Swatches(
    value: RwSignal<String>,
    presets: &'static [&'static str],
    label: &'static str,
    disabled: Signal<bool>,
) -> impl IntoView {
    view! {
        <div class="color-row__swatches">
            {presets.iter().copied().map(|preset| {
                let is_selected = move || value.get().eq_ignore_ascii_case(preset);
                view! {
                    <button
                        type="button"
                        class=move || if is_selected() { "swatch is-selected" } else { "swatch" }
                        style=format!("background-color: {preset}")
                        aria-label=format!("Set {label} to {preset}")
                        aria-pressed=move || is_selected().to_string()
                        title=preset
                        disabled=move || disabled.get()
                        on:click=move |_| value.set(preset.to_string())
                    >
                        <span class="swatch__check" aria-hidden="true">
                            <Check size=12 />
                        </span>
                    </button>
                }
            }).collect_view()}
        </div>
    }
}

/// Readout that copies its colour value when clicked.
#[component]
fn HexValue(
    state: StudioState,
    value: RwSignal<String>,
    label: &'static str,
    disabled: Signal<bool>,
) -> impl IntoView {
    view! {
        <button
            type="button"
            class="color-row__hex"
            title=move || format!("Copy {}", value.get())
            aria-label=move || format!("Copy {label} {}", value.get())
            disabled=move || disabled.get()
            on:click=move |_| {
                let hex = value.get_untracked();
                let message = format!("{label} {hex} copied");
                let outcome = browser::copy_text(
                    &hex,
                    move || state.notify(&message, false),
                    move |error| state.notify(&error, true),
                );
                if let Err(error) = outcome {
                    state.notify(&error, true);
                }
            }
        >
            {move || value.get()}
        </button>
    }
}

/// Colour pickers for every coloured region of the code.
#[component]
pub(crate) fn PalettePanel(state: StudioState, contrast: Memo<ContrastRatios>) -> impl IntoView {
    let transparent = Signal::derive(move || state.is_transparent_background.get());

    view! {
        <div class="panel">
            <div class="panel__head">
                <h2 class="panel__title">"Palette"</h2>
                <button
                    type="button"
                    class="link-button"
                    aria-label="Reset the palette to the default colours"
                    on:click=move |_| state.reset_palette()
                >
                    "Reset"
                </button>
            </div>
            <div class="panel__body space-y-4">
                <div class="palette-grid">
                    <div class="color-row">
                        <div class="color-row__head">
                            <span class="color-row__name">"Modules"</span>
                            <span class="color-row__meta">
                                {move || ratio_badge(contrast.get().modules, "Module")}
                                <HexValue
                                    state=state
                                    value=state.module_color
                                    label="Module colour"
                                    disabled=Signal::derive(|| false)
                                />
                            </span>
                        </div>
                        <CustomColor
                            value=state.module_color
                            label="Module colour"
                            disabled=Signal::derive(|| false)
                        />
                        <Swatches
                            value=state.module_color
                            presets=&CODE_PRESETS
                            label="modules"
                            disabled=Signal::derive(|| false)
                        />
                    </div>

                    <div class="color-row">
                        <div class="color-row__head">
                            <span class="color-row__name">"Eye frame"</span>
                            <span class="color-row__meta">
                                {move || ratio_badge(contrast.get().eye_frame, "Eye frame")}
                                <button
                                    type="button"
                                    class="icon-link"
                                    title="Use the module colour"
                                    aria-label="Use the module colour"
                                    disabled=move || {
                                        state.eye_color.get() == state.module_color.get()
                                    }
                                    on:click=move |_| state.eye_color.set(state.module_color.get())
                                >
                                    <Copy size=11 />
                                </button>
                                <HexValue
                                    state=state
                                    value=state.eye_color
                                    label="Eye frame colour"
                                    disabled=Signal::derive(|| false)
                                />
                            </span>
                        </div>
                        <CustomColor
                            value=state.eye_color
                            label="Eye frame colour"
                            disabled=Signal::derive(|| false)
                        />
                        <Swatches
                            value=state.eye_color
                            presets=&CODE_PRESETS
                            label="eye frame"
                            disabled=Signal::derive(|| false)
                        />
                    </div>

                    <div class="color-row">
                        <div class="color-row__head">
                            <span class="color-row__name">"Eye dot"</span>
                            <span class="color-row__meta">
                                {move || ratio_badge(contrast.get().eye_dot, "Eye dot")}
                                <button
                                    type="button"
                                    class="icon-link"
                                    title="Use the eye frame colour"
                                    aria-label="Use the eye frame colour"
                                    disabled=move || {
                                        state.dot_color.get() == state.eye_color.get()
                                    }
                                    on:click=move |_| state.dot_color.set(state.eye_color.get())
                                >
                                    <Copy size=11 />
                                </button>
                                <HexValue
                                    state=state
                                    value=state.dot_color
                                    label="Eye dot colour"
                                    disabled=Signal::derive(|| false)
                                />
                            </span>
                        </div>
                        <CustomColor
                            value=state.dot_color
                            label="Eye dot colour"
                            disabled=Signal::derive(|| false)
                        />
                        <Swatches
                            value=state.dot_color
                            presets=&DOT_PRESETS
                            label="eye dot"
                            disabled=Signal::derive(|| false)
                        />
                    </div>

                    <div class="color-row">
                        <div class="color-row__head">
                            <span class="color-row__name">"Background"</span>
                            <span class="color-row__meta">
                                <label class="check check--compact">
                                    <input
                                        type="checkbox"
                                        prop:checked=move || state.is_transparent_background.get()
                                        on:change=move |event| {
                                            state.is_transparent_background.set(event_target_checked(&event));
                                        }
                                    />
                                    <span>"Transparent"</span>
                                </label>
                                <HexValue
                                    state=state
                                    value=state.background_color
                                    label="Background colour"
                                    disabled=transparent
                                />
                            </span>
                        </div>
                        <CustomColor
                            value=state.background_color
                            label="Background colour"
                            disabled=transparent
                        />
                        <Swatches
                            value=state.background_color
                            presets=&BACKGROUND_PRESETS
                            label="background"
                            disabled=transparent
                        />
                    </div>
                </div>

                {move || {
                    let ratios = contrast.get();
                    let failures = ratios.failures();
                    if failures == 0 {
                        view! { <span /> }.into_any()
                    } else {
                        view! {
                            <p class="ratio ratio--fail w-full justify-start gap-1.5 py-1.5">
                                <Warning size=12 />
                                {if failures == 1 {
                                    "1 region is below the 3:1 minimum.".to_string()
                                } else {
                                    format!("{failures} regions are below the 3:1 minimum.")
                                }}
                            </p>
                        }.into_any()
                    }
                }}

                <p class="field-hint">
                    {move || if transparent.get() {
                        "Contrast is checked against white. Transparent exports need a light surface. Click a hex to copy."
                    } else {
                        "Contrast is measured against the background. Click a hex to copy."
                    }}
                </p>
            </div>
        </div>
    }
}
