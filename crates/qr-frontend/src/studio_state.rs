//! Reactive state shared by the studio panels.

use std::time::Duration;

use leptos::prelude::*;
use qr_core::contrast;
use qr_core::contrast::MIN_GRAPHIC_CONTRAST;
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

/// URL encoded on first load so the studio opens with a working example.
pub(crate) const DEFAULT_URL: &str = "https://example.com";

/// Default module colour.
pub(crate) const DEFAULT_MODULE_COLOR: &str = "#000000";
/// Default finder frame colour.
pub(crate) const DEFAULT_EYE_COLOR: &str = "#000000";
/// Default finder dot colour.
pub(crate) const DEFAULT_DOT_COLOR: &str = "#008b99";
/// Default background colour.
pub(crate) const DEFAULT_BACKGROUND_COLOR: &str = "#ffffff";

/// Parses a latitude and longitude pair, returning `None` when either value is
/// missing, non-finite, or outside its valid range.
pub(crate) fn parse_geo_pair(latitude: &str, longitude: &str) -> Option<(f64, f64)> {
    let latitude = latitude.trim().parse::<f64>().ok()?;
    let longitude = longitude.trim().parse::<f64>().ok()?;
    let in_range = latitude.is_finite()
        && longitude.is_finite()
        && latitude.abs() <= 90.0
        && longitude.abs() <= 180.0;
    in_range.then_some((latitude, longitude))
}

/// Returns a payload that encodes nothing, used when a form state is invalid.
fn empty_payload() -> Payload {
    Payload::Text {
        text: String::new(),
    }
}

/// A rendered code and the SVG that represents it.
#[derive(Debug, Clone, PartialEq)]
pub(crate) struct RenderedCode {
    /// The matrix that was rendered.
    pub(crate) matrix: QrMatrix,
    /// The standalone SVG document.
    pub(crate) svg: String,
}

/// What the preview should show for the current payload.
#[derive(Debug, Clone, PartialEq)]
pub(crate) enum PreviewState {
    /// The payload is empty, so no code is generated.
    Empty,
    /// A code was generated.
    Ready(RenderedCode),
    /// Encoding failed.
    Error(String),
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
    /// Returns whether every region meets the minimum ratio.
    pub(crate) fn is_acceptable(self) -> bool {
        self.modules >= MIN_GRAPHIC_CONTRAST
            && self.eye_frame >= MIN_GRAPHIC_CONTRAST
            && self.eye_dot >= MIN_GRAPHIC_CONTRAST
    }

    /// Returns the number of regions below the minimum ratio.
    pub(crate) fn failures(self) -> usize {
        [self.modules, self.eye_frame, self.eye_dot]
            .into_iter()
            .filter(|ratio| *ratio < MIN_GRAPHIC_CONTRAST)
            .count()
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
    pub(crate) notification: RwSignal<Option<Notification>>,
    notification_sequence: RwSignal<u32>,
}

/// A short message shown under the export controls.
#[derive(Debug, Clone, PartialEq, Eq)]
pub(crate) struct Notification {
    /// Text to show.
    pub(crate) message: String,
    /// Whether the message reports a failure.
    pub(crate) is_error: bool,
}

impl StudioState {
    /// Creates the initial studio state.
    pub(crate) fn new() -> Self {
        Self {
            active_tab: RwSignal::new(PayloadTab::Url),
            url_input: RwSignal::new(DEFAULT_URL.to_string()),
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
            module_color: RwSignal::new(DEFAULT_MODULE_COLOR.to_string()),
            eye_color: RwSignal::new(DEFAULT_EYE_COLOR.to_string()),
            dot_color: RwSignal::new(DEFAULT_DOT_COLOR.to_string()),
            is_transparent_background: RwSignal::new(false),
            background_color: RwSignal::new(DEFAULT_BACKGROUND_COLOR.to_string()),
            logo_data_url: RwSignal::new(None),
            target_resolution: RwSignal::new(1024),
            notification: RwSignal::new(None),
            notification_sequence: RwSignal::new(0),
        }
    }

    /// Shows a status message and schedules its removal.
    ///
    /// Every message gets a sequence number so a late timer cannot clear a
    /// newer message.
    pub(crate) fn notify(self, message: &str, is_error: bool) {
        let next = self.notification_sequence.get_untracked() + 1;
        self.notification_sequence.set(next);
        self.notification.set(Some(Notification {
            message: message.to_string(),
            is_error,
        }));

        let notification = self.notification;
        let sequence = self.notification_sequence;
        set_timeout(
            move || {
                if sequence.get_untracked() == next {
                    notification.set(None);
                }
            },
            Duration::from_millis(2600),
        );
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
            PayloadTab::Geo => {
                let latitude = self.geo_latitude.get();
                let longitude = self.geo_longitude.get();
                let query = self.geo_query.get();
                // An invalid or incomplete pair must not silently encode a
                // clamped or zero coordinate. The field hints explain why.
                match parse_geo_pair(&latitude, &longitude) {
                    Some((latitude, longitude)) => Payload::Geo {
                        latitude,
                        longitude,
                        query,
                    },
                    None => empty_payload(),
                }
            }
            PayloadTab::Crypto => {
                let amount = self.crypto_amount.get();
                // An invalid amount must not produce a code that looks like a
                // payment request while silently dropping the amount.
                let amount_is_valid =
                    amount.trim().is_empty() || payload::normalize_crypto_amount(&amount).is_some();
                if amount_is_valid {
                    Payload::Crypto {
                        currency: self.crypto_currency.get(),
                        address: self.crypto_address.get(),
                        amount,
                        message: self.crypto_message.get(),
                        label: String::new(),
                    }
                } else {
                    empty_payload()
                }
            }
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

    /// Encodes `content`, boosting error correction when a logo is present.
    ///
    /// This is deliberately separate from [`render_svg`](Self::render_svg) so
    /// that styling changes do not repeat the encoding work.
    pub(crate) fn encode(self, content: &str) -> Result<QrMatrix, EncodeError> {
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
            margin_modules: 4,
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

    /// Restores the default palette.
    pub(crate) fn reset_palette(self) {
        self.module_color.set(DEFAULT_MODULE_COLOR.to_string());
        self.eye_color.set(DEFAULT_EYE_COLOR.to_string());
        self.dot_color.set(DEFAULT_DOT_COLOR.to_string());
        self.background_color
            .set(DEFAULT_BACKGROUND_COLOR.to_string());
        self.is_transparent_background.set(false);
    }

    /// Restores the default shapes.
    pub(crate) fn reset_shapes(self) {
        self.module_shape.set(ModuleShape::Classy);
        self.eye_frame_shape.set(EyeFrameShape::Rounded);
        self.eye_dot_shape.set(EyeDotShape::Circle);
    }
}

#[cfg(test)]
mod tests {
    use super::parse_geo_pair;

    #[test]
    fn parses_coordinates_inside_their_ranges() {
        assert_eq!(
            parse_geo_pair("41.0082", "28.9784"),
            Some((41.0082, 28.9784))
        );
        assert_eq!(parse_geo_pair("-90", "180"), Some((-90.0, 180.0)));
    }

    #[test]
    fn rejects_missing_and_out_of_range_coordinates() {
        assert_eq!(parse_geo_pair("", "28.9784"), None);
        assert_eq!(parse_geo_pair("41.0082", ""), None);
        assert_eq!(parse_geo_pair("90.1", "0"), None);
        assert_eq!(parse_geo_pair("0", "-180.1"), None);
        assert_eq!(parse_geo_pair("north", "0"), None);
    }
}
