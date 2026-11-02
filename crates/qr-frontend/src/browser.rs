//! Browser-only helpers for clipboard access and file downloads.

use js_sys::Array;
use js_sys::Promise;
use wasm_bindgen::closure::Closure;
use wasm_bindgen::prelude::wasm_bindgen;
use wasm_bindgen::JsCast;
use wasm_bindgen::JsValue;
use web_sys::Blob;
use web_sys::BlobPropertyBag;
use web_sys::CanvasRenderingContext2d;
use web_sys::Document;
use web_sys::HtmlAnchorElement;
use web_sys::HtmlCanvasElement;
use web_sys::HtmlImageElement;
use web_sys::Url;

#[wasm_bindgen(inline_js = r#"
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
    #[wasm_bindgen(js_name = copyCanvasToClipboard)]
    fn copy_canvas_to_clipboard(canvas: &HtmlCanvasElement) -> Promise;
}

/// Returns the document of the current window.
fn document() -> Option<Document> {
    web_sys::window()?.document()
}

/// Creates an object URL for an SVG document.
fn create_svg_object_url(svg: &str) -> Result<String, String> {
    let parts = Array::new();
    parts.push(&JsValue::from_str(svg));

    let options = BlobPropertyBag::new();
    options.set_type("image/svg+xml;charset=utf-8");
    let blob = Blob::new_with_str_sequence_and_options(&parts, &options)
        .map_err(|_| "Failed to create SVG blob".to_string())?;
    Url::create_object_url_with_blob(&blob).map_err(|_| "Failed to create object URL".to_string())
}

/// Starts a download of `href` under `filename`.
fn trigger_download(href: &str, filename: &str) -> Result<(), String> {
    let document = document().ok_or("Document is unavailable")?;
    let element = document
        .create_element("a")
        .map_err(|_| "Failed to create anchor element")?;
    let anchor = element.unchecked_into::<HtmlAnchorElement>();
    anchor.set_href(href);
    anchor.set_download(filename);
    anchor.click();
    Ok(())
}

/// Attaches success and error callbacks to `promise`.
fn handle_promise(
    promise: &Promise,
    on_success: impl Fn() + 'static,
    on_error: impl Fn() + 'static,
) {
    let success =
        Closure::wrap(Box::new(move |_: JsValue| on_success()) as Box<dyn FnMut(JsValue)>);
    let error = Closure::wrap(Box::new(move |_: JsValue| on_error()) as Box<dyn FnMut(JsValue)>);
    let _ = promise.then2(&success, &error);
    success.forget();
    error.forget();
}

/// Rasterises `svg` into a canvas and passes the canvas to `on_ready`.
fn render_svg_to_canvas(
    svg: &str,
    resolution: u32,
    on_ready: impl FnOnce(HtmlCanvasElement) + 'static,
) -> Result<(), String> {
    let document = document().ok_or("Document is unavailable")?;
    let element = document
        .create_element("canvas")
        .map_err(|_| "Failed to create canvas element")?;
    let canvas = element.unchecked_into::<HtmlCanvasElement>();
    canvas.set_width(resolution);
    canvas.set_height(resolution);

    let context = canvas
        .get_context("2d")
        .map_err(|_| "Failed to access canvas context")?
        .ok_or("Canvas 2D context is unavailable")?
        .dyn_into::<CanvasRenderingContext2d>()
        .map_err(|_| "Unexpected canvas context type")?;

    let url = create_svg_object_url(svg)?;
    let image = HtmlImageElement::new().map_err(|_| "Failed to create image element")?;

    let callback_canvas = canvas.clone();
    let callback_image = image.clone();
    let callback_url = url.clone();
    let mut on_ready = Some(on_ready);
    let onload = Closure::wrap(Box::new(move || {
        let _ = context.draw_image_with_html_image_element_and_dw_and_dh(
            &callback_image,
            0.0,
            0.0,
            resolution as f64,
            resolution as f64,
        );
        let _ = Url::revoke_object_url(&callback_url);
        if let Some(callback) = on_ready.take() {
            callback(callback_canvas.clone());
        }
    }) as Box<dyn FnMut()>);

    image.set_onload(Some(onload.as_ref().unchecked_ref()));
    onload.forget();
    image.set_src(&url);
    Ok(())
}

/// Downloads `svg` as a standalone SVG file.
pub(crate) fn download_svg(svg: &str) -> Result<(), String> {
    let url = create_svg_object_url(svg)?;
    let result = trigger_download(&url, "qrcode.svg");
    let _ = Url::revoke_object_url(&url);
    result
}

/// Downloads `svg` rasterised to `resolution` square pixels as a PNG file.
pub(crate) fn download_png(svg: &str, resolution: u32) -> Result<(), String> {
    render_svg_to_canvas(svg, resolution, move |canvas| {
        let Ok(data_url) = canvas.to_data_url_with_type("image/png") else {
            return;
        };
        let _ = trigger_download(&data_url, &format!("qrcode_{resolution}px.png"));
    })
}

/// Copies `svg` to the clipboard as text.
pub(crate) fn copy_svg(
    svg: &str,
    on_success: impl Fn() + 'static,
    on_error: impl Fn() + 'static,
) -> Result<(), String> {
    let clipboard = web_sys::window()
        .ok_or("Window is unavailable")?
        .navigator()
        .clipboard();
    let promise = clipboard.write_text(svg);
    handle_promise(&promise, on_success, on_error);
    Ok(())
}

/// Copies `svg` to the clipboard rasterised to `resolution` square pixels.
pub(crate) fn copy_png(
    svg: &str,
    resolution: u32,
    on_success: impl Fn() + 'static,
    on_error: impl Fn() + 'static,
) -> Result<(), String> {
    render_svg_to_canvas(svg, resolution, move |canvas| {
        let promise = copy_canvas_to_clipboard(&canvas);
        handle_promise(&promise, on_success, on_error);
    })
}
