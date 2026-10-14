use leptos::prelude::*;
use qr_core::matrix::{EccLevel, QrMatrix};
use qr_core::payload::{inspect_url_query, CryptoCurrency, Payload, WifiAuth};
use qr_core::vector::{
    EyeDotShape, EyeFrameShape, LogoConfig, LogoMaskShape, ModuleShape, VectorRenderConfig,
    VectorRenderer,
};
use qr_core::verifier::calculate_contrast_ratio;
use wasm_bindgen::JsCast;
use web_sys::{Blob, BlobPropertyBag, HtmlAnchorElement, HtmlCanvasElement, HtmlImageElement, Url};

pub mod components;
pub use components::glide_select::PayloadTab;
pub use components::payload_tabs::PayloadEditor;

#[wasm_bindgen::prelude::wasm_bindgen(inline_js = r#"
export function copyCanvasToClipboard(canvas) {
    return new Promise((resolve, reject) => {
        if (!navigator.clipboard || !window.ClipboardItem) {
            reject(new Error("Async Clipboard API not supported"));
            return;
        }
        canvas.toBlob((blob) => {
            if (!blob) {
                reject(new Error("Canvas toBlob failed"));
                return;
            }
            const item = new ClipboardItem({ "image/png": blob });
            navigator.clipboard.write([item]).then(resolve).catch(reject);
        }, "image/png");
    });
}
"#)]
extern "C" {
    fn copyCanvasToClipboard(canvas: &HtmlCanvasElement) -> js_sys::Promise;
}

#[component]
pub fn App() -> impl IntoView {
    // Payload form state
    let active_tab = RwSignal::new(PayloadTab::Url);
    let url_input = RwSignal::new(String::new());
    let text_input = RwSignal::new(String::new());
    let wifi_ssid = RwSignal::new(String::new());
    let wifi_password = RwSignal::new(String::new());
    let wifi_auth = RwSignal::new(WifiAuth::Wpa);
    let wifi_hidden = RwSignal::new(false);
    let vcard_first_name = RwSignal::new(String::new());
    let vcard_last_name = RwSignal::new(String::new());
    let vcard_phone = RwSignal::new(String::new());
    let vcard_email = RwSignal::new(String::new());
    let vcard_org = RwSignal::new(String::new());
    let vcard_title = RwSignal::new(String::new());
    let vcard_url = RwSignal::new(String::new());
    let vcard_note = RwSignal::new(String::new());
    let email_to = RwSignal::new(String::new());
    let email_subject = RwSignal::new(String::new());
    let email_body = RwSignal::new(String::new());
    let phone_number = RwSignal::new(String::new());
    let geo_lat = RwSignal::new(String::new());
    let geo_lng = RwSignal::new(String::new());
    let geo_query = RwSignal::new(String::new());
    let crypto_currency = RwSignal::new(CryptoCurrency::Bitcoin);
    let crypto_address = RwSignal::new(String::new());
    let crypto_amount = RwSignal::new(String::new());
    let crypto_message = RwSignal::new(String::new());

    // Styling parameters
    let (module_shape, set_module_shape) = signal(ModuleShape::Classy);
    let (eye_frame_shape, set_eye_frame_shape) = signal(EyeFrameShape::Rounded);
    let (eye_dot_shape, set_eye_dot_shape) = signal(EyeDotShape::Circle);

    // Palette defaults
    let (module_color, set_module_color) = signal("#000000".to_string());
    let (eye_color, set_eye_color) = signal("#000000".to_string());
    let (dot_color, set_dot_color) = signal("#008b99".to_string());
    let (is_transparent_bg, set_is_transparent_bg) = signal(false);
    let (bg_color, set_bg_color) = signal("#ffffff".to_string());

    // Optional Center Logo
    let (logo_data_url, set_logo_data_url) = signal(Option::<String>::None);

    // Export Options
    let (target_res, set_target_res) = signal(1024u32);
    let (notification_msg, set_notification_msg) = signal(Option::<String>::None);

    // Reactive Payload Construction
    let active_payload = Memo::new(move |_| match active_tab.get() {
        PayloadTab::Url => Payload::Url {
            url: url_input.get(),
            strip_tracking: false,
        },
        PayloadTab::Text => Payload::Text {
            text: text_input.get(),
        },
        PayloadTab::Wifi => Payload::Wifi {
            ssid: wifi_ssid.get(),
            password: wifi_password.get(),
            auth_type: wifi_auth.get(),
            hidden: wifi_hidden.get(),
        },
        PayloadTab::VCard => Payload::VCard {
            first_name: vcard_first_name.get(),
            last_name: vcard_last_name.get(),
            phone: vcard_phone.get(),
            email: vcard_email.get(),
            organization: vcard_org.get(),
            title: vcard_title.get(),
            url: vcard_url.get(),
            note: vcard_note.get(),
        },
        PayloadTab::Email => Payload::Email {
            to: email_to.get(),
            subject: email_subject.get(),
            body: email_body.get(),
        },
        PayloadTab::Phone => Payload::Phone {
            number: phone_number.get(),
        },
        PayloadTab::Geo => Payload::Geo {
            latitude: geo_lat.get().parse().unwrap_or(0.0),
            longitude: geo_lng.get().parse().unwrap_or(0.0),
            query: geo_query.get(),
        },
        PayloadTab::Crypto => Payload::Crypto {
            currency: crypto_currency.get(),
            address: crypto_address.get(),
            amount: crypto_amount.get(),
            message: crypto_message.get(),
            label: String::new(),
        },
    });

    let query_info = Memo::new(move |_| {
        if active_tab.get() == PayloadTab::Url {
            inspect_url_query(&url_input.get())
        } else {
            None
        }
    });

    // Decoupled QR Matrix Memo (only recalculates matrix math on payload or logo presence change)
    let qr_matrix_memo = Memo::new(move |_| {
        let payload = active_payload.get();
        let sanitized = payload.sanitize();
        let content = sanitized.content;
        let payload_str = if content.is_empty() {
            if active_tab.get() == PayloadTab::Url {
                "https://den1zz.dev".to_string()
            } else {
                String::new()
            }
        } else {
            content
        };

        let has_logo = logo_data_url.get().is_some();
        let ecc = if has_logo {
            EccLevel::H
        } else {
            EccLevel::Auto
        };

        QrMatrix::new(&payload_str, ecc, has_logo)
    });

    // Render SVG on styling or matrix updates
    let qr_result_memo = Memo::new(move |_| {
        let matrix = match qr_matrix_memo.get() {
            Ok(m) => m,
            Err(e) => return Err(e),
        };

        let logo = logo_data_url.get().map(|data_url| LogoConfig {
            data_url,
            shape: LogoMaskShape::Rounded,
            scale_percent: 22.0,
        });

        let bg = if is_transparent_bg.get() {
            None
        } else {
            Some(bg_color.get())
        };

        let config = VectorRenderConfig {
            scale: 20.0,
            margin_modules: 2,
            module_shape: module_shape.get(),
            eye_frame_shape: eye_frame_shape.get(),
            eye_dot_shape: eye_dot_shape.get(),
            code_color: module_color.get(),
            gradient: None,
            eye_color: Some(eye_color.get()),
            eye_dot_color: Some(dot_color.get()),
            bg_color: bg,
            logo,
            cta: None,
            alt_text: Some("QR Code".to_string()),
        };

        let svg = VectorRenderer::render_svg(&matrix, &config);
        Ok((matrix, svg))
    });

    // Contrast calculation for Module, Eye Frame, and Eye Dot against Background
    let contrast_ratios = Memo::new(move |_| {
        let mod_c = module_color.get();
        let eye_c = eye_color.get();
        let dot_c = dot_color.get();
        let bg = if is_transparent_bg.get() {
            "#FFFFFF"
        } else {
            &bg_color.get()
        };
        let r_mod = calculate_contrast_ratio(&mod_c, bg);
        let r_eye = calculate_contrast_ratio(&eye_c, bg);
        let r_dot = calculate_contrast_ratio(&dot_c, bg);
        (r_mod, r_eye, r_dot)
    });

    // Temporary notification helper
    let show_toast = move |msg: String| {
        set_notification_msg.set(Some(msg));
        set_timeout(
            move || {
                set_notification_msg.set(None);
            },
            std::time::Duration::from_millis(2500),
        );
    };

    // Client-side SVG to Clipboard
    let copy_svg_to_clipboard = move |svg_content: String| {
        if let Some(window) = web_sys::window() {
            let clipboard = window.navigator().clipboard();
            let succ_toast = show_toast;
            let fail_toast = show_toast;

            let promise = clipboard.write_text(&svg_content);
            let on_ok = wasm_bindgen::closure::Closure::wrap(Box::new(move |_| {
                succ_toast("COPIED SVG TO CLIPBOARD".to_string());
            })
                as Box<dyn FnMut(wasm_bindgen::JsValue)>);

            let on_err = wasm_bindgen::closure::Closure::wrap(Box::new(move |_| {
                fail_toast("CLIPBOARD PERMISSION DENIED".to_string());
            })
                as Box<dyn FnMut(wasm_bindgen::JsValue)>);

            let _ = promise.then2(&on_ok, &on_err);
            on_ok.forget();
            on_err.forget();
        }
    };

    // Client-side SVG download
    let download_svg = move |svg_content: String| {
        if let Some(window) = web_sys::window() {
            if let Some(doc) = window.document() {
                let blob_parts = js_sys::Array::new();
                blob_parts.push(&wasm_bindgen::JsValue::from_str(&svg_content));
                let bag = BlobPropertyBag::new();
                bag.set_type("image/svg+xml;charset=utf-8");
                if let Ok(blob) = Blob::new_with_str_sequence_and_options(&blob_parts, &bag) {
                    if let Ok(url) = Url::create_object_url_with_blob(&blob) {
                        if let Ok(a) = doc.create_element("a") {
                            let a = a.unchecked_into::<HtmlAnchorElement>();
                            a.set_href(&url);
                            a.set_download("qrcode.svg");
                            a.click();
                            let _ = Url::revoke_object_url(&url);
                        }
                    }
                }
            }
        }
    };

    // Client-side Canvas Rasterizer to PNG download
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
                        let blob_parts = js_sys::Array::new();
                        blob_parts.push(&wasm_bindgen::JsValue::from_str(&svg_content));
                        let bag = BlobPropertyBag::new();
                        bag.set_type("image/svg+xml;charset=utf-8");
                        if let Ok(blob) = Blob::new_with_str_sequence_and_options(&blob_parts, &bag)
                        {
                            if let Ok(url) = Url::create_object_url_with_blob(&blob) {
                                let img = HtmlImageElement::new().unwrap();
                                let img_clone = img.clone();
                                let url_clone = url.clone();
                                let canvas_clone = canvas.clone();

                                let onload =
                                    wasm_bindgen::closure::Closure::wrap(Box::new(move || {
                                        let _ = ctx
                                            .draw_image_with_html_image_element_and_dw_and_dh(
                                                &img_clone, 0.0, 0.0, res as f64, res as f64,
                                            );
                                        let _ = Url::revoke_object_url(&url_clone);

                                        let data_url = canvas_clone
                                            .to_data_url_with_type("image/png")
                                            .unwrap();
                                        if let Some(doc_inner) =
                                            web_sys::window().and_then(|w| w.document())
                                        {
                                            if let Ok(a_inner) = doc_inner.create_element("a") {
                                                let a_inner =
                                                    a_inner.unchecked_into::<HtmlAnchorElement>();
                                                a_inner.set_href(&data_url);
                                                a_inner
                                                    .set_download(&format!("qrcode_{res}px.png"));
                                                a_inner.click();
                                            }
                                        }
                                    })
                                        as Box<dyn FnMut()>);

                                img.set_onload(Some(onload.as_ref().unchecked_ref()));
                                onload.forget();
                                img.set_src(&url);
                            }
                        }
                    }
                }
            }
        }
    };

    // Copy PNG to Clipboard
    let copy_png_to_clipboard = move |svg_content: String, res: u32| {
        #[cfg(target_arch = "wasm32")]
        {
            if let Some(window) = web_sys::window() {
                if let Some(doc) = window.document() {
                    if let Ok(canvas_el) = doc.create_element("canvas") {
                        let canvas = canvas_el.unchecked_into::<HtmlCanvasElement>();
                        canvas.set_width(res);
                        canvas.set_height(res);

                        let ctx =
                            canvas.get_context("2d").ok().flatten().and_then(|c| {
                                c.dyn_into::<web_sys::CanvasRenderingContext2d>().ok()
                            });

                        if let Some(ctx) = ctx {
                            let blob_parts = js_sys::Array::new();
                            blob_parts.push(&wasm_bindgen::JsValue::from_str(&svg_content));
                            let bag = BlobPropertyBag::new();
                            bag.set_type("image/svg+xml;charset=utf-8");
                            if let Ok(blob) =
                                Blob::new_with_str_sequence_and_options(&blob_parts, &bag)
                            {
                                if let Ok(url) = Url::create_object_url_with_blob(&blob) {
                                    let img = HtmlImageElement::new().unwrap();
                                    let img_clone = img.clone();
                                    let url_clone = url.clone();
                                    let canvas_clone = canvas.clone();
                                    let show_toast_succ = show_toast;
                                    let show_toast_fail = show_toast;

                                    let onload =
                                        wasm_bindgen::closure::Closure::wrap(Box::new(move || {
                                            let _ = ctx
                                                .draw_image_with_html_image_element_and_dw_and_dh(
                                                    &img_clone, 0.0, 0.0, res as f64, res as f64,
                                                );
                                            let _ = Url::revoke_object_url(&url_clone);

                                            let promise = copyCanvasToClipboard(&canvas_clone);
                                            let succ_toast = show_toast_succ;
                                            let fail_toast = show_toast_fail;

                                            let on_ok = wasm_bindgen::closure::Closure::wrap(
                                                Box::new(move |_| {
                                                    succ_toast(
                                                        "COPIED PNG TO CLIPBOARD".to_string(),
                                                    );
                                                })
                                                    as Box<dyn FnMut(wasm_bindgen::JsValue)>,
                                            );

                                            let on_err = wasm_bindgen::closure::Closure::wrap(
                                                Box::new(move |_| {
                                                    fail_toast(
                                                        "CLIPBOARD PERMISSION DENIED".to_string(),
                                                    );
                                                })
                                                    as Box<dyn FnMut(wasm_bindgen::JsValue)>,
                                            );

                                            let _ = promise.then2(&on_ok, &on_err);
                                            on_ok.forget();
                                            on_err.forget();
                                        })
                                            as Box<dyn FnMut()>);

                                    img.set_onload(Some(onload.as_ref().unchecked_ref()));
                                    onload.forget();
                                    img.set_src(&url);
                                }
                            }
                        }
                    }
                }
            }
        }
        #[cfg(not(target_arch = "wasm32"))]
        {
            let _ = (svg_content, res);
        }
    };

    view! {
        <div class="min-h-screen bg-[#000000] text-[#ffffff] pb-16 selection:bg-[#00f0ff]/25 selection:text-[#00f0ff]">
            // Main container
            <main class="w-full max-w-[680px] mx-auto px-4 pt-8 sm:pt-12 space-y-5">
                // Header
                <div class="text-center space-y-2">
                    <h1 class="text-3xl sm:text-4xl font-bold tracking-tight">
                        <span class="text-[#00f0ff]">"qr."</span>
                        <span class="text-[#ffffff]">"den1zz.dev"</span>
                    </h1>
                    <p class="text-xs sm:text-sm text-[#a0a0a0] max-w-md mx-auto leading-relaxed">
                        "Local, client-side QR studio built in pure Rust & WebAssembly."
                    </p>
                </div>

                // Payload configuration
                <PayloadEditor
                    active_payload=active_payload
                    active_tab=active_tab
                    url_input=url_input
                    text_input=text_input
                    wifi_ssid=wifi_ssid
                    wifi_password=wifi_password
                    wifi_auth=wifi_auth
                    wifi_hidden=wifi_hidden
                    vcard_first_name=vcard_first_name
                    vcard_last_name=vcard_last_name
                    vcard_phone=vcard_phone
                    vcard_email=vcard_email
                    vcard_org=vcard_org
                    vcard_title=vcard_title
                    vcard_url=vcard_url
                    vcard_note=vcard_note
                    email_to=email_to
                    email_subject=email_subject
                    email_body=email_body
                    phone_number=phone_number
                    geo_lat=geo_lat
                    geo_lng=geo_lng
                    geo_query=geo_query
                    crypto_currency=crypto_currency
                    crypto_address=crypto_address
                    crypto_amount=crypto_amount
                    crypto_message=crypto_message
                    query_info=query_info
                />

                // QR preview and export
                <div class="glass-panel p-4 sm:p-6 flex flex-col items-center justify-center relative space-y-4 sm:space-y-5">
                    {move || match qr_result_memo.get() {
                        Ok((matrix, svg)) => {
                            let (mod_ratio, eye_ratio, dot_ratio) = contrast_ratios.get();
                            let is_mod_ok = mod_ratio >= 3.0;
                            let is_eye_ok = eye_ratio >= 3.0;
                            let is_dot_ok = dot_ratio >= 3.0;
                            let all_ok = is_mod_ok && is_eye_ok && is_dot_ok;

                            let svg_for_png = svg.clone();
                            let svg_for_copy_png = svg.clone();
                            let svg_for_download = svg.clone();
                            let svg_for_copy = svg.clone();

                            view! {
                                <div class="w-full flex flex-col items-center justify-center space-y-4">
                                    // Scannability status pill
                                    <div class="flex items-center justify-center gap-1 sm:gap-2 text-[10px] sm:text-xs px-2.5 sm:px-3 py-1 rounded-full bg-[#050505] border border-[rgba(255,255,255,0.12)] whitespace-nowrap max-w-full overflow-x-auto [scrollbar-width:none] [&::-webkit-scrollbar]:hidden">
                                        <span class=if all_ok {
                                            "w-2 h-2 shrink-0 rounded-full bg-emerald-400 shadow-[0_0_8px_#34d399]"
                                        } else {
                                            "w-2 h-2 shrink-0 rounded-full bg-amber-400 shadow-[0_0_8px_#fbbf24]"
                                        }></span>
                                        <span class="text-[#ffffff] shrink-0 whitespace-nowrap">
                                            {if all_ok { "Ready to Scan" } else { "Low Contrast" }}
                                        </span>
                                        <span class="hidden sm:inline text-[#555555] shrink-0">"•"</span>
                                        <span class="hidden sm:inline text-[#ffffff] shrink-0" title="Module Contrast">{format!("M:{mod_ratio:.1}:1")}</span>
                                        <span class="hidden sm:inline text-[#555555] shrink-0">"•"</span>
                                        <span class="hidden sm:inline text-[#ffffff] shrink-0" title="Eye Frame Contrast">{format!("E:{eye_ratio:.1}:1")}</span>
                                        <span class="hidden sm:inline text-[#555555] shrink-0">"•"</span>
                                        <span class="hidden sm:inline text-[#ffffff] shrink-0" title="Eye Dot Contrast">{format!("D:{dot_ratio:.1}:1")}</span>
                                        <span class="hidden sm:inline text-[#555555] shrink-0">"•"</span>
                                        <span class="hidden sm:inline text-[#a0a0a0] font-mono shrink-0">{format!("v{} ({}×{})", matrix.version, matrix.size, matrix.size)}</span>
                                    </div>

                                    // Viewfinder Frame
                                    <div class="viewfinder-frame">
                                        <div class="qr-svg-wrapper" inner_html=svg></div>
                                    </div>

                                    // Export resolution and format controls
                                    <div class="w-full max-w-[480px] space-y-3 pt-2 border-t border-[rgba(255,255,255,0.08)]">
                                        <div class="flex items-center justify-between text-xs">
                                            <span class="text-[#a0a0a0] font-semibold uppercase text-[11px]">"Resolution"</span>
                                            <div class="flex gap-1">
                                                {[512u32, 1024, 2048].into_iter().map(|res| {
                                                    let is_active = move || target_res.get() == res;
                                                    view! {
                                                        <button
                                                            class=move || {
                                                                if is_active() {
                                                                    "px-2.5 py-0.5 rounded text-[11px] font-bold bg-[#00f0ff] text-[#000000]"
                                                                } else {
                                                                    "px-2.5 py-0.5 rounded text-[11px] text-[#a0a0a0] hover:text-[#ffffff] hover:bg-[#151515]"
                                                                }
                                                            }
                                                            on:click=move |_| set_target_res.set(res)
                                                        >
                                                            {format!("{res}px")}
                                                        </button>
                                                    }
                                                }).collect_view()}
                                            </div>
                                        </div>

                                        <div class="grid grid-cols-2 sm:grid-cols-4 gap-2">
                                            <button
                                                class="button button-primary w-full py-2 text-center text-xs font-bold"
                                                on:click=move |_| {
                                                    let res = target_res.get();
                                                    download_png(svg_for_png.clone(), res);
                                                }
                                            >
                                                {move || format!("↓ PNG ({}px)", target_res.get())}
                                            </button>

                                            <button
                                                class="button button-white w-full py-2 text-center text-xs font-semibold"
                                                on:click=move |_| {
                                                    let res = target_res.get();
                                                    copy_png_to_clipboard(svg_for_copy_png.clone(), res);
                                                }
                                            >
                                                "Copy PNG"
                                            </button>

                                            <button
                                                class="button button-secondary w-full py-2 px-3 text-xs font-semibold text-center"
                                                on:click=move |_| download_svg(svg_for_download.clone())
                                            >
                                                "↓ SVG"
                                            </button>

                                            <button
                                                class="button button-secondary w-full py-2 px-3 text-xs font-semibold text-center"
                                                on:click=move |_| copy_svg_to_clipboard(svg_for_copy.clone())
                                            >
                                                "Copy SVG"
                                            </button>
                                        </div>
                                    </div>
                                </div>
                            }.into_any()
                        }
                        Err(e) => view! {
                            <div class="p-6 text-center text-rose-400 text-sm">
                                {format!("Generation error: {e}")}
                            </div>
                        }.into_any()
                    }}

                    // Toast Notification
                    {move || notification_msg.get().map(|msg| {
                        view! {
                            <div class="absolute bottom-4 px-4 py-1.5 rounded-full bg-[#050505] border border-[#00f0ff] text-[#00f0ff] text-xs font-bold shadow-[0_0_15px_rgba(0,240,255,0.35)] z-20">
                                {msg}
                            </div>
                        }
                    })}
                </div>

                // Shapes
                <div class="glass-panel p-4 sm:p-5 space-y-4">
                    <h3 class="text-xs font-semibold uppercase tracking-wider text-[#a0a0a0]">
                        "Shapes & Style"
                    </h3>
                    <div class="grid grid-cols-1 sm:grid-cols-3 gap-3">
                        // Module Shape
                        <div class="space-y-1.5">
                            <span class="text-[11px] text-[#a0a0a0]">"Module Shape"</span>
                            <div class="flex flex-col gap-1">
                                {[
                                    (ModuleShape::Classy, "Classy"),
                                    (ModuleShape::Smooth, "Smooth"),
                                    (ModuleShape::Dots, "Dots"),
                                    (ModuleShape::Square, "Square"),
                                ].into_iter().map(|(shape, label)| {
                                    let is_sel = move || module_shape.get() == shape;
                                    view! {
                                        <button
                                            class=move || {
                                                if is_sel() {
                                                    "w-full py-1.5 px-3 rounded-lg text-xs font-bold bg-[#00f0ff] text-[#000000] border border-[#00f0ff] text-left transition-all"
                                                } else {
                                                    "w-full py-1.5 px-3 rounded-lg text-xs font-medium text-[#a0a0a0] bg-[#111111] border border-[rgba(255,255,255,0.08)] hover:text-[#ffffff] text-left transition-all"
                                                }
                                            }
                                            on:click=move |_| set_module_shape.set(shape)
                                        >
                                            {label}
                                        </button>
                                    }
                                }).collect_view()}
                            </div>
                        </div>

                        // Eye Frame Shape
                        <div class="space-y-1.5">
                            <span class="text-[11px] text-[#a0a0a0]">"Eye Frame Shape"</span>
                            <div class="flex flex-col gap-1">
                                {[
                                    (EyeFrameShape::Rounded, "Round Frame"),
                                    (EyeFrameShape::Circle, "Circle Frame"),
                                    (EyeFrameShape::Square, "Square Frame"),
                                ].into_iter().map(|(shape, label)| {
                                    let is_sel = move || eye_frame_shape.get() == shape;
                                    view! {
                                        <button
                                            class=move || {
                                                if is_sel() {
                                                    "w-full py-1.5 px-3 rounded-lg text-xs font-bold bg-[#00f0ff] text-[#000000] border border-[#00f0ff] text-left transition-all"
                                                } else {
                                                    "w-full py-1.5 px-3 rounded-lg text-xs font-medium text-[#a0a0a0] bg-[#111111] border border-[rgba(255,255,255,0.08)] hover:text-[#ffffff] text-left transition-all"
                                                }
                                            }
                                            on:click=move |_| set_eye_frame_shape.set(shape)
                                        >
                                            {label}
                                        </button>
                                    }
                                }).collect_view()}
                            </div>
                        </div>

                        // Eye Dot Shape
                        <div class="space-y-1.5">
                            <span class="text-[11px] text-[#a0a0a0]">"Eye Dot Shape"</span>
                            <div class="flex flex-col gap-1">
                                {[
                                    (EyeDotShape::Circle, "Circle Dot"),
                                    (EyeDotShape::Rounded, "Round Dot"),
                                    (EyeDotShape::Square, "Square Dot"),
                                ].into_iter().map(|(shape, label)| {
                                    let is_sel = move || eye_dot_shape.get() == shape;
                                    view! {
                                        <button
                                            class=move || {
                                                if is_sel() {
                                                    "w-full py-1.5 px-3 rounded-lg text-xs font-bold bg-[#00f0ff] text-[#000000] border border-[#00f0ff] text-left transition-all"
                                                } else {
                                                    "w-full py-1.5 px-3 rounded-lg text-xs font-medium text-[#a0a0a0] bg-[#111111] border border-[rgba(255,255,255,0.08)] hover:text-[#ffffff] text-left transition-all"
                                                }
                                            }
                                            on:click=move |_| set_eye_dot_shape.set(shape)
                                        >
                                            {label}
                                        </button>
                                    }
                                }).collect_view()}
                            </div>
                        </div>
                    </div>
                </div>

                // Colors
                <div class="glass-panel p-4 sm:p-5 space-y-4">
                    <h3 class="text-xs font-semibold uppercase tracking-wider text-[#a0a0a0]">
                        "Colors & Palette"
                    </h3>
                    <div class="grid grid-cols-1 sm:grid-cols-2 gap-4">
                        // Module Color
                        <div class="space-y-1.5">
                            <div class="flex items-center justify-between text-xs">
                                <span class="text-[#a0a0a0]">"Code / Modules"</span>
                                <span class="font-mono text-[11px] text-[#555555]">{move || module_color.get()}</span>
                            </div>
                            <div class="flex items-center gap-2">
                                <input
                                    type="color"
                                    class="w-8 h-8 rounded cursor-pointer bg-transparent border-0"
                                    prop:value=move || module_color.get()
                                    on:input=move |ev| set_module_color.set(event_target_value(&ev))
                                />
                                <div class="flex gap-1.5 flex-wrap">
                                    {["#000000", "#008b99", "#00f0ff", "#ffffff"].into_iter().map(|preset| {
                                        view! {
                                            <button
                                                class="w-6 h-6 rounded-full border border-[rgba(255,255,255,0.2)] transition-transform hover:scale-110"
                                                style=format!("background-color: {preset}")
                                                title=format!("Set color to {preset}")
                                                on:click=move |_| set_module_color.set(preset.to_string())
                                            />
                                        }
                                    }).collect_view()}
                                </div>
                            </div>
                        </div>

                        // Eye Frame Color
                        <div class="space-y-1.5">
                            <div class="flex items-center justify-between text-xs">
                                <span class="text-[#a0a0a0]">"Eye Frame"</span>
                                <button
                                    class="text-[11px] text-[#00f0ff] hover:underline"
                                    on:click=move |_| set_eye_color.set(module_color.get())
                                >
                                    "Match Code"
                                </button>
                            </div>
                            <div class="flex items-center gap-2">
                                <input
                                    type="color"
                                    class="w-8 h-8 rounded cursor-pointer bg-transparent border-0"
                                    prop:value=move || eye_color.get()
                                    on:input=move |ev| set_eye_color.set(event_target_value(&ev))
                                />
                                <div class="flex gap-1.5 flex-wrap">
                                    {["#000000", "#008b99", "#00f0ff", "#ffffff"].into_iter().map(|preset| {
                                        view! {
                                            <button
                                                class="w-6 h-6 rounded-full border border-[rgba(255,255,255,0.2)] transition-transform hover:scale-110"
                                                style=format!("background-color: {preset}")
                                                title=format!("Set color to {preset}")
                                                on:click=move |_| set_eye_color.set(preset.to_string())
                                            />
                                        }
                                    }).collect_view()}
                                </div>
                            </div>
                        </div>

                        // Eye Dot Color
                        <div class="space-y-1.5">
                            <div class="flex items-center justify-between text-xs">
                                <span class="text-[#a0a0a0]">"Eye Dot"</span>
                                <button
                                    class="text-[11px] text-[#00f0ff] hover:underline"
                                    on:click=move |_| set_dot_color.set(eye_color.get())
                                >
                                    "Match Frame"
                                </button>
                            </div>
                            <div class="flex items-center gap-2">
                                <input
                                    type="color"
                                    class="w-8 h-8 rounded cursor-pointer bg-transparent border-0"
                                    prop:value=move || dot_color.get()
                                    on:input=move |ev| set_dot_color.set(event_target_value(&ev))
                                />
                                <div class="flex gap-1.5 flex-wrap">
                                    {["#008b99", "#00f0ff", "#000000", "#ffffff"].into_iter().map(|preset| {
                                        view! {
                                            <button
                                                class="w-6 h-6 rounded-full border border-[rgba(255,255,255,0.2)] transition-transform hover:scale-110"
                                                style=format!("background-color: {preset}")
                                                title=format!("Set color to {preset}")
                                                on:click=move |_| set_dot_color.set(preset.to_string())
                                            />
                                        }
                                    }).collect_view()}
                                </div>
                            </div>
                        </div>

                        // Background Color
                        <div class="space-y-1.5">
                            <div class="flex items-center justify-between text-xs">
                                <span class="text-[#a0a0a0]">"Background"</span>
                                <label class="flex items-center gap-1.5 cursor-pointer text-[11px] text-[#00f0ff]">
                                    <input
                                        type="checkbox"
                                        class="rounded accent-[#00f0ff] prec"
                                        prop:checked=move || is_transparent_bg.get()
                                        on:change=move |ev| set_is_transparent_bg.set(event_target_checked(&ev))
                                    />
                                    <span>"Transparent"</span>
                                </label>
                            </div>
                            <div class="flex items-center gap-2">
                                <input
                                    type="color"
                                    class="w-8 h-8 rounded cursor-pointer bg-transparent border-0 disabled:opacity-40"
                                    disabled=move || is_transparent_bg.get()
                                    prop:value=move || bg_color.get()
                                    on:input=move |ev| set_bg_color.set(event_target_value(&ev))
                                />
                                <div class="flex gap-1.5 flex-wrap">
                                    {["#ffffff", "#000000", "#f8fafc", "#0a0a0a"].into_iter().map(|preset| {
                                        view! {
                                            <button
                                                class="w-6 h-6 rounded-full border border-[rgba(255,255,255,0.2)] transition-transform hover:scale-110 disabled:opacity-30"
                                                disabled=move || is_transparent_bg.get()
                                                style=format!("background-color: {preset}")
                                                title=format!("Set color to {preset}")
                                                on:click=move |_| set_bg_color.set(preset.to_string())
                                            />
                                        }
                                    }).collect_view()}
                                </div>
                            </div>
                        </div>
                    </div>
                </div>

                // Center logo
                <div class="glass-panel p-4 sm:p-5 space-y-3">
                    <div class="flex items-center justify-between">
                        <h3 class="text-xs font-semibold uppercase tracking-wider text-[#a0a0a0]">
                            "Center Logo (Optional)"
                        </h3>
                        {move || if logo_data_url.get().is_some() {
                            view! {
                                <button
                                    class="text-xs text-rose-400 hover:text-rose-300"
                                    on:click=move |_| set_logo_data_url.set(None)
                                >
                                    "Remove Logo"
                                </button>
                            }.into_any()
                        } else {
                            view! { <span /> }.into_any()
                        }}
                    </div>
                    <div class="flex items-center gap-3">
                        <label class="button button-secondary text-xs cursor-pointer py-1.5 px-3">
                            <span>"Upload PNG/SVG"</span>
                            <input
                                type="file"
                                accept="image/png,image/jpeg,image/svg+xml"
                                class="hidden"
                                on:change=move |ev| {
                                    if let Some(target) = ev.target() {
                                        let input = target.unchecked_into::<web_sys::HtmlInputElement>();
                                        if let Some(files) = input.files() {
                                            if let Some(file) = files.get(0) {
                                                let reader = web_sys::FileReader::new().unwrap();
                                                let reader_clone = reader.clone();
                                                let onload = wasm_bindgen::closure::Closure::wrap(Box::new(move |_e: web_sys::ProgressEvent| {
                                                    if let Ok(res) = reader_clone.result() {
                                                        if let Some(s) = res.as_string() {
                                                            set_logo_data_url.set(Some(s));
                                                        }
                                                    }
                                                }) as Box<dyn FnMut(web_sys::ProgressEvent)>);
                                                reader.set_onload(Some(onload.as_ref().unchecked_ref()));
                                                onload.forget();
                                                let _ = reader.read_as_data_url(&file);
                                            }
                                        }
                                    }
                                }
                            />
                        </label>
                        <span class="text-xs text-[#555555]">
                            {move || if logo_data_url.get().is_some() {
                                "Logo loaded (Ecc auto-boosted to Level H)"
                            } else {
                                "No logo selected"
                            }}
                        </span>
                    </div>
                </div>

            <footer class="w-full max-w-[680px] mx-auto px-4 pt-4 pb-12 text-center text-xs text-[#a0a0a0] space-y-2">
                <p>
                    "Open source software released under the MIT License."
                </p>
                <p>
                    <a
                        href="https://github.com/Den1zzDev/qr.den1zz.dev"
                        target="_blank"
                        rel="noopener noreferrer"
                        class="text-[#00f0ff] hover:underline"
                    >
                        "GitHub repository"
                    </a>
                </p>
            </footer>
            </main>
        </div>
    }
}

#[cfg(feature = "hydrate")]
#[wasm_bindgen::prelude::wasm_bindgen(start)]
pub fn start_hydrate() {
    console_error_panic_hook::set_once();
    leptos::mount::hydrate_body(App);
}

#[cfg(feature = "csr")]
#[wasm_bindgen::prelude::wasm_bindgen(start)]
pub fn start_csr() {
    console_error_panic_hook::set_once();
    leptos::mount::mount_to_body(App);
}
