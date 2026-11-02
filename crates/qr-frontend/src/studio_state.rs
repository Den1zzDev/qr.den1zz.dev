//! Reactive state shared by the studio panels.

use leptos::prelude::*;
use qr_core::contrast;
use qr_core::matrix::EccLevel;
use qr_core::matrix::EncodeError;
use qr_core::matrix::Overlay;
use qr_core::matrix::QrMatrix;
use qr_core::payload;
use qr_core::payload::CryptoCurrency;
use qr_core::payload::Payload;
use qr_core::payload::UrlQueryInfo;
use qr_core::payload::WifiAuth;
use qr_core::vector;
use qr_core::vector::EyeDotShape;
use qr_core::vector::EyeFrameShape;
use qr_core::vector::LogoConfig;
use qr_core::vector::LogoMaskShape;
use qr_core::vector::ModuleShape;
use qr_core::vector::VectorRenderConfig;

use crate::components::glide_select::PayloadTab;

/// URL encoded when the URL tab is empty so that the preview is never blank.
const DEFAULT_URL: &str = "https://den1zz.dev";

/// A rendered code and the SVG that represents it.
#[derive(Debug, Clone, PartialEq)]
pub(crate) struct RenderedCode {
    /// The matrix that was rendered.
    pub(crate) matrix: QrMatrix,
    /// The standalone SVG document.
    pub(crate) svg: String,
}

/// Contrast ratios for the three independently coloured code regions.
#[derive(Debug, Clone, Copy, PartialEq)]
pub(crate) struct ContrastRatios {
    /// Ratio between module colour and background.
    pub(crate) modules: f32,
    /// Ratio between finder frame colour and background.
    pub(crate) eye_frame: f32,
    /// Ratio between finder dot colour and background.
    pub(crate) eye_dot: f32,
}

impl ContrastRatios {
    /// Returns whether every region meets the 3:1 minimum ratio.
    pub(crate) fn is_acceptable(self) -> bool {
        self.modules >= 3.0 && self.eye_frame >= 3.0 && self.eye_dot >= 3.0
    }
}

/// Every reactive value used by the studio.
#[derive(Clone, Copy)]
pub(crate) struct StudioState {
    pub(crate) active_tab: RwSignal<PayloadTab>,
    pub(crate) url_input: RwSignal<String>,
    pub(crate) text_input: RwSignal<String>,
    pub(crate) wifi_ssid: RwSignal<String>,
    pub(crate) wifi_password: RwSignal<String>,
    pub(crate) wifi_auth: RwSignal<WifiAuth>,
    pub(crate) wifi_hidden: RwSignal<bool>,
    pub(crate) vcard_first_name: RwSignal<String>,
    pub(crate) vcard_last_name: RwSignal<String>,
    pub(crate) vcard_phone: RwSignal<String>,
    pub(crate) vcard_email: RwSignal<String>,
    pub(crate) vcard_organization: RwSignal<String>,
    pub(crate) vcard_title: RwSignal<String>,
    pub(crate) vcard_url: RwSignal<String>,
    pub(crate) vcard_note: RwSignal<String>,
    pub(crate) email_to: RwSignal<String>,
    pub(crate) email_subject: RwSignal<String>,
    pub(crate) email_body: RwSignal<String>,
    pub(crate) phone_number: RwSignal<String>,
    pub(crate) geo_latitude: RwSignal<String>,
    pub(crate) geo_longitude: RwSignal<String>,
    pub(crate) geo_query: RwSignal<String>,
    pub(crate) crypto_currency: RwSignal<CryptoCurrency>,
    pub(crate) crypto_address: RwSignal<String>,
    pub(crate) crypto_amount: RwSignal<String>,
    pub(crate) crypto_message: RwSignal<String>,
    pub(crate) module_shape: RwSignal<ModuleShape>,
    pub(crate) eye_frame_shape: RwSignal<EyeFrameShape>,
    pub(crate) eye_dot_shape: RwSignal<EyeDotShape>,
    pub(crate) module_color: RwSignal<String>,
    pub(crate) eye_color: RwSignal<String>,
    pub(crate) dot_color: RwSignal<String>,
    pub(crate) is_transparent_background: RwSignal<bool>,
    pub(crate) background_color: RwSignal<String>,
    pub(crate) logo_data_url: RwSignal<Option<String>>,
    pub(crate) target_resolution: RwSignal<u32>,
    pub(crate) notification: RwSignal<Option<String>>,
}

impl StudioState {
    /// Creates the initial studio state.
    pub(crate) fn new() -> Self {
        Self {
            active_tab: RwSignal::new(PayloadTab::Url),
            url_input: RwSignal::new(String::new()),
            text_input: RwSignal::new(String::new()),
            wifi_ssid: RwSignal::new(String::new()),
            wifi_password: RwSignal::new(String::new()),
            wifi_auth: RwSignal::new(WifiAuth::Wpa),
            wifi_hidden: RwSignal::new(false),
            vcard_first_name: RwSignal::new(String::new()),
            vcard_last_name: RwSignal::new(String::new()),
            vcard_phone: RwSignal::new(String::new()),
            vcard_email: RwSignal::new(String::new()),
            vcard_organization: RwSignal::new(String::new()),
            vcard_title: RwSignal::new(String::new()),
            vcard_url: RwSignal::new(String::new()),
            vcard_note: RwSignal::new(String::new()),
            email_to: RwSignal::new(String::new()),
            email_subject: RwSignal::new(String::new()),
            email_body: RwSignal::new(String::new()),
            phone_number: RwSignal::new(String::new()),
            geo_latitude: RwSignal::new(String::new()),
            geo_longitude: RwSignal::new(String::new()),
            geo_query: RwSignal::new(String::new()),
            crypto_currency: RwSignal::new(CryptoCurrency::Bitcoin),
            crypto_address: RwSignal::new(String::new()),
            crypto_amount: RwSignal::new(String::new()),
            crypto_message: RwSignal::new(String::new()),
            module_shape: RwSignal::new(ModuleShape::Classy),
            eye_frame_shape: RwSignal::new(EyeFrameShape::Rounded),
            eye_dot_shape: RwSignal::new(EyeDotShape::Circle),
            module_color: RwSignal::new("#000000".to_string()),
            eye_color: RwSignal::new("#000000".to_string()),
            dot_color: RwSignal::new("#008b99".to_string()),
            is_transparent_background: RwSignal::new(false),
            background_color: RwSignal::new("#ffffff".to_string()),
            logo_data_url: RwSignal::new(None),
            target_resolution: RwSignal::new(1024),
            notification: RwSignal::new(None),
        }
    }

    /// Returns the payload described by the active tab.
    pub(crate) fn payload(self) -> Payload {
        match self.active_tab.get() {
            PayloadTab::Url => Payload::Url {
                url: self.url_input.get(),
                strip_tracking: false,
            },
            PayloadTab::Text => Payload::Text {
                text: self.text_input.get(),
            },
            PayloadTab::Wifi => Payload::Wifi {
                ssid: self.wifi_ssid.get(),
                password: self.wifi_password.get(),
                auth_type: self.wifi_auth.get(),
                hidden: self.wifi_hidden.get(),
            },
            PayloadTab::VCard => Payload::VCard {
                first_name: self.vcard_first_name.get(),
                last_name: self.vcard_last_name.get(),
                phone: self.vcard_phone.get(),
                email: self.vcard_email.get(),
                organization: self.vcard_organization.get(),
                title: self.vcard_title.get(),
                url: self.vcard_url.get(),
                note: self.vcard_note.get(),
            },
            PayloadTab::Email => Payload::Email {
                to: self.email_to.get(),
                subject: self.email_subject.get(),
                body: self.email_body.get(),
            },
            PayloadTab::Phone => Payload::Phone {
                number: self.phone_number.get(),
            },
            PayloadTab::Geo => Payload::Geo {
                latitude: self.geo_latitude.get().parse().unwrap_or(0.0),
                longitude: self.geo_longitude.get().parse().unwrap_or(0.0),
                query: self.geo_query.get(),
            },
            PayloadTab::Crypto => Payload::Crypto {
                currency: self.crypto_currency.get(),
                address: self.crypto_address.get(),
                amount: self.crypto_amount.get(),
                message: self.crypto_message.get(),
                label: String::new(),
            },
        }
    }

    /// Returns query metadata for the URL tab, or `None` for other tabs.
    pub(crate) fn url_query_info(self) -> Option<UrlQueryInfo> {
        if self.active_tab.get() == PayloadTab::Url {
            payload::inspect_url_query(&self.url_input.get())
        } else {
            None
        }
    }

    /// Encodes the current payload.
    ///
    /// This is deliberately separate from [`render_svg`](Self::render_svg) so
    /// that styling changes do not repeat the encoding work.
    pub(crate) fn matrix(self) -> Result<QrMatrix, EncodeError> {
        let sanitized = self.payload().sanitize();
        let content = if sanitized.content.is_empty() {
            if self.active_tab.get() == PayloadTab::Url {
                DEFAULT_URL
            } else {
                ""
            }
        } else {
            sanitized.content.as_str()
        };
        let overlay = if self.logo_data_url.get().is_some() {
            Overlay::Image
        } else {
            Overlay::None
        };

        QrMatrix::try_new(content, EccLevel::Auto, overlay)
    }

    /// Renders `matrix` with the current style configuration.
    pub(crate) fn render_svg(self, matrix: &QrMatrix) -> String {
        let logo = self.logo_data_url.get().map(|data_url| LogoConfig {
            data_url,
            shape: LogoMaskShape::Rounded,
            scale_percent: 22.0,
        });
        let background = if self.is_transparent_background.get() {
            None
        } else {
            Some(self.background_color.get())
        };
        let config = VectorRenderConfig {
            module_shape: self.module_shape.get(),
            eye_frame_shape: self.eye_frame_shape.get(),
            eye_dot_shape: self.eye_dot_shape.get(),
            code_color: self.module_color.get(),
            eye_color: Some(self.eye_color.get()),
            eye_dot_color: Some(self.dot_color.get()),
            bg_color: background,
            logo,
            ..VectorRenderConfig::default()
        };

        vector::render_svg(matrix, &config)
    }

    /// Returns contrast ratios for the current palette.
    pub(crate) fn contrast_ratios(self) -> ContrastRatios {
        let background_color = self.background_color.get();
        let background = if self.is_transparent_background.get() {
            "#FFFFFF"
        } else {
            background_color.as_str()
        };

        ContrastRatios {
            modules: contrast::contrast_ratio(&self.module_color.get(), background),
            eye_frame: contrast::contrast_ratio(&self.eye_color.get(), background),
            eye_dot: contrast::contrast_ratio(&self.dot_color.get(), background),
        }
    }
}
