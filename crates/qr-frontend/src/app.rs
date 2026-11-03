//! Root component for the studio.

use std::time::Duration;

use leptos::prelude::*;

use crate::components::icons::External;
use crate::components::icons::Github;
use crate::components::logo_panel::LogoPanel;
use crate::components::mobile_dock::MobileDock;
use crate::components::palette_panel::PalettePanel;
use crate::components::payload_editor::PayloadEditor;
use crate::components::preview_panel::PreviewPanel;
use crate::components::shape_panel::ShapePanel;
use crate::studio_state::PreviewState;
use crate::studio_state::StudioState;

const REPOSITORY_URL: &str = "https://github.com/Den1zzDev/qr.den1zz.dev";

/// Root component for the QR studio.
#[component]
pub fn App() -> impl IntoView {
    let state = StudioState::new();
    let payload = Memo::new(move |_| state.payload());
    let sanitized = Memo::new(move |_| payload.get().sanitize());
    let query_info = Memo::new(move |_| state.url_query_info());

    // Encoding and colour work is deferred while the user is typing. A 120ms
    // pause keeps the input responsive and still feels immediate.
    let (preview_content, set_preview_content) = signal(sanitized.get_untracked().content);
    let debounce_generation = RwSignal::new(0_u32);
    Effect::new(move |_| {
        let content = sanitized.get().content;
        let generation = debounce_generation.get_untracked() + 1;
        debounce_generation.set(generation);

        if content.is_empty() {
            set_preview_content.set(String::new());
            return;
        }

        if preview_content.get_untracked() == content {
            return;
        }

        set_timeout(
            move || {
                if debounce_generation.get_untracked() == generation {
                    set_preview_content.set(content);
                }
            },
            Duration::from_millis(120),
        );
    });

    let matrix = Memo::new(move |_| {
        let content = preview_content.get();
        if content.is_empty() {
            return None;
        }
        Some(state.encode(&content))
    });

    let preview = Memo::new(move |_| match matrix.get() {
        None => PreviewState::Empty,
        Some(Err(error)) => PreviewState::Error(error.to_string()),
        Some(Ok(matrix)) => {
            let svg = state.render_svg(&matrix);
            PreviewState::Ready(crate::studio_state::RenderedCode { matrix, svg })
        }
    });
    let contrast = Memo::new(move |_| state.contrast_ratios());

    view! {
        <noscript>
            <p style="margin:0;padding:12px 16px;border-bottom:1px solid rgba(255,255,255,0.12);background:#0a0a0a;color:#adadad;font:13px/1.5 ui-monospace,monospace;">
                "This studio needs JavaScript for live generation. The page is still readable, and no data is uploaded when scripting is off."
            </p>
        </noscript>
        <p class="sr-only" role="status" aria-live="polite">
            {move || state.notification.get().map(|notification| notification.message)}
        </p>

        <a class="skip-link" href="#studio">"Skip to the studio"</a>

        <header class="site-header">
            <div class="site-header__inner">
                <div>
                    <a class="wordmark" href="/">
                        <span class="wordmark__accent">"qr."</span>
                        <span>"den1zz.dev"</span>
                    </a>
                    <p class="site-tagline">"Local QR studio. Nothing leaves your browser."</p>
                </div>
                <div class="header-actions">
                    <span
                        class="header-readout"
                        title="Generation and contrast checks run in this tab. No payload data is uploaded."
                    >
                        <span class="header-readout__led" aria-hidden="true"></span>
                        "Local · WASM · no upload"
                    </span>
                    <a
                        class="icon-button"
                        href=REPOSITORY_URL
                        target="_blank"
                        rel="noopener noreferrer"
                        aria-label="Open the GitHub repository in a new tab"
                    >
                        <Github size=18 />
                    </a>
                </div>
            </div>
        </header>

        <main id="studio" class="studio-shell">
            <div class="studio-intro">
                <h1 class="studio-intro__title">"Design a QR code for print and screen."</h1>
                <p class="studio-intro__copy">
                    "Pick a payload, tune the modules and colours, then export a PNG or SVG at print resolution. Everything runs in this tab."
                </p>
            </div>

            <div class="studio-grid">
                <div class="studio-grid__payload">
                    <PayloadEditor state=state query_info=query_info sanitized=sanitized />
                </div>

                <div class="studio-grid__preview">
                    <PreviewPanel state=state result=preview contrast=contrast sanitized=sanitized />
                </div>

                <div class="studio-grid__style">
                    <ShapePanel state=state />
                    <PalettePanel state=state contrast=contrast />
                    <LogoPanel state=state />
                </div>
            </div>
        </main>

        <footer class="site-footer">
            <div class="site-footer__inner">
                <span>"MIT licensed. Built with Rust and WebAssembly."</span>
                <a
                    href=REPOSITORY_URL
                    target="_blank"
                    rel="noopener noreferrer"
                    class="inline-flex items-center gap-1"
                >
                    "Source"
                    <External size=12 />
                </a>
            </div>
        </footer>

        <MobileDock
            state=state
            result=preview
            contrast=contrast
            sanitized=sanitized
        />
    }
}
