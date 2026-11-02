//! Palette controls for module, finder, and background colours.

use leptos::prelude::*;

use crate::studio_state::StudioState;

/// Colours offered for modules and finder patterns.
const CODE_PRESETS: [&str; 4] = ["#000000", "#008b99", "#00f0ff", "#ffffff"];

/// Colours offered for the finder dot.
const DOT_PRESETS: [&str; 4] = ["#008b99", "#00f0ff", "#000000", "#ffffff"];

/// Colours offered for the background.
const BACKGROUND_PRESETS: [&str; 4] = ["#ffffff", "#000000", "#f8fafc", "#0a0a0a"];

/// Colour pickers for every coloured region of the code.
#[component]
pub(crate) fn PalettePanel(state: StudioState) -> impl IntoView {
    view! {
        <div class="glass-panel p-4 sm:p-5 space-y-4">
            <h3 class="text-xs font-semibold uppercase tracking-wider text-[#a0a0a0]">
                "Colors & Palette"
            </h3>
            <div class="grid grid-cols-1 sm:grid-cols-2 gap-4">
                <div class="space-y-1.5">
                    <div class="flex items-center justify-between text-xs">
                        <span class="text-[#a0a0a0]">"Code / Modules"</span>
                        <span class="font-mono text-[11px] text-[#555555]">
                            {move || state.module_color.get()}
                        </span>
                    </div>
                    <div class="flex items-center gap-2">
                        <input
                            type="color"
                            class="w-8 h-8 rounded cursor-pointer bg-transparent border-0"
                            prop:value=move || state.module_color.get()
                            on:input=move |event| {
                                state.module_color.set(event_target_value(&event));
                            }
                        />
                        <div class="flex gap-1.5 flex-wrap">
                            {CODE_PRESETS.into_iter().map(|preset| {
                                view! {
                                    <button
                                        class="w-6 h-6 rounded-full border border-[rgba(255,255,255,0.2)] transition-transform hover:scale-110"
                                        style=format!("background-color: {preset}")
                                        title=format!("Set color to {preset}")
                                        on:click=move |_| state.module_color.set(preset.to_string())
                                    />
                                }
                            }).collect_view()}
                        </div>
                    </div>
                </div>

                <div class="space-y-1.5">
                    <div class="flex items-center justify-between text-xs">
                        <span class="text-[#a0a0a0]">"Eye Frame"</span>
                        <button
                            class="text-[11px] text-[#00f0ff] hover:underline"
                            on:click=move |_| state.eye_color.set(state.module_color.get())
                        >
                            "Match Code"
                        </button>
                    </div>
                    <div class="flex items-center gap-2">
                        <input
                            type="color"
                            class="w-8 h-8 rounded cursor-pointer bg-transparent border-0"
                            prop:value=move || state.eye_color.get()
                            on:input=move |event| {
                                state.eye_color.set(event_target_value(&event));
                            }
                        />
                        <div class="flex gap-1.5 flex-wrap">
                            {CODE_PRESETS.into_iter().map(|preset| {
                                view! {
                                    <button
                                        class="w-6 h-6 rounded-full border border-[rgba(255,255,255,0.2)] transition-transform hover:scale-110"
                                        style=format!("background-color: {preset}")
                                        title=format!("Set color to {preset}")
                                        on:click=move |_| state.eye_color.set(preset.to_string())
                                    />
                                }
                            }).collect_view()}
                        </div>
                    </div>
                </div>

                <div class="space-y-1.5">
                    <div class="flex items-center justify-between text-xs">
                        <span class="text-[#a0a0a0]">"Eye Dot"</span>
                        <button
                            class="text-[11px] text-[#00f0ff] hover:underline"
                            on:click=move |_| state.dot_color.set(state.eye_color.get())
                        >
                            "Match Frame"
                        </button>
                    </div>
                    <div class="flex items-center gap-2">
                        <input
                            type="color"
                            class="w-8 h-8 rounded cursor-pointer bg-transparent border-0"
                            prop:value=move || state.dot_color.get()
                            on:input=move |event| {
                                state.dot_color.set(event_target_value(&event));
                            }
                        />
                        <div class="flex gap-1.5 flex-wrap">
                            {DOT_PRESETS.into_iter().map(|preset| {
                                view! {
                                    <button
                                        class="w-6 h-6 rounded-full border border-[rgba(255,255,255,0.2)] transition-transform hover:scale-110"
                                        style=format!("background-color: {preset}")
                                        title=format!("Set color to {preset}")
                                        on:click=move |_| state.dot_color.set(preset.to_string())
                                    />
                                }
                            }).collect_view()}
                        </div>
                    </div>
                </div>

                <div class="space-y-1.5">
                    <div class="flex items-center justify-between text-xs">
                        <span class="text-[#a0a0a0]">"Background"</span>
                        <label class="flex items-center gap-1.5 cursor-pointer text-[11px] text-[#00f0ff]">
                            <input
                                type="checkbox"
                                class="rounded accent-[#00f0ff] prec"
                                prop:checked=move || state.is_transparent_background.get()
                                on:change=move |event| {
                                    state.is_transparent_background.set(event_target_checked(&event));
                                }
                            />
                            <span>"Transparent"</span>
                        </label>
                    </div>
                    <div class="flex items-center gap-2">
                        <input
                            type="color"
                            class="w-8 h-8 rounded cursor-pointer bg-transparent border-0 disabled:opacity-40"
                            disabled=move || state.is_transparent_background.get()
                            prop:value=move || state.background_color.get()
                            on:input=move |event| {
                                state.background_color.set(event_target_value(&event));
                            }
                        />
                        <div class="flex gap-1.5 flex-wrap">
                            {BACKGROUND_PRESETS.into_iter().map(|preset| {
                                view! {
                                    <button
                                        class="w-6 h-6 rounded-full border border-[rgba(255,255,255,0.2)] transition-transform hover:scale-110 disabled:opacity-30"
                                        disabled=move || state.is_transparent_background.get()
                                        style=format!("background-color: {preset}")
                                        title=format!("Set color to {preset}")
                                        on:click=move |_| state.background_color.set(preset.to_string())
                                    />
                                }
                            }).collect_view()}
                        </div>
                    </div>
                </div>
            </div>
        </div>
    }
}
