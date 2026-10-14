use leptos::prelude::*;
use qr_core::payload::{
    strip_all_query, strip_tracking_query, CryptoCurrency, Payload, UrlQueryInfo, WifiAuth,
};

pub use crate::components::glide_select::{GlideSelect, PayloadTab};

#[component]
pub fn PayloadEditor(
    active_payload: Memo<Payload>,
    active_tab: RwSignal<PayloadTab>,
    url_input: RwSignal<String>,
    text_input: RwSignal<String>,
    wifi_ssid: RwSignal<String>,
    wifi_password: RwSignal<String>,
    wifi_auth: RwSignal<WifiAuth>,
    wifi_hidden: RwSignal<bool>,
    vcard_first_name: RwSignal<String>,
    vcard_last_name: RwSignal<String>,
    vcard_phone: RwSignal<String>,
    vcard_email: RwSignal<String>,
    vcard_org: RwSignal<String>,
    vcard_title: RwSignal<String>,
    vcard_url: RwSignal<String>,
    vcard_note: RwSignal<String>,
    email_to: RwSignal<String>,
    email_subject: RwSignal<String>,
    email_body: RwSignal<String>,
    phone_number: RwSignal<String>,
    geo_lat: RwSignal<String>,
    geo_lng: RwSignal<String>,
    geo_query: RwSignal<String>,
    crypto_currency: RwSignal<CryptoCurrency>,
    crypto_address: RwSignal<String>,
    crypto_amount: RwSignal<String>,
    crypto_message: RwSignal<String>,
    query_info: Memo<Option<UrlQueryInfo>>,
) -> impl IntoView {
    let (show_wifi_password, set_show_wifi_password) = signal(false);

    view! {
        <div class="glass-panel p-4 sm:p-5 space-y-3">
            // Compact Header: Title + GlideSelect Chip Dropdown
            <div class="flex items-center justify-between">
                <label class="text-xs font-semibold uppercase tracking-wider text-[#a0a0a0]">
                    {move || active_tab.get().header_title()}
                </label>
                <GlideSelect active_tab=active_tab />
            </div>

            // Active Tab Content
            <div>
                {move || match active_tab.get() {
                    PayloadTab::Url => view! {
                        <div class="space-y-3">
                            <div class="relative flex items-center">
                                <input
                                    type="url"
                                    class="w-full text-sm font-mono pr-8 py-2.5 px-3 rounded-lg bg-[#050505] border border-[rgba(255,255,255,0.12)] focus:border-[#00f0ff] text-[#ffffff] placeholder:text-[#555555] outline-none"
                                    placeholder="https://den1zz.dev"
                                    prop:value=move || url_input.get()
                                    on:input=move |ev| url_input.set(event_target_value(&ev))
                                />
                                {move || if !url_input.get().is_empty() {
                                    view! {
                                        <button
                                            type="button"
                                            title="Clear URL"
                                            class="absolute right-2 text-[#555555] hover:text-[#ffffff] text-sm px-1.5 py-0.5 rounded transition-colors"
                                            on:click=move |_| url_input.set(String::new())
                                        >
                                            "✕"
                                        </button>
                                    }.into_any()
                                } else {
                                    view! { <span /> }.into_any()
                                }}
                            </div>

                            // Tracking Parameter Inspector Notification (No emojis, clean AMOLED design)
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
                                                            let (cleaned, _) = strip_tracking_query(&url_input.get());
                                                            url_input.set(cleaned);
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
                                                    let cleaned = strip_all_query(&url_input.get());
                                                    url_input.set(cleaned);
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
                                prop:value=move || text_input.get()
                                on:input=move |ev| text_input.set(event_target_value(&ev))
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
                                    prop:value=move || wifi_ssid.get()
                                    on:input=move |ev| wifi_ssid.set(event_target_value(&ev))
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
                                        let is_sel = move || wifi_auth.get() == auth;
                                        view! {
                                            <button
                                                type="button"
                                                on:click=move |_| wifi_auth.set(auth)
                                                class=move || {
                                                    if is_sel() {
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

                            {move || if wifi_auth.get() != WifiAuth::Nopass {
                                view! {
                                    <div>
                                        <div class="flex items-center justify-between mb-1">
                                            <label class="text-xs text-[#a0a0a0]">
                                                "Password"
                                            </label>
                                            <button
                                                type="button"
                                                class="text-[11px] text-[#00f0ff] hover:underline"
                                                on:click=move |_| set_show_wifi_password.update(|v| *v = !*v)
                                            >
                                                {move || if show_wifi_password.get() { "Hide" } else { "Show" }}
                                            </button>
                                        </div>
                                        <input
                                            type=move || if show_wifi_password.get() { "text" } else { "password" }
                                            class="w-full text-sm font-mono"
                                            prop:value=move || wifi_password.get()
                                            on:input=move |ev| wifi_password.set(event_target_value(&ev))
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
                                    prop:checked=move || wifi_hidden.get()
                                    on:change=move |ev| wifi_hidden.set(event_target_checked(&ev))
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
                                        prop:value=move || vcard_first_name.get()
                                        on:input=move |ev| vcard_first_name.set(event_target_value(&ev))
                                    />
                                </div>
                                <div>
                                    <label class="text-xs text-[#a0a0a0] block mb-1">
                                        "Last Name"
                                    </label>
                                    <input
                                        type="text"
                                        class="w-full text-sm font-mono"
                                        prop:value=move || vcard_last_name.get()
                                        on:input=move |ev| vcard_last_name.set(event_target_value(&ev))
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
                                        prop:value=move || vcard_phone.get()
                                        on:input=move |ev| vcard_phone.set(event_target_value(&ev))
                                    />
                                </div>
                                <div>
                                    <label class="text-xs text-[#a0a0a0] block mb-1">
                                        "Email"
                                    </label>
                                    <input
                                        type="email"
                                        class="w-full text-sm font-mono"
                                        prop:value=move || vcard_email.get()
                                        on:input=move |ev| vcard_email.set(event_target_value(&ev))
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
                                        prop:value=move || vcard_org.get()
                                        on:input=move |ev| vcard_org.set(event_target_value(&ev))
                                    />
                                </div>
                                <div>
                                    <label class="text-xs text-[#a0a0a0] block mb-1">
                                        "Job Title"
                                    </label>
                                    <input
                                        type="text"
                                        class="w-full text-sm font-mono"
                                        prop:value=move || vcard_title.get()
                                        on:input=move |ev| vcard_title.set(event_target_value(&ev))
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
                                    prop:value=move || vcard_url.get()
                                    on:input=move |ev| vcard_url.set(event_target_value(&ev))
                                />
                            </div>

                            <div>
                                <label class="text-xs text-[#a0a0a0] block mb-1">
                                    "Note"
                                </label>
                                <input
                                    type="text"
                                    class="w-full text-sm font-mono"
                                    prop:value=move || vcard_note.get()
                                    on:input=move |ev| vcard_note.set(event_target_value(&ev))
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
                                    prop:value=move || email_to.get()
                                    on:input=move |ev| email_to.set(event_target_value(&ev))
                                />
                            </div>
                            <div>
                                <label class="text-xs text-[#a0a0a0] block mb-1">
                                    "Subject"
                                </label>
                                <input
                                    type="text"
                                    class="w-full text-sm font-mono"
                                    prop:value=move || email_subject.get()
                                    on:input=move |ev| email_subject.set(event_target_value(&ev))
                                />
                            </div>
                            <div>
                                <label class="text-xs text-[#a0a0a0] block mb-1">
                                    "Message"
                                </label>
                                <textarea
                                    class="w-full text-sm font-mono"
                                    rows="3"
                                    prop:value=move || email_body.get()
                                    on:input=move |ev| email_body.set(event_target_value(&ev))
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
                                    prop:value=move || phone_number.get()
                                    on:input=move |ev| phone_number.set(event_target_value(&ev))
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
                                        prop:value=move || geo_lat.get()
                                        on:input=move |ev| geo_lat.set(event_target_value(&ev))
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
                                        prop:value=move || geo_lng.get()
                                        on:input=move |ev| geo_lng.set(event_target_value(&ev))
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
                                    prop:value=move || geo_query.get()
                                    on:input=move |ev| geo_query.set(event_target_value(&ev))
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
                                    ].into_iter().map(|(curr, label)| {
                                        let is_sel = move || crypto_currency.get() == curr;
                                        view! {
                                            <button
                                                type="button"
                                                on:click=move |_| crypto_currency.set(curr)
                                                class=move || {
                                                    if is_sel() {
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
                                    prop:value=move || crypto_address.get()
                                    on:input=move |ev| crypto_address.set(event_target_value(&ev))
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
                                        prop:value=move || crypto_amount.get()
                                        on:input=move |ev| crypto_amount.set(event_target_value(&ev))
                                    />
                                </div>
                                <div>
                                    <label class="text-xs text-[#a0a0a0] block mb-1">
                                        "Note (Optional)"
                                    </label>
                                    <input
                                        type="text"
                                        class="w-full text-sm font-mono"
                                        prop:value=move || crypto_message.get()
                                        on:input=move |ev| crypto_message.set(event_target_value(&ev))
                                    />
                                </div>
                            </div>
                        </div>
                    }.into_any(),
                }}
            </div>

            // Payload Encoded String & Byte Count Footer
            <div class="pt-2 border-t border-[rgba(255,255,255,0.06)] flex items-center justify-between text-[11px] text-[#555555]">
                <span class="truncate max-w-[280px] sm:max-w-md font-mono">
                    {move || {
                        let sanitized = active_payload.get().sanitize();
                        sanitized.content
                    }}
                </span>
                <span class="font-mono text-[#a0a0a0] shrink-0">
                    {move || {
                        let sanitized = active_payload.get().sanitize();
                        format!("{} bytes", sanitized.cleaned_bytes)
                    }}
                </span>
            </div>
        </div>
    }
}
