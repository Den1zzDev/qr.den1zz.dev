//! Chip-style dropdown for selecting the active payload tab.

use leptos::prelude::*;
use web_sys::KeyboardEvent;

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
    "transform: translateY(calc(0 * (var(--gs-row, 28px) + 1px))); opacity: 1;",
    "transform: translateY(calc(1 * (var(--gs-row, 28px) + 1px))); opacity: 1;",
    "transform: translateY(calc(2 * (var(--gs-row, 28px) + 1px))); opacity: 1;",
    "transform: translateY(calc(3 * (var(--gs-row, 28px) + 1px))); opacity: 1;",
    "transform: translateY(calc(4 * (var(--gs-row, 28px) + 1px))); opacity: 1;",
    "transform: translateY(calc(5 * (var(--gs-row, 28px) + 1px))); opacity: 1;",
    "transform: translateY(calc(6 * (var(--gs-row, 28px) + 1px))); opacity: 1;",
    "transform: translateY(calc(7 * (var(--gs-row, 28px) + 1px))); opacity: 1;",
];

/// Dropdown that selects the active payload tab.
#[component]
pub(crate) fn GlideSelect(active_tab: RwSignal<PayloadTab>) -> impl IntoView {
    let (is_open, set_is_open) = signal(false);
    let (hover_index, set_hover_index) = signal(Option::<usize>::None);

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
                set_is_open.set(false);
                set_hover_index.set(None);
            }
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
                set_is_open.set(false);
                set_hover_index.set(None);
            }
            _ => {}
        }
    };

    view! {
        <div class="glide-select" on:keydown=handle_keydown>
            // Backdrop for click-outside dismissal.
            {move || if is_open.get() {
                view! {
                    <div
                        class="fixed inset-0 z-40 bg-transparent cursor-default"
                        on:click=move |_| {
                            set_is_open.set(false);
                            set_hover_index.set(None);
                        }
                    />
                }.into_any()
            } else {
                view! { <span /> }.into_any()
            }}

            <button
                type="button"
                role="combobox"
                aria-haspopup="listbox"
                aria-expanded=move || is_open.get()
                class="glide-select__trigger"
                on:click=move |_| {
                    set_is_open.update(|open| *open = !*open);
                    set_hover_index.set(None);
                }
            >
                <span class="glide-select__label">
                    {move || active_tab.get().label()}
                </span>
                <span class="glide-select__chevron" aria-hidden="true">
                    <svg width="12" height="12" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2.5" stroke-linecap="round" stroke-linejoin="round">
                        <polyline points="6 9 12 15 18 9"></polyline>
                    </svg>
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
                            role="listbox"
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
                                        role="option"
                                        aria-selected=is_selected
                                        data-index=index
                                        class="glide-select__option"
                                        on:mouseenter=move |_| set_hover_index.set(Some(index))
                                        on:mouseover=move |_| set_hover_index.set(Some(index))
                                        on:click=move |_| {
                                            active_tab.set(tab);
                                            set_is_open.set(false);
                                            set_hover_index.set(None);
                                        }
                                    >
                                        <span class="glide-select__name">{tab.label()}</span>
                                        <span
                                            class="glide-select__check"
                                            data-on=move || is_selected().then_some("")
                                            style=move || if is_selected() {
                                                "visibility: visible;"
                                            } else {
                                                "visibility: hidden;"
                                            }
                                            aria-hidden="true"
                                        >
                                            <svg width="13" height="13" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2.5" stroke-linecap="round" stroke-linejoin="round">
                                                <polyline points="20 6 9 17 4 12"></polyline>
                                            </svg>
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
