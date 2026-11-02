//! Root component for the studio.

use leptos::prelude::*;

use crate::components::logo_panel::LogoPanel;
use crate::components::palette_panel::PalettePanel;
use crate::components::payload_editor::PayloadEditor;
use crate::components::preview_panel::PreviewPanel;
use crate::components::shape_panel::ShapePanel;
use crate::studio_state::RenderedCode;
use crate::studio_state::StudioState;

/// Root component for the QR studio.
#[component]
pub fn App() -> impl IntoView {
    let state = StudioState::new();
    let payload = Memo::new(move |_| state.payload());
    let query_info = Memo::new(move |_| state.url_query_info());
    let matrix = Memo::new(move |_| state.matrix());
    let rendered = Memo::new(move |_| {
        let matrix = matrix.get()?;
        let svg = state.render_svg(&matrix);
        Ok(RenderedCode { matrix, svg })
    });
    let contrast = Memo::new(move |_| state.contrast_ratios());

    view! {
        <div class="min-h-screen bg-[#000000] text-[#ffffff] pb-16 selection:bg-[#00f0ff]/25 selection:text-[#00f0ff]">
            <main class="w-full max-w-[680px] mx-auto px-4 pt-8 sm:pt-12 space-y-5">
                <div class="text-center space-y-2">
                    <h1 class="text-3xl sm:text-4xl font-bold tracking-tight">
                        <span class="text-[#00f0ff]">"qr."</span>
                        <span class="text-[#ffffff]">"den1zz.dev"</span>
                    </h1>
                    <p class="text-xs sm:text-sm text-[#a0a0a0] max-w-md mx-auto leading-relaxed">
                        "Local, client-side QR studio built in pure Rust & WebAssembly."
                    </p>
                </div>

                <PayloadEditor state=state payload=payload query_info=query_info />
                <PreviewPanel state=state result=rendered contrast=contrast />
                <ShapePanel state=state />
                <PalettePanel state=state />
                <LogoPanel state=state />

                <footer class="w-full max-w-[680px] mx-auto px-4 pt-4 pb-12 text-center text-xs text-[#a0a0a0] space-y-2">
                    <p>"Open source software released under the MIT License."</p>
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
