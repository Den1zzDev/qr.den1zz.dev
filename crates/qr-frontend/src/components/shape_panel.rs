//! Controls for module and finder pattern shapes.

use leptos::prelude::*;
use qr_core::vector::EyeDotShape;
use qr_core::vector::EyeFrameShape;
use qr_core::vector::ModuleShape;

use crate::studio_state::StudioState;

/// Class applied to the selected option button.
const SELECTED_BUTTON: &str = "w-full py-1.5 px-3 rounded-lg text-xs font-bold bg-[#00f0ff] text-[#000000] border border-[#00f0ff] text-left transition-all";

/// Class applied to an unselected option button.
const IDLE_BUTTON: &str = "w-full py-1.5 px-3 rounded-lg text-xs font-medium text-[#a0a0a0] bg-[#111111] border border-[rgba(255,255,255,0.08)] hover:text-[#ffffff] text-left transition-all";

/// Shape selectors for modules and finder patterns.
#[component]
pub(crate) fn ShapePanel(state: StudioState) -> impl IntoView {
    view! {
        <div class="glass-panel p-4 sm:p-5 space-y-4">
            <h3 class="text-xs font-semibold uppercase tracking-wider text-[#a0a0a0]">
                "Shapes & Style"
            </h3>
            <div class="grid grid-cols-1 sm:grid-cols-3 gap-3">
                <div class="space-y-1.5">
                    <span class="text-[11px] text-[#a0a0a0]">"Module Shape"</span>
                    <div class="flex flex-col gap-1">
                        {[
                            (ModuleShape::Classy, "Classy"),
                            (ModuleShape::Smooth, "Smooth"),
                            (ModuleShape::Dots, "Dots"),
                            (ModuleShape::Square, "Square"),
                        ].into_iter().map(|(shape, label)| {
                            let is_selected = move || state.module_shape.get() == shape;
                            view! {
                                <button
                                    class=move || if is_selected() {
                                        SELECTED_BUTTON
                                    } else {
                                        IDLE_BUTTON
                                    }
                                    on:click=move |_| state.module_shape.set(shape)
                                >
                                    {label}
                                </button>
                            }
                        }).collect_view()}
                    </div>
                </div>

                <div class="space-y-1.5">
                    <span class="text-[11px] text-[#a0a0a0]">"Eye Frame Shape"</span>
                    <div class="flex flex-col gap-1">
                        {[
                            (EyeFrameShape::Rounded, "Round Frame"),
                            (EyeFrameShape::Circle, "Circle Frame"),
                            (EyeFrameShape::Square, "Square Frame"),
                        ].into_iter().map(|(shape, label)| {
                            let is_selected = move || state.eye_frame_shape.get() == shape;
                            view! {
                                <button
                                    class=move || if is_selected() {
                                        SELECTED_BUTTON
                                    } else {
                                        IDLE_BUTTON
                                    }
                                    on:click=move |_| state.eye_frame_shape.set(shape)
                                >
                                    {label}
                                </button>
                            }
                        }).collect_view()}
                    </div>
                </div>

                <div class="space-y-1.5">
                    <span class="text-[11px] text-[#a0a0a0]">"Eye Dot Shape"</span>
                    <div class="flex flex-col gap-1">
                        {[
                            (EyeDotShape::Circle, "Circle Dot"),
                            (EyeDotShape::Rounded, "Round Dot"),
                            (EyeDotShape::Square, "Square Dot"),
                        ].into_iter().map(|(shape, label)| {
                            let is_selected = move || state.eye_dot_shape.get() == shape;
                            view! {
                                <button
                                    class=move || if is_selected() {
                                        SELECTED_BUTTON
                                    } else {
                                        IDLE_BUTTON
                                    }
                                    on:click=move |_| state.eye_dot_shape.set(shape)
                                >
                                    {label}
                                </button>
                            }
                        }).collect_view()}
                    </div>
                </div>
            </div>
        </div>
    }
}
