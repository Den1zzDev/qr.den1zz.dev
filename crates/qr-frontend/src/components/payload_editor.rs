//! Payload editor for every supported payload kind.

use leptos::html;
use leptos::prelude::*;
use qr_core::payload;
use qr_core::payload::CryptoCurrency;
use qr_core::payload::SanitizedPayload;
use qr_core::payload::UrlQueryInfo;
use qr_core::payload::WifiAuth;

use crate::components::glide_select::GlideSelect;
use crate::components::glide_select::PayloadTab;
use crate::components::icons::Close;
use crate::studio_state::parse_geo_pair;
use crate::studio_state::StudioState;
use crate::studio_state::DEFAULT_URL;

/// Editor for the payload that will be encoded.
#[component]
pub(crate) fn PayloadEditor(
    state: StudioState,
    query_info: Memo<Option<UrlQueryInfo>>,
    sanitized: Memo<SanitizedPayload>,
) -> impl IntoView {
    let (show_wifi_password, set_show_wifi_password) = signal(false);
    let url_ref = NodeRef::<html::Input>::new();
    let url_initialised = RwSignal::new(false);
    let geo_invalid = Signal::derive(move || {
        let latitude = state.geo_latitude.get();
        let longitude = state.geo_longitude.get();
        let empty = latitude.trim().is_empty() || longitude.trim().is_empty();
        let started = !latitude.trim().is_empty() || !longitude.trim().is_empty();
        started && parse_geo_pair(&latitude, &longitude).is_none() && !empty
    });
    let geo_incomplete = Signal::derive(move || {
        let latitude = state.geo_latitude.get();
        let longitude = state.geo_longitude.get();
        let empty = latitude.trim().is_empty() || longitude.trim().is_empty();
        let started = !latitude.trim().is_empty() || !longitude.trim().is_empty();
        started && empty
    });
    let crypto_amount_invalid = Signal::derive(move || {
        let amount = state.crypto_amount.get();
        !amount.trim().is_empty() && payload::normalize_crypto_amount(&amount).is_none()
    });

    // The URL field is uncontrolled so text typed before hydration is kept.
    // Programmatic changes (clear, tracking cleanup, sample link) sync through
    // the signal and are written back to the element here.
    Effect::new(move |_| {
        let value = state.url_input.get();
        let Some(input) = url_ref.get() else {
            return;
        };

        if !url_initialised.get_untracked() {
            let typed = input.value();
            if !typed.is_empty() && typed != value {
                state.url_input.set(typed);
            }
            url_initialised.set(true);
            return;
        }

        if input.value() != value {
            input.set_value(&value);
        }
    });

    view! {
        <div class="panel">
            <div class="panel__head">
                <div>
                    <h2 class="panel__title">{move || state.active_tab.get().header_title()}</h2>
                </div>
                <GlideSelect active_tab=state.active_tab />
            </div>

            <div class="panel__body space-y-3">
                {move || match state.active_tab.get() {
                    PayloadTab::Url => view! {
                        <div class="space-y-3">
                            <div>
                                <label class="field-label" for="target-url">"Destination"</label>
                                <div class="field-wrap">
                                    <input
                                        id="target-url"
                                        type="url"
                                        class="field"
                                        node_ref=url_ref
                                        value=DEFAULT_URL
                                        placeholder="https://example.com"
                                        inputmode="url"
                                        autocomplete="url"
                                        autocapitalize="off"
                                        spellcheck="false"
                                        aria-describedby="target-url-hint"
                                        on:input=move |event| {
                                            state.url_input.set(event_target_value(&event));
                                        }
                                    />
                                    {move || if !state.url_input.get().is_empty() {
                                        view! {
                                            <button
                                                type="button"
                                                class="field-clear"
                                                aria-label="Clear the destination"
                                                title="Clear"
                                                on:click=move |_| state.url_input.set(String::new())
                                            >
                                                <Close size=14 />
                                            </button>
                                        }.into_any()
                                    } else {
                                        view! { <span /> }.into_any()
                                    }}
                                </div>
                                <p id="target-url-hint" class="field-hint">
                                    "HTTPS is added when the scheme is missing."
                                </p>
                            </div>

                            {move || if let Some(info) = query_info.get() {
                                let total = info.total_param_count;
                                let tracking = info.tracking_param_count;
                                view! {
                                    <div class="url-note">
                                        <div class="url-note__head">
                                            <span>
                                                <span class="url-note__count">{format!("{total}")}</span>
                                                {if total == 1 { " query parameter" } else { " query parameters" }}
                                                " in this link"
                                            </span>
                                            <span>
                                                {if tracking == 1 {
                                                    "1 tracking parameter".to_string()
                                                } else {
                                                    format!("{tracking} tracking parameters")
                                                }}
                                            </span>
                                        </div>
                                        <div class="url-note__actions">
                                            {if tracking > 0 {
                                                view! {
                                                    <button
                                                        type="button"
                                                        class="btn btn--sm btn--secondary"
                                                        on:click=move |_| {
                                                            let cleaned = payload::strip_tracking_query(&state.url_input.get());
                                                            state.url_input.set(cleaned.url);
                                                        }
                                                    >
                                                        {if tracking == 1 {
                                                            "Remove the tracking parameter".to_string()
                                                        } else {
                                                            format!("Remove {tracking} tracking parameters")
                                                        }}
                                                    </button>
                                                }.into_any()
                                            } else {
                                                view! { <span /> }.into_any()
                                            }}
                                            <button
                                                type="button"
                                                class="btn btn--sm btn--ghost"
                                                on:click=move |_| {
                                                    let cleaned = payload::strip_all_query(&state.url_input.get());
                                                    state.url_input.set(cleaned);
                                                }
                                            >
                                                "Remove all parameters"
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
                        <div>
                            <label class="field-label" for="payload-text">"Content"</label>
                            <textarea
                                id="payload-text"
                                class="field"
                                rows="4"
                                placeholder="Any text, emoji, or structured data"
                                prop:value=move || state.text_input.get()
                                on:input=move |event| state.text_input.set(event_target_value(&event))
                            />
                            <p class="field-hint">"Line breaks and Unicode are preserved."</p>
                        </div>
                    }.into_any(),

                    PayloadTab::Wifi => view! {
                        <div class="space-y-3">
                            <div>
                                <label class="field-label" for="wifi-ssid">"Network name (SSID)"</label>
                                <input
                                    id="wifi-ssid"
                                    type="text"
                                    class="field"
                                    autocomplete="off"
                                    autocapitalize="none"
                                    spellcheck="false"
                                    prop:value=move || state.wifi_ssid.get()
                                    on:input=move |event| state.wifi_ssid.set(event_target_value(&event))
                                />
                            </div>

                            <fieldset class="option-group">
                                <legend class="field-label">"Security"</legend>
                                <div class="segmented flex-wrap">
                                    {[
                                        (WifiAuth::Wpa, "WPA"),
                                        (WifiAuth::Wep, "WEP"),
                                        (WifiAuth::Nopass, "Open"),
                                    ].into_iter().map(|(auth, label)| {
                                        view! {
                                            <label class="seg">
                                                <input
                                                    type="radio"
                                                    name="wifi-auth"
                                                    checked=move || state.wifi_auth.get() == auth
                                                    on:change=move |_| state.wifi_auth.set(auth)
                                                />
                                                <span>{label}</span>
                                            </label>
                                        }
                                    }).collect_view()}
                                </div>
                                <p class="field-hint">"WPA covers WPA, WPA2, and WPA3 networks."</p>
                            </fieldset>

                            {move || if state.wifi_auth.get() != WifiAuth::Nopass {
                                view! {
                                    <div>
                                        <div class="flex items-center justify-between">
                                            <label class="field-label" for="wifi-password">"Password"</label>
                                            <button
                                                type="button"
                                                class="link-button"
                                                aria-pressed=move || show_wifi_password.get().to_string()
                                                on:click=move |_| {
                                                    set_show_wifi_password.update(|visible| *visible = !*visible);
                                                }
                                            >
                                                {move || if show_wifi_password.get() { "Hide" } else { "Show" }}
                                            </button>
                                        </div>
                                        <input
                                            id="wifi-password"
                                            type=move || if show_wifi_password.get() { "text" } else { "password" }
                                            class="field"
                                            autocomplete="off"
                                            spellcheck="false"
                                            prop:value=move || state.wifi_password.get()
                                            on:input=move |event| state.wifi_password.set(event_target_value(&event))
                                        />
                                    </div>
                                }.into_any()
                            } else {
                                view! { <span /> }.into_any()
                            }}

                            <label class="check">
                                <input
                                    type="checkbox"
                                    prop:checked=move || state.wifi_hidden.get()
                                    on:change=move |event| {
                                        state.wifi_hidden.set(event_target_checked(&event));
                                    }
                                />
                                <span>"Hidden network"</span>
                            </label>
                        </div>
                    }.into_any(),

                    PayloadTab::VCard => view! {
                        <div class="space-y-3">
                            <div class="grid grid-cols-1 gap-3 sm:grid-cols-2">
                                <div>
                                    <label class="field-label" for="vcard-first">"First name"</label>
                                    <input
                                        id="vcard-first"
                                        type="text"
                                        class="field"
                                        autocomplete="given-name"
                                        prop:value=move || state.vcard_first_name.get()
                                        on:input=move |event| state.vcard_first_name.set(event_target_value(&event))
                                    />
                                </div>
                                <div>
                                    <label class="field-label" for="vcard-last">"Last name"</label>
                                    <input
                                        id="vcard-last"
                                        type="text"
                                        class="field"
                                        autocomplete="family-name"
                                        prop:value=move || state.vcard_last_name.get()
                                        on:input=move |event| state.vcard_last_name.set(event_target_value(&event))
                                    />
                                </div>
                                <div>
                                    <label class="field-label" for="vcard-phone">"Phone"</label>
                                    <input
                                        id="vcard-phone"
                                        type="tel"
                                        class="field"
                                        inputmode="tel"
                                        autocomplete="tel"
                                        prop:value=move || state.vcard_phone.get()
                                        on:input=move |event| state.vcard_phone.set(event_target_value(&event))
                                    />
                                </div>
                                <div>
                                    <label class="field-label" for="vcard-email">"Email"</label>
                                    <input
                                        id="vcard-email"
                                        type="email"
                                        class="field"
                                        inputmode="email"
                                        autocomplete="email"
                                        autocapitalize="none"
                                        spellcheck="false"
                                        prop:value=move || state.vcard_email.get()
                                        on:input=move |event| state.vcard_email.set(event_target_value(&event))
                                    />
                                </div>
                                <div>
                                    <label class="field-label" for="vcard-org">"Company"</label>
                                    <input
                                        id="vcard-org"
                                        type="text"
                                        class="field"
                                        autocomplete="organization"
                                        prop:value=move || state.vcard_organization.get()
                                        on:input=move |event| state.vcard_organization.set(event_target_value(&event))
                                    />
                                </div>
                                <div>
                                    <label class="field-label" for="vcard-title">"Job title"</label>
                                    <input
                                        id="vcard-title"
                                        type="text"
                                        class="field"
                                        autocomplete="organization-title"
                                        prop:value=move || state.vcard_title.get()
                                        on:input=move |event| state.vcard_title.set(event_target_value(&event))
                                    />
                                </div>
                            </div>

                            <div>
                                <label class="field-label" for="vcard-url">"Website"</label>
                                <input
                                    id="vcard-url"
                                    type="url"
                                    class="field"
                                    inputmode="url"
                                    autocomplete="url"
                                    autocapitalize="off"
                                    spellcheck="false"
                                    prop:value=move || state.vcard_url.get()
                                    on:input=move |event| state.vcard_url.set(event_target_value(&event))
                                />
                            </div>

                            <div>
                                <label class="field-label" for="vcard-note">"Note"</label>
                                <input
                                    id="vcard-note"
                                    type="text"
                                    class="field"
                                    prop:value=move || state.vcard_note.get()
                                    on:input=move |event| state.vcard_note.set(event_target_value(&event))
                                />
                            </div>
                            <p class="field-hint">"Only the fields you fill are encoded."</p>
                        </div>
                    }.into_any(),

                    PayloadTab::Email => view! {
                        <div class="space-y-3">
                            <div>
                                <label class="field-label" for="email-to">"Recipient"</label>
                                <input
                                    id="email-to"
                                    type="email"
                                    class="field"
                                    inputmode="email"
                                    autocomplete="email"
                                    autocapitalize="none"
                                    spellcheck="false"
                                    prop:value=move || state.email_to.get()
                                    on:input=move |event| state.email_to.set(event_target_value(&event))
                                />
                            </div>
                            <div>
                                <label class="field-label" for="email-subject">"Subject"</label>
                                <input
                                    id="email-subject"
                                    type="text"
                                    class="field"
                                    prop:value=move || state.email_subject.get()
                                    on:input=move |event| state.email_subject.set(event_target_value(&event))
                                />
                            </div>
                            <div>
                                <label class="field-label" for="email-body">"Message"</label>
                                <textarea
                                    id="email-body"
                                    class="field"
                                    rows="4"
                                    prop:value=move || state.email_body.get()
                                    on:input=move |event| state.email_body.set(event_target_value(&event))
                                />
                            </div>
                        </div>
                    }.into_any(),

                    PayloadTab::Phone => view! {
                        <div>
                            <label class="field-label" for="phone-number">"Number"</label>
                            <input
                                id="phone-number"
                                type="tel"
                                class="field"
                                inputmode="tel"
                                autocomplete="tel"
                                placeholder="+1 555 0100"
                                prop:value=move || state.phone_number.get()
                                on:input=move |event| state.phone_number.set(event_target_value(&event))
                            />
                            <p class="field-hint">"Extensions can be appended as ;ext=123."</p>
                        </div>
                    }.into_any(),

                    PayloadTab::Geo => view! {
                        <div class="space-y-3">
                            <div class="grid grid-cols-2 gap-3">
                                <div>
                                    <label class="field-label" for="geo-latitude">"Latitude"</label>
                                    <input
                                        id="geo-latitude"
                                        type="number"
                                        step="any"
                                        min="-90"
                                        max="90"
                                        class="field"
                                        inputmode="decimal"
                                        aria-describedby="geo-error"
                                        aria-invalid=move || {
                                            (geo_invalid.get() || geo_incomplete.get()).to_string()
                                        }
                                        placeholder="41.0082"
                                        prop:value=move || state.geo_latitude.get()
                                        on:input=move |event| state.geo_latitude.set(event_target_value(&event))
                                    />
                                </div>
                                <div>
                                    <label class="field-label" for="geo-longitude">"Longitude"</label>
                                    <input
                                        id="geo-longitude"
                                        type="number"
                                        step="any"
                                        min="-180"
                                        max="180"
                                        class="field"
                                        inputmode="decimal"
                                        aria-describedby="geo-error"
                                        aria-invalid=move || {
                                            (geo_invalid.get() || geo_incomplete.get()).to_string()
                                        }
                                        placeholder="28.9784"
                                        prop:value=move || state.geo_longitude.get()
                                        on:input=move |event| state.geo_longitude.set(event_target_value(&event))
                                    />
                                </div>
                            </div>
                            {move || {
                                if geo_incomplete.get() {
                                    view! {
                                        <p id="geo-error" class="field-hint field-hint--error" role="alert">
                                            "Enter both coordinates to encode a location."
                                        </p>
                                    }.into_any()
                                } else if geo_invalid.get() {
                                    view! {
                                        <p id="geo-error" class="field-hint field-hint--error" role="alert">
                                            "Latitude must be between -90 and 90, and longitude between -180 and 180."
                                        </p>
                                    }.into_any()
                                } else {
                                    view! { <span /> }.into_any()
                                }
                            }}
                            <div>
                                <label class="field-label" for="geo-label">"Place label (optional)"</label>
                                <input
                                    id="geo-label"
                                    type="text"
                                    class="field"
                                    placeholder="Istanbul"
                                    prop:value=move || state.geo_query.get()
                                    on:input=move |event| state.geo_query.set(event_target_value(&event))
                                />
                            </div>
                        </div>
                    }.into_any(),

                    PayloadTab::Crypto => view! {
                        <div class="space-y-3">
                            <fieldset class="option-group">
                                <legend class="field-label">"Currency"</legend>
                                <div class="segmented flex-wrap">
                                    {[
                                        (CryptoCurrency::Bitcoin, "BTC"),
                                        (CryptoCurrency::Ethereum, "ETH"),
                                        (CryptoCurrency::Solana, "SOL"),
                                        (CryptoCurrency::Monero, "XMR"),
                                        (CryptoCurrency::Litecoin, "LTC"),
                                    ].into_iter().map(|(currency, label)| {
                                        view! {
                                            <label class="seg">
                                                <input
                                                    type="radio"
                                                    name="crypto-currency"
                                                    checked=move || state.crypto_currency.get() == currency
                                                    on:change=move |_| state.crypto_currency.set(currency)
                                                />
                                                <span>{label}</span>
                                            </label>
                                        }
                                    }).collect_view()}
                                </div>
                            </fieldset>

                            <div>
                                <label class="field-label" for="crypto-address">"Wallet address"</label>
                                <input
                                    id="crypto-address"
                                    type="text"
                                    class="field"
                                    autocomplete="off"
                                    autocapitalize="none"
                                    spellcheck="false"
                                    prop:value=move || state.crypto_address.get()
                                    on:input=move |event| state.crypto_address.set(event_target_value(&event))
                                />
                            </div>

                            <div class="grid grid-cols-1 gap-3 sm:grid-cols-2">
                                <div>
                                    <label class="field-label" for="crypto-amount">"Amount (optional)"</label>
                                    <input
                                        id="crypto-amount"
                                        type="text"
                                        class="field"
                                        inputmode="decimal"
                                        aria-describedby="crypto-amount-error"
                                        aria-invalid=move || crypto_amount_invalid.get().to_string()
                                        placeholder="0.015"
                                        prop:value=move || state.crypto_amount.get()
                                        on:input=move |event| state.crypto_amount.set(event_target_value(&event))
                                    />
                                    {move || if crypto_amount_invalid.get() {
                                        view! {
                                            <p id="crypto-amount-error" class="field-hint field-hint--error" role="alert">
                                                "Use a positive decimal such as 0.015."
                                            </p>
                                        }.into_any()
                                    } else {
                                        view! { <span /> }.into_any()
                                    }}
                                </div>
                                <div>
                                    <label class="field-label" for="crypto-note">"Note (optional)"</label>
                                    <input
                                        id="crypto-note"
                                        type="text"
                                        class="field"
                                        prop:value=move || state.crypto_message.get()
                                        on:input=move |event| state.crypto_message.set(event_target_value(&event))
                                    />
                                </div>
                            </div>
                        </div>
                    }.into_any(),
                }}
            </div>

            <div class="px-4 pb-4 sm:px-5 sm:pb-5">
                {move || {
                    let data = sanitized.get();
                    let has_content = !data.content.is_empty();

                    // Sensitive payloads are summarised instead of echoed, so a
                    // Wi-Fi password or an email body never appears on screen.
                    let preview = if !has_content {
                        "Nothing to encode yet".to_string()
                    } else {
                        match state.active_tab.get() {
                            PayloadTab::Wifi => {
                                let ssid = state.wifi_ssid.get();
                                if ssid.is_empty() {
                                    "Wi-Fi network".to_string()
                                } else {
                                    format!("Wi-Fi network · {ssid}")
                                }
                            }
                            PayloadTab::VCard => {
                                let first = state.vcard_first_name.get();
                                let last = state.vcard_last_name.get();
                                let name = format!("{first} {last}").trim().to_string();
                                if name.is_empty() {
                                    "Contact card".to_string()
                                } else {
                                    format!("Contact card · {name}")
                                }
                            }
                            PayloadTab::Email => {
                                let recipient = state.email_to.get();
                                if recipient.is_empty() {
                                    "Email draft".to_string()
                                } else {
                                    format!("Email draft · {recipient}")
                                }
                            }
                            PayloadTab::Crypto => {
                                let currency = state.crypto_currency.get().label();
                                let address = state.crypto_address.get();
                                if address.is_empty() {
                                    currency.to_string()
                                } else {
                                    format!("{currency} · {address}")
                                }
                            }
                            _ => data.content.clone(),
                        }
                    };

                    view! {
                        <div class="payload-meta">
                            <span class="payload-meta__content">{preview}</span>
                            <span class="payload-meta__bytes">
                                {format!("{} B", data.cleaned_bytes)}
                            </span>
                        </div>
                    }
                }}
            </div>
        </div>
    }
}
