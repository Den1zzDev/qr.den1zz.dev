//! Payload editor for every supported payload kind.

use leptos::prelude::*;
use qr_core::payload;
use qr_core::payload::CryptoCurrency;
use qr_core::payload::Payload;
use qr_core::payload::UrlQueryInfo;
use qr_core::payload::WifiAuth;

use crate::components::glide_select::GlideSelect;
use crate::components::glide_select::PayloadTab;
use crate::studio_state::StudioState;

/// Editor for the payload that will be encoded.
#[component]
pub(crate) fn PayloadEditor(
    state: StudioState,
    payload: Memo<Payload>,
    query_info: Memo<Option<UrlQueryInfo>>,
) -> impl IntoView {
    let (show_wifi_password, set_show_wifi_password) = signal(false);
    let sanitized = Memo::new(move |_| payload.get().sanitize());

    view! {
        <div class="glass-panel p-4 sm:p-5 space-y-3">
            <div class="flex items-center justify-between">
                <label class="text-xs font-semibold uppercase tracking-wider text-[#a0a0a0]">
                    {move || state.active_tab.get().header_title()}
                </label>
                <GlideSelect active_tab=state.active_tab />
            </div>

            <div>
                {move || match state.active_tab.get() {
                    PayloadTab::Url => view! {
                        <div class="space-y-3">
                            <div class="relative flex items-center">
                                <input
                                    type="url"
                                    class="w-full text-sm font-mono pr-8 py-2.5 px-3 rounded-lg bg-[#050505] border border-[rgba(255,255,255,0.12)] focus:border-[#00f0ff] text-[#ffffff] placeholder:text-[#555555] outline-none"
                                    placeholder="https://den1zz.dev"
                                    prop:value=move || state.url_input.get()
                                    on:input=move |event| {
                                        state.url_input.set(event_target_value(&event));
                                    }
                                />
                                {move || if !state.url_input.get().is_empty() {
                                    view! {
                                        <button
                                            type="button"
                                            title="Clear URL"
                                            class="absolute right-2 text-[#555555] hover:text-[#ffffff] text-sm px-1.5 py-0.5 rounded transition-colors"
                                            on:click=move |_| state.url_input.set(String::new())
                                        >
                                            "✕"
                                        </button>
                                    }.into_any()
                                } else {
                                    view! { <span /> }.into_any()
                                }}
                            </div>

                            {move || if let Some(info) = query_info.get() {
                                view! {
                                    <div class="p-3 rounded-lg bg-[#0c0c0c] border border-[rgba(0,240,255,0.25)] text-xs space-y-2">
                                        <div class="flex items-center justify-between">
                                            <span class="text-[#00f0ff] font-semibold">
                                                {format!("Detected {} query parameter(s)", info.total_param_count)}
                                            </span>
                                            <span class="text-[#777777]">
                                                {format!("{} tracking tags", info.tracking_param_count)}
                                            </span>
                                        </div>
                                        <div class="flex items-center gap-2 pt-0.5">
                                            {if info.tracking_param_count > 0 {
                                                view! {
                                                    <button
                                                        type="button"
                                                        class="button button-primary !py-1 !px-2.5 !text-xs !min-h-0"
                                                        on:click=move |_| {
                                                            let cleaned = payload::strip_tracking_query(&state.url_input.get());
                                                            state.url_input.set(cleaned.url);
                                                        }
                                                    >
                                                        {format!("Strip {} Tracking Tag(s)", info.tracking_param_count)}
                                                    </button>
                                                }.into_any()
                                            } else {
                                                view! { <span /> }.into_any()
                                            }}
                                            <button
                                                type="button"
                                                class="button button-secondary !py-1 !px-2.5 !text-xs !min-h-0"
                                                on:click=move |_| {
                                                    let cleaned = payload::strip_all_query(&state.url_input.get());
                                                    state.url_input.set(cleaned);
                                                }
                                            >
                                                "Remove All Parameters"
                                            </button>
                                        </div>
                                    </div>
                                }.into_any()
                            } else {
                                view! { <span /> }.into_any()
                            }}
                        </div>
                    }.into_any(),

                    PayloadTab::Text => view! {
                        <div class="space-y-2">
                            <textarea
                                class="w-full text-sm font-mono py-2 px-3 rounded-lg bg-[#050505] border border-[rgba(255,255,255,0.12)] focus:border-[#00f0ff] text-[#ffffff] outline-none"
                                rows="3"
                                prop:value=move || state.text_input.get()
                                on:input=move |event| state.text_input.set(event_target_value(&event))
                            />
                        </div>
                    }.into_any(),

                    PayloadTab::Wifi => view! {
                        <div class="space-y-3">
                            <div>
                                <label class="text-xs text-[#a0a0a0] block mb-1">
                                    "Network Name (SSID)"
                                </label>
                                <input
                                    type="text"
                                    class="w-full text-sm font-mono"
                                    prop:value=move || state.wifi_ssid.get()
                                    on:input=move |event| state.wifi_ssid.set(event_target_value(&event))
                                />
                            </div>

                            <div class="space-y-1">
                                <label class="text-xs text-[#a0a0a0] block">
                                    "Security"
                                </label>
                                <div class="grid grid-cols-3 gap-2">
                                    {[
                                        (WifiAuth::Wpa, "WPA / WPA2 / WPA3"),
                                        (WifiAuth::Wep, "WEP"),
                                        (WifiAuth::Nopass, "Open"),
                                    ].into_iter().map(|(auth, label)| {
                                        let is_selected = move || state.wifi_auth.get() == auth;
                                        view! {
                                            <button
                                                type="button"
                                                on:click=move |_| state.wifi_auth.set(auth)
                                                class=move || {
                                                    if is_selected() {
                                                        "py-1.5 px-2 rounded-lg text-xs font-bold bg-[#00f0ff] text-[#000000] border border-[#00f0ff]"
                                                    } else {
                                                        "py-1.5 px-2 rounded-lg text-xs font-medium text-[#a0a0a0] bg-[#111111] border border-[rgba(255,255,255,0.08)] hover:text-[#ffffff]"
                                                    }
                                                }
                                            >
                                                {label}
                                            </button>
                                        }
                                    }).collect_view()}
                                </div>
                            </div>

                            {move || if state.wifi_auth.get() != WifiAuth::Nopass {
                                view! {
                                    <div>
                                        <div class="flex items-center justify-between mb-1">
                                            <label class="text-xs text-[#a0a0a0]">
                                                "Password"
                                            </label>
                                            <button
                                                type="button"
                                                class="text-[11px] text-[#00f0ff] hover:underline"
                                                on:click=move |_| {
                                                    set_show_wifi_password.update(|visible| *visible = !*visible);
                                                }
                                            >
                                                {move || if show_wifi_password.get() { "Hide" } else { "Show" }}
                                            </button>
                                        </div>
                                        <input
                                            type=move || if show_wifi_password.get() { "text" } else { "password" }
                                            class="w-full text-sm font-mono"
                                            prop:value=move || state.wifi_password.get()
                                            on:input=move |event| {
                                                state.wifi_password.set(event_target_value(&event));
                                            }
                                        />
                                    </div>
                                }.into_any()
                            } else {
                                view! { <span /> }.into_any()
                            }}

                            <div class="flex items-center gap-2 pt-0.5">
                                <input
                                    type="checkbox"
                                    id="wifi_hidden"
                                    class="rounded accent-[#00f0ff]"
                                    prop:checked=move || state.wifi_hidden.get()
                                    on:change=move |event| {
                                        state.wifi_hidden.set(event_target_checked(&event));
                                    }
                                />
                                <label for="wifi_hidden" class="text-xs text-[#a0a0a0] cursor-pointer">
                                    "Hidden network"
                                </label>
                            </div>
                        </div>
                    }.into_any(),

                    PayloadTab::VCard => view! {
                        <div class="space-y-3">
                            <div class="grid grid-cols-1 sm:grid-cols-2 gap-3">
                                <div>
                                    <label class="text-xs text-[#a0a0a0] block mb-1">
                                        "First Name"
                                    </label>
                                    <input
                                        type="text"
                                        class="w-full text-sm font-mono"
                                        prop:value=move || state.vcard_first_name.get()
                                        on:input=move |event| {
                                            state.vcard_first_name.set(event_target_value(&event));
                                        }
                                    />
                                </div>
                                <div>
                                    <label class="text-xs text-[#a0a0a0] block mb-1">
                                        "Last Name"
                                    </label>
                                    <input
                                        type="text"
                                        class="w-full text-sm font-mono"
                                        prop:value=move || state.vcard_last_name.get()
                                        on:input=move |event| {
                                            state.vcard_last_name.set(event_target_value(&event));
                                        }
                                    />
                                </div>
                            </div>

                            <div class="grid grid-cols-1 sm:grid-cols-2 gap-3">
                                <div>
                                    <label class="text-xs text-[#a0a0a0] block mb-1">
                                        "Phone Number"
                                    </label>
                                    <input
                                        type="tel"
                                        class="w-full text-sm font-mono"
                                        prop:value=move || state.vcard_phone.get()
                                        on:input=move |event| state.vcard_phone.set(event_target_value(&event))
                                    />
                                </div>
                                <div>
                                    <label class="text-xs text-[#a0a0a0] block mb-1">
                                        "Email"
                                    </label>
                                    <input
                                        type="email"
                                        class="w-full text-sm font-mono"
                                        prop:value=move || state.vcard_email.get()
                                        on:input=move |event| state.vcard_email.set(event_target_value(&event))
                                    />
                                </div>
                            </div>

                            <div class="grid grid-cols-1 sm:grid-cols-2 gap-3">
                                <div>
                                    <label class="text-xs text-[#a0a0a0] block mb-1">
                                        "Company"
                                    </label>
                                    <input
                                        type="text"
                                        class="w-full text-sm font-mono"
                                        prop:value=move || state.vcard_organization.get()
                                        on:input=move |event| {
                                            state.vcard_organization.set(event_target_value(&event));
                                        }
                                    />
                                </div>
                                <div>
                                    <label class="text-xs text-[#a0a0a0] block mb-1">
                                        "Job Title"
                                    </label>
                                    <input
                                        type="text"
                                        class="w-full text-sm font-mono"
                                        prop:value=move || state.vcard_title.get()
                                        on:input=move |event| state.vcard_title.set(event_target_value(&event))
                                    />
                                </div>
                            </div>

                            <div>
                                <label class="text-xs text-[#a0a0a0] block mb-1">
                                    "Website"
                                </label>
                                <input
                                    type="url"
                                    class="w-full text-sm font-mono"
                                    prop:value=move || state.vcard_url.get()
                                    on:input=move |event| state.vcard_url.set(event_target_value(&event))
                                />
                            </div>

                            <div>
                                <label class="text-xs text-[#a0a0a0] block mb-1">
                                    "Note"
                                </label>
                                <input
                                    type="text"
                                    class="w-full text-sm font-mono"
                                    prop:value=move || state.vcard_note.get()
                                    on:input=move |event| state.vcard_note.set(event_target_value(&event))
                                />
                            </div>
                        </div>
                    }.into_any(),

                    PayloadTab::Email => view! {
                        <div class="space-y-3">
                            <div>
                                <label class="text-xs text-[#a0a0a0] block mb-1">
                                    "Recipient"
                                </label>
                                <input
                                    type="email"
                                    class="w-full text-sm font-mono"
                                    prop:value=move || state.email_to.get()
                                    on:input=move |event| state.email_to.set(event_target_value(&event))
                                />
                            </div>
                            <div>
                                <label class="text-xs text-[#a0a0a0] block mb-1">
                                    "Subject"
                                </label>
                                <input
                                    type="text"
                                    class="w-full text-sm font-mono"
                                    prop:value=move || state.email_subject.get()
                                    on:input=move |event| state.email_subject.set(event_target_value(&event))
                                />
                            </div>
                            <div>
                                <label class="text-xs text-[#a0a0a0] block mb-1">
                                    "Message"
                                </label>
                                <textarea
                                    class="w-full text-sm font-mono"
                                    rows="3"
                                    prop:value=move || state.email_body.get()
                                    on:input=move |event| state.email_body.set(event_target_value(&event))
                                />
                            </div>
                        </div>
                    }.into_any(),

                    PayloadTab::Phone => view! {
                        <div class="space-y-3">
                            <div>
                                <label class="text-xs text-[#a0a0a0] block mb-1">
                                    "Phone Number"
                                </label>
                                <input
                                    type="tel"
                                    class="w-full text-sm font-mono"
                                    prop:value=move || state.phone_number.get()
                                    on:input=move |event| state.phone_number.set(event_target_value(&event))
                                />
                            </div>
                        </div>
                    }.into_any(),

                    PayloadTab::Geo => view! {
                        <div class="space-y-3">
                            <div class="grid grid-cols-2 gap-3">
                                <div>
                                    <label class="text-xs text-[#a0a0a0] block mb-1">
                                        "Latitude"
                                    </label>
                                    <input
                                        type="number"
                                        step="any"
                                        class="w-full text-sm font-mono"
                                        prop:value=move || state.geo_latitude.get()
                                        on:input=move |event| {
                                            state.geo_latitude.set(event_target_value(&event));
                                        }
                                    />
                                </div>
                                <div>
                                    <label class="text-xs text-[#a0a0a0] block mb-1">
                                        "Longitude"
                                    </label>
                                    <input
                                        type="number"
                                        step="any"
                                        class="w-full text-sm font-mono"
                                        prop:value=move || state.geo_longitude.get()
                                        on:input=move |event| {
                                            state.geo_longitude.set(event_target_value(&event));
                                        }
                                    />
                                </div>
                            </div>
                            <div>
                                <label class="text-xs text-[#a0a0a0] block mb-1">
                                    "Label (Optional)"
                                </label>
                                <input
                                    type="text"
                                    class="w-full text-sm font-mono"
                                    prop:value=move || state.geo_query.get()
                                    on:input=move |event| state.geo_query.set(event_target_value(&event))
                                />
                            </div>
                        </div>
                    }.into_any(),

                    PayloadTab::Crypto => view! {
                        <div class="space-y-3">
                            <div class="space-y-1">
                                <label class="text-xs text-[#a0a0a0] block">
                                    "Cryptocurrency"
                                </label>
                                <div class="grid grid-cols-2 sm:grid-cols-5 gap-1.5">
                                    {[
                                        (CryptoCurrency::Bitcoin, "BTC"),
                                        (CryptoCurrency::Ethereum, "ETH"),
                                        (CryptoCurrency::Solana, "SOL"),
                                        (CryptoCurrency::Monero, "XMR"),
                                        (CryptoCurrency::Litecoin, "LTC"),
                                    ].into_iter().map(|(currency, label)| {
                                        let is_selected = move || state.crypto_currency.get() == currency;
                                        view! {
                                            <button
                                                type="button"
                                                on:click=move |_| state.crypto_currency.set(currency)
                                                class=move || {
                                                    if is_selected() {
                                                        "py-1.5 px-2 rounded-lg text-xs font-bold bg-[#00f0ff] text-[#000000] border border-[#00f0ff]"
                                                    } else {
                                                        "py-1.5 px-2 rounded-lg text-xs font-medium text-[#a0a0a0] bg-[#111111] border border-[rgba(255,255,255,0.08)] hover:text-[#ffffff]"
                                                    }
                                                }
                                            >
                                                {label}
                                            </button>
                                        }
                                    }).collect_view()}
                                </div>
                            </div>

                            <div>
                                <label class="text-xs text-[#a0a0a0] block mb-1">
                                    "Wallet Address"
                                </label>
                                <input
                                    type="text"
                                    class="w-full text-sm font-mono"
                                    prop:value=move || state.crypto_address.get()
                                    on:input=move |event| {
                                        state.crypto_address.set(event_target_value(&event));
                                    }
                                />
                            </div>

                            <div class="grid grid-cols-1 sm:grid-cols-2 gap-3">
                                <div>
                                    <label class="text-xs text-[#a0a0a0] block mb-1">
                                        "Amount (Optional)"
                                    </label>
                                    <input
                                        type="text"
                                        class="w-full text-sm font-mono"
                                        prop:value=move || state.crypto_amount.get()
                                        on:input=move |event| {
                                            state.crypto_amount.set(event_target_value(&event));
                                        }
                                    />
                                </div>
                                <div>
                                    <label class="text-xs text-[#a0a0a0] block mb-1">
                                        "Note (Optional)"
                                    </label>
                                    <input
                                        type="text"
                                        class="w-full text-sm font-mono"
                                        prop:value=move || state.crypto_message.get()
                                        on:input=move |event| {
                                            state.crypto_message.set(event_target_value(&event));
                                        }
                                    />
                                </div>
                            </div>
                        </div>
                    }.into_any(),
                }}
            </div>

            <div class="pt-2 border-t border-[rgba(255,255,255,0.06)] flex items-center justify-between text-[11px] text-[#555555]">
                <span class="truncate max-w-[280px] sm:max-w-md font-mono">
                    {move || sanitized.get().content}
                </span>
                <span class="font-mono text-[#a0a0a0] shrink-0">
                    {move || format!("{} bytes", sanitized.get().cleaned_bytes)}
                </span>
            </div>
        </div>
    }
}
