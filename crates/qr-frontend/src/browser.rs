//! Browser-only helpers for clipboard access and file downloads.

use std::cell::RefCell;
use std::rc::Rc;

use js_sys::Promise;
use wasm_bindgen::closure::Closure;
use wasm_bindgen::prelude::*;
use wasm_bindgen::JsCast;
use wasm_bindgen::JsValue;
use web_sys::CanvasRenderingContext2d;
use web_sys::HtmlCanvasElement;
use web_sys::HtmlImageElement;
use web_sys::Url;

#[wasm_bindgen(inline_js = r#"
export function copyTextToClipboard(text) {
    if (!navigator.clipboard || !navigator.clipboard.writeText) {
        return Promise.reject(new Error("Clipboard access is unavailable. Serve the page over HTTPS or localhost."));
    }
    return navigator.clipboard.writeText(text);
}

export function copyCanvasToClipboard(canvas) {
    return new Promise((resolve, reject) => {
        if (!navigator.clipboard || !window.ClipboardItem) {
            reject(new Error("Image clipboard access is unavailable in this browser."));
            return;
        }
        canvas.toBlob((blob) => {
            if (!blob) {
                reject(new Error("The canvas could not be encoded as PNG."));
                return;
            }
            const item = new ClipboardItem({ "image/png": blob });
            navigator.clipboard.write([item]).then(resolve).catch(reject);
        }, "image/png");
    });
}

export function downloadText(text, filename, mime) {
    const blob = new Blob([text], { type: mime });
    const url = URL.createObjectURL(blob);
    const anchor = document.createElement("a");
    anchor.href = url;
    anchor.download = filename;
    document.body.appendChild(anchor);
    anchor.click();
    anchor.remove();
    setTimeout(() => URL.revokeObjectURL(url), 1000);
}

export function downloadCanvas(canvas, filename) {
    return new Promise((resolve, reject) => {
        canvas.toBlob((blob) => {
            if (!blob) {
                reject(new Error("The canvas could not be encoded as PNG."));
                return;
            }
            const url = URL.createObjectURL(blob);
            const anchor = document.createElement("a");
            anchor.href = url;
            anchor.download = filename;
            document.body.appendChild(anchor);
            anchor.click();
            anchor.remove();
            setTimeout(() => URL.revokeObjectURL(url), 1000);
            resolve();
        }, "image/png");
    });
}
"#)]
extern "C" {
    #[wasm_bindgen(js_name = copyTextToClipboard)]
    fn copy_text_to_clipboard(text: &str) -> Promise;

    #[wasm_bindgen(js_name = copyCanvasToClipboard)]
    fn copy_canvas_to_clipboard(canvas: &HtmlCanvasElement) -> Promise;

    #[wasm_bindgen(js_name = downloadText)]
    fn download_text(text: &str, filename: &str, mime: &str);

    #[wasm_bindgen(js_name = downloadCanvas)]
    fn download_canvas(canvas: &HtmlCanvasElement, filename: &str) -> Promise;
}

/// Replaces browser clipboard errors with an actionable message.
fn friendly_clipboard_error(message: String) -> String {
    let lower = message.to_lowercase();
    if lower.contains("permission")
        || lower.contains("denied")
        || lower.contains("not allowed")
        || lower.contains("writetext")
    {
        "Clipboard permission denied. Download the file instead.".to_string()
    } else if lower.contains("unavailable") {
        "Clipboard access is unavailable in this browser. Download the file instead.".to_string()
    } else {
        message
    }
}

/// Returns the most useful message from a rejected promise.
fn error_message(value: &JsValue) -> String {
    value
        .as_string()
        .or_else(|| {
            js_sys::Reflect::get(value, &JsValue::from_str("message"))
                .ok()
                .and_then(|message| message.as_string())
        })
        .filter(|message| !message.trim().is_empty())
        .unwrap_or_else(|| "The browser refused the request.".to_string())
}

/// Attaches success and error callbacks to `promise`.
fn handle_promise(
    promise: &Promise,
    on_success: impl Fn() + 'static,
    on_error: impl Fn(String) + 'static,
) {
    let success =
        Closure::wrap(Box::new(move |_: JsValue| on_success()) as Box<dyn FnMut(JsValue)>);
    let error = Closure::wrap(
        Box::new(move |value: JsValue| on_error(error_message(&value))) as Box<dyn FnMut(JsValue)>,
    );
    let _ = promise.then2(&success, &error);
    success.forget();
    error.forget();
}

/// Returns the document of the current window.
fn document() -> Result<web_sys::Document, String> {
    web_sys::window()
        .and_then(|window| window.document())
        .ok_or_else(|| "The document is unavailable.".to_string())
}

/// Creates an object URL for `svg`.
fn create_svg_object_url(svg: &str) -> Result<String, String> {
    let array = js_sys::Array::new();
    array.push(&JsValue::from_str(svg));
    let options = web_sys::BlobPropertyBag::new();
    options.set_type("image/svg+xml;charset=utf-8");
    let blob = web_sys::Blob::new_with_str_sequence_and_options(&array, &options)
        .map_err(|_| "The SVG could not be prepared for rasterising.".to_string())?;
    Url::create_object_url_with_blob(&blob)
        .map_err(|_| "The SVG could not be prepared for rasterising.".to_string())
}

/// Rasterises `svg` into a canvas and passes the canvas to `on_ready`.
fn render_svg_to_canvas(
    svg: &str,
    resolution: u32,
    on_ready: Box<dyn FnOnce(HtmlCanvasElement)>,
    on_error: Box<dyn Fn(String)>,
) -> Result<(), String> {
    let document = document()?;
    let element = document
        .create_element("canvas")
        .map_err(|_| "A canvas could not be created.".to_string())?;
    let canvas = element.unchecked_into::<HtmlCanvasElement>();
    canvas.set_width(resolution);
    canvas.set_height(resolution);

    let context = canvas
        .get_context("2d")
        .map_err(|_| "The canvas context is unavailable.".to_string())?
        .ok_or_else(|| "The canvas context is unavailable.".to_string())?
        .dyn_into::<CanvasRenderingContext2d>()
        .map_err(|_| "The canvas returned an unexpected context.".to_string())?;

    let url = create_svg_object_url(svg)?;
    let image = HtmlImageElement::new()
        .map_err(|_| "The SVG could not be loaded for rasterising.".to_string())?;

    let callback_canvas = canvas.clone();
    let callback_image = image.clone();
    let callback_url = url.clone();
    let ready = Rc::new(RefCell::new(Some(on_ready)));
    let onload = Closure::wrap(Box::new(move || {
        let _ = context.draw_image_with_html_image_element_and_dw_and_dh(
            &callback_image,
            0.0,
            0.0,
            resolution as f64,
            resolution as f64,
        );
        let _ = Url::revoke_object_url(&callback_url);
        if let Some(callback) = ready.borrow_mut().take() {
            callback(callback_canvas.clone());
        }
    }) as Box<dyn FnMut()>);

    let callback_url = url.clone();
    let error = Rc::new(on_error);
    let onerror = Closure::wrap(Box::new(move || {
        let _ = Url::revoke_object_url(&callback_url);
        error("The SVG could not be drawn into the canvas.".to_string());
    }) as Box<dyn FnMut()>);

    image.set_onload(Some(onload.as_ref().unchecked_ref()));
    image.set_onerror(Some(onerror.as_ref().unchecked_ref()));
    onload.forget();
    onerror.forget();
    image.set_src(&url);
    Ok(())
}

/// Downloads `svg` as a standalone SVG file.
pub(crate) fn download_svg(svg: &str, on_success: impl Fn() + 'static) -> Result<(), String> {
    download_text(svg, "qrcode.svg", "image/svg+xml;charset=utf-8");
    on_success();
    Ok(())
}

/// Downloads `svg` rasterised to `resolution` square pixels as a PNG file.
pub(crate) fn download_png(
    svg: &str,
    resolution: u32,
    on_success: impl Fn() + Clone + 'static,
    on_error: impl Fn(String) + Clone + 'static,
) -> Result<(), String> {
    let error = on_error.clone();
    render_svg_to_canvas(
        svg,
        resolution,
        Box::new(move |canvas| {
            let outcome = download_canvas(&canvas, &format!("qrcode_{resolution}px.png"));
            let success = on_success.clone();
            let error = error.clone();
            handle_promise(&outcome, success, error);
        }),
        Box::new(on_error),
    )
}

/// Copies `svg` to the clipboard as text.
pub(crate) fn copy_svg(
    svg: &str,
    on_success: impl Fn() + 'static,
    on_error: impl Fn(String) + 'static,
) -> Result<(), String> {
    let promise = copy_text_to_clipboard(svg);
    handle_promise(&promise, on_success, move |message| {
        on_error(friendly_clipboard_error(message));
    });
    Ok(())
}

/// Copies `svg` to the clipboard rasterised to `resolution` square pixels.
pub(crate) fn copy_png(
    svg: &str,
    resolution: u32,
    on_success: impl Fn() + Clone + 'static,
    on_error: impl Fn(String) + Clone + 'static,
) -> Result<(), String> {
    let success = on_success.clone();
    let error = on_error.clone();
    render_svg_to_canvas(
        svg,
        resolution,
        Box::new(move |canvas| {
            let promise = copy_canvas_to_clipboard(&canvas);
            let success = success.clone();
            let error = error.clone();
            handle_promise(&promise, success, move |message| {
                error(friendly_clipboard_error(message));
            });
        }),
        Box::new(on_error),
    )
}
