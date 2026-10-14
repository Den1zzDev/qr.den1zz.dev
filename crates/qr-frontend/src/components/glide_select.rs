use leptos::prelude::*;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum PayloadTab {
    Url,
    Text,
    Wifi,
    VCard,
    Email,
    Phone,
    Geo,
    Crypto,
}

impl PayloadTab {
    pub const ALL: [PayloadTab; 8] = [
        PayloadTab::Url,
        PayloadTab::Text,
        PayloadTab::Wifi,
        PayloadTab::VCard,
        PayloadTab::Email,
        PayloadTab::Phone,
        PayloadTab::Geo,
        PayloadTab::Crypto,
    ];

    pub fn label(&self) -> &'static str {
        match self {
            PayloadTab::Url => "Link",
            PayloadTab::Text => "Text",
            PayloadTab::Wifi => "Wi-Fi",
            PayloadTab::VCard => "vCard",
            PayloadTab::Email => "Email",
            PayloadTab::Phone => "Phone",
            PayloadTab::Geo => "Location",
            PayloadTab::Crypto => "Crypto",
        }
    }

    pub fn header_title(&self) -> &'static str {
        match self {
            PayloadTab::Url => "Target URL",
            PayloadTab::Text => "Text / Raw Data",
            PayloadTab::Wifi => "Wi-Fi Network",
            PayloadTab::VCard => "Contact Card (vCard)",
            PayloadTab::Email => "Email Draft",
            PayloadTab::Phone => "Phone Number",
            PayloadTab::Geo => "Location Coordinates",
            PayloadTab::Crypto => "Cryptocurrency Payment",
        }
    }
}

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

#[component]
pub fn GlideSelect(active_tab: RwSignal<PayloadTab>) -> impl IntoView {
    let (is_open, set_is_open) = signal(false);
    let (hover_index, set_hover_index) = signal(Option::<usize>::None);

    let selected_index = move || {
        PayloadTab::ALL
            .iter()
            .position(|t| *t == active_tab.get())
            .unwrap_or(0)
    };

    let active_pill_index = move || hover_index.get().unwrap_or_else(selected_index);

    // Pill transform calculation using static table
    let pill_style = move || {
        let idx = active_pill_index().min(PILL_STYLES.len() - 1);
        PILL_STYLES[idx]
    };

    let handle_keydown = move |ev: web_sys::KeyboardEvent| {
        let key = ev.key();
        if !is_open.get() {
            if matches!(key.as_str(), "ArrowDown" | "ArrowUp" | "Enter" | " ") {
                ev.prevent_default();
                set_is_open.set(true);
                set_hover_index.set(Some(selected_index()));
            }
        } else {
            match key.as_str() {
                "Escape" => {
                    ev.prevent_default();
                    set_is_open.set(false);
                    set_hover_index.set(None);
                }
                "ArrowDown" => {
                    ev.prevent_default();
                    let cur = active_pill_index();
                    set_hover_index.set(Some((cur + 1) % PayloadTab::ALL.len()));
                }
                "ArrowUp" => {
                    ev.prevent_default();
                    let cur = active_pill_index();
                    set_hover_index.set(Some(if cur == 0 {
                        PayloadTab::ALL.len() - 1
                    } else {
                        cur - 1
                    }));
                }
                "Home" => {
                    ev.prevent_default();
                    set_hover_index.set(Some(0));
                }
                "End" => {
                    ev.prevent_default();
                    set_hover_index.set(Some(PayloadTab::ALL.len() - 1));
                }
                "Enter" | " " => {
                    ev.prevent_default();
                    let idx = active_pill_index().min(PayloadTab::ALL.len() - 1);
                    active_tab.set(PayloadTab::ALL[idx]);
                    set_is_open.set(false);
                    set_hover_index.set(None);
                }
                _ => {}
            }
        }
    };

    view! {
        <div class="glide-select" on:keydown=handle_keydown>
            // Backdrop for click-outside dismissal
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

            // Trigger Chip Button
            <button
                type="button"
                role="combobox"
                aria-haspopup="listbox"
                aria-expanded=move || is_open.get()
                class="glide-select__trigger"
                on:click=move |_| {
                    set_is_open.update(|v| *v = !*v);
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

            // Animated Dropdown Menu
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
                            // Gliding Pill Highlight
                            <span
                                class="glide-select__pill"
                                style=pill_style
                                aria-hidden="true"
                            />

                            // Menu Option Items
                            {PayloadTab::ALL.into_iter().enumerate().map(|(i, tab)| {
                                let is_selected = move || active_tab.get() == tab;
                                view! {
                                    <div
                                        role="option"
                                        aria-selected=is_selected
                                        data-index=i
                                        class="glide-select__option"
                                        on:mouseenter=move |_| set_hover_index.set(Some(i))
                                        on:mouseover=move |_| set_hover_index.set(Some(i))
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
                                            style=move || if is_selected() { "visibility: visible;" } else { "visibility: hidden;" }
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
