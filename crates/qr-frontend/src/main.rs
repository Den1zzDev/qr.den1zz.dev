use leptos::prelude::*;
use qr_core::{
    matrix::{EccLevel, QrMatrix},
    payload::{inspect_url_query, strip_all_query, strip_tracking_query},
    vector::{
        EyeDotShape, EyeFrameShape, LogoConfig, LogoMaskShape, ModuleShape, VectorRenderConfig,
        VectorRenderer,
    },
    verifier::calculate_contrast_ratio,
};
use wasm_bindgen::JsCast;
use web_sys::{Blob, BlobPropertyBag, HtmlAnchorElement, HtmlCanvasElement, HtmlImageElement, Url};

#[component]
pub fn App() -> impl IntoView {
    // Primary Input State
    let (input_text, set_input_text) = signal("https://den1zz.dev".to_string());

    // Essential Styling State (Ente Classy as default)
    let (module_shape, set_module_shape) = signal(ModuleShape::Classy);
    let (eye_frame_shape, set_eye_frame_shape) = signal(EyeFrameShape::Rounded);
    let (eye_dot_shape, set_eye_dot_shape) = signal(EyeDotShape::Circle);

    // Sensible Default Colors: AMOLED Black background, White modules & eye frames, Cyan eye dots
    let (module_color, set_module_color) = signal("#ffffff".to_string());
    let (eye_color, set_eye_color) = signal("#ffffff".to_string());
    let (dot_color, set_dot_color) = signal("#00f0ff".to_string());
    let (is_transparent_bg, set_is_transparent_bg) = signal(false);
    let (bg_color, set_bg_color) = signal("#000000".to_string());

    // Optional Center Logo
    let (logo_data_url, set_logo_data_url) = signal(Option::<String>::None);

    // Export Options
    let (target_res, set_target_res) = signal(1024u32);
    let (notification_msg, set_notification_msg) = signal(Option::<String>::None);

    // Dynamic Query Parameter Inspector (ha.mr & tools.ralite.dev philosophy)
    let query_info = Memo::new(move |_| {
        let text = input_text.get();
        inspect_url_query(&text)
    });

    // Dynamic QR Matrix and SVG Memo
    let qr_result_memo = Memo::new(move |_| {
        let content = input_text.get().trim().to_string();
        let payload_str = if content.is_empty() {
            "https://qr.den1zz.dev".to_string()
        } else {
            content
        };

        let has_logo = logo_data_url.get().is_some();
        let ecc = if has_logo {
            EccLevel::H
        } else {
            EccLevel::Auto
        };

        let matrix = match QrMatrix::new(&payload_str, ecc, has_logo) {
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
        wasm_bindgen_futures::spawn_local(async move {
            gloo_timers::future::TimeoutFuture::new(2200).await;
            set_notification_msg.set(None);
        });
    };

    // Clipboard copy helper
    let copy_text_to_clipboard = move |text: String, label: &'static str| {
        if let Some(window) = web_sys::window() {
            let nav = window.navigator();
            let cb = nav.clipboard();
            let _ = cb.write_text(&text);
            show_toast(format!("COPIED {label}"));
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

    let color_presets = ["#ffffff", "#00f0ff", "#38ef7d", "#ff3b80", "#f59e0b", "#a855f7"];

    view! {
        <div class="min-h-screen bg-[#000000] text-[#ffffff] pb-16 selection:bg-[#00f0ff]/25 selection:text-[#00f0ff]">
            // Main Focused Container (KISS Philosophy)
            <main class="w-full max-w-[680px] mx-auto px-4 pt-8 sm:pt-12 space-y-5">
                // Clean Header
                <div class="text-center space-y-2">
                    <h1 class="text-3xl sm:text-4xl font-bold tracking-tight">
                        <span class="text-[#00f0ff]">"qr."</span>
                        <span class="text-[#ffffff]">"den1zz.dev"</span>
                    </h1>
                    <p class="text-xs sm:text-sm text-[#a0a0a0] max-w-md mx-auto leading-relaxed">
                        "Local, client-side QR studio built in pure Rust & WebAssembly. URL query cleaner, Ente classy modules, and zero network calls."
                    </p>
                </div>

                // 1. Primary Input Box
                <div class="glass-panel p-4 sm:p-5 space-y-3">
                    <div class="relative flex items-center">
                        <input
                            type="text"
                            class="w-full text-sm sm:text-base py-3 pl-4 pr-10 rounded-xl bg-[#050505] border border-[rgba(255,255,255,0.12)] focus:border-[#00f0ff] text-[#ffffff] placeholder:text-[#555555] outline-none transition-all shadow-inner"
                            placeholder="Paste URL or enter text..."
                            prop:value=move || input_text.get()
                            on:input=move |ev| set_input_text.set(event_target_value(&ev))
                        />
                        {move || if !input_text.get().is_empty() {
                            view! {
                                <button
                                    class="absolute right-3 text-[#a0a0a0] hover:text-[#ffffff] p-1 text-sm"
                                    title="Clear input"
                                    on:click=move |_| set_input_text.set(String::new())
                                >
                                    "✕"
                                </button>
                            }.into_any()
                        } else {
                            view! {}.into_any()
                        }}
                    </div>

                    // Query Cleaner Prompt (ha.mr & tools.ralite.dev style)
                    {move || query_info.get().map(|info| {
                        view! {
                            <div class="p-4 rounded-xl bg-[#080808] border border-[#00f0ff]/40 space-y-2.5 text-left transition-all">
                                <div class="flex items-center justify-between text-xs">
                                    <span class="font-bold text-[#00f0ff] flex items-center gap-1.5 tracking-wide">
                                        <span>"⚠️"</span>
                                        <span>"Consider removing parameters..."</span>
                                    </span>
                                    <span class="text-[11px] text-[#a0a0a0]">
                                        {format!("{} query parameters detected", info.total_param_count)}
                                    </span>
                                </div>
                                <p class="text-xs text-[#a0a0a0] leading-relaxed">
                                    "Removing tracking tokens or transient URL query parameters shortens the link, generating a simpler and faster-to-scan QR code."
                                </p>
                                <div class="flex flex-wrap items-center gap-2 pt-1">
                                    <button
                                        class="button button-primary text-xs py-1.5 px-3.5"
                                        on:click=move |_| {
                                            let cur = input_text.get();
                                            let cleaned = strip_all_query(&cur);
                                            let saved = cur.len().saturating_sub(cleaned.len());
                                            set_input_text.set(cleaned);
                                            show_toast(format!("Removed all query parameters (-{saved} chars)"));
                                        }
                                    >
                                        "Remove All Parameters (?)"
                                    </button>

                                    {if info.tracking_param_count > 0 {
                                        view! {
                                            <button
                                                class="button button-secondary text-xs py-1.5 px-3.5"
                                                on:click=move |_| {
                                                    let cur = input_text.get();
                                                    let (cleaned, count) = strip_tracking_query(&cur);
                                                    let saved = cur.len().saturating_sub(cleaned.len());
                                                    set_input_text.set(cleaned);
                                                    show_toast(format!("Stripped {count} tracking tags (-{saved} chars)"));
                                                }
                                            >
                                                {format!("Strip Tracking Only ({} tags)", info.tracking_param_count)}
                                            </button>
                                        }.into_any()
                                    } else {
                                        view! {}.into_any()
                                    }}
                                </div>
                            </div>
                        }
                    })}
                </div>

                // 2. Live AMOLED QR Viewport Stage with Integrated Download Controls
                <div class="glass-panel p-5 sm:p-6 flex flex-col items-center justify-center relative space-y-5">
                    {move || match qr_result_memo.get() {
                        Ok((matrix, svg)) => {
                            let (mod_ratio, eye_ratio, dot_ratio) = contrast_ratios.get();
                            let is_mod_ok = mod_ratio >= 3.0;
                            let is_eye_ok = eye_ratio >= 3.0;
                            let is_dot_ok = dot_ratio >= 3.0;
                            let all_ok = is_mod_ok && is_eye_ok && is_dot_ok;

                            view! {
                                <div class="w-full flex flex-col items-center justify-center space-y-4">
                                    // Scannability status pill
                                    <div class="flex items-center gap-2 text-xs px-3 py-1 rounded-full bg-[#050505] border border-[rgba(255,255,255,0.12)]">
                                        <span class=if all_ok {
                                            "w-2 h-2 rounded-full bg-emerald-400 shadow-[0_0_8px_#34d399]"
                                        } else {
                                            "w-2 h-2 rounded-full bg-amber-400 shadow-[0_0_8px_#fbbf24]"
                                        }></span>
                                        <span class="text-[#ffffff]">
                                            {if all_ok { "Ready to Scan" } else { "Low Contrast" }}
                                        </span>
                                        <span class="text-[#555555]">"•"</span>
                                        <span class="text-[#ffffff]" title="Module Contrast">{format!("M:{mod_ratio:.1}:1")}</span>
                                        <span class="text-[#555555]">"•"</span>
                                        <span class="text-[#ffffff]" title="Eye Frame Contrast">{format!("E:{eye_ratio:.1}:1")}</span>
                                        <span class="text-[#555555]">"•"</span>
                                        <span class="text-[#00f0ff]" title="Eye Dot Contrast">{format!("D:{dot_ratio:.1}:1")}</span>
                                        <span class="text-[#555555]">"•"</span>
                                        <span class="text-[#a0a0a0]">{format!("v{} ({}×{})", matrix.version, matrix.size, matrix.size)}</span>
                                    </div>

                                    // Viewfinder Frame (clean, no cyan corners)
                                    <div class="viewfinder-frame">
                                        <div
                                            class="qr-svg-wrapper"
                                            inner_html=svg
                                        ></div>
                                    </div>
                                </div>
                            }.into_any()
                        }
                        Err(e) => view! {
                            <div class="text-rose-400 text-xs py-8">{format!("Error: {e}")}</div>
                        }.into_any()
                    }}

                    // Download Bar positioned directly below the QR code
                    <div class="w-full max-w-[420px] space-y-3 pt-2 border-t border-[rgba(255,255,255,0.08)]">
                        <div class="flex items-center justify-between text-xs">
                            <span class="text-[#a0a0a0] font-semibold uppercase text-[11px]">"Resolution"</span>
                            <div class="flex gap-1">
                                {
                                    let resolutions = [512u32, 1024u32, 2048u32];
                                    resolutions.into_iter().map(|res| {
                                        view! {
                                            <button
                                                class=move || {
                                                    let is_act = target_res.get() == res;
                                                    if is_act {
                                                        "px-2.5 py-0.5 rounded-full bg-[#00f0ff] text-[#000000] font-bold text-[11px]"
                                                    } else {
                                                        "px-2.5 py-0.5 rounded-full text-[#a0a0a0] hover:text-[#ffffff] text-[11px]"
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
                                class="button button-primary w-full sm:flex-1 py-2 text-center text-xs font-bold"
                                on:click=move |_| {
                                    let res = target_res.get();
                                    if let Ok((_, svg)) = qr_result_memo.get() {
                                        download_png(svg, res);
                                    }
                                }
                            >
                                {move || format!("↓ Download PNG ({}px)", target_res.get())}
                            </button>

                            <button
                                class="button button-white w-full sm:w-auto py-2 px-4 text-xs font-semibold"
                                on:click=move |_| {
                                    if let Ok((_, svg)) = qr_result_memo.get() {
                                        download_svg(svg);
                                    }
                                }
                            >
                                "↓ SVG"
                            </button>

                            <button
                                class="button button-secondary w-full sm:w-auto py-2 px-4 text-xs font-semibold"
                                on:click=move |_| {
                                    if let Ok((_, svg)) = qr_result_memo.get() {
                                        copy_text_to_clipboard(svg, "SVG");
                                    }
                                }
                            >
                                "Copy SVG"
                            </button>
                        </div>
                    </div>

                    // Toast Notification
                    {move || notification_msg.get().map(|msg| {
                        view! {
                            <div class="absolute bottom-4 px-4 py-1.5 rounded-full bg-[#050505] border border-[#00f0ff] text-[#00f0ff] text-xs font-bold shadow-[0_0_15px_rgba(0,240,255,0.35)]">
                                {msg}
                            </div>
                        }
                    })}
                </div>

                // 3. Shape Configuration: Modules, Eye Frame, and Eye Dot
                <div class="glass-panel p-4 sm:p-5 space-y-4">
                    <div class="text-[11px] uppercase font-semibold text-[#a0a0a0] tracking-wider pb-1 border-b border-[rgba(255,255,255,0.08)]">
                        "Shapes & Style"
                    </div>

                    <div class="grid grid-cols-1 sm:grid-cols-3 gap-4">
                        // Module Shape
                        <div>
                            <label class="block text-[#a0a0a0] text-[11px] mb-1.5 uppercase font-semibold">"Module Shape"</label>
                            <div class="grid grid-cols-2 gap-1">
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
                                                        "py-1.5 text-xs rounded-md border border-[#00f0ff] bg-[#00f0ff]/15 text-[#00f0ff] font-bold"
                                                    } else {
                                                        "py-1.5 text-xs rounded-md border border-[rgba(255,255,255,0.12)] bg-[#050505] text-[#a0a0a0] hover:text-[#ffffff]"
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

                        // Eye Frame Shape
                        <div>
                            <label class="block text-[#a0a0a0] text-[11px] mb-1.5 uppercase font-semibold">"Eye Frame Shape"</label>
                            <div class="grid grid-cols-1 gap-1">
                                {
                                    let frames = [
                                        (EyeFrameShape::Rounded, "Round Frame"),
                                        (EyeFrameShape::Circle, "Circle Frame"),
                                        (EyeFrameShape::Square, "Square Frame"),
                                    ];
                                    frames.into_iter().map(|(f, label)| {
                                        view! {
                                            <button
                                                class=move || {
                                                    let active = eye_frame_shape.get() == f;
                                                    if active {
                                                        "py-1.5 text-xs rounded-md border border-[#00f0ff] bg-[#00f0ff]/15 text-[#00f0ff] font-bold"
                                                    } else {
                                                        "py-1.5 text-xs rounded-md border border-[rgba(255,255,255,0.12)] bg-[#050505] text-[#a0a0a0] hover:text-[#ffffff]"
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

                        // Eye Dot Shape
                        <div>
                            <label class="block text-[#a0a0a0] text-[11px] mb-1.5 uppercase font-semibold">"Eye Dot Shape"</label>
                            <div class="grid grid-cols-1 gap-1">
                                {
                                    let dots = [
                                        (EyeDotShape::Circle, "Circle Dot"),
                                        (EyeDotShape::Rounded, "Round Dot"),
                                        (EyeDotShape::Square, "Square Dot"),
                                    ];
                                    dots.into_iter().map(|(d, label)| {
                                        view! {
                                            <button
                                                class=move || {
                                                    let active = eye_dot_shape.get() == d;
                                                    if active {
                                                        "py-1.5 text-xs rounded-md border border-[#00f0ff] bg-[#00f0ff]/15 text-[#00f0ff] font-bold"
                                                    } else {
                                                        "py-1.5 text-xs rounded-md border border-[rgba(255,255,255,0.12)] bg-[#050505] text-[#a0a0a0] hover:text-[#ffffff]"
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
                </div>

                // 4. Color Pickers: Module Color, Eye Frame Color, and Eye Dot Color
                <div class="glass-panel p-4 sm:p-5 space-y-4">
                    <div class="text-[11px] uppercase font-semibold text-[#a0a0a0] tracking-wider pb-1 border-b border-[rgba(255,255,255,0.08)]">
                        "Colors & Palette"
                    </div>

                    // 4a. Module Color Picker
                    <div class="space-y-2">
                        <div class="flex items-center justify-between">
                            <span class="text-xs font-semibold text-[#ffffff] flex items-center gap-1.5">
                                <span class="w-2.5 h-2.5 rounded-full border border-white/20" style=move || format!("background-color: {}", module_color.get())></span>
                                <span>"Module Color (Data Matrix)"</span>
                            </span>
                            <div class="flex items-center gap-2">
                                <input
                                    type="color"
                                    class="w-6 h-6 rounded bg-transparent border border-white/20 cursor-pointer"
                                    prop:value=move || module_color.get()
                                    on:input=move |ev| set_module_color.set(event_target_value(&ev))
                                />
                                <input
                                    type="text"
                                    class="w-24 text-xs uppercase bg-[#050505] border border-[rgba(255,255,255,0.12)] rounded px-2 py-1 text-center"
                                    prop:value=move || module_color.get()
                                    on:input=move |ev| set_module_color.set(event_target_value(&ev))
                                />
                            </div>
                        </div>

                        <div class="flex items-center gap-1.5">
                            {
                                color_presets.into_iter().map(|hex| {
                                    view! {
                                        <button
                                            class="w-6 h-6 rounded-full border border-white/20 transition-transform hover:scale-110 cursor-pointer"
                                            style=format!("background-color: {hex}")
                                            title=format!("Set module color to {hex}")
                                            on:click=move |_| set_module_color.set(hex.to_string())
                                        ></button>
                                    }
                                }).collect_view()
                            }
                        </div>
                    </div>

                    // 4b. Eye Frame Color Picker
                    <div class="space-y-2 pt-3 border-t border-[rgba(255,255,255,0.08)]">
                        <div class="flex items-center justify-between">
                            <span class="text-xs font-semibold text-[#ffffff] flex items-center gap-1.5">
                                <span class="w-2.5 h-2.5 rounded-full border border-white/20" style=move || format!("background-color: {}", eye_color.get())></span>
                                <span>"Eye Frame Color (Outer Frame)"</span>
                            </span>
                            <div class="flex items-center gap-2">
                                <button
                                    class="text-[11px] text-[#00f0ff] hover:underline cursor-pointer mr-1"
                                    title="Sync with module color"
                                    on:click=move |_| set_eye_color.set(module_color.get())
                                >
                                    "Sync"
                                </button>
                                <input
                                    type="color"
                                    class="w-6 h-6 rounded bg-transparent border border-white/20 cursor-pointer"
                                    prop:value=move || eye_color.get()
                                    on:input=move |ev| set_eye_color.set(event_target_value(&ev))
                                />
                                <input
                                    type="text"
                                    class="w-24 text-xs uppercase bg-[#050505] border border-[rgba(255,255,255,0.12)] rounded px-2 py-1 text-center"
                                    prop:value=move || eye_color.get()
                                    on:input=move |ev| set_eye_color.set(event_target_value(&ev))
                                />
                            </div>
                        </div>

                        <div class="flex items-center gap-1.5">
                            {
                                color_presets.into_iter().map(|hex| {
                                    view! {
                                        <button
                                            class="w-6 h-6 rounded-full border border-white/20 transition-transform hover:scale-110 cursor-pointer"
                                            style=format!("background-color: {hex}")
                                            title=format!("Set eye frame color to {hex}")
                                            on:click=move |_| set_eye_color.set(hex.to_string())
                                        ></button>
                                    }
                                }).collect_view()
                            }
                        </div>
                    </div>

                    // 4c. Eye Dot Color Picker
                    <div class="space-y-2 pt-3 border-t border-[rgba(255,255,255,0.08)]">
                        <div class="flex items-center justify-between">
                            <span class="text-xs font-semibold text-[#ffffff] flex items-center gap-1.5">
                                <span class="w-2.5 h-2.5 rounded-full border border-white/20" style=move || format!("background-color: {}", dot_color.get())></span>
                                <span>"Eye Dot Color (Inner Center)"</span>
                            </span>
                            <div class="flex items-center gap-2">
                                <button
                                    class="text-[11px] text-[#00f0ff] hover:underline cursor-pointer mr-1"
                                    title="Sync with module color"
                                    on:click=move |_| set_dot_color.set(module_color.get())
                                >
                                    "Sync"
                                </button>
                                <input
                                    type="color"
                                    class="w-6 h-6 rounded bg-transparent border border-white/20 cursor-pointer"
                                    prop:value=move || dot_color.get()
                                    on:input=move |ev| set_dot_color.set(event_target_value(&ev))
                                />
                                <input
                                    type="text"
                                    class="w-24 text-xs uppercase bg-[#050505] border border-[rgba(255,255,255,0.12)] rounded px-2 py-1 text-center"
                                    prop:value=move || dot_color.get()
                                    on:input=move |ev| set_dot_color.set(event_target_value(&ev))
                                />
                            </div>
                        </div>

                        <div class="flex items-center gap-1.5">
                            {
                                color_presets.into_iter().map(|hex| {
                                    view! {
                                        <button
                                            class="w-6 h-6 rounded-full border border-white/20 transition-transform hover:scale-110 cursor-pointer"
                                            style=format!("background-color: {hex}")
                                            title=format!("Set eye dot color to {hex}")
                                            on:click=move |_| set_dot_color.set(hex.to_string())
                                        ></button>
                                    }
                                }).collect_view()
                            }
                        </div>
                    </div>

                    // Background & Optional Logo
                    <div class="pt-3 border-t border-[rgba(255,255,255,0.08)] flex flex-wrap items-center justify-between gap-4">
                        // Background Controls
                        <div class="flex items-center gap-3">
                            <label class="flex items-center gap-2 text-xs text-[#a0a0a0] hover:text-[#ffffff] cursor-pointer select-none">
                                <input
                                    type="checkbox"
                                    class="rounded accent-[#00f0ff]"
                                    prop:checked=move || is_transparent_bg.get()
                                    on:change=move |ev| set_is_transparent_bg.set(event_target_checked(&ev))
                                />
                                <span>"Transparent BG"</span>
                            </label>

                            {move || if !is_transparent_bg.get() {
                                view! {
                                    <div class="flex items-center gap-1.5 text-xs text-[#a0a0a0]">
                                        <span>"BG:"</span>
                                        <input
                                            type="color"
                                            class="w-5 h-5 rounded bg-transparent border border-white/20 cursor-pointer"
                                            title="Background Color"
                                            prop:value=move || bg_color.get()
                                            on:input=move |ev| set_bg_color.set(event_target_value(&ev))
                                        />
                                    </div>
                                }.into_any()
                            } else {
                                view! {}.into_any()
                            }}
                        </div>

                        // Optional Center Logo
                        <div class="flex items-center gap-2">
                            {move || if logo_data_url.get().is_some() {
                                view! {
                                    <button
                                        class="text-xs text-rose-400 hover:underline cursor-pointer"
                                        on:click=move |_| set_logo_data_url.set(None)
                                    >
                                        "Remove Logo"
                                    </button>
                                }.into_any()
                            } else {
                                view! {
                                    <label class="text-xs text-[#00f0ff] hover:underline cursor-pointer">
                                        <span>"+ Add Center Logo"</span>
                                        <input
                                            type="file"
                                            accept="image/*"
                                            class="hidden"
                                            on:change=move |ev| {
                                                let target = event_target::<web_sys::HtmlInputElement>(&ev);
                                                if let Some(files) = target.files() {
                                                    if let Some(file) = files.get(0) {
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
                                    </label>
                                }.into_any()
                            }}
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
