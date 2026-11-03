//! Inline icon set shared by the studio panels.
//!
//! Every icon uses a 24-unit view box, no fill, round caps and joins, and a
//! 1.6-unit stroke so the set stays visually consistent at 12–24 pixels.

use leptos::prelude::*;
use qr_core::vector::EyeDotShape;
use qr_core::vector::EyeFrameShape;
use qr_core::vector::ModuleShape;

use crate::components::glide_select::PayloadTab;

/// Wraps icon paths in a consistently configured SVG element.
macro_rules! icon_svg {
    ($size:expr, $body:expr) => {
        view! {
            <svg
                width=$size
                height=$size
                viewBox="0 0 24 24"
                fill="none"
                stroke="currentColor"
                stroke-width="1.6"
                stroke-linecap="round"
                stroke-linejoin="round"
                aria-hidden="true"
                focusable="false"
            >
                {$body}
            </svg>
        }
    };
}

/// Chevron used by disclosure controls.
#[component]
pub(crate) fn ChevronDown(#[prop(default = 14)] size: u16) -> impl IntoView {
    icon_svg!(size, view! { <polyline points="6 9 12 15 18 9" /> })
}

/// Check mark used for selected options and confirmations.
#[component]
pub(crate) fn Check(#[prop(default = 14)] size: u16) -> impl IntoView {
    icon_svg!(size, view! { <polyline points="20 6 9 17 4 12" /> })
}

/// Close mark used by clear buttons.
#[component]
pub(crate) fn Close(#[prop(default = 14)] size: u16) -> impl IntoView {
    icon_svg!(
        size,
        view! {
            <line x1="18" y1="6" x2="6" y2="18" />
            <line x1="6" y1="6" x2="18" y2="18" />
        }
    )
}

/// Download arrow.
#[component]
pub(crate) fn Download(#[prop(default = 16)] size: u16) -> impl IntoView {
    icon_svg!(
        size,
        view! {
            <path d="M12 3v12" />
            <path d="m7 11 5 5 5-5" />
            <path d="M4 20h16" />
        }
    )
}

/// Copy action.
#[component]
pub(crate) fn Copy(#[prop(default = 16)] size: u16) -> impl IntoView {
    icon_svg!(
        size,
        view! {
            <rect x="9" y="9" width="11" height="11" rx="2" />
            <path d="M15 5.5A2.5 2.5 0 0 0 12.5 3H6a2 2 0 0 0-2 2v6.5A2.5 2.5 0 0 0 6.5 14" />
        }
    )
}

/// Warning triangle for error and contrast states.
#[component]
pub(crate) fn Warning(#[prop(default = 14)] size: u16) -> impl IntoView {
    icon_svg!(
        size,
        view! {
            <path d="M12 4 2.5 20h19L12 4Z" />
            <path d="M12 10v4.5" />
            <path d="M12 17.6v.1" />
        }
    )
}

/// Empty-state mark.
#[component]
pub(crate) fn QrFrame(#[prop(default = 20)] size: u16) -> impl IntoView {
    icon_svg!(
        size,
        view! {
            <rect x="3" y="3" width="7" height="7" rx="1" />
            <rect x="14" y="3" width="7" height="7" rx="1" />
            <rect x="3" y="14" width="7" height="7" rx="1" />
            <path d="M14 14h3v3h-3zM19 19h2v2h-2z" />
        }
    )
}

/// Colour picker.
#[component]
pub(crate) fn Pipette(#[prop(default = 12)] size: u16) -> impl IntoView {
    icon_svg!(
        size,
        view! {
            <path d="m14.5 4.5 5 5" />
            <path d="M17 2.5 21.5 7l-9.8 9.8-4.5-4.5L17 2.5Z" />
            <path d="m7 13 4 4" />
            <path d="M5 17.5 3.5 21l3.5-1.5" />
        }
    )
}

/// Upload action.
#[component]
pub(crate) fn Upload(#[prop(default = 16)] size: u16) -> impl IntoView {
    icon_svg!(
        size,
        view! {
            <path d="M12 17V5" />
            <path d="m7 10 5-5 5 5" />
            <path d="M4 20h16" />
        }
    )
}

/// External link mark.
#[component]
pub(crate) fn External(#[prop(default = 14)] size: u16) -> impl IntoView {
    icon_svg!(
        size,
        view! {
            <path d="M14 4h6v6" />
            <path d="M20 4 11 13" />
            <path d="M19 14v5a1 1 0 0 1-1 1H5a1 1 0 0 1-1-1V6a1 1 0 0 1 1-1h5" />
        }
    )
}

/// GitHub mark. This one is filled rather than stroked, matching the brand.
#[component]
pub(crate) fn Github(#[prop(default = 18)] size: u16) -> impl IntoView {
    view! {
        <svg
            width=size
            height=size
            viewBox="0 0 24 24"
            fill="currentColor"
            aria-hidden="true"
            focusable="false"
        >
            <path d="M12 2C6.48 2 2 6.58 2 12.26c0 4.53 2.87 8.37 6.84 9.73.5.1.68-.22.68-.49 0-.24-.01-1.04-.01-1.88-2.78.62-3.37-1.21-3.37-1.21-.45-1.18-1.11-1.5-1.11-1.5-.91-.63.07-.62.07-.62 1 .07 1.53 1.06 1.53 1.06.9 1.57 2.35 1.12 2.92.86.09-.66.35-1.12.63-1.38-2.22-.26-4.56-1.14-4.56-5.07 0-1.12.39-2.03 1.03-2.75-.1-.26-.45-1.3.1-2.71 0 0 .84-.27 2.75 1.05a9.32 9.32 0 0 1 5 0c1.91-1.32 2.75-1.05 2.75-1.05.55 1.41.2 2.45.1 2.71.64.72 1.03 1.63 1.03 2.75 0 3.94-2.34 4.81-4.57 5.06.36.32.68.94.68 1.9 0 1.37-.01 2.47-.01 2.81 0 .27.18.6.69.49A10.26 10.26 0 0 0 22 12.26C22 6.58 17.52 2 12 2Z" />
        </svg>
    }
}

/// Link payload type.
#[component]
pub(crate) fn LinkIcon(#[prop(default = 16)] size: u16) -> impl IntoView {
    icon_svg!(
        size,
        view! {
            <path d="M10.5 13.5a4 4 0 0 0 5.66 0l2.83-2.83a4 4 0 1 0-5.66-5.66L11.9 6.4" />
            <path d="M13.5 10.5a4 4 0 0 0-5.66 0L5 13.34a4 4 0 1 0 5.66 5.66l1.43-1.43" />
        }
    )
}

/// Raw text payload type.
#[component]
pub(crate) fn TextIcon(#[prop(default = 16)] size: u16) -> impl IntoView {
    icon_svg!(
        size,
        view! {
            <path d="M4 6h16" />
            <path d="M4 12h12" />
            <path d="M4 18h8" />
        }
    )
}

/// Wi-Fi payload type.
#[component]
pub(crate) fn WifiIcon(#[prop(default = 16)] size: u16) -> impl IntoView {
    icon_svg!(
        size,
        view! {
            <path d="M2.5 8.5a14 14 0 0 1 19 0" />
            <path d="M5.5 12a10 10 0 0 1 13 0" />
            <path d="M8.5 15.5a6 6 0 0 1 7 0" />
            <path d="M12 19h.01" />
        }
    )
}

/// Contact card payload type.
#[component]
pub(crate) fn ContactIcon(#[prop(default = 16)] size: u16) -> impl IntoView {
    icon_svg!(
        size,
        view! {
            <rect x="3" y="4" width="18" height="16" rx="2" />
            <circle cx="9" cy="10" r="2" />
            <path d="M6 16.5a3.5 3.5 0 0 1 6 0" />
            <path d="M15 9h3M15 13h3" />
        }
    )
}

/// Email payload type.
#[component]
pub(crate) fn MailIcon(#[prop(default = 16)] size: u16) -> impl IntoView {
    icon_svg!(
        size,
        view! {
            <rect x="3" y="5" width="18" height="14" rx="2" />
            <path d="m3.5 6.5 8.5 6 8.5-6" />
        }
    )
}

/// Phone payload type.
#[component]
pub(crate) fn PhoneIcon(#[prop(default = 16)] size: u16) -> impl IntoView {
    icon_svg!(
        size,
        view! {
            <path d="M6.2 3h3l1.4 4.2-2 1.4a12 12 0 0 0 5.8 5.8l1.4-2L20 13.8v3a2.2 2.2 0 0 1-2.4 2.2A16 16 0 0 1 4 5.4 2.2 2.2 0 0 1 6.2 3Z" />
        }
    )
}

/// Location payload type.
#[component]
pub(crate) fn MapPinIcon(#[prop(default = 16)] size: u16) -> impl IntoView {
    icon_svg!(
        size,
        view! {
            <path d="M12 21s-6.5-5.6-6.5-11a6.5 6.5 0 0 1 13 0C18.5 15.4 12 21 12 21Z" />
            <circle cx="12" cy="10" r="2.3" />
        }
    )
}

/// Cryptocurrency payload type.
#[component]
pub(crate) fn WalletIcon(#[prop(default = 16)] size: u16) -> impl IntoView {
    icon_svg!(
        size,
        view! {
            <rect x="3" y="6" width="18" height="13" rx="2" />
            <path d="M3 9h18" />
            <path d="M16.5 14.5h2" />
            <path d="M6 6V5a2 2 0 0 1 2-2h9" />
        }
    )
}

/// Returns the icon for a payload type.
#[component]
pub(crate) fn PayloadIcon(tab: PayloadTab, #[prop(default = 16)] size: u16) -> impl IntoView {
    match tab {
        PayloadTab::Url => view! { <LinkIcon size=size /> }.into_any(),
        PayloadTab::Text => view! { <TextIcon size=size /> }.into_any(),
        PayloadTab::Wifi => view! { <WifiIcon size=size /> }.into_any(),
        PayloadTab::VCard => view! { <ContactIcon size=size /> }.into_any(),
        PayloadTab::Email => view! { <MailIcon size=size /> }.into_any(),
        PayloadTab::Phone => view! { <PhoneIcon size=size /> }.into_any(),
        PayloadTab::Geo => view! { <MapPinIcon size=size /> }.into_any(),
        PayloadTab::Crypto => view! { <WalletIcon size=size /> }.into_any(),
    }
}

/// Miniature preview of a module shape.
#[component]
pub(crate) fn ModuleShapeGlyph(shape: ModuleShape) -> impl IntoView {
    let path = "M2 11V6.5A4.5 4.5 0 0 1 6.5 2H11v9Z";
    match shape {
        ModuleShape::Square => view! {
            <svg width="24" height="24" viewBox="0 0 24 24" aria-hidden="true">
                <g fill="currentColor">
                    <rect x="2" y="2" width="9" height="9" />
                    <rect x="13" y="2" width="9" height="9" />
                    <rect x="2" y="13" width="9" height="9" />
                    <rect x="13" y="13" width="9" height="9" />
                </g>
            </svg>
        }
        .into_any(),
        ModuleShape::Dots => view! {
            <svg width="24" height="24" viewBox="0 0 24 24" aria-hidden="true">
                <g fill="currentColor">
                    <circle cx="6.5" cy="6.5" r="4.2" />
                    <circle cx="17.5" cy="6.5" r="4.2" />
                    <circle cx="6.5" cy="17.5" r="4.2" />
                    <circle cx="17.5" cy="17.5" r="4.2" />
                </g>
            </svg>
        }
        .into_any(),
        ModuleShape::Smooth => view! {
            <svg width="24" height="24" viewBox="0 0 24 24" aria-hidden="true">
                <g fill="currentColor">
                    <rect x="2" y="2" width="9" height="9" rx="3" />
                    <rect x="13" y="2" width="9" height="9" rx="3" />
                    <rect x="2" y="13" width="9" height="9" rx="3" />
                    <rect x="13" y="13" width="9" height="9" rx="3" />
                </g>
            </svg>
        }
        .into_any(),
        ModuleShape::Classy => view! {
            <svg width="24" height="24" viewBox="0 0 24 24" aria-hidden="true">
                <g fill="currentColor">
                    <path d={path} />
                    <path d={path} transform="rotate(90 12 12)" />
                    <path d={path} transform="rotate(180 12 12)" />
                    <path d={path} transform="rotate(270 12 12)" />
                </g>
            </svg>
        }
        .into_any(),
        _ => view! {
            <svg width="24" height="24" viewBox="0 0 24 24" aria-hidden="true">
                <rect x="3" y="3" width="7" height="7" fill="currentColor" />
                <rect x="14" y="3" width="7" height="7" fill="currentColor" />
                <rect x="3" y="14" width="7" height="7" fill="currentColor" />
                <rect x="14" y="14" width="7" height="7" fill="currentColor" />
            </svg>
        }
        .into_any(),
    }
}

/// Miniature preview of a finder frame shape.
#[component]
pub(crate) fn EyeFrameGlyph(shape: EyeFrameShape) -> impl IntoView {
    match shape {
        EyeFrameShape::Square => view! {
            <svg width="24" height="24" viewBox="0 0 24 24" aria-hidden="true">
                <rect x="4" y="4" width="16" height="16" fill="none" stroke="currentColor" stroke-width="3.5" />
            </svg>
        }
        .into_any(),
        EyeFrameShape::Rounded => view! {
            <svg width="24" height="24" viewBox="0 0 24 24" aria-hidden="true">
                <rect x="4" y="4" width="16" height="16" rx="5" fill="none" stroke="currentColor" stroke-width="3.5" />
            </svg>
        }
        .into_any(),
        EyeFrameShape::Circle => view! {
            <svg width="24" height="24" viewBox="0 0 24 24" aria-hidden="true">
                <circle cx="12" cy="12" r="8" fill="none" stroke="currentColor" stroke-width="3.5" />
            </svg>
        }
        .into_any(),
        _ => view! {
            <svg width="24" height="24" viewBox="0 0 24 24" aria-hidden="true">
                <rect x="4" y="4" width="16" height="16" fill="none" stroke="currentColor" stroke-width="3.5" />
            </svg>
        }
        .into_any(),
    }
}

/// Miniature preview of a finder dot shape.
#[component]
pub(crate) fn EyeDotGlyph(shape: EyeDotShape) -> impl IntoView {
    match shape {
        EyeDotShape::Square => view! {
            <svg width="24" height="24" viewBox="0 0 24 24" aria-hidden="true">
                <rect x="7" y="7" width="10" height="10" fill="currentColor" />
            </svg>
        }
        .into_any(),
        EyeDotShape::Rounded => view! {
            <svg width="24" height="24" viewBox="0 0 24 24" aria-hidden="true">
                <rect x="6" y="6" width="12" height="12" rx="4" fill="currentColor" />
            </svg>
        }
        .into_any(),
        EyeDotShape::Circle => view! {
            <svg width="24" height="24" viewBox="0 0 24 24" aria-hidden="true">
                <circle cx="12" cy="12" r="6" fill="currentColor" />
            </svg>
        }
        .into_any(),
        _ => view! {
            <svg width="24" height="24" viewBox="0 0 24 24" aria-hidden="true">
                <rect x="7" y="7" width="10" height="10" fill="currentColor" />
            </svg>
        }
        .into_any(),
    }
}
