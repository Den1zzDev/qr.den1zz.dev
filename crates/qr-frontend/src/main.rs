use base64::Engine;
use image::DynamicImage;
use leptos::prelude::*;
use qr_core::{
    halftone::{DitherAlgorithm, HalftoneColorMode, HalftoneConfig, HalftoneEngine},
    matrix::{EccLevel, QrMatrix},
    payload::{Payload, WifiAuth},
    vector::{
        CtaBannerConfig, EyeDotShape, EyeFrameShape, GradientConfig, LogoConfig, LogoMaskShape,
        ModuleShape, VectorRenderConfig, VectorRenderer,
    },
    verifier::{calculate_contrast_ratio, ScanStatus, ScanVerifier},
};
use wasm_bindgen::JsCast;
use web_sys::{Blob, BlobPropertyBag, HtmlAnchorElement, HtmlCanvasElement, HtmlImageElement, Url};

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum AppMode {
    Vector,
    Halftone,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum PayloadType {
    Url,
    Text,
    Wifi,
    VCard,
    Email,
}

#[component]
pub fn App() -> impl IntoView {
    // Mode Switcher
    let (app_mode, set_app_mode) = signal(AppMode::Vector);

    // Payload State
    let (payload_type, set_payload_type) = signal(PayloadType::Url);
    let (url_input, set_url_input) = signal("https://qr.den1zz.dev".to_string());
    let (strip_tracking, set_strip_tracking) = signal(true);

    let (text_input, set_text_input) = signal("Precision QR Studio".to_string());

    let (wifi_ssid, set_wifi_ssid) = signal("Studio-Network".to_string());
    let (wifi_pass, set_wifi_pass) = signal("cyber-pass".to_string());
    let (wifi_auth, set_wifi_auth) = signal(WifiAuth::Wpa);
    let (wifi_hidden, set_wifi_hidden) = signal(false);

    let (vcard_name, set_vcard_name) = signal("Den1zz".to_string());
    let (vcard_phone, set_vcard_phone) = signal("+123456789".to_string());
    let (vcard_email, set_vcard_email) = signal("hello@den1zz.dev".to_string());
    let (vcard_org, set_vcard_org) = signal("Studio".to_string());

    let (email_to, set_email_to) = signal("hello@den1zz.dev".to_string());
    let (email_subject, set_email_subject) = signal("QR Studio Inquiry".to_string());

    // Vector Mode Styling State
    // Defaulting to Classy module shape (matching ente-toys/qr) and den1zz cyan palette!
    let (module_shape, set_module_shape) = signal(ModuleShape::Classy);
    let (eye_frame_shape, set_eye_frame_shape) = signal(EyeFrameShape::Rounded);
    let (eye_dot_shape, set_eye_dot_shape) = signal(EyeDotShape::Circle);

    let (code_color, set_code_color) = signal("#3fd9ff".to_string());
    let (is_gradient, set_is_gradient) = signal(false);
    let (gradient_type_radial, set_gradient_type_radial) = signal(false);
    let (gradient_angle, set_gradient_angle) = signal(135.0f32);
    let (gradient_end_color, set_gradient_end_color) = signal("#0077ff".to_string());

    let (use_custom_eyes, set_use_custom_eyes) = signal(false);
    let (eye_color, set_eye_color) = signal("#3fd9ff".to_string());

    let (is_transparent_bg, set_is_transparent_bg) = signal(false);
    let (bg_color, set_bg_color) = signal("#04070d".to_string());

    let (margin_modules, set_margin_modules) = signal(2usize);
    let (ecc_level, set_ecc_level) = signal(EccLevel::Auto);

    // Center Logo State
    let (logo_data_url, set_logo_data_url) = signal(Option::<String>::None);
    let (_logo_filename, set_logo_filename) = signal(Option::<String>::None);
    let (logo_shape, set_logo_shape) = signal(LogoMaskShape::Rounded);
    let (logo_scale, set_logo_scale) = signal(22.0f32);

    // CTA Banner State
    let (cta_preset, set_cta_preset) = signal("none".to_string());
    let (cta_custom_text, set_cta_custom_text) = signal(String::new());
    let (cta_font_size, set_cta_font_size) = signal(16.0f32);
    let (cta_color, set_cta_color) = signal("#3fd9ff".to_string());

    // Halftone Mode State
    let (halftone_img_data_url, set_halftone_img_data_url) = signal(Option::<String>::None);
    let (_halftone_filename, set_halftone_filename) = signal(Option::<String>::None);
    let (halftone_mode, set_halftone_mode) = signal(HalftoneColorMode::Color);
    let (dither_algo, set_dither_algo) = signal(DitherAlgorithm::Clustered);
    let (strength_bias, set_strength_bias) = signal(50i32);
    let (cell_core_size, set_cell_core_size) = signal(3u32);
    let (halftone_dark_color, set_halftone_dark_color) = signal("#080d16".to_string());
    let (clahe_enabled, set_clahe_enabled) = signal(true);
    let (preserve_structure, set_preserve_structure) = signal(true);

    // Export Options
    let (target_res, set_target_res) = signal(1024u32);
    let (copied_msg, set_copied_msg) = signal(Option::<String>::None);

    // Dynamic Payload Memo
    let active_payload = Memo::new(move |_| match payload_type.get() {
        PayloadType::Url => Payload::Url {
            url: url_input.get(),
            strip_tracking: strip_tracking.get(),
        },
        PayloadType::Text => Payload::Text {
            text: text_input.get(),
        },
        PayloadType::Wifi => Payload::Wifi {
            ssid: wifi_ssid.get(),
            password: wifi_pass.get(),
            auth_type: wifi_auth.get(),
            hidden: wifi_hidden.get(),
        },
        PayloadType::VCard => Payload::VCard {
            name: vcard_name.get(),
            phone: vcard_phone.get(),
            email: vcard_email.get(),
            organization: vcard_org.get(),
        },
        PayloadType::Email => Payload::Email {
            to: email_to.get(),
            subject: email_subject.get(),
        },
    });

    let sanitized_memo = Memo::new(move |_| active_payload.get().sanitize());

    // Generated Vector SVG Memo
    let vector_result_memo = Memo::new(move |_| {
        let sanitized = sanitized_memo.get();
        let has_logo = logo_data_url.get().is_some();
        let ecc = ecc_level.get();

        let matrix = match QrMatrix::new(&sanitized.content, ecc, has_logo) {
            Ok(m) => m,
            Err(e) => return Err(e),
        };

        let grad = if is_gradient.get() {
            if gradient_type_radial.get() {
                Some(GradientConfig::Radial {
                    color_start: code_color.get(),
                    color_end: gradient_end_color.get(),
                })
            } else {
                Some(GradientConfig::Linear {
                    angle: gradient_angle.get(),
                    color_start: code_color.get(),
                    color_end: gradient_end_color.get(),
                })
            }
        } else {
            None
        };

        let logo = logo_data_url.get().map(|data_url| LogoConfig {
            data_url,
            shape: logo_shape.get(),
            scale_percent: logo_scale.get(),
        });

        let cta_text_resolved = match cta_preset.get().as_str() {
            "none" => None,
            "custom" => {
                let t = cta_custom_text.get().trim().to_string();
                if t.is_empty() {
                    None
                } else {
                    Some(t)
                }
            }
            preset => Some(preset.to_uppercase()),
        };

        let cta = cta_text_resolved.map(|text| CtaBannerConfig {
            text,
            font_size: cta_font_size.get(),
            color: cta_color.get(),
        });

        let bg = if is_transparent_bg.get() {
            None
        } else {
            Some(bg_color.get())
        };

        let config = VectorRenderConfig {
            scale: 20.0,
            margin_modules: margin_modules.get(),
            module_shape: module_shape.get(),
            eye_frame_shape: eye_frame_shape.get(),
            eye_dot_shape: eye_dot_shape.get(),
            code_color: code_color.get(),
            gradient: grad,
            eye_color: if use_custom_eyes.get() {
                Some(eye_color.get())
            } else {
                None
            },
            bg_color: bg,
            logo,
            cta,
            alt_text: Some("Precision QR Code".to_string()),
        };

        let svg = VectorRenderer::render_svg(&matrix, &config);
        Ok((matrix, svg))
    });

    // Halftone Result Memo
    let halftone_result_memo = Memo::new(move |_| {
        let img_url = match halftone_img_data_url.get() {
            Some(u) => u,
            None => return None,
        };

        let raw_b64 = img_url
            .trim_start_matches("data:image/png;base64,")
            .trim_start_matches("data:image/jpeg;base64,")
            .trim_start_matches("data:image/webp;base64,");

        let img_bytes = base64::engine::general_purpose::STANDARD
            .decode(raw_b64)
            .ok()?;

        let dyn_img = image::load_from_memory(&img_bytes).ok()?;

        let sanitized = sanitized_memo.get();
        let matrix = QrMatrix::new(&sanitized.content, EccLevel::H, true).ok()?;

        let config = HalftoneConfig {
            mode: halftone_mode.get(),
            dither: dither_algo.get(),
            strength_bias: strength_bias.get(),
            cell_core_size: cell_core_size.get(),
            dark_color: halftone_dark_color.get(),
            clahe: clahe_enabled.get(),
            preserve_structure: preserve_structure.get(),
            crisp_cores: false,
        };

        let (rendered_img, svg) = HalftoneEngine::render(&matrix, &dyn_img, &config);
        let dyn_rendered = DynamicImage::ImageRgba8(rendered_img);

        let verification = ScanVerifier::verify(
            &dyn_rendered,
            &sanitized.content,
            &config.dark_color,
            "#FFFFFF",
        );

        Some((matrix, svg, verification))
    });

    // Contrast calculation for Vector Mode
    let vector_contrast_ratio = Memo::new(move |_| {
        let code = code_color.get();
        let bg = if is_transparent_bg.get() {
            "#FFFFFF"
        } else {
            &bg_color.get()
        };
        calculate_contrast_ratio(&code, bg)
    });

    // Clipboard copy helper
    let copy_text_to_clipboard = move |text: String, label: &'static str| {
        if let Some(window) = web_sys::window() {
            let nav = window.navigator();
            let cb = nav.clipboard();
            let _ = cb.write_text(&text);
            set_copied_msg.set(Some(format!("COPIED {label}")));
            wasm_bindgen_futures::spawn_local(async move {
                gloo_timers::future::TimeoutFuture::new(1800).await;
                set_copied_msg.set(None);
            });
        }
    };

    // Download SVG
    let download_svg = move |svg_content: String| {
        if let Some(window) = web_sys::window() {
            if let Some(doc) = window.document() {
                let blob_parts = js_sys::Array::new();
                blob_parts.push(&wasm_bindgen::JsValue::from_str(&svg_content));
                let bag = BlobPropertyBag::new();
                bag.set_type("image/svg+xml");
                if let Ok(blob) = Blob::new_with_str_sequence_and_options(&blob_parts, &bag) {
                    if let Ok(url) = Url::create_object_url_with_blob(&blob) {
                        if let Ok(a) = doc.create_element("a") {
                            let link = a.unchecked_into::<HtmlAnchorElement>();
                            link.set_href(&url);
                            link.set_download("qr-code.svg");
                            link.click();
                            let _ = Url::revoke_object_url(&url);
                        }
                    }
                }
            }
        }
    };

    // Download PNG
    let download_png = move |svg_content: String, res: u32| {
        if let Some(window) = web_sys::window() {
            if let Some(doc) = window.document() {
                if let Ok(canvas_el) = doc.create_element("canvas") {
                    let canvas = canvas_el.unchecked_into::<HtmlCanvasElement>();
                    canvas.set_width(res);
                    canvas.set_height(res);

                    let ctx = canvas
                        .get_context("2d")
                        .ok()
                        .flatten()
                        .and_then(|c| c.dyn_into::<web_sys::CanvasRenderingContext2d>().ok());

                    if let Some(ctx) = ctx {
                        let img = HtmlImageElement::new().unwrap();
                        let img_clone = img.clone();
                        let encoded = js_sys::encode_uri_component(&svg_content);
                        let src = format!("data:image/svg+xml;utf8,{encoded}");

                        let onload = wasm_bindgen::closure::Closure::wrap(Box::new(move || {
                            let _ = ctx.draw_image_with_html_image_element_and_dw_and_dh(
                                &img_clone, 0.0, 0.0, res as f64, res as f64,
                            );
                            if let Ok(data_url) = canvas.to_data_url_with_type("image/png") {
                                if let Ok(a_el) = doc.create_element("a") {
                                    let a = a_el.unchecked_into::<HtmlAnchorElement>();
                                    a.set_href(&data_url);
                                    a.set_download(&format!("qr-code-{res}x{res}.png"));
                                    a.click();
                                }
                            }
                        }) as Box<dyn FnMut()>);

                        img.set_onload(Some(onload.as_ref().unchecked_ref()));
                        onload.forget();
                        img.set_src(&src);
                    }
                }
            }
        }
    };

    view! {
        <div class="min-h-screen bg-[#04070d] text-[#e6f1f5] pb-16 selection:bg-[#3fd9ff]/25 selection:text-[#3fd9ff]">
            // Floating Pill Navigation Bar
            <div class="site-nav-wrap mb-8">
                <nav class="site-nav">
                    <a href="/" class="flex items-center gap-2 text-sm font-bold tracking-tight">
                        <span class="chromatic-text" data-text="qr.den1zz.dev">"qr.den1zz.dev"</span>
                    </a>

                    // Mode Switcher Toggle
                    <div class="flex items-center gap-1 bg-[#080d16]/80 p-1 rounded-full border border-[rgba(63,217,255,0.18)]">
                        <button
                            class=move || {
                                let base = "px-3.5 py-1 text-xs font-semibold rounded-full transition-all duration-150";
                                if app_mode.get() == AppMode::Vector {
                                    format!("{base} bg-[#3fd9ff] text-[#04070d] shadow-[0_0_16px_rgba(63,217,255,0.4)]")
                                } else {
                                    format!("{base} text-[#93a8b3] hover:text-[#e6f1f5]")
                                }
                            }
                            on:click=move |_| set_app_mode.set(AppMode::Vector)
                        >
                            "Vector & Shapes"
                        </button>
                        <button
                            class=move || {
                                let base = "px-3.5 py-1 text-xs font-semibold rounded-full transition-all duration-150";
                                if app_mode.get() == AppMode::Halftone {
                                    format!("{base} bg-[#3fd9ff] text-[#04070d] shadow-[0_0_16px_rgba(63,217,255,0.4)]")
                                } else {
                                    format!("{base} text-[#93a8b3] hover:text-[#e6f1f5]")
                                }
                            }
                            on:click=move |_| set_app_mode.set(AppMode::Halftone)
                        >
                            "Photo Halftone"
                        </button>
                    </div>

                    // Quick Outlink
                    <a
                        href="https://den1zz.dev"
                        target="_blank"
                        rel="noreferrer"
                        class="hidden sm:inline-flex items-center gap-1 button button-secondary text-xs py-1 px-3"
                    >
                        <span>"den1zz.dev"</span>
                        <span class="text-[#3fd9ff]">"↗"</span>
                    </a>
                </nav>
            </div>

            // Main Content Container
            <main class="w-full max-w-[1280px] mx-auto px-4 sm:px-6">
                // Studio Hero Eyebrow
                <div class="mb-8">
                    <div class="status-badge mb-2">
                        <span class="status-dot-active"></span>
                        <span>"ONLINE // PURE RUST ENGINE"</span>
                    </div>
                    <h1 class="text-3xl sm:text-4xl font-bold tracking-tight text-[#e6f1f5]">
                        <span class="chromatic-text" data-text="High Precision QR Studio">"High Precision QR Studio"</span>
                    </h1>
                    <p class="text-xs sm:text-sm text-[#93a8b3] mt-1.5 max-w-2xl font-sans">
                        "Ente-grade classy modules, customizable vector geometry, and sub-module photo halftoning running client-side in WebAssembly."
                    </p>
                </div>

                // Two Column Grid
                <div class="grid grid-cols-1 lg:grid-cols-12 gap-6 items-start">
                    // Left Column: Controls & Configuration
                    <div class="lg:col-span-6 xl:col-span-5 space-y-4">
                        // 1. PAYLOAD DECK
                        <div class="glass-panel p-5 space-y-4">
                            <div class="flex items-center justify-between border-b border-[rgba(63,217,255,0.18)] pb-3">
                                <span class="text-xs font-bold tracking-wider text-[#3fd9ff] uppercase">"01 // Payload"</span>
                                <div class="flex gap-1">
                                    {
                                        let tabs = [
                                            (PayloadType::Url, "URL"),
                                            (PayloadType::Text, "Text"),
                                            (PayloadType::Wifi, "WiFi"),
                                            (PayloadType::VCard, "vCard"),
                                            (PayloadType::Email, "Email"),
                                        ];
                                        tabs.into_iter().map(|(pt, label)| {
                                            view! {
                                                <button
                                                    class=move || {
                                                        let active = payload_type.get() == pt;
                                                        if active {
                                                            "text-[11px] px-2 py-0.5 rounded-full bg-[#3fd9ff]/15 text-[#3fd9ff] border border-[#3fd9ff]/40 font-bold"
                                                        } else {
                                                            "text-[11px] px-2 py-0.5 rounded-full text-[#93a8b3] hover:text-[#e6f1f5]"
                                                        }
                                                    }
                                                    on:click=move |_| set_payload_type.set(pt)
                                                >
                                                    {label}
                                                </button>
                                            }
                                        }).collect_view()
                                    }
                                </div>
                            </div>

                            // Inputs by Payload Type
                            {move || match payload_type.get() {
                                PayloadType::Url => view! {
                                    <div class="space-y-2.5">
                                        <input
                                            type="text"
                                            class="w-full"
                                            placeholder="https://example.com"
                                            prop:value=move || url_input.get()
                                            on:input=move |ev| set_url_input.set(event_target_value(&ev))
                                        />
                                        <label class="flex items-center gap-2 text-xs text-[#93a8b3] hover:text-[#e6f1f5] cursor-pointer select-none">
                                            <input
                                                type="checkbox"
                                                class="rounded accent-[#3fd9ff]"
                                                prop:checked=move || strip_tracking.get()
                                                on:change=move |ev| set_strip_tracking.set(event_target_checked(&ev))
                                            />
                                            <span>"Strip tracking query params (utm_*, fbclid, gclid)"</span>
                                        </label>

                                        {move || {
                                            let s = sanitized_memo.get();
                                            if s.stripped_params_count > 0 {
                                                view! {
                                                    <div class="px-3 py-2 rounded bg-[#080d16] border border-[#3fd9ff]/30 text-xs text-[#3fd9ff] flex items-center justify-between">
                                                        <span>{format!("{} tracking tags cleaned (-{} bytes)", s.stripped_params_count, s.raw_bytes.saturating_sub(s.cleaned_bytes))}</span>
                                                        <span class="text-[#93a8b3] font-mono">{format!("{} B → {} B", s.raw_bytes, s.cleaned_bytes)}</span>
                                                    </div>
                                                }.into_any()
                                            } else {
                                                view! {
                                                    <div class="text-[11px] text-[#56636b] font-mono">
                                                        {format!("Payload length: {} bytes", s.cleaned_bytes)}
                                                    </div>
                                                }.into_any()
                                            }
                                        }}
                                    </div>
                                }.into_any(),

                                PayloadType::Text => view! {
                                    <div>
                                        <textarea
                                            class="w-full h-24"
                                            placeholder="Enter plain text..."
                                            prop:value=move || text_input.get()
                                            on:input=move |ev| set_text_input.set(event_target_value(&ev))
                                        ></textarea>
                                    </div>
                                }.into_any(),

                                PayloadType::Wifi => view! {
                                    <div class="grid grid-cols-2 gap-2.5 text-xs">
                                        <div class="col-span-2">
                                            <label class="block text-[#93a8b3] text-[11px] mb-1">"SSID / Network Name"</label>
                                            <input
                                                type="text"
                                                class="w-full"
                                                prop:value=move || wifi_ssid.get()
                                                on:input=move |ev| set_wifi_ssid.set(event_target_value(&ev))
                                            />
                                        </div>
                                        <div>
                                            <label class="block text-[#93a8b3] text-[11px] mb-1">"Password"</label>
                                            <input
                                                type="password"
                                                class="w-full"
                                                prop:value=move || wifi_pass.get()
                                                on:input=move |ev| set_wifi_pass.set(event_target_value(&ev))
                                            />
                                        </div>
                                        <div>
                                            <label class="block text-[#93a8b3] text-[11px] mb-1">"Auth"</label>
                                            <select
                                                class="w-full"
                                                on:change=move |ev| {
                                                    match event_target_value(&ev).as_str() {
                                                        "WEP" => set_wifi_auth.set(WifiAuth::Wep),
                                                        "NOPASS" => set_wifi_auth.set(WifiAuth::Nopass),
                                                        _ => set_wifi_auth.set(WifiAuth::Wpa),
                                                    }
                                                }
                                            >
                                                <option value="WPA" selected=move || wifi_auth.get() == WifiAuth::Wpa>"WPA / WPA2"</option>
                                                <option value="WEP" selected=move || wifi_auth.get() == WifiAuth::Wep>"WEP"</option>
                                                <option value="NOPASS" selected=move || wifi_auth.get() == WifiAuth::Nopass>"Open / None"</option>
                                            </select>
                                        </div>
                                        <div class="col-span-2">
                                            <label class="flex items-center gap-2 cursor-pointer text-[#93a8b3] hover:text-[#e6f1f5] select-none">
                                                <input
                                                    type="checkbox"
                                                    class="rounded accent-[#3fd9ff]"
                                                    prop:checked=move || wifi_hidden.get()
                                                    on:change=move |ev| set_wifi_hidden.set(event_target_checked(&ev))
                                                />
                                                <span>"Hidden Network"</span>
                                            </label>
                                        </div>
                                    </div>
                                }.into_any(),

                                PayloadType::VCard => view! {
                                    <div class="grid grid-cols-2 gap-2 text-xs">
                                        <div>
                                            <label class="block text-[#93a8b3] text-[11px] mb-1">"Full Name"</label>
                                            <input
                                                type="text"
                                                class="w-full"
                                                prop:value=move || vcard_name.get()
                                                on:input=move |ev| set_vcard_name.set(event_target_value(&ev))
                                            />
                                        </div>
                                        <div>
                                            <label class="block text-[#93a8b3] text-[11px] mb-1">"Phone"</label>
                                            <input
                                                type="text"
                                                class="w-full"
                                                prop:value=move || vcard_phone.get()
                                                on:input=move |ev| set_vcard_phone.set(event_target_value(&ev))
                                            />
                                        </div>
                                        <div>
                                            <label class="block text-[#93a8b3] text-[11px] mb-1">"Email"</label>
                                            <input
                                                type="email"
                                                class="w-full"
                                                prop:value=move || vcard_email.get()
                                                on:input=move |ev| set_vcard_email.set(event_target_value(&ev))
                                            />
                                        </div>
                                        <div>
                                            <label class="block text-[#93a8b3] text-[11px] mb-1">"Organization"</label>
                                            <input
                                                type="text"
                                                class="w-full"
                                                prop:value=move || vcard_org.get()
                                                on:input=move |ev| set_vcard_org.set(event_target_value(&ev))
                                            />
                                        </div>
                                    </div>
                                }.into_any(),

                                PayloadType::Email => view! {
                                    <div class="space-y-2 text-xs">
                                        <input
                                            type="email"
                                            placeholder="recipient@domain.com"
                                            class="w-full"
                                            prop:value=move || email_to.get()
                                            on:input=move |ev| set_email_to.set(event_target_value(&ev))
                                        />
                                        <input
                                            type="text"
                                            placeholder="Subject line"
                                            class="w-full"
                                            prop:value=move || email_subject.get()
                                            on:input=move |ev| set_email_subject.set(event_target_value(&ev))
                                        />
                                    </div>
                                }.into_any(),
                            }}
                        </div>

                        // 2. STYLING DECK
                        {move || match app_mode.get() {
                            AppMode::Vector => view! {
                                <div class="glass-panel p-5 space-y-4">
                                    <div class="flex items-center justify-between border-b border-[rgba(63,217,255,0.18)] pb-3">
                                        <span class="text-xs font-bold tracking-wider text-[#3fd9ff] uppercase">"02 // Shapes & Palette"</span>
                                        <div class="flex items-center gap-1 text-[11px]">
                                            <span class="text-[#93a8b3]">"ECC:"</span>
                                            {
                                                let eccs = [
                                                    (EccLevel::Auto, "Auto"),
                                                    (EccLevel::L, "L"),
                                                    (EccLevel::M, "M"),
                                                    (EccLevel::Q, "Q"),
                                                    (EccLevel::H, "H"),
                                                ];
                                                eccs.into_iter().map(|(e, label)| {
                                                    view! {
                                                        <button
                                                            class=move || {
                                                                if ecc_level.get() == e {
                                                                    "px-2 py-0.5 rounded-full bg-[#3fd9ff] text-[#04070d] font-bold text-[10px]"
                                                                } else {
                                                                    "px-2 py-0.5 rounded-full text-[#93a8b3] hover:text-[#e6f1f5] text-[10px]"
                                                                }
                                                            }
                                                            on:click=move |_| set_ecc_level.set(e)
                                                        >
                                                            {label}
                                                        </button>
                                                    }
                                                }).collect_view()
                                            }
                                        </div>
                                    </div>

                                    // Module Shapes (Featuring Classy!)
                                    <div>
                                        <label class="block text-[#93a8b3] text-[11px] mb-1.5 uppercase font-semibold">"Module Shape"</label>
                                        <div class="grid grid-cols-4 gap-1.5">
                                            {
                                                let shapes = [
                                                    (ModuleShape::Classy, "Classy"),
                                                    (ModuleShape::Smooth, "Smooth"),
                                                    (ModuleShape::Dots, "Dots"),
                                                    (ModuleShape::Square, "Square"),
                                                ];
                                                shapes.into_iter().map(|(sh, label)| {
                                                    view! {
                                                        <button
                                                            class=move || {
                                                                let active = module_shape.get() == sh;
                                                                if active {
                                                                    "py-2 text-xs rounded-lg border border-[#3fd9ff] bg-[#3fd9ff]/15 text-[#3fd9ff] font-bold shadow-[0_0_12px_rgba(63,217,255,0.25)]"
                                                                } else {
                                                                    "py-2 text-xs rounded-lg border border-[rgba(63,217,255,0.18)] bg-[#080d16] text-[#93a8b3] hover:text-[#e6f1f5]"
                                                                }
                                                            }
                                                            on:click=move |_| set_module_shape.set(sh)
                                                        >
                                                            {label}
                                                        </button>
                                                    }
                                                }).collect_view()
                                            }
                                        </div>
                                    </div>

                                    // Eyes (Frame & Dot)
                                    <div class="grid grid-cols-2 gap-3">
                                        <div>
                                            <label class="block text-[#93a8b3] text-[11px] mb-1.5 uppercase font-semibold">"Eye Frame"</label>
                                            <div class="grid grid-cols-3 gap-1">
                                                {
                                                    let frames = [
                                                        (EyeFrameShape::Rounded, "Round"),
                                                        (EyeFrameShape::Circle, "Circle"),
                                                        (EyeFrameShape::Square, "Square"),
                                                    ];
                                                    frames.into_iter().map(|(f, label)| {
                                                        view! {
                                                            <button
                                                                class=move || {
                                                                    let active = eye_frame_shape.get() == f;
                                                                    if active {
                                                                        "py-1.5 text-[11px] rounded-md border border-[#3fd9ff] bg-[#3fd9ff]/15 text-[#3fd9ff] font-bold"
                                                                    } else {
                                                                        "py-1.5 text-[11px] rounded-md border border-[rgba(63,217,255,0.18)] bg-[#080d16] text-[#93a8b3] hover:text-[#e6f1f5]"
                                                                    }
                                                                }
                                                                on:click=move |_| set_eye_frame_shape.set(f)
                                                            >
                                                                {label}
                                                            </button>
                                                        }
                                                    }).collect_view()
                                                }
                                            </div>
                                        </div>

                                        <div>
                                            <label class="block text-[#93a8b3] text-[11px] mb-1.5 uppercase font-semibold">"Eye Dot"</label>
                                            <div class="grid grid-cols-3 gap-1">
                                                {
                                                    let dots = [
                                                        (EyeDotShape::Circle, "Circle"),
                                                        (EyeDotShape::Rounded, "Round"),
                                                        (EyeDotShape::Square, "Square"),
                                                    ];
                                                    dots.into_iter().map(|(d, label)| {
                                                        view! {
                                                            <button
                                                                class=move || {
                                                                    let active = eye_dot_shape.get() == d;
                                                                    if active {
                                                                        "py-1.5 text-[11px] rounded-md border border-[#3fd9ff] bg-[#3fd9ff]/15 text-[#3fd9ff] font-bold"
                                                                    } else {
                                                                        "py-1.5 text-[11px] rounded-md border border-[rgba(63,217,255,0.18)] bg-[#080d16] text-[#93a8b3] hover:text-[#e6f1f5]"
                                                                    }
                                                                }
                                                                on:click=move |_| set_eye_dot_shape.set(d)
                                                            >
                                                                {label}
                                                            </button>
                                                        }
                                                    }).collect_view()
                                                }
                                            </div>
                                        </div>
                                    </div>

                                    // Color & Swatches
                                    <div class="pt-3 border-t border-[rgba(63,217,255,0.18)] space-y-3">
                                        <div class="flex items-center justify-between">
                                            <label class="text-xs text-[#93a8b3] font-semibold">"Code Color"</label>
                                            <div class="flex items-center gap-2">
                                                <input
                                                    type="color"
                                                    class="w-7 h-7 rounded bg-transparent border border-[rgba(63,217,255,0.25)] cursor-pointer"
                                                    prop:value=move || code_color.get()
                                                    on:input=move |ev| set_code_color.set(event_target_value(&ev))
                                                />
                                                <input
                                                    type="text"
                                                    class="w-24 text-xs font-mono uppercase"
                                                    prop:value=move || code_color.get()
                                                    on:input=move |ev| set_code_color.set(event_target_value(&ev))
                                                />
                                            </div>
                                        </div>

                                        // Quick Swatches
                                        <div class="flex items-center gap-1.5">
                                            {
                                                let swatches = ["#3fd9ff", "#38ef7d", "#ff3b80", "#a855f7", "#f59e0b", "#ffffff"];
                                                swatches.into_iter().map(|hex| {
                                                    view! {
                                                        <button
                                                            class="w-6 h-6 rounded-full border border-white/20 transition-transform hover:scale-110"
                                                            style=format!("background-color: {hex}")
                                                            on:click=move |_| set_code_color.set(hex.to_string())
                                                        ></button>
                                                    }
                                                }).collect_view()
                                            }
                                        </div>

                                        // Gradient Toggle
                                        <div class="space-y-2">
                                            <label class="flex items-center gap-2 text-xs text-[#93a8b3] hover:text-[#e6f1f5] cursor-pointer select-none">
                                                <input
                                                    type="checkbox"
                                                    class="rounded accent-[#3fd9ff]"
                                                    prop:checked=move || is_gradient.get()
                                                    on:change=move |ev| set_is_gradient.set(event_target_checked(&ev))
                                                />
                                                <span>"Enable Gradient"</span>
                                            </label>

                                            {move || if is_gradient.get() {
                                                view! {
                                                    <div class="p-3 rounded-lg bg-[#080d16] border border-[rgba(63,217,255,0.18)] space-y-2.5 text-xs">
                                                        <div class="flex items-center justify-between">
                                                            <span class="text-[#93a8b3] text-[11px]">"Secondary Color"</span>
                                                            <div class="flex items-center gap-2">
                                                                <input
                                                                    type="color"
                                                                    class="w-6 h-6 rounded bg-transparent border border-white/20 cursor-pointer"
                                                                    prop:value=move || gradient_end_color.get()
                                                                    on:input=move |ev| set_gradient_end_color.set(event_target_value(&ev))
                                                                />
                                                                <input
                                                                    type="text"
                                                                    class="w-24 text-xs font-mono uppercase"
                                                                    prop:value=move || gradient_end_color.get()
                                                                    on:input=move |ev| set_gradient_end_color.set(event_target_value(&ev))
                                                                />
                                                            </div>
                                                        </div>
                                                        <div class="flex items-center gap-2">
                                                            <button
                                                                class=move || {
                                                                    if !gradient_type_radial.get() {
                                                                        "px-2.5 py-1 rounded bg-[#3fd9ff]/15 text-[#3fd9ff] border border-[#3fd9ff]/40 text-xs font-semibold"
                                                                    } else {
                                                                        "px-2.5 py-1 rounded bg-[#080d16] text-[#93a8b3] border border-[rgba(63,217,255,0.18)] text-xs"
                                                                    }
                                                                }
                                                                on:click=move |_| set_gradient_type_radial.set(false)
                                                            >
                                                                "Linear"
                                                            </button>
                                                            <button
                                                                class=move || {
                                                                    if gradient_type_radial.get() {
                                                                        "px-2.5 py-1 rounded bg-[#3fd9ff]/15 text-[#3fd9ff] border border-[#3fd9ff]/40 text-xs font-semibold"
                                                                    } else {
                                                                        "px-2.5 py-1 rounded bg-[#080d16] text-[#93a8b3] border border-[rgba(63,217,255,0.18)] text-xs"
                                                                    }
                                                                }
                                                                on:click=move |_| set_gradient_type_radial.set(true)
                                                            >
                                                                "Radial"
                                                            </button>
                                                        </div>
                                                        {move || if !gradient_type_radial.get() {
                                                            view! {
                                                                <div>
                                                                    <div class="flex justify-between text-[11px] text-[#93a8b3] mb-1">
                                                                        <span>"Linear Angle"</span>
                                                                        <span>{format!("{:.0}°", gradient_angle.get())}</span>
                                                                    </div>
                                                                    <input
                                                                        type="range"
                                                                        min="0"
                                                                        max="360"
                                                                        prop:value=move || gradient_angle.get().to_string()
                                                                        on:input=move |ev| {
                                                                            if let Ok(v) = event_target_value(&ev).parse::<f32>() {
                                                                                set_gradient_angle.set(v);
                                                                            }
                                                                        }
                                                                        class="w-full"
                                                                    />
                                                                </div>
                                                            }.into_any()
                                                        } else {
                                                            view! {}.into_any()
                                                        }}
                                                    </div>
                                                }.into_any()
                                            } else {
                                                view! {}.into_any()
                                            }}
                                        </div>

                                        // Custom Eye Color
                                        <div class="space-y-2">
                                            <label class="flex items-center gap-2 text-xs text-[#93a8b3] hover:text-[#e6f1f5] cursor-pointer select-none">
                                                <input
                                                    type="checkbox"
                                                    class="rounded accent-[#3fd9ff]"
                                                    prop:checked=move || use_custom_eyes.get()
                                                    on:change=move |ev| set_use_custom_eyes.set(event_target_checked(&ev))
                                                />
                                                <span>"Custom Eye Color"</span>
                                            </label>

                                            {move || if use_custom_eyes.get() {
                                                view! {
                                                    <div class="p-3 rounded-lg bg-[#080d16] border border-[rgba(63,217,255,0.18)] flex items-center justify-between text-xs">
                                                        <span class="text-[#93a8b3] text-[11px]">"Eye Tint"</span>
                                                        <div class="flex items-center gap-2">
                                                            <input
                                                                type="color"
                                                                class="w-6 h-6 rounded bg-transparent border border-white/20 cursor-pointer"
                                                                prop:value=move || eye_color.get()
                                                                on:input=move |ev| set_eye_color.set(event_target_value(&ev))
                                                            />
                                                            <input
                                                                type="text"
                                                                class="w-24 text-xs font-mono uppercase"
                                                                prop:value=move || eye_color.get()
                                                                on:input=move |ev| set_eye_color.set(event_target_value(&ev))
                                                            />
                                                        </div>
                                                    </div>
                                                }.into_any()
                                            } else {
                                                view! {}.into_any()
                                            }}
                                        </div>

                                        // Background & Quiet Zone
                                        <div class="pt-3 border-t border-[rgba(63,217,255,0.18)] space-y-3">
                                            <div class="flex items-center justify-between">
                                                <label class="text-xs text-[#93a8b3] font-semibold">"Background"</label>
                                                <div class="flex items-center gap-2">
                                                    {move || if !is_transparent_bg.get() {
                                                        view! {
                                                            <input
                                                                type="color"
                                                                class="w-7 h-7 rounded bg-transparent border border-[rgba(63,217,255,0.25)] cursor-pointer"
                                                                prop:value=move || bg_color.get()
                                                                on:input=move |ev| set_bg_color.set(event_target_value(&ev))
                                                            />
                                                        }.into_any()
                                                    } else {
                                                        view! {}.into_any()
                                                    }}
                                                    <label class="flex items-center gap-1.5 text-xs text-[#93a8b3] cursor-pointer select-none">
                                                        <input
                                                            type="checkbox"
                                                            class="rounded accent-[#3fd9ff]"
                                                            prop:checked=move || is_transparent_bg.get()
                                                            on:change=move |ev| set_is_transparent_bg.set(event_target_checked(&ev))
                                                        />
                                                        <span>"Transparent"</span>
                                                    </label>
                                                </div>
                                            </div>

                                            <div>
                                                <div class="flex justify-between text-[11px] text-[#93a8b3] mb-1">
                                                    <span>"Quiet Zone (Margin)"</span>
                                                    <span>{format!("{} modules", margin_modules.get())}</span>
                                                </div>
                                                <input
                                                    type="range"
                                                    min="0"
                                                    max="6"
                                                    prop:value=move || margin_modules.get().to_string()
                                                    on:input=move |ev| {
                                                        if let Ok(v) = event_target_value(&ev).parse::<usize>() {
                                                            set_margin_modules.set(v);
                                                        }
                                                    }
                                                    class="w-full"
                                                />
                                            </div>
                                        </div>
                                    </div>
                                </div>
                            }.into_any(),

                            AppMode::Halftone => view! {
                                <div class="glass-panel p-5 space-y-4">
                                    <div class="flex items-center justify-between border-b border-[rgba(63,217,255,0.18)] pb-3">
                                        <span class="text-xs font-bold tracking-wider text-[#3fd9ff] uppercase">"02 // Photo Halftone"</span>
                                        {move || if halftone_img_data_url.get().is_some() {
                                            view! {
                                                <button
                                                    class="text-[11px] text-rose-400 hover:underline"
                                                    on:click=move |_| {
                                                        set_halftone_img_data_url.set(None);
                                                        set_halftone_filename.set(None);
                                                    }
                                                >
                                                    "Remove Photo"
                                                </button>
                                            }.into_any()
                                        } else {
                                            view! {}.into_any()
                                        }}
                                    </div>

                                    // Upload Photo
                                    <div>
                                        <input
                                            type="file"
                                            accept="image/*"
                                            class="w-full text-xs text-[#93a8b3] file:mr-2 file:py-1.5 file:px-3 file:rounded-full file:border file:border-[#3fd9ff]/30 file:text-xs file:bg-[#3fd9ff]/10 file:text-[#3fd9ff] hover:file:bg-[#3fd9ff]/20 cursor-pointer"
                                            on:change=move |ev| {
                                                let target = event_target::<web_sys::HtmlInputElement>(&ev);
                                                if let Some(files) = target.files() {
                                                    if let Some(file) = files.get(0) {
                                                        set_halftone_filename.set(Some(file.name()));
                                                        let reader = web_sys::FileReader::new().unwrap();
                                                        let r_clone = reader.clone();
                                                        let onload = wasm_bindgen::closure::Closure::wrap(Box::new(move |_: web_sys::ProgressEvent| {
                                                            if let Ok(res) = r_clone.result() {
                                                                if let Some(s) = res.as_string() {
                                                                    set_halftone_img_data_url.set(Some(s));
                                                                }
                                                            }
                                                        }) as Box<dyn FnMut(_)>);
                                                        reader.set_onload(Some(onload.as_ref().unchecked_ref()));
                                                        onload.forget();
                                                        let _ = reader.read_as_data_url(&file);
                                                    }
                                                }
                                            }
                                        />
                                    </div>

                                    // Halftone Mode Select
                                    <div>
                                        <label class="block text-[#93a8b3] text-[11px] mb-1.5 uppercase font-semibold">"Render Mode"</label>
                                        <div class="grid grid-cols-3 gap-1.5">
                                            {
                                                let modes = [
                                                    (HalftoneColorMode::Color, "Color"),
                                                    (HalftoneColorMode::Sampled, "Sampled"),
                                                    (HalftoneColorMode::Bw, "B/W Dither"),
                                                ];
                                                modes.into_iter().map(|(m, label)| {
                                                    view! {
                                                        <button
                                                            class=move || {
                                                                let is_act = halftone_mode.get() == m;
                                                                if is_act {
                                                                    "py-2 text-xs rounded-lg border border-[#3fd9ff] bg-[#3fd9ff]/15 text-[#3fd9ff] font-bold"
                                                                } else {
                                                                    "py-2 text-xs rounded-lg border border-[rgba(63,217,255,0.18)] bg-[#080d16] text-[#93a8b3] hover:text-[#e6f1f5]"
                                                                }
                                                            }
                                                            on:click=move |_| set_halftone_mode.set(m)
                                                        >
                                                            {label}
                                                        </button>
                                                    }
                                                }).collect_view()
                                            }
                                        </div>
                                    </div>

                                    // Color / Sampled parameters
                                    {move || if halftone_mode.get() != HalftoneColorMode::Bw {
                                        view! {
                                            <div class="space-y-3 pt-2 border-t border-[rgba(63,217,255,0.18)]">
                                                <div class="flex items-center justify-between">
                                                    <label class="text-xs text-[#93a8b3]">"Dark Module Fill"</label>
                                                    <div class="flex items-center gap-2">
                                                        <input
                                                            type="color"
                                                            class="w-6 h-6 rounded bg-transparent border border-[rgba(63,217,255,0.25)] cursor-pointer"
                                                            prop:value=move || halftone_dark_color.get()
                                                            on:input=move |ev| set_halftone_dark_color.set(event_target_value(&ev))
                                                        />
                                                        <input
                                                            type="text"
                                                            class="w-24 text-xs font-mono uppercase"
                                                            prop:value=move || halftone_dark_color.get()
                                                            on:input=move |ev| set_halftone_dark_color.set(event_target_value(&ev))
                                                        />
                                                    </div>
                                                </div>
                                                <div>
                                                    <div class="flex justify-between text-[11px] text-[#93a8b3] mb-1">
                                                        <span>"Core Fill Size"</span>
                                                        <span>{cell_core_size.get()}</span>
                                                    </div>
                                                    <input
                                                        type="range"
                                                        min="1"
                                                        max="5"
                                                        prop:value=move || cell_core_size.get().to_string()
                                                        on:input=move |ev| {
                                                            if let Ok(v) = event_target_value(&ev).parse::<u32>() {
                                                                set_cell_core_size.set(v);
                                                            }
                                                        }
                                                        class="w-full"
                                                    />
                                                </div>
                                            </div>
                                        }.into_any()
                                    } else {
                                        view! {
                                            <div class="space-y-3 pt-2 border-t border-[rgba(63,217,255,0.18)]">
                                                <div>
                                                    <div class="flex justify-between text-[11px] text-[#93a8b3] mb-1">
                                                        <span>"QR Strength / Bias"</span>
                                                        <span>{strength_bias.get()}</span>
                                                    </div>
                                                    <input
                                                        type="range"
                                                        min="0"
                                                        max="120"
                                                        step="5"
                                                        prop:value=move || strength_bias.get().to_string()
                                                        on:input=move |ev| {
                                                            if let Ok(v) = event_target_value(&ev).parse::<i32>() {
                                                                set_strength_bias.set(v);
                                                            }
                                                        }
                                                        class="w-full"
                                                    />
                                                </div>
                                                <div>
                                                    <label class="block text-[#93a8b3] text-[11px] mb-1.5 uppercase font-semibold">"Dither Kernel"</label>
                                                    <div class="grid grid-cols-4 gap-1">
                                                        {
                                                            let d_algos = [
                                                                (DitherAlgorithm::Clustered, "Cluster"),
                                                                (DitherAlgorithm::Floyd, "Floyd-St"),
                                                                (DitherAlgorithm::Bayer4, "Bayer 4"),
                                                                (DitherAlgorithm::Bayer8, "Bayer 8"),
                                                            ];
                                                            d_algos.into_iter().map(|(da, label)| {
                                                                view! {
                                                                    <button
                                                                        class=move || {
                                                                            let act = dither_algo.get() == da;
                                                                            if act {
                                                                                "py-1 text-[11px] rounded border border-[#3fd9ff] bg-[#3fd9ff]/15 text-[#3fd9ff] font-bold"
                                                                            } else {
                                                                                "py-1 text-[11px] rounded border border-[rgba(63,217,255,0.18)] bg-[#080d16] text-[#93a8b3] hover:text-[#e6f1f5]"
                                                                            }
                                                                        }
                                                                        on:click=move |_| set_dither_algo.set(da)
                                                                    >
                                                                        {label}
                                                                    </button>
                                                                }
                                                            }).collect_view()
                                                        }
                                                    </div>
                                                </div>
                                            </div>
                                        }.into_any()
                                    }}

                                    <div class="space-y-2 pt-2 border-t border-[rgba(63,217,255,0.18)] text-xs">
                                        <label class="flex items-center gap-2 text-[#93a8b3] hover:text-[#e6f1f5] cursor-pointer select-none">
                                            <input
                                                type="checkbox"
                                                class="rounded accent-[#3fd9ff]"
                                                prop:checked=move || preserve_structure.get()
                                                on:change=move |ev| set_preserve_structure.set(event_target_checked(&ev))
                                            />
                                            <span>"Preserve structural finders and timing pattern"</span>
                                        </label>
                                        <label class="flex items-center gap-2 text-[#93a8b3] hover:text-[#e6f1f5] cursor-pointer select-none">
                                            <input
                                                type="checkbox"
                                                class="rounded accent-[#3fd9ff]"
                                                prop:checked=move || clahe_enabled.get()
                                                on:change=move |ev| set_clahe_enabled.set(event_target_checked(&ev))
                                            />
                                            <span>"Adaptive histogram equalization (CLAHE)"</span>
                                        </label>
                                    </div>
                                </div>
                            }.into_any(),
                        }}

                        // 3. LOGO & BRANDING DECK
                        <div class="glass-panel p-5 space-y-4">
                            <span class="text-xs font-bold tracking-wider text-[#3fd9ff] uppercase block border-b border-[rgba(63,217,255,0.18)] pb-3">
                                "03 // Branding & Badge"
                            </span>

                            // CTA Text Banner
                            <div class="space-y-2">
                                <label class="text-xs text-[#93a8b3] font-semibold block">"CTA Banner Text"</label>
                                <div class="grid grid-cols-3 gap-1">
                                    {
                                        let presets = [
                                            ("none", "None"),
                                            ("scan me", "SCAN ME"),
                                            ("wifi", "WIFI"),
                                            ("menu", "MENU"),
                                            ("visit us", "VISIT US"),
                                            ("custom", "Custom"),
                                        ];
                                        presets.into_iter().map(|(p, label)| {
                                            view! {
                                                <button
                                                    class=move || {
                                                        let active = cta_preset.get() == p;
                                                        if active {
                                                            "py-1 text-[11px] rounded-md border border-[#3fd9ff] bg-[#3fd9ff]/15 text-[#3fd9ff] font-bold"
                                                        } else {
                                                            "py-1 text-[11px] rounded-md border border-[rgba(63,217,255,0.18)] bg-[#080d16] text-[#93a8b3] hover:text-[#e6f1f5]"
                                                        }
                                                    }
                                                    on:click=move |_| set_cta_preset.set(p.to_string())
                                                >
                                                    {label}
                                                </button>
                                            }
                                        }).collect_view()
                                    }
                                </div>

                                {move || if cta_preset.get() == "custom" {
                                    view! {
                                        <input
                                            type="text"
                                            placeholder="Custom badge text..."
                                            class="w-full text-xs uppercase mt-2"
                                            prop:value=move || cta_custom_text.get()
                                            on:input=move |ev| set_cta_custom_text.set(event_target_value(&ev))
                                        />
                                    }.into_any()
                                } else {
                                    view! {}.into_any()
                                }}

                                {move || if cta_preset.get() != "none" {
                                    view! {
                                        <div class="grid grid-cols-2 gap-2 pt-1">
                                            <div>
                                                <label class="block text-[10px] text-[#93a8b3] mb-0.5">"Font Size"</label>
                                                <input
                                                    type="range"
                                                    min="10"
                                                    max="28"
                                                    prop:value=move || cta_font_size.get().to_string()
                                                    on:input=move |ev| {
                                                        if let Ok(v) = event_target_value(&ev).parse::<f32>() {
                                                            set_cta_font_size.set(v);
                                                        }
                                                    }
                                                    class="w-full"
                                                />
                                            </div>
                                            <div>
                                                <label class="block text-[10px] text-[#93a8b3] mb-0.5">"Banner Color"</label>
                                                <input
                                                    type="color"
                                                    class="w-6 h-6 rounded bg-transparent border border-white/20 cursor-pointer"
                                                    prop:value=move || cta_color.get()
                                                    on:input=move |ev| set_cta_color.set(event_target_value(&ev))
                                                />
                                            </div>
                                        </div>
                                    }.into_any()
                                } else {
                                    view! {}.into_any()
                                }}
                            </div>

                            // Center Logo
                            <div class="pt-3 border-t border-[rgba(63,217,255,0.18)] space-y-2">
                                <div class="flex items-center justify-between">
                                    <span class="text-xs text-[#93a8b3] font-semibold">"Center Logo"</span>
                                    {move || if logo_data_url.get().is_some() {
                                        view! {
                                            <button
                                                class="text-[11px] text-rose-400 hover:underline"
                                                on:click=move |_| {
                                                    set_logo_data_url.set(None);
                                                    set_logo_filename.set(None);
                                                }
                                            >
                                                "Remove Logo"
                                            </button>
                                        }.into_any()
                                    } else {
                                        view! {}.into_any()
                                    }}
                                </div>

                                <input
                                    type="file"
                                    accept="image/*"
                                    class="w-full text-xs text-[#93a8b3] file:mr-2 file:py-1 file:px-2.5 file:rounded-full file:border file:border-[#3fd9ff]/30 file:text-xs file:bg-[#3fd9ff]/10 file:text-[#3fd9ff] hover:file:bg-[#3fd9ff]/20 cursor-pointer"
                                    on:change=move |ev| {
                                        let target = event_target::<web_sys::HtmlInputElement>(&ev);
                                        if let Some(files) = target.files() {
                                            if let Some(file) = files.get(0) {
                                                set_logo_filename.set(Some(file.name()));
                                                let reader = web_sys::FileReader::new().unwrap();
                                                let r_clone = reader.clone();
                                                let onload = wasm_bindgen::closure::Closure::wrap(Box::new(move |_: web_sys::ProgressEvent| {
                                                    if let Ok(res) = r_clone.result() {
                                                        if let Some(s) = res.as_string() {
                                                            set_logo_data_url.set(Some(s));
                                                        }
                                                    }
                                                }) as Box<dyn FnMut(_)>);
                                                reader.set_onload(Some(onload.as_ref().unchecked_ref()));
                                                onload.forget();
                                                let _ = reader.read_as_data_url(&file);
                                            }
                                        }
                                    }
                                />

                                {move || if logo_data_url.get().is_some() {
                                    view! {
                                        <div class="space-y-2 pt-1 text-xs">
                                            <div class="flex items-center justify-between">
                                                <span class="text-[#93a8b3] text-[11px]">"Mask Shape"</span>
                                                <div class="flex gap-1">
                                                    {
                                                        let shapes = [
                                                            (LogoMaskShape::Rounded, "Round"),
                                                            (LogoMaskShape::Circle, "Circle"),
                                                            (LogoMaskShape::Square, "Square"),
                                                        ];
                                                        shapes.into_iter().map(|(s, label)| {
                                                            view! {
                                                                <button
                                                                    class=move || {
                                                                        let is_act = logo_shape.get() == s;
                                                                        if is_act {
                                                                            "px-2 py-0.5 rounded border border-[#3fd9ff] bg-[#3fd9ff]/15 text-[#3fd9ff] text-[11px] font-bold"
                                                                        } else {
                                                                            "px-2 py-0.5 rounded border border-[rgba(63,217,255,0.18)] bg-[#080d16] text-[#93a8b3] text-[11px]"
                                                                        }
                                                                    }
                                                                    on:click=move |_| set_logo_shape.set(s)
                                                                >
                                                                    {label}
                                                                </button>
                                                            }
                                                        }).collect_view()
                                                    }
                                                </div>
                                            </div>
                                            <div>
                                                <div class="flex justify-between text-[11px] text-[#93a8b3] mb-1">
                                                    <span>"Logo Scale"</span>
                                                    <span>{format!("{:.0}%", logo_scale.get())}</span>
                                                </div>
                                                <input
                                                    type="range"
                                                    min="15"
                                                    max="30"
                                                    prop:value=move || logo_scale.get().to_string()
                                                    on:input=move |ev| {
                                                        if let Ok(v) = event_target_value(&ev).parse::<f32>() {
                                                            set_logo_scale.set(v);
                                                        }
                                                    }
                                                    class="w-full"
                                                />
                                            </div>
                                        </div>
                                    }.into_any()
                                } else {
                                    view! {}.into_any()
                                }}
                            </div>
                        </div>
                    </div>

                    // Right Column: Preview Stage & HUD
                    <div class="lg:col-span-6 xl:col-span-7 space-y-4 lg:sticky lg:top-24">
                        // Scannability & Contrast Status Bar
                        {move || {
                            if app_mode.get() == AppMode::Vector {
                                let ratio = vector_contrast_ratio.get();
                                let is_ok = ratio >= 3.5;
                                view! {
                                    <div class="glass-panel px-4 py-3 flex items-center justify-between text-xs">
                                        <div class="flex items-center gap-2.5">
                                            <span class=if is_ok {
                                                "w-2.5 h-2.5 rounded-full bg-emerald-400 shadow-[0_0_8px_#34d399] shrink-0"
                                            } else {
                                                "w-2.5 h-2.5 rounded-full bg-amber-400 shadow-[0_0_8px_#fbbf24] shrink-0"
                                            }></span>
                                            <span class="font-semibold text-[#e6f1f5]">
                                                {if is_ok { "Ready to scan (high contrast)" } else { "Low contrast warning" }}
                                            </span>
                                        </div>
                                        <span class="font-mono text-[#3fd9ff]">{format!("Contrast {ratio:.1}:1")}</span>
                                    </div>
                                }.into_any()
                            } else {
                                match halftone_result_memo.get() {
                                    Some((_, _, report)) => {
                                        let (dot_class, text) = match report.status {
                                            ScanStatus::Verified => ("bg-emerald-400 shadow-[0_0_8px_#34d399]", "Verified scannable (100% bit match)"),
                                            ScanStatus::Uncertain { .. } => ("bg-amber-400 shadow-[0_0_8px_#fbbf24]", "Scan uncertain — test in scanner"),
                                            ScanStatus::Unscannable => ("bg-rose-400 shadow-[0_0_8px_#f43f5e]", "Unscannable — increase bias or core size"),
                                        };
                                        view! {
                                            <div class="glass-panel px-4 py-3 flex items-center justify-between text-xs">
                                                <div class="flex items-center gap-2.5">
                                                    <span class=format!("w-2.5 h-2.5 rounded-full {dot_class} shrink-0")></span>
                                                    <span class="font-semibold text-[#e6f1f5]">{text}</span>
                                                </div>
                                                <span class="font-mono text-[#3fd9ff]">{format!("Contrast {:.1}:1", report.contrast_ratio)}</span>
                                            </div>
                                        }.into_any()
                                    }
                                    None => view! {
                                        <div class="glass-panel px-4 py-3 flex items-center gap-2 text-xs text-[#93a8b3]">
                                            <span class="w-2.5 h-2.5 rounded-full bg-[#56636b] shrink-0"></span>
                                            <span>"Upload an image in the left panel to generate halftone QR"</span>
                                        </div>
                                    }.into_any()
                                }
                            }
                        }}

                        // The Viewfinder QR Stage Frame (Guaranteed Fit)
                        <div class="glass-panel p-6 flex flex-col items-center justify-center relative min-h-[460px]">
                            // HUD Corner Reticles
                            <div class="hud-reticle hud-tl"></div>
                            <div class="hud-reticle hud-tr"></div>
                            <div class="hud-reticle hud-bl"></div>
                            <div class="hud-reticle hud-br"></div>

                            {move || {
                                if app_mode.get() == AppMode::Vector {
                                    match vector_result_memo.get() {
                                        Ok((matrix, svg)) => {
                                            view! {
                                                <div class="w-full flex flex-col items-center justify-center">
                                                    <div class="viewfinder-frame">
                                                        <div class="hud-reticle hud-tl"></div>
                                                        <div class="hud-reticle hud-tr"></div>
                                                        <div class="hud-reticle hud-bl"></div>
                                                        <div class="hud-reticle hud-br"></div>
                                                        <div
                                                            class="qr-svg-wrapper"
                                                            inner_html=svg
                                                        ></div>
                                                    </div>
                                                    <div class="mt-4 font-mono text-xs text-[#93a8b3]">
                                                        {format!("QR Version {} • {}×{} modules", matrix.version, matrix.size, matrix.size)}
                                                    </div>
                                                </div>
                                            }.into_any()
                                        }
                                        Err(e) => view! {
                                            <div class="text-rose-400 text-xs font-mono">{format!("Error: {e}")}</div>
                                        }.into_any()
                                    }
                                } else {
                                    match halftone_result_memo.get() {
                                        Some((matrix, svg, _)) => {
                                            view! {
                                                <div class="w-full flex flex-col items-center justify-center">
                                                    <div class="viewfinder-frame">
                                                        <div class="hud-reticle hud-tl"></div>
                                                        <div class="hud-reticle hud-tr"></div>
                                                        <div class="hud-reticle hud-bl"></div>
                                                        <div class="hud-reticle hud-br"></div>
                                                        <div
                                                            class="qr-svg-wrapper"
                                                            inner_html=svg
                                                        ></div>
                                                    </div>
                                                    <div class="mt-4 font-mono text-xs text-[#93a8b3]">
                                                        {format!("Halftone v{} • {}×{} modules • 5×5 sub-grid", matrix.version, matrix.size, matrix.size)}
                                                    </div>
                                                </div>
                                            }.into_any()
                                        }
                                        None => view! {
                                            <div class="text-center py-16 text-[#93a8b3]">
                                                <span class="text-4xl block mb-3 opacity-30">"📸"</span>
                                                <span class="text-xs font-mono block text-[#93a8b3]">"Awaiting photo upload"</span>
                                                <span class="text-[11px] text-[#56636b] block mt-1">"Select any PNG, JPG, or WebP to render halftone"</span>
                                            </div>
                                        }.into_any()
                                    }
                                }
                            }}

                            // Toast copy notification
                            {move || copied_msg.get().map(|msg| {
                                view! {
                                    <div class="absolute bottom-4 px-3.5 py-1.5 rounded-full bg-[#080d16] border border-[#3fd9ff] text-[#3fd9ff] text-xs font-mono font-bold shadow-[0_0_20px_rgba(63,217,255,0.4)]">
                                        {msg}
                                    </div>
                                }
                            })}
                        </div>

                        // Actions & Export Bar
                        <div class="glass-panel p-5 space-y-4">
                            <div class="flex items-center justify-between text-xs pb-3 border-b border-[rgba(63,217,255,0.18)]">
                                <span class="text-[#93a8b3] font-semibold uppercase text-[11px]">"Export Resolution"</span>
                                <div class="flex gap-1">
                                    {
                                        let resolutions = [512u32, 1024u32, 2048u32, 4096u32];
                                        resolutions.into_iter().map(|res| {
                                            view! {
                                                <button
                                                    class=move || {
                                                        let is_act = target_res.get() == res;
                                                        if is_act {
                                                            "px-2.5 py-0.5 rounded-full bg-[#3fd9ff] text-[#04070d] font-bold text-[11px] shadow-[0_0_12px_rgba(63,217,255,0.35)]"
                                                        } else {
                                                            "px-2.5 py-0.5 rounded-full text-[#93a8b3] hover:text-[#e6f1f5] text-[11px]"
                                                        }
                                                    }
                                                    on:click=move |_| set_target_res.set(res)
                                                >
                                                    {format!("{res}px")}
                                                </button>
                                            }
                                        }).collect_view()
                                    }
                                </div>
                            </div>

                            <div class="flex flex-col sm:flex-row items-center gap-2.5">
                                <button
                                    class="button button-primary w-full sm:flex-1 py-2.5 text-center text-xs font-bold"
                                    on:click=move |_| {
                                        let res = target_res.get();
                                        if app_mode.get() == AppMode::Vector {
                                            if let Ok((_, svg)) = vector_result_memo.get() {
                                                download_png(svg, res);
                                            }
                                        } else if let Some((_, svg, _)) = halftone_result_memo.get() {
                                            download_png(svg, res);
                                        }
                                    }
                                >
                                    {move || format!("↓ Download PNG ({}px)", target_res.get())}
                                </button>

                                <button
                                    class="button button-secondary w-full sm:w-auto py-2.5 px-4 text-xs font-semibold"
                                    on:click=move |_| {
                                        if app_mode.get() == AppMode::Vector {
                                            if let Ok((_, svg)) = vector_result_memo.get() {
                                                download_svg(svg);
                                            }
                                        } else if let Some((_, svg, _)) = halftone_result_memo.get() {
                                            download_svg(svg);
                                        }
                                    }
                                >
                                    "↓ SVG"
                                </button>

                                <button
                                    class="button button-secondary w-full sm:w-auto py-2.5 px-4 text-xs font-semibold"
                                    on:click=move |_| {
                                        if app_mode.get() == AppMode::Vector {
                                            if let Ok((_, svg)) = vector_result_memo.get() {
                                                copy_text_to_clipboard(svg, "SVG");
                                            }
                                        } else if let Some((_, svg, _)) = halftone_result_memo.get() {
                                            copy_text_to_clipboard(svg, "SVG");
                                        }
                                    }
                                >
                                    "Copy SVG"
                                </button>
                            </div>
                        </div>
                    </div>
                </div>
            </main>
        </div>
    }
}

fn main() {
    console_error_panic_hook::set_once();
    mount_to_body(|| view! { <App /> });
}
