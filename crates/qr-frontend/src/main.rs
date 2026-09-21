#[cfg(feature = "csr")]
fn main() {
    console_error_panic_hook::set_once();
    leptos::mount::mount_to_body(|| leptos::prelude::view! { <qr_frontend::App /> });
}

#[cfg(not(feature = "csr"))]
fn main() {}
