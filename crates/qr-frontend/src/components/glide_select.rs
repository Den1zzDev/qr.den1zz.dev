//! Chip-style dropdown for selecting the active payload tab.

use std::time::Duration;

use leptos::html;
use leptos::prelude::*;
use web_sys::KeyboardEvent;

use crate::components::icons::Check;
use crate::components::icons::ChevronDown;
use crate::components::icons::PayloadIcon;

/// Payload kinds supported by the editor.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) enum PayloadTab {
    /// Website link.
    Url,
    /// Plain text.
    Text,
    /// Wi-Fi credentials.
    Wifi,
    /// Contact card.
    VCard,
    /// Email draft.
    Email,
    /// Telephone number.
    Phone,
    /// Geographic coordinates.
    Geo,
    /// Cryptocurrency payment.
    Crypto,
}

impl PayloadTab {
    /// Every tab, in display order.
    pub(crate) const ALL: [Self; 8] = [
        Self::Url,
        Self::Text,
        Self::Wifi,
        Self::VCard,
        Self::Email,
        Self::Phone,
        Self::Geo,
        Self::Crypto,
    ];

    /// Returns the short label shown in the dropdown.
    pub(crate) fn label(&self) -> &'static str {
        match self {
            Self::Url => "Link",
            Self::Text => "Text",
            Self::Wifi => "Wi-Fi",
            Self::VCard => "vCard",
            Self::Email => "Email",
            Self::Phone => "Phone",
            Self::Geo => "Location",
            Self::Crypto => "Crypto",
        }
    }

    /// Returns the heading shown above the editor.
    pub(crate) fn header_title(&self) -> &'static str {
        match self {
            Self::Url => "Target URL",
            Self::Text => "Text / Raw Data",
            Self::Wifi => "Wi-Fi Network",
            Self::VCard => "Contact Card (vCard)",
            Self::Email => "Email Draft",
            Self::Phone => "Phone Number",
            Self::Geo => "Location Coordinates",
            Self::Crypto => "Cryptocurrency Payment",
        }
    }
}

/// Vertical offsets for the animated selection pill.
const PILL_STYLES: [&str; 8] = [
    "transform: translateY(calc(0 * (var(--gs-row, 32px) + 1px))); opacity: 1;",
    "transform: translateY(calc(1 * (var(--gs-row, 32px) + 1px))); opacity: 1;",
    "transform: translateY(calc(2 * (var(--gs-row, 32px) + 1px))); opacity: 1;",
    "transform: translateY(calc(3 * (var(--gs-row, 32px) + 1px))); opacity: 1;",
    "transform: translateY(calc(4 * (var(--gs-row, 32px) + 1px))); opacity: 1;",
    "transform: translateY(calc(5 * (var(--gs-row, 32px) + 1px))); opacity: 1;",
    "transform: translateY(calc(6 * (var(--gs-row, 32px) + 1px))); opacity: 1;",
    "transform: translateY(calc(7 * (var(--gs-row, 32px) + 1px))); opacity: 1;",
];

/// Dropdown that selects the active payload tab.
#[component]
pub(crate) fn GlideSelect(active_tab: RwSignal<PayloadTab>) -> impl IntoView {
    let (is_open, set_is_open) = signal(false);
    let (hover_index, set_hover_index) = signal(Option::<usize>::None);
    let (is_swapping, set_is_swapping) = signal(false);
    let trigger_ref = NodeRef::<html::Button>::new();

    let selected_index = move || {
        PayloadTab::ALL
            .iter()
            .position(|tab| *tab == active_tab.get())
            .unwrap_or(0)
    };
    let active_pill_index = move || hover_index.get().unwrap_or_else(selected_index);
    let pill_style = move || {
        let index = active_pill_index().min(PILL_STYLES.len() - 1);
        PILL_STYLES[index]
    };

    Effect::new(move |_| {
        let _ = active_tab.get();
        set_is_swapping.set(true);
        set_timeout(
            move || set_is_swapping.set(false),
            Duration::from_millis(180),
        );
    });

    let close_menu = move || {
        set_is_open.set(false);
        set_hover_index.set(None);
    };

    let handle_keydown = move |event: KeyboardEvent| {
        let key = event.key();
        if !is_open.get() {
            if matches!(key.as_str(), "ArrowDown" | "ArrowUp" | "Enter" | " ") {
                event.prevent_default();
                set_is_open.set(true);
                set_hover_index.set(Some(selected_index()));
            }
            return;
        }

        match key.as_str() {
            "Escape" => {
                event.prevent_default();
                close_menu();
                if let Some(trigger) = trigger_ref.get() {
                    let _ = trigger.focus();
                }
            }
            "Tab" => close_menu(),
            "ArrowDown" => {
                event.prevent_default();
                let current = active_pill_index();
                set_hover_index.set(Some((current + 1) % PayloadTab::ALL.len()));
            }
            "ArrowUp" => {
                event.prevent_default();
                let current = active_pill_index();
                set_hover_index.set(Some(if current == 0 {
                    PayloadTab::ALL.len() - 1
                } else {
                    current - 1
                }));
            }
            "Home" => {
                event.prevent_default();
                set_hover_index.set(Some(0));
            }
            "End" => {
                event.prevent_default();
                set_hover_index.set(Some(PayloadTab::ALL.len() - 1));
            }
            "Enter" | " " => {
                event.prevent_default();
                let index = active_pill_index().min(PayloadTab::ALL.len() - 1);
                active_tab.set(PayloadTab::ALL[index]);
                close_menu();
            }
            _ => {}
        }
    };

    view! {
        <div
            class="glide-select"
            on:keydown=handle_keydown
            data-swap=move || is_swapping.get().then_some("")
        >
            // Backdrop for click-outside dismissal.
            {move || if is_open.get() {
                view! {
                    <div
                        class="fixed inset-0 z-40 bg-transparent cursor-default"
                        aria-hidden="true"
                        on:click=move |_| close_menu()
                    />
                }.into_any()
            } else {
                view! { <span /> }.into_any()
            }}

            <button
                type="button"
                node_ref=trigger_ref
                role="combobox"
                aria-label="Content type"
                aria-haspopup="listbox"
                aria-controls="payload-type-list"
                aria-expanded=move || is_open.get().to_string()
                aria-activedescendant=move || {
                    is_open
                        .get()
                        .then(|| format!("payload-type-{}", active_pill_index()))
                }
                class="glide-select__trigger"
                on:click=move |_| {
                    set_is_open.update(|open| *open = !*open);
                    set_hover_index.set(Some(selected_index()));
                }
            >
                <span class="glide-select__trigger-icon" aria-hidden="true">
                    {move || view! { <PayloadIcon tab=active_tab.get() size=14 /> }}
                </span>
                <span class="glide-select__label">{move || active_tab.get().label()}</span>
                <span class="glide-select__chevron" aria-hidden="true">
                    <ChevronDown size=12 />
                </span>
            </button>

            {move || if is_open.get() {
                view! {
                    <div
                        class="glide-select__menu"
                        data-state="open"
                        data-side="bottom"
                        data-align="right"
                    >
                        <div
                            id="payload-type-list"
                            role="listbox"
                            aria-label="Content type"
                            class="glide-select__list"
                            data-live=""
                            on:mouseleave=move |_| set_hover_index.set(None)
                        >
                            <span
                                class="glide-select__pill"
                                style=pill_style
                                aria-hidden="true"
                            />

                            {PayloadTab::ALL.into_iter().enumerate().map(|(index, tab)| {
                                let is_selected = move || active_tab.get() == tab;
                                view! {
                                    <div
                                        id=format!("payload-type-{index}")
                                        role="option"
                                        aria-selected=move || is_selected().to_string()
                                        data-index=index
                                        class="glide-select__option"
                                        on:mouseenter=move |_| set_hover_index.set(Some(index))
                                        on:mouseover=move |_| set_hover_index.set(Some(index))
                                        on:click=move |_| {
                                            active_tab.set(tab);
                                            close_menu();
                                        }
                                    >
                                        <span class="glide-select__option-icon" aria-hidden="true">
                                            <PayloadIcon tab=tab size=14 />
                                        </span>
                                        <span class="glide-select__name">{tab.label()}</span>
                                        <span
                                            class="glide-select__check"
                                            aria-hidden="true"
                                            style=move || if is_selected() {
                                                "visibility: visible;"
                                            } else {
                                                "visibility: hidden;"
                                            }
                                        >
                                            <Check size=13 />
                                        </span>
                                    </div>
                                }
                            }).collect_view()}
                        </div>
                    </div>
                }.into_any()
            } else {
                view! { <span /> }.into_any()
            }}
        </div>
    }
}
