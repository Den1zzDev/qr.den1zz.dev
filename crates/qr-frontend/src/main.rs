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
    // Top-level Mode
    let (app_mode, set_app_mode) = signal(AppMode::Vector);

    // Payload State
    let (payload_type, set_payload_type) = signal(PayloadType::Url);
    let (url_input, set_url_input) = signal("https://qr.den1zz.dev".to_string());
    let (strip_tracking, set_strip_tracking) = signal(true);

    let (text_input, set_text_input) = signal("QR Studio".to_string());

    let (wifi_ssid, set_wifi_ssid) = signal("WiFi-Network".to_string());
    let (wifi_pass, set_wifi_pass) = signal("pass1234".to_string());
    let (wifi_auth, set_wifi_auth) = signal(WifiAuth::Wpa);
    let (wifi_hidden, set_wifi_hidden) = signal(false);

    let (vcard_name, set_vcard_name) = signal("Den1zz".to_string());
    let (vcard_phone, set_vcard_phone) = signal("+123456789".to_string());
    let (vcard_email, set_vcard_email) = signal("hello@den1zz.dev".to_string());
    let (vcard_org, set_vcard_org) = signal("Studio".to_string());

    let (email_to, set_email_to) = signal("hello@den1zz.dev".to_string());
    let (email_subject, set_email_subject) = signal("Contact".to_string());

    // Vector Mode Styling State
    let (module_shape, set_module_shape) = signal(ModuleShape::Smooth);
    let (eye_frame_shape, set_eye_frame_shape) = signal(EyeFrameShape::Rounded);
    let (eye_dot_shape, set_eye_dot_shape) = signal(EyeDotShape::Circle);

    let (code_color, set_code_color) = signal("#A855F7".to_string());
    let (is_gradient, set_is_gradient) = signal(false);
    let (gradient_type_radial, set_gradient_type_radial) = signal(false);
    let (gradient_angle, set_gradient_angle) = signal(135.0f32);
    let (gradient_end_color, set_gradient_end_color) = signal("#EC4899".to_string());

    let (use_custom_eyes, set_use_custom_eyes) = signal(false);
    let (eye_color, set_eye_color) = signal("#F43F5E".to_string());

    let (is_transparent_bg, set_is_transparent_bg) = signal(false);
    let (bg_color, set_bg_color) = signal("#09090B".to_string());

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
    let (cta_color, set_cta_color) = signal("#F4F4F5".to_string());

    // Halftone Mode State
    let (halftone_img_data_url, set_halftone_img_data_url) = signal(Option::<String>::None);
    let (_halftone_filename, set_halftone_filename) = signal(Option::<String>::None);
    let (halftone_mode, set_halftone_mode) = signal(HalftoneColorMode::Color);
    let (dither_algo, set_dither_algo) = signal(DitherAlgorithm::Clustered);
    let (strength_bias, set_strength_bias) = signal(50i32);
    let (cell_core_size, set_cell_core_size) = signal(3u32);
    let (halftone_dark_color, set_halftone_dark_color) = signal("#18181B".to_string());
    let (clahe_enabled, set_clahe_enabled) = signal(true);
    let (preserve_structure, set_preserve_structure) = signal(true);

    // Export Options
    let (target_res, set_target_res) = signal(1024u32);
    let (alt_text, _set_alt_text) = signal("QR Code".to_string());
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
            alt_text: Some(alt_text.get()),
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

    // Helper: Contrast for Vector Mode
    let vector_contrast_ratio = Memo::new(move |_| {
        let code = code_color.get();
        let bg = if is_transparent_bg.get() {
            "#FFFFFF"
        } else {
            &bg_color.get()
        };
        calculate_contrast_ratio(&code, bg)
    });

    // Copy to clipboard helper
    let copy_text_to_clipboard = move |text: String, label: &'static str| {
        if let Some(window) = web_sys::window() {
            let nav = window.navigator();
            let cb = nav.clipboard();
            let _ = cb.write_text(&text);
            set_copied_msg.set(Some(format!("Copied {label}!")));
            wasm_bindgen_futures::spawn_local(async move {
                gloo_timers::future::TimeoutFuture::new(1500).await;
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

    // Download PNG at target resolution
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
        <div class="max-w-[1280px] mx-auto px-4 py-8">
            // Header
            <header class="mb-8 flex flex-col sm:flex-row sm:items-center justify-between gap-4 border-b border-edge pb-6">
                <div>
                    <div class="flex items-center gap-3">
                        <span class="w-3 h-3 rounded-sm bg-purple"></span>
                        <h1 class="text-2xl font-bold tracking-tight text-ink">"QR Code Studio"</h1>
                        <span class="text-xs px-2 py-0.5 rounded bg-purple-subtle text-purple border border-purple/30 font-medium">"Pure Rust"</span>
                    </div>
                    <p class="text-xs text-muted mt-1.5 font-mono">
                        "Vector customization and binary photo halftoning united in a single engine."
                    </p>
                </div>

                // Engine Mode Switcher Tabs
                <div class="flex items-center bg-surface border border-edge rounded p-1">
                    <button
                        class=move || {
                            let base = "px-4 py-1.5 text-xs font-medium rounded transition-colors";
                            if app_mode.get() == AppMode::Vector {
                                format!("{base} bg-purple text-ink shadow-sm")
                            } else {
                                format!("{base} text-muted hover:text-ink")
                            }
                        }
                        on:click=move |_| set_app_mode.set(AppMode::Vector)
                    >
                        "Vector & Logo"
                    </button>
                    <button
                        class=move || {
                            let base = "px-4 py-1.5 text-xs font-medium rounded transition-colors";
                            if app_mode.get() == AppMode::Halftone {
                                format!("{base} bg-purple text-ink shadow-sm")
                            } else {
                                format!("{base} text-muted hover:text-ink")
                            }
                        }
                        on:click=move |_| set_app_mode.set(AppMode::Halftone)
                    >
                        "Photo Halftone"
                    </button>
                </div>
            </header>

            // Main Studio Layout
            <div class="grid grid-cols-1 lg:grid-cols-12 gap-6">
                // Left Configuration Column
                <div class="lg:col-span-6 xl:col-span-5 space-y-4">
                    // 1. Payload Panel
                    <div class="bg-surface border border-edge rounded p-4 space-y-3">
                        <div class="flex items-center justify-between border-b border-edge pb-2">
                            <span class="text-xs font-semibold uppercase tracking-wider text-muted">"1. Payload Content"</span>
                            // Format tabs
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
                                                    let is_active = payload_type.get() == pt;
                                                    if is_active {
                                                        "text-[11px] px-2 py-0.5 rounded bg-purple-subtle text-purple border border-purple/40 font-medium"
                                                    } else {
                                                        "text-[11px] px-2 py-0.5 rounded text-muted hover:text-ink"
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

                        // Payload Type Inputs
                        {move || match payload_type.get() {
                            PayloadType::Url => view! {
                                <div class="space-y-2">
                                    <input
                                        type="text"
                                        class="w-full bg-bg border border-edge rounded px-3 py-2 text-xs text-ink focus:border-purple focus:outline-none"
                                        placeholder="https://..."
                                        prop:value=move || url_input.get()
                                        on:input=move |ev| set_url_input.set(event_target_value(&ev))
                                    />
                                    <label class="flex items-center gap-2 cursor-pointer text-xs text-muted hover:text-ink select-none">
                                        <input
                                            type="checkbox"
                                            prop:checked=move || strip_tracking.get()
                                            on:change=move |ev| set_strip_tracking.set(event_target_checked(&ev))
                                            class="rounded border-edge"
                                        />
                                        <span>"Strip tracking query params (utm_*, fbclid, gclid, etc.)"</span>
                                    </label>
                                    // Live cleaned stats
                                    {move || {
                                        let s = sanitized_memo.get();
                                        if s.stripped_params_count > 0 {
                                            view! {
                                                <div class="p-2 rounded bg-bg border border-purple/30 text-[11px] text-purple flex items-center justify-between">
                                                    <span>{format!("{} tracking params stripped (-{} B)", s.stripped_params_count, s.raw_bytes.saturating_sub(s.cleaned_bytes))}</span>
                                                    <span class="text-muted font-mono">{format!("{} B → {} B", s.raw_bytes, s.cleaned_bytes)}</span>
                                                </div>
                                            }.into_any()
                                        } else {
                                            view! {
                                                <div class="text-[11px] text-muted font-mono">
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
                                        class="w-full bg-bg border border-edge rounded p-2 text-xs text-ink focus:border-purple focus:outline-none h-20"
                                        placeholder="Enter plain text..."
                                        prop:value=move || text_input.get()
                                        on:input=move |ev| set_text_input.set(event_target_value(&ev))
                                    ></textarea>
                                </div>
                            }.into_any(),
                            PayloadType::Wifi => view! {
                                <div class="grid grid-cols-2 gap-2 text-xs">
                                    <div class="col-span-2">
                                        <label class="block text-muted text-[11px] mb-1">"SSID / Network Name"</label>
                                        <input
                                            type="text"
                                            class="w-full bg-bg border border-edge rounded px-2.5 py-1.5 text-xs text-ink focus:border-purple focus:outline-none"
                                            prop:value=move || wifi_ssid.get()
                                            on:input=move |ev| set_wifi_ssid.set(event_target_value(&ev))
                                        />
                                    </div>
                                    <div>
                                        <label class="block text-muted text-[11px] mb-1">"Password"</label>
                                        <input
                                            type="password"
                                            class="w-full bg-bg border border-edge rounded px-2.5 py-1.5 text-xs text-ink focus:border-purple focus:outline-none"
                                            prop:value=move || wifi_pass.get()
                                            on:input=move |ev| set_wifi_pass.set(event_target_value(&ev))
                                        />
                                    </div>
                                    <div>
                                        <label class="block text-muted text-[11px] mb-1">"Auth"</label>
                                        <select
                                            class="w-full bg-bg border border-edge rounded px-2 py-1.5 text-xs text-ink focus:border-purple focus:outline-none"
                                            on:change=move |ev| {
                                                match event_target_value(&ev).as_str() {
                                                    "WEP" => set_wifi_auth.set(WifiAuth::Wep),
                                                    "NOPASS" => set_wifi_auth.set(WifiAuth::Nopass),
                                                    _ => set_wifi_auth.set(WifiAuth::Wpa),
                                                }
                                            }
                                        >
                                            <option value="WPA" selected=move || wifi_auth.get() == WifiAuth::Wpa>"WPA/WPA2"</option>
                                            <option value="WEP" selected=move || wifi_auth.get() == WifiAuth::Wep>"WEP"</option>
                                            <option value="NOPASS" selected=move || wifi_auth.get() == WifiAuth::Nopass>"None"</option>
                                        </select>
                                    </div>
                                    <div class="col-span-2">
                                        <label class="flex items-center gap-2 cursor-pointer text-xs text-muted hover:text-ink select-none">
                                            <input
                                                type="checkbox"
                                                prop:checked=move || wifi_hidden.get()
                                                on:change=move |ev| set_wifi_hidden.set(event_target_checked(&ev))
                                                class="rounded border-edge"
                                            />
                                            <span>"Hidden Network"</span>
                                        </label>
                                    </div>
                                </div>
                            }.into_any(),
                            PayloadType::VCard => view! {
                                <div class="grid grid-cols-2 gap-2 text-xs">
                                    <div>
                                        <label class="block text-muted text-[11px] mb-1">"Full Name"</label>
                                        <input
                                            type="text"
                                            class="w-full bg-bg border border-edge rounded px-2.5 py-1 text-xs text-ink focus:border-purple focus:outline-none"
                                            prop:value=move || vcard_name.get()
                                            on:input=move |ev| set_vcard_name.set(event_target_value(&ev))
                                        />
                                    </div>
                                    <div>
                                        <label class="block text-muted text-[11px] mb-1">"Phone"</label>
                                        <input
                                            type="text"
                                            class="w-full bg-bg border border-edge rounded px-2.5 py-1 text-xs text-ink focus:border-purple focus:outline-none"
                                            prop:value=move || vcard_phone.get()
                                            on:input=move |ev| set_vcard_phone.set(event_target_value(&ev))
                                        />
                                    </div>
                                    <div>
                                        <label class="block text-muted text-[11px] mb-1">"Email"</label>
                                        <input
                                            type="email"
                                            class="w-full bg-bg border border-edge rounded px-2.5 py-1 text-xs text-ink focus:border-purple focus:outline-none"
                                            prop:value=move || vcard_email.get()
                                            on:input=move |ev| set_vcard_email.set(event_target_value(&ev))
                                        />
                                    </div>
                                    <div>
                                        <label class="block text-muted text-[11px] mb-1">"Organization"</label>
                                        <input
                                            type="text"
                                            class="w-full bg-bg border border-edge rounded px-2.5 py-1 text-xs text-ink focus:border-purple focus:outline-none"
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
                                        class="w-full bg-bg border border-edge rounded px-2.5 py-1.5 text-xs text-ink focus:border-purple focus:outline-none"
                                        prop:value=move || email_to.get()
                                        on:input=move |ev| set_email_to.set(event_target_value(&ev))
                                    />
                                    <input
                                        type="text"
                                        placeholder="Subject"
                                        class="w-full bg-bg border border-edge rounded px-2.5 py-1.5 text-xs text-ink focus:border-purple focus:outline-none"
                                        prop:value=move || email_subject.get()
                                        on:input=move |ev| set_email_subject.set(event_target_value(&ev))
                                    />
                                </div>
                            }.into_any(),
                        }}
                    </div>

                    // 2. Engine Controls (Vector vs Halftone)
                    {move || match app_mode.get() {
                        AppMode::Vector => view! {
                            <div class="bg-surface border border-edge rounded p-4 space-y-4">
                                <div class="border-b border-edge pb-2 flex items-center justify-between">
                                    <span class="text-xs font-semibold uppercase tracking-wider text-muted">"2. Geometry & Shapes"</span>
                                    // ECC selector
                                    <div class="flex items-center gap-1 text-[11px]">
                                        <span class="text-muted">"ECC:"</span>
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
                                                                "px-1.5 py-0.5 rounded bg-purple-subtle text-purple border border-purple/30 font-medium"
                                                            } else {
                                                                "px-1.5 py-0.5 rounded text-muted hover:text-ink"
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

                                // Module Shapes
                                <div>
                                    <label class="block text-muted text-[11px] mb-1.5">"Module Shape"</label>
                                    <div class="grid grid-cols-4 gap-1.5">
                                        {
                                            let shapes = [
                                                (ModuleShape::Square, "Square"),
                                                (ModuleShape::Smooth, "Smooth"),
                                                (ModuleShape::Dots, "Dots"),
                                                (ModuleShape::Classy, "Classy"),
                                            ];
                                            shapes.into_iter().map(|(sh, label)| {
                                                view! {
                                                    <button
                                                        class=move || {
                                                            let active = module_shape.get() == sh;
                                                            if active {
                                                                "py-1.5 text-xs rounded border border-purple bg-purple/20 text-purple font-medium"
                                                            } else {
                                                                "py-1.5 text-xs rounded border border-edge bg-bg text-muted hover:text-ink"
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

                                // Eye Frames & Dots
                                <div class="grid grid-cols-2 gap-3">
                                    <div>
                                        <label class="block text-muted text-[11px] mb-1.5">"Eye Frame"</label>
                                        <div class="grid grid-cols-3 gap-1">
                                            {
                                                let frames = [
                                                    (EyeFrameShape::Square, "Square"),
                                                    (EyeFrameShape::Rounded, "Round"),
                                                    (EyeFrameShape::Circle, "Circle"),
                                                ];
                                                frames.into_iter().map(|(f, label)| {
                                                    view! {
                                                        <button
                                                            class=move || {
                                                                let active = eye_frame_shape.get() == f;
                                                                if active {
                                                                  "py-1 text-[11px] rounded border border-purple bg-purple/20 text-purple font-medium"
                                                                } else {
                                                                  "py-1 text-[11px] rounded border border-edge bg-bg text-muted hover:text-ink"
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
                                        <label class="block text-muted text-[11px] mb-1.5">"Eye Dot"</label>
                                        <div class="grid grid-cols-3 gap-1">
                                            {
                                                let dots = [
                                                    (EyeDotShape::Square, "Square"),
                                                    (EyeDotShape::Rounded, "Round"),
                                                    (EyeDotShape::Circle, "Circle"),
                                                ];
                                                dots.into_iter().map(|(d, label)| {
                                                    view! {
                                                        <button
                                                            class=move || {
                                                                let active = eye_dot_shape.get() == d;
                                                                if active {
                                                                  "py-1 text-[11px] rounded border border-purple bg-purple/20 text-purple font-medium"
                                                                } else {
                                                                  "py-1 text-[11px] rounded border border-edge bg-bg text-muted hover:text-ink"
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

                                // Color & Gradient
                                <div class="pt-2 border-t border-edge space-y-3">
                                    <div class="flex items-center justify-between">
                                        <label class="text-xs text-muted">"Code Color"</label>
                                        <div class="flex items-center gap-2">
                                            <input
                                                type="color"
                                                class="w-6 h-6 rounded bg-transparent border border-edge cursor-pointer"
                                                prop:value=move || code_color.get()
                                                on:input=move |ev| set_code_color.set(event_target_value(&ev))
                                            />
                                            <input
                                                type="text"
                                                class="w-20 bg-bg border border-edge rounded px-1.5 py-0.5 text-xs text-ink font-mono uppercase"
                                                prop:value=move || code_color.get()
                                                on:input=move |ev| set_code_color.set(event_target_value(&ev))
                                            />
                                        </div>
                                    </div>

                                    // Gradient Toggle
                                    <div class="space-y-2">
                                        <label class="flex items-center gap-2 text-xs text-muted hover:text-ink cursor-pointer select-none">
                                            <input
                                                type="checkbox"
                                                prop:checked=move || is_gradient.get()
                                                on:change=move |ev| set_is_gradient.set(event_target_checked(&ev))
                                            />
                                            <span>"Enable Gradient"</span>
                                        </label>

                                        {move || if is_gradient.get() {
                                            view! {
                                                <div class="p-2.5 rounded bg-bg border border-edge space-y-2 text-xs">
                                                    <div class="flex items-center justify-between">
                                                        <span class="text-muted text-[11px]">"Secondary Color"</span>
                                                        <input
                                                            type="color"
                                                            class="w-5 h-5 rounded bg-transparent border border-edge cursor-pointer"
                                                            prop:value=move || gradient_end_color.get()
                                                            on:input=move |ev| set_gradient_end_color.set(event_target_value(&ev))
                                                        />
                                                    </div>
                                                    <div class="flex items-center gap-2">
                                                        <button
                                                            class=move || {
                                                                if !gradient_type_radial.get() {
                                                                    "px-2 py-0.5 rounded border border-purple bg-purple/20 text-purple text-[11px]"
                                                                } else {
                                                                    "px-2 py-0.5 rounded border border-edge text-muted text-[11px]"
                                                                }
                                                            }
                                                            on:click=move |_| set_gradient_type_radial.set(false)
                                                        >
                                                            "Linear"
                                                        </button>
                                                        <button
                                                            class=move || {
                                                                if gradient_type_radial.get() {
                                                                    "px-2 py-0.5 rounded border border-purple bg-purple/20 text-purple text-[11px]"
                                                                } else {
                                                                    "px-2 py-0.5 rounded border border-edge text-muted text-[11px]"
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
                                                                <div class="flex justify-between text-[11px] text-muted mb-1">
                                                                    <span>"Angle"</span>
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

                                    // Custom Eye Color Toggle
                                    <div class="space-y-2">
                                        <label class="flex items-center gap-2 text-xs text-muted hover:text-ink cursor-pointer select-none">
                                            <input
                                                type="checkbox"
                                                prop:checked=move || use_custom_eyes.get()
                                                on:change=move |ev| set_use_custom_eyes.set(event_target_checked(&ev))
                                            />
                                            <span>"Custom Eye Color"</span>
                                        </label>

                                        {move || if use_custom_eyes.get() {
                                            view! {
                                                <div class="flex items-center justify-between p-2 rounded bg-bg border border-edge">
                                                    <span class="text-[11px] text-muted">"Eye Color"</span>
                                                    <div class="flex items-center gap-2">
                                                        <input
                                                            type="color"
                                                            class="w-5 h-5 rounded bg-transparent border border-edge cursor-pointer"
                                                            prop:value=move || eye_color.get()
                                                            on:input=move |ev| set_eye_color.set(event_target_value(&ev))
                                                        />
                                                        <input
                                                            type="text"
                                                            class="w-20 bg-bg border border-edge rounded px-1.5 py-0.5 text-xs text-ink font-mono uppercase"
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

                                    // Background Color & Quiet Zone
                                    <div class="pt-2 border-t border-edge space-y-2">
                                        <div class="flex items-center justify-between">
                                            <label class="text-xs text-muted">"Background"</label>
                                            <div class="flex items-center gap-2">
                                                {move || if !is_transparent_bg.get() {
                                                    view! {
                                                        <input
                                                            type="color"
                                                            class="w-6 h-6 rounded bg-transparent border border-edge cursor-pointer"
                                                            prop:value=move || bg_color.get()
                                                            on:input=move |ev| set_bg_color.set(event_target_value(&ev))
                                                        />
                                                    }.into_any()
                                                } else {
                                                    view! {}.into_any()
                                                }}
                                                <label class="flex items-center gap-1.5 text-xs text-muted cursor-pointer select-none">
                                                    <input
                                                        type="checkbox"
                                                        prop:checked=move || is_transparent_bg.get()
                                                        on:change=move |ev| set_is_transparent_bg.set(event_target_checked(&ev))
                                                    />
                                                    <span>"Transparent"</span>
                                                </label>
                                            </div>
                                        </div>

                                        <div>
                                            <div class="flex justify-between text-[11px] text-muted mb-1">
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

                                    // CTA Banner Section
                                    <div class="pt-2 border-t border-edge space-y-2">
                                        <span class="text-xs text-muted">"CTA Text Badge"</span>
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
                                                                    "py-1 text-[11px] rounded border border-purple bg-purple/20 text-purple font-medium"
                                                                } else {
                                                                    "py-1 text-[11px] rounded border border-edge bg-bg text-muted hover:text-ink"
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
                                                    class="w-full bg-bg border border-edge rounded px-2.5 py-1 text-xs text-ink focus:border-purple focus:outline-none uppercase"
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
                                                        <label class="block text-[10px] text-muted mb-0.5">"Font Size"</label>
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
                                                        <label class="block text-[10px] text-muted mb-0.5">"Badge Color"</label>
                                                        <input
                                                            type="color"
                                                            class="w-6 h-6 rounded bg-transparent border border-edge cursor-pointer"
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

                                    // Center Logo Section
                                    <div class="pt-2 border-t border-edge space-y-2">
                                        <div class="flex items-center justify-between">
                                            <span class="text-xs text-muted">"Center Logo"</span>
                                            {move || if logo_data_url.get().is_some() {
                                                view! {
                                                    <button
                                                        class="text-[11px] text-rose-400 hover:underline"
                                                        on:click=move |_| {
                                                            set_logo_data_url.set(None);
                                                            set_logo_filename.set(None);
                                                        }
                                                    >
                                                        "Remove"
                                                    </button>
                                                }.into_any()
                                            } else {
                                                view! {}.into_any()
                                            }}
                                        </div>

                                        <input
                                            type="file"
                                            accept="image/*"
                                            class="w-full text-xs text-muted file:mr-2 file:py-1 file:px-2 file:rounded file:border-0 file:text-xs file:bg-surface-hover file:text-ink hover:file:bg-edge cursor-pointer"
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
                                                        <span class="text-muted text-[11px]">"Logo Mask"</span>
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
                                                                                    "px-2 py-0.5 rounded border border-purple bg-purple/20 text-purple text-[11px]"
                                                                                } else {
                                                                                    "px-2 py-0.5 rounded border border-edge text-muted text-[11px]"
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
                                                        <div class="flex justify-between text-[11px] text-muted mb-1">
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
                        }.into_any(),

                        AppMode::Halftone => view! {
                            <div class="bg-surface border border-edge rounded p-4 space-y-4">
                                <div class="border-b border-edge pb-2 flex items-center justify-between">
                                    <span class="text-xs font-semibold uppercase tracking-wider text-muted">"2. Photo Halftoning"</span>
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

                                // Photo Upload
                                <div>
                                    <input
                                        type="file"
                                        accept="image/*"
                                        class="w-full text-xs text-muted file:mr-2 file:py-1.5 file:px-3 file:rounded file:border-0 file:text-xs file:bg-surface-hover file:text-ink hover:file:bg-edge cursor-pointer"
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

                                // Mode Selection
                                <div>
                                    <label class="block text-muted text-[11px] mb-1.5">"Halftone Render Mode"</label>
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
                                                                "py-1.5 text-xs rounded border border-purple bg-purple/20 text-purple font-medium"
                                                            } else {
                                                                "py-1.5 text-xs rounded border border-edge bg-bg text-muted hover:text-ink"
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

                                // Color Mode / Sampled settings
                                {move || if halftone_mode.get() != HalftoneColorMode::Bw {
                                    view! {
                                        <div class="space-y-3 pt-2 border-t border-edge">
                                            <div class="flex items-center justify-between">
                                                <label class="text-xs text-muted">"Dark Core Fill"</label>
                                                <div class="flex items-center gap-2">
                                                    <input
                                                        type="color"
                                                        class="w-6 h-6 rounded bg-transparent border border-edge cursor-pointer"
                                                        prop:value=move || halftone_dark_color.get()
                                                        on:input=move |ev| set_halftone_dark_color.set(event_target_value(&ev))
                                                    />
                                                    <input
                                                        type="text"
                                                        class="w-20 bg-bg border border-edge rounded px-1.5 py-0.5 text-xs text-ink font-mono uppercase"
                                                        prop:value=move || halftone_dark_color.get()
                                                        on:input=move |ev| set_halftone_dark_color.set(event_target_value(&ev))
                                                    />
                                                </div>
                                            </div>
                                            <div>
                                                <div class="flex justify-between text-[11px] text-muted mb-1">
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
                                        <div class="space-y-3 pt-2 border-t border-edge">
                                            <div>
                                                <div class="flex justify-between text-[11px] text-muted mb-1">
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
                                                <label class="block text-muted text-[11px] mb-1.5">"Dither Kernel"</label>
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
                                                                            "py-1 text-[11px] rounded border border-purple bg-purple/20 text-purple font-medium"
                                                                        } else {
                                                                            "py-1 text-[11px] rounded border border-edge bg-bg text-muted hover:text-ink"
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

                                // Advanced Protection Checkboxes
                                <div class="space-y-1.5 pt-2 border-t border-edge text-xs">
                                    <label class="flex items-center gap-2 text-muted hover:text-ink cursor-pointer select-none">
                                        <input
                                            type="checkbox"
                                            prop:checked=move || preserve_structure.get()
                                            on:change=move |ev| set_preserve_structure.set(event_target_checked(&ev))
                                        />
                                        <span>"Preserve structure (finders + timing + alignment)"</span>
                                    </label>
                                    <label class="flex items-center gap-2 text-muted hover:text-ink cursor-pointer select-none">
                                        <input
                                            type="checkbox"
                                            prop:checked=move || clahe_enabled.get()
                                            on:change=move |ev| set_clahe_enabled.set(event_target_checked(&ev))
                                        />
                                        <span>"CLAHE adaptive histogram equalization"</span>
                                    </label>
                                </div>
                            </div>
                        }.into_any(),
                    }}
                </div>

                // Right Viewport / Preview Column
                <div class="lg:col-span-6 xl:col-span-7 space-y-4">
                    // Verification & Status Bar
                    {move || {
                        if app_mode.get() == AppMode::Vector {
                            let ratio = vector_contrast_ratio.get();
                            let is_ok = ratio >= 3.5;
                            view! {
                                <div class="bg-surface border border-edge rounded px-4 py-3 flex items-center justify-between text-xs">
                                    <div class="flex items-center gap-2.5">
                                        <span class=if is_ok { "w-2.5 h-2.5 rounded-full bg-emerald-500 shrink-0" } else { "w-2.5 h-2.5 rounded-full bg-amber-500 shrink-0" }></span>
                                        <span class="font-medium text-ink">
                                            {if is_ok { "Ready to scan (high contrast)" } else { "Low contrast warning" }}
                                        </span>
                                    </div>
                                    <span class="font-mono text-muted">{format!("Contrast {ratio:.1}:1")}</span>
                                </div>
                            }.into_any()
                        } else {
                            // Halftone verification report
                            match halftone_result_memo.get() {
                                Some((_, _, report)) => {
                                    let (dot_class, text) = match report.status {
                                        ScanStatus::Verified => ("bg-emerald-500", "Verified scannable (100% match)"),
                                        ScanStatus::Uncertain { .. } => ("bg-amber-500", "Scan uncertain"),
                                        ScanStatus::Unscannable => ("bg-rose-500", "Unscannable — increase strength or core size"),
                                    };
                                    view! {
                                        <div class="bg-surface border border-edge rounded px-4 py-3 flex items-center justify-between text-xs">
                                            <div class="flex items-center gap-2.5">
                                                <span class=format!("w-2.5 h-2.5 rounded-full {dot_class} shrink-0")></span>
                                                <span class="font-medium text-ink">{text}</span>
                                            </div>
                                            <span class="font-mono text-muted">{format!("Contrast {:.1}:1", report.contrast_ratio)}</span>
                                        </div>
                                    }.into_any()
                                }
                                None => view! {
                                    <div class="bg-surface border border-edge rounded px-4 py-3 flex items-center gap-2 text-xs text-muted">
                                        <span class="w-2.5 h-2.5 rounded-full bg-zinc-600 shrink-0"></span>
                                        <span>"Upload a photo to render halftone QR"</span>
                                    </div>
                                }.into_any()
                            }
                        }
                    }}

                    // Preview Card
                    <div class="bg-surface border border-edge rounded p-6 flex flex-col items-center justify-center min-h-[460px] relative">
                        {move || {
                            if app_mode.get() == AppMode::Vector {
                                match vector_result_memo.get() {
                                    Ok((matrix, svg)) => {
                                        view! {
                                            <div class="w-full flex flex-col items-center">
                                                <div
                                                    class="max-w-[400px] w-full p-3 rounded bg-surface border border-edge shadow-md"
                                                    inner_html=svg
                                                ></div>
                                                <div class="mt-4 font-mono text-[11px] text-muted">
                                                    {format!("QR v{} · {}×{} modules", matrix.version, matrix.size, matrix.size)}
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
                                            <div class="w-full flex flex-col items-center">
                                                <div
                                                    class="max-w-[400px] w-full p-3 rounded bg-surface border border-edge shadow-md"
                                                    inner_html=svg
                                                ></div>
                                                <div class="mt-4 font-mono text-[11px] text-muted">
                                                    {format!("Halftone v{} · {}×{} modules · 5×5 sub-grid", matrix.version, matrix.size, matrix.size)}
                                                </div>
                                            </div>
                                        }.into_any()
                                    }
                                    None => view! {
                                        <div class="text-center py-12 text-muted">
                                            <span class="text-3xl block mb-2 opacity-40">"🖼️"</span>
                                            <span class="text-xs font-mono">"Upload a photo to preview halftoning"</span>
                                        </div>
                                    }.into_any()
                                }
                            }
                        }}

                        // Toast copy message
                        {move || copied_msg.get().map(|msg| {
                            view! {
                                <div class="absolute bottom-4 px-3 py-1.5 rounded bg-emerald-950 border border-emerald-500/40 text-emerald-300 text-xs font-mono shadow-lg">
                                    {msg}
                                </div>
                            }
                        })}
                    </div>

                    // Actions & Export Bar
                    <div class="bg-surface border border-edge rounded p-4 space-y-3">
                        <div class="flex items-center justify-between text-xs pb-2 border-b border-edge">
                            <span class="text-muted">"PNG Resolution"</span>
                            <div class="flex gap-1">
                                {
                                    let resolutions = [512u32, 1024u32, 2048u32, 4096u32];
                                    resolutions.into_iter().map(|res| {
                                        view! {
                                            <button
                                                class=move || {
                                                    let is_act = target_res.get() == res;
                                                    if is_act {
                                                        "px-2 py-0.5 rounded bg-purple-subtle text-purple border border-purple/30 text-[11px] font-medium"
                                                    } else {
                                                        "px-2 py-0.5 rounded text-muted hover:text-ink text-[11px]"
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

                        <div class="flex flex-col sm:flex-row items-center gap-2">
                            <button
                                class="w-full sm:w-auto flex-1 bg-purple hover:bg-purple/90 text-ink text-xs font-semibold py-2.5 px-4 rounded transition-colors text-center"
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
                                class="w-full sm:w-auto border border-edge hover:border-purple text-ink text-xs font-medium py-2.5 px-4 rounded transition-colors text-center"
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
                                class="w-full sm:w-auto border border-edge hover:border-purple text-ink text-xs font-medium py-2.5 px-4 rounded transition-colors text-center"
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
        </div>
    }
}

fn main() {
    console_error_panic_hook::set_once();
    mount_to_body(|| view! { <App /> });
}
