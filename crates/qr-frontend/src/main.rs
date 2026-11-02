//! Native entry point for the frontend crate.

#[cfg(feature = "csr")]
fn main() {
    console_error_panic_hook::set_once();
    leptos::mount::mount_to_body(qr_frontend::App);
}

#[cfg(not(feature = "csr"))]
fn main() {}
