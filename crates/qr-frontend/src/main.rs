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

    // Essential Styling State (KISS: Ente Classy as default!)
    let (module_shape, set_module_shape) = signal(ModuleShape::Classy);
    let (eye_frame_shape, set_eye_frame_shape) = signal(EyeFrameShape::Rounded);
    let (code_color, set_code_color) = signal("#3fd9ff".to_string());
    let (is_transparent_bg, set_is_transparent_bg) = signal(false);
    let (bg_color, set_bg_color) = signal("#04070d".to_string());

    // Optional Center Logo
    let (logo_data_url, set_logo_data_url) = signal(Option::<String>::None);

    // Export Options
    let (target_res, set_target_res) = signal(1024u32);
    let (notification_msg, set_notification_msg) = signal(Option::<String>::None);

    // Dynamic Query Parameter Inspector (ha.mr / tools.ralite.dev philosophy)
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
            eye_dot_shape: EyeDotShape::Circle,
            code_color: code_color.get(),
            gradient: None,
            eye_color: None,
            bg_color: bg,
            logo,
            cta: None,
            alt_text: Some("QR Code".to_string()),
        };

        let svg = VectorRenderer::render_svg(&matrix, &config);
        Ok((matrix, svg))
    });

    // Contrast calculation
    let contrast_ratio = Memo::new(move |_| {
        let code = code_color.get();
        let bg = if is_transparent_bg.get() {
            "#FFFFFF"
        } else {
            &bg_color.get()
        };
        calculate_contrast_ratio(&code, bg)
    });

    // Temporary notification helper
    let show_toast = move |msg: String| {
        set_notification_msg.set(Some(msg));
        wasm_bindgen_futures::spawn_local(async move {
            gloo_timers::future::TimeoutFuture::new(2400).await;
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

    view! {
        <div class="min-h-screen bg-[#04070d] text-[#e6f1f5] pb-16 selection:bg-[#3fd9ff]/25 selection:text-[#3fd9ff]">
            // Floating Pill Navigation Bar
            <div class="site-nav-wrap mb-8">
                <nav class="site-nav">
                    <a href="/" class="flex items-center gap-2 text-sm font-bold tracking-tight">
                        <span class="chromatic-text" data-text="qr.den1zz.dev">"qr.den1zz.dev"</span>
                    </a>

                    <div class="flex items-center gap-2">
                        <div class="status-badge py-0.5 px-2.5 text-[11px]">
                            <span class="status-dot-active"></span>
                            <span>"ONLINE // PURE RUST"</span>
                        </div>
                    </div>

                    <a
                        href="https://den1zz.dev"
                        target="_blank"
                        rel="noreferrer"
                        class="button button-secondary text-xs py-1 px-3"
                    >
                        <span>"den1zz.dev"</span>
                        <span class="text-[#3fd9ff]">"↗"</span>
                    </a>
                </nav>
            </div>

            // Main Focused Container (KISS Philosophy)
            <main class="w-full max-w-[720px] mx-auto px-4 space-y-6">
                // Clean Hero Eyebrow
                <div class="text-center space-y-2">
                    <h1 class="text-3xl sm:text-4xl font-bold tracking-tight">
                        <span class="chromatic-text" data-text="Simple QR Code Studio">"Simple QR Code Studio"</span>
                    </h1>
                    <p class="text-xs sm:text-sm text-[#93a8b3] font-sans max-w-md mx-auto">
                        "Fast, private, client-side QR generation with Ente classy modules and smart URL parameter stripping."
                    </p>
                </div>

                // 1. Primary Input Box
                <div class="glass-panel p-5 space-y-3">
                    <div class="relative flex items-center">
                        <input
                            type="text"
                            class="w-full text-sm sm:text-base py-3 pl-4 pr-10 rounded-xl bg-[#080d16] border border-[rgba(63,217,255,0.22)] focus:border-[#3fd9ff] text-[#e6f1f5] placeholder:text-[#56636b] outline-none transition-all shadow-inner font-mono"
                            placeholder="Paste link or enter text to encode..."
                            prop:value=move || input_text.get()
                            on:input=move |ev| set_input_text.set(event_target_value(&ev))
                        />
                        {move || if !input_text.get().is_empty() {
                            view! {
                                <button
                                    class="absolute right-3 text-[#93a8b3] hover:text-[#e6f1f5] p-1 text-sm font-mono"
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

                    // 2. ha.mr & tools.ralite.dev Query Warning and Parameter Stripper
                    {move || query_info.get().map(|info| {
                        view! {
                            <div class="p-4 rounded-xl bg-[#080d16]/95 border border-[#3fd9ff]/40 space-y-2.5 text-left transition-all">
                                <div class="flex items-center justify-between text-xs">
                                    <span class="font-bold text-[#3fd9ff] flex items-center gap-1.5 tracking-wide">
                                        <span>"⚠️"</span>
                                        <span>"Consider removing parameters..."</span>
                                    </span>
                                    <span class="text-[11px] font-mono text-[#93a8b3]">
                                        {format!("{} query parameters detected", info.total_param_count)}
                                    </span>
                                </div>
                                <p class="text-xs text-[#93a8b3] leading-relaxed font-sans">
                                    "Some links include data that is not necessary for the link to function, such as tracking tokens, analytics, or page state. Removing these segments shortens the URL, resulting in a cleaner, much faster-to-scan QR code."
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

                // 3. Live QR Viewport Stage
                <div class="glass-panel p-6 flex flex-col items-center justify-center relative">
                    <div class="hud-reticle hud-tl"></div>
                    <div class="hud-reticle hud-tr"></div>
                    <div class="hud-reticle hud-bl"></div>
                    <div class="hud-reticle hud-br"></div>

                    {move || match qr_result_memo.get() {
                        Ok((matrix, svg)) => {
                            let ratio = contrast_ratio.get();
                            let is_high_contrast = ratio >= 3.5;
                            view! {
                                <div class="w-full flex flex-col items-center justify-center space-y-4">
                                    // Scannability status pill
                                    <div class="flex items-center gap-2 text-xs font-mono px-3 py-1 rounded-full bg-[#080d16] border border-[rgba(63,217,255,0.18)]">
                                        <span class=if is_high_contrast {
                                            "w-2 h-2 rounded-full bg-emerald-400 shadow-[0_0_8px_#34d399]"
                                        } else {
                                            "w-2 h-2 rounded-full bg-amber-400 shadow-[0_0_8px_#fbbf24]"
                                        }></span>
                                        <span class="text-[#e6f1f5]">
                                            {if is_high_contrast { "High Contrast" } else { "Low Contrast" }}
                                        </span>
                                        <span class="text-[#56636b]">"•"</span>
                                        <span class="text-[#3fd9ff]">{format!("{ratio:.1}:1")}</span>
                                        <span class="text-[#56636b]">"•"</span>
                                        <span class="text-[#93a8b3]">{format!("v{} ({}×{})", matrix.version, matrix.size, matrix.size)}</span>
                                    </div>

                                    // Viewfinder Frame
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
                                </div>
                            }.into_any()
                        }
                        Err(e) => view! {
                            <div class="text-rose-400 text-xs font-mono py-8">{format!("Error: {e}")}</div>
                        }.into_any()
                    }}

                    // Toast Notification
                    {move || notification_msg.get().map(|msg| {
                        view! {
                            <div class="absolute bottom-4 px-4 py-1.5 rounded-full bg-[#080d16] border border-[#3fd9ff] text-[#3fd9ff] text-xs font-mono font-bold shadow-[0_0_20px_rgba(63,217,255,0.4)]">
                                {msg}
                            </div>
                        }
                    })}
                </div>

                // 4. Essential Customization Toolbar (KISS)
                <div class="glass-panel p-5 space-y-4">
                    // Shape & Corners in a clean row
                    <div class="grid grid-cols-1 sm:grid-cols-2 gap-4">
                        // Module Shape
                        <div>
                            <label class="block text-[#93a8b3] text-[11px] mb-1.5 uppercase font-semibold">"Module Shape"</label>
                            <div class="grid grid-cols-4 gap-1">
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
                                                        "py-1.5 text-xs rounded-md border border-[#3fd9ff] bg-[#3fd9ff]/15 text-[#3fd9ff] font-bold shadow-[0_0_10px_rgba(63,217,255,0.2)]"
                                                    } else {
                                                        "py-1.5 text-xs rounded-md border border-[rgba(63,217,255,0.18)] bg-[#080d16] text-[#93a8b3] hover:text-[#e6f1f5]"
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
                            <label class="block text-[#93a8b3] text-[11px] mb-1.5 uppercase font-semibold">"Eye Shape"</label>
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
                                                        "py-1.5 text-xs rounded-md border border-[#3fd9ff] bg-[#3fd9ff]/15 text-[#3fd9ff] font-bold shadow-[0_0_10px_rgba(63,217,255,0.2)]"
                                                    } else {
                                                        "py-1.5 text-xs rounded-md border border-[rgba(63,217,255,0.18)] bg-[#080d16] text-[#93a8b3] hover:text-[#e6f1f5]"
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
                    </div>

                    // Color & Logo row
                    <div class="pt-3 border-t border-[rgba(63,217,255,0.18)] flex flex-wrap items-center justify-between gap-4">
                        // Color Swatches
                        <div class="flex items-center gap-2">
                            <label class="text-xs text-[#93a8b3] font-semibold">"Color:"</label>
                            <div class="flex items-center gap-1.5">
                                {
                                    let swatches = ["#3fd9ff", "#ffffff", "#38ef7d", "#ff3b80", "#f59e0b", "#a855f7"];
                                    swatches.into_iter().map(|hex| {
                                        view! {
                                            <button
                                                class="w-6 h-6 rounded-full border border-white/20 transition-transform hover:scale-110 cursor-pointer"
                                                style=format!("background-color: {hex}")
                                                on:click=move |_| set_code_color.set(hex.to_string())
                                            ></button>
                                        }
                                    }).collect_view()
                                }
                            </div>
                            <input
                                type="color"
                                class="w-6 h-6 rounded bg-transparent border border-[rgba(63,217,255,0.25)] cursor-pointer ml-1"
                                prop:value=move || code_color.get()
                                on:input=move |ev| set_code_color.set(event_target_value(&ev))
                            />
                        </div>

                        // Background Color & Transparent Toggle
                        <div class="flex items-center gap-2">
                            <label class="flex items-center gap-2 text-xs text-[#93a8b3] hover:text-[#e6f1f5] cursor-pointer select-none">
                                <input
                                    type="checkbox"
                                    class="rounded accent-[#3fd9ff]"
                                    prop:checked=move || is_transparent_bg.get()
                                    on:change=move |ev| set_is_transparent_bg.set(event_target_checked(&ev))
                                />
                                <span>"Transparent BG"</span>
                            </label>

                            {move || if !is_transparent_bg.get() {
                                view! {
                                    <input
                                        type="color"
                                        class="w-6 h-6 rounded bg-transparent border border-[rgba(63,217,255,0.25)] cursor-pointer"
                                        title="Background Color"
                                        prop:value=move || bg_color.get()
                                        on:input=move |ev| set_bg_color.set(event_target_value(&ev))
                                    />
                                }.into_any()
                            } else {
                                view! {}.into_any()
                            }}
                        </div>

                        // Optional Center Logo upload
                        <div class="flex items-center gap-2">
                            {move || if logo_data_url.get().is_some() {
                                view! {
                                    <button
                                        class="text-xs text-rose-400 hover:underline"
                                        on:click=move |_| set_logo_data_url.set(None)
                                    >
                                        "Remove Logo"
                                    </button>
                                }.into_any()
                            } else {
                                view! {
                                    <label class="text-xs text-[#3fd9ff] hover:underline cursor-pointer">
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

                // 5. One-Click Export Bar
                <div class="glass-panel p-5 space-y-3">
                    <div class="flex items-center justify-between text-xs pb-2 border-b border-[rgba(63,217,255,0.18)]">
                        <span class="text-[#93a8b3] font-semibold uppercase text-[11px]">"Resolution"</span>
                        <div class="flex gap-1">
                            {
                                let resolutions = [512u32, 1024u32, 2048u32];
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
                                if let Ok((_, svg)) = qr_result_memo.get() {
                                    download_png(svg, res);
                                }
                            }
                        >
                            {move || format!("↓ Download PNG ({}px)", target_res.get())}
                        </button>

                        <button
                            class="button button-secondary w-full sm:w-auto py-2.5 px-4 text-xs font-semibold"
                            on:click=move |_| {
                                if let Ok((_, svg)) = qr_result_memo.get() {
                                    download_svg(svg);
                                }
                            }
                        >
                            "↓ SVG"
                        </button>

                        <button
                            class="button button-secondary w-full sm:w-auto py-2.5 px-4 text-xs font-semibold"
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
            </main>
        </div>
    }
}

fn main() {
    console_error_panic_hook::set_once();
    mount_to_body(|| view! { <App /> });
}
