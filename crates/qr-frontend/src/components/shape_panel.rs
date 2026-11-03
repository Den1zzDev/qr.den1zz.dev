//! Controls for module and finder pattern shapes.

use leptos::prelude::*;
use qr_core::vector::EyeDotShape;
use qr_core::vector::EyeFrameShape;
use qr_core::vector::ModuleShape;

use crate::components::icons::Check;
use crate::components::icons::EyeDotGlyph;
use crate::components::icons::EyeFrameGlyph;
use crate::components::icons::ModuleShapeGlyph;
use crate::components::icons::Warning;
use crate::studio_state::StudioState;

/// Shape selectors for modules and finder patterns.
#[component]
pub(crate) fn ShapePanel(state: StudioState) -> impl IntoView {
    view! {
        <div class="panel">
            <div class="panel__head">
                <h2 class="panel__title">"Modules & eyes"</h2>
                <button
                    type="button"
                    class="link-button"
                    aria-label="Reset module and eye shapes"
                    on:click=move |_| state.reset_shapes()
                >
                    "Reset"
                </button>
            </div>
            <div class="panel__body">
                <div class="grid grid-cols-1 gap-4 sm:grid-cols-3">
                    <fieldset class="option-group">
                        <legend>"Module shape"</legend>
                        <div class="option-list">
                            {[
                                (ModuleShape::Classy, "Classy"),
                                (ModuleShape::Smooth, "Smooth"),
                                (ModuleShape::Dots, "Dots"),
                                (ModuleShape::Square, "Square"),
                            ].into_iter().map(|(shape, label)| {
                                let is_selected = move || state.module_shape.get() == shape;
                                view! {
                                    <label class=move || if is_selected() { "option is-selected" } else { "option" }>
                                        <input
                                            type="radio"
                                            name="module-shape"
                                            prop:checked=is_selected
                                            on:change=move |_| state.module_shape.set(shape)
                                        />
                                        <span class="option__icon" aria-hidden="true">
                                            <ModuleShapeGlyph shape=shape />
                                        </span>
                                        <span class="option__label">{label}</span>
                                        <span class="option__check" aria-hidden="true">
                                            <Check size=13 />
                                        </span>
                                    </label>
                                }
                            }).collect_view()}
                        </div>
                    </fieldset>

                    <fieldset class="option-group">
                        <legend>"Eye frame"</legend>
                        <div class="option-list">
                            {[
                                (EyeFrameShape::Rounded, "Rounded"),
                                (EyeFrameShape::Circle, "Circle"),
                                (EyeFrameShape::Square, "Square"),
                            ].into_iter().map(|(shape, label)| {
                                let is_selected = move || state.eye_frame_shape.get() == shape;
                                view! {
                                    <label class=move || if is_selected() { "option is-selected" } else { "option" }>
                                        <input
                                            type="radio"
                                            name="eye-frame-shape"
                                            prop:checked=is_selected
                                            on:change=move |_| state.eye_frame_shape.set(shape)
                                        />
                                        <span class="option__icon" aria-hidden="true">
                                            <EyeFrameGlyph shape=shape />
                                        </span>
                                        <span class="option__label">{label}</span>
                                        <span class="option__check" aria-hidden="true">
                                            <Check size=13 />
                                        </span>
                                    </label>
                                }
                            }).collect_view()}
                        </div>
                    </fieldset>

                    <fieldset class="option-group">
                        <legend>"Eye dot"</legend>
                        <div class="option-list">
                            {[
                                (EyeDotShape::Circle, "Circle"),
                                (EyeDotShape::Rounded, "Rounded"),
                                (EyeDotShape::Square, "Square"),
                            ].into_iter().map(|(shape, label)| {
                                let is_selected = move || state.eye_dot_shape.get() == shape;
                                view! {
                                    <label class=move || if is_selected() { "option is-selected" } else { "option" }>
                                        <input
                                            type="radio"
                                            name="eye-dot-shape"
                                            prop:checked=is_selected
                                            on:change=move |_| state.eye_dot_shape.set(shape)
                                        />
                                        <span class="option__icon" aria-hidden="true">
                                            <EyeDotGlyph shape=shape />
                                        </span>
                                        <span class="option__label">{label}</span>
                                        <span class="option__check" aria-hidden="true">
                                            <Check size=13 />
                                        </span>
                                    </label>
                                }
                            }).collect_view()}
                        </div>
                    </fieldset>
                </div>
                {move || (state.eye_frame_shape.get() == EyeFrameShape::Circle).then(|| view! {
                    <p class="field-hint field-hint--warn" role="status">
                        <Warning size=12 />
                        "Circular eye frames may fail some scanners. Test-scan before printing."
                    </p>
                })}
            </div>
        </div>
    }
}
