//! Payload builders that produce scannable payload strings.
//!
//! Each [`Payload`] variant knows how to turn user input into the byte string
//! that will be encoded in a QR code. Sanitisation trims values, escapes
//! reserved characters, and optionally removes tracking parameters from URLs.

use std::fmt::Write as _;

use serde::Deserialize;
use serde::Serialize;
use url::Url;

/// Cryptocurrency supported by the payment payload builder.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[non_exhaustive]
pub enum CryptoCurrency {
    /// Bitcoin.
    Bitcoin,
    /// Ethereum.
    Ethereum,
    /// Solana.
    Solana,
    /// Monero.
    Monero,
    /// Litecoin.
    Litecoin,
}

impl CryptoCurrency {
    /// Every supported currency, in display order.
    pub const ALL: [Self; 5] = [
        Self::Bitcoin,
        Self::Ethereum,
        Self::Solana,
        Self::Monero,
        Self::Litecoin,
    ];

    /// Returns the human-readable name and ticker.
    #[must_use]
    pub fn label(&self) -> &'static str {
        match self {
            Self::Bitcoin => "Bitcoin (BTC)",
            Self::Ethereum => "Ethereum (ETH)",
            Self::Solana => "Solana (SOL)",
            Self::Monero => "Monero (XMR)",
            Self::Litecoin => "Litecoin (LTC)",
        }
    }

    /// Returns the URI scheme used by payment requests.
    #[must_use]
    pub fn scheme(&self) -> &'static str {
        match self {
            Self::Bitcoin => "bitcoin",
            Self::Ethereum => "ethereum",
            Self::Solana => "solana",
            Self::Monero => "monero",
            Self::Litecoin => "litecoin",
        }
    }
}

/// A payload that can be encoded in a QR code.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(tag = "type", rename_all = "lowercase")]
#[non_exhaustive]
pub enum Payload {
    /// A web address, optionally stripped of tracking parameters.
    Url {
        /// The URL to encode.
        url: String,
        /// Whether known tracking parameters are removed before encoding.
        strip_tracking: bool,
    },
    /// Arbitrary text.
    Text {
        /// The text to encode.
        text: String,
    },
    /// Credentials for joining a Wi-Fi network.
    Wifi {
        /// Network name (SSID).
        ssid: String,
        /// Network password. Ignored when `auth_type` is [`WifiAuth::Nopass`].
        password: String,
        /// Authentication scheme.
        auth_type: WifiAuth,
        /// Whether the network hides its SSID.
        #[serde(default)]
        hidden: bool,
    },
    /// A contact card in vCard 3.0 format.
    VCard {
        /// Given name.
        #[serde(default)]
        first_name: String,
        /// Family name.
        #[serde(default)]
        last_name: String,
        /// Telephone number.
        #[serde(default)]
        phone: String,
        /// Email address.
        #[serde(default)]
        email: String,
        /// Organisation name.
        #[serde(default)]
        organization: String,
        /// Job title.
        #[serde(default)]
        title: String,
        /// Website address.
        #[serde(default)]
        url: String,
        /// Free-form note.
        #[serde(default)]
        note: String,
    },
    /// A pre-filled email draft.
    Email {
        /// Recipient address.
        to: String,
        /// Subject line.
        #[serde(default)]
        subject: String,
        /// Message body.
        #[serde(default)]
        body: String,
    },
    /// A telephone number.
    Phone {
        /// Number in RFC 3966 form.
        number: String,
    },
    /// Geographic coordinates.
    Geo {
        /// Latitude in degrees.
        latitude: f64,
        /// Longitude in degrees.
        longitude: f64,
        /// Optional place name.
        #[serde(default)]
        query: String,
    },
    /// A cryptocurrency payment request.
    Crypto {
        /// Currency of the request.
        currency: CryptoCurrency,
        /// Wallet address.
        address: String,
        /// Requested amount as a decimal string.
        #[serde(default)]
        amount: String,
        /// Payment description.
        #[serde(default)]
        message: String,
        /// Recipient label.
        #[serde(default)]
        label: String,
    },
}

/// Wi-Fi authentication scheme.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "UPPERCASE")]
#[non_exhaustive]
pub enum WifiAuth {
    /// WPA, WPA2, or WPA3.
    Wpa,
    /// WEP, retained for legacy networks.
    Wep,
    /// No authentication.
    Nopass,
}

impl WifiAuth {
    /// Every supported authentication scheme, in display order.
    pub const ALL: [Self; 3] = [Self::Wpa, Self::Wep, Self::Nopass];

    /// Returns the human-readable name.
    #[must_use]
    pub fn label(&self) -> &'static str {
        match self {
            Self::Wpa => "WPA / WPA2 / WPA3",
            Self::Wep => "WEP (Legacy)",
            Self::Nopass => "None (Open)",
        }
    }
}

/// A payload string together with byte-count metadata.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[non_exhaustive]
pub struct SanitizedPayload {
    /// Encoded payload string ready for a QR code.
    pub content: String,
    /// Byte count of the input before sanitisation.
    pub raw_bytes: usize,
    /// Byte count of [`content`](Self::content).
    pub cleaned_bytes: usize,
    /// Number of query parameters removed from a URL.
    pub stripped_params_count: usize,
}

/// Summary of the query parameters found in a URL.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[non_exhaustive]
pub struct UrlQueryInfo {
    /// Always `true` for values returned by [`inspect_url_query`].
    pub has_query: bool,
    /// Total number of query parameters.
    pub total_param_count: usize,
    /// Number of parameters recognised as tracking tags.
    pub tracking_param_count: usize,
    /// The URL with tracking parameters removed.
    pub stripped_tracking_url: String,
    /// The URL with every query parameter removed.
    pub stripped_all_url: String,
    /// Byte length of the original input.
    pub original_len: usize,
    /// Byte length of [`stripped_tracking_url`](Self::stripped_tracking_url).
    pub tracking_cleaned_len: usize,
    /// Byte length of [`stripped_all_url`](Self::stripped_all_url).
    pub all_cleaned_len: usize,
}

/// A URL after optional removal of tracking parameters.
#[derive(Debug, Clone, PartialEq, Eq)]
#[non_exhaustive]
pub struct CleanedUrl {
    /// The cleaned URL.
    pub url: String,
    /// Number of query parameters that were removed.
    pub removed_count: usize,
}

/// Tracking parameters that are matched exactly, case-insensitively.
const TRACKING_EXACT: &[&str] = &[
    "fbclid",
    "gclid",
    "gbraid",
    "wbraid",
    "mc_cid",
    "mc_eid",
    "_ga",
    "_gl",
    "msclkid",
    "twclid",
    "ref",
    "ref_",
    "ref_src",
    "ref_url",
    "source",
    "source_id",
    "igshid",
    "ysclid",
    "sc_cid",
    "ttclid",
    "rdid",
    "dclid",
    "_hsenc",
    "_hsmi",
    "hsctatracking",
    "vero_id",
    "vero_conv",
    "nr_email_referer",
    "si",
    "feature",
    "tag",
    "spm",
    "scm",
    "qid",
    "sr",
    "dchild",
    "ved",
    "ei",
    "usg",
];

/// Tracking parameter prefixes, matched case-insensitively.
const TRACKING_PREFIXES: &[&str] = &["utm_", "pd_rd_", "pf_rd_", "at_", "itm_"];

/// Returns whether `name` is a known tracking query parameter.
#[must_use]
pub fn is_tracking_param(name: &str) -> bool {
    let lower_name = name.to_ascii_lowercase();
    TRACKING_PREFIXES
        .iter()
        .any(|prefix| lower_name.starts_with(prefix))
        || TRACKING_EXACT.contains(&lower_name.as_str())
}

/// Removes every query parameter from `raw`, keeping the fragment.
#[must_use]
pub fn strip_all_query(raw: &str) -> String {
    let trimmed = raw.trim();
    let Some(query_index) = trimmed.find('?') else {
        return trimmed.to_string();
    };

    let before_query = &trimmed[..query_index];
    match trimmed[query_index..].find('#') {
        Some(fragment_offset) => {
            let fragment = &trimmed[query_index + fragment_offset..];
            format!("{before_query}{fragment}")
        }
        None => before_query.to_string(),
    }
}

/// Removes known tracking parameters from `raw`.
///
/// The URL is returned unchanged when it has no query string, cannot be parsed,
/// or contains no tracking parameters.
#[must_use]
pub fn strip_tracking_query(raw: &str) -> CleanedUrl {
    let trimmed = raw.trim();
    if !trimmed.contains('?') {
        return CleanedUrl {
            url: trimmed.to_string(),
            removed_count: 0,
        };
    }

    let has_scheme = trimmed.contains("://");
    let to_parse = if has_scheme {
        trimmed.to_string()
    } else {
        format!("https://{trimmed}")
    };

    let Ok(mut parsed) = Url::parse(&to_parse) else {
        return CleanedUrl {
            url: trimmed.to_string(),
            removed_count: 0,
        };
    };

    let pairs: Vec<(String, String)> = parsed.query_pairs().into_owned().collect();
    let mut removed_count = 0;
    let filtered: Vec<(String, String)> = pairs
        .into_iter()
        .filter(|(name, _)| {
            if is_tracking_param(name) {
                removed_count += 1;
                false
            } else {
                true
            }
        })
        .collect();

    if removed_count == 0 {
        return CleanedUrl {
            url: trimmed.to_string(),
            removed_count: 0,
        };
    }

    if filtered.is_empty() {
        parsed.set_query(None);
    } else {
        let mut query = String::new();
        for (index, (name, value)) in filtered.iter().enumerate() {
            if index > 0 {
                query.push('&');
            }
            query.push_str(name);
            if !value.is_empty() {
                query.push('=');
                query.push_str(value);
            }
        }
        parsed.set_query(Some(&query));
    }

    let cleaned = parsed.to_string();
    let without_scheme = cleaned.strip_prefix("https://").map(str::to_string);
    let url = if has_scheme {
        cleaned
    } else {
        without_scheme.unwrap_or(cleaned)
    };

    CleanedUrl { url, removed_count }
}

/// Describes the query parameters of `raw`, or returns `None` when it has none.
#[must_use]
pub fn inspect_url_query(raw: &str) -> Option<UrlQueryInfo> {
    let trimmed = raw.trim();
    if !trimmed.contains('?') {
        return None;
    }

    let tracking_cleaned = strip_tracking_query(trimmed);
    let stripped_all = strip_all_query(trimmed);
    let has_scheme = trimmed.contains("://");
    let to_parse = if has_scheme {
        trimmed.to_string()
    } else {
        format!("https://{trimmed}")
    };

    let total_params = Url::parse(&to_parse)
        .map(|url| url.query_pairs().count())
        .unwrap_or(0);
    if total_params == 0 {
        return None;
    }

    let tracking_cleaned_len = tracking_cleaned.url.len();
    let all_cleaned_len = stripped_all.len();
    Some(UrlQueryInfo {
        has_query: true,
        total_param_count: total_params,
        tracking_param_count: tracking_cleaned.removed_count,
        stripped_tracking_url: tracking_cleaned.url,
        stripped_all_url: stripped_all,
        original_len: trimmed.len(),
        tracking_cleaned_len,
        all_cleaned_len,
    })
}

/// Trims a user-entered URL and adds `https://` when no scheme is present.
#[must_use]
pub fn clean_url(raw: &str) -> String {
    let trimmed = raw.trim();
    if trimmed.is_empty() {
        return String::new();
    }
    if trimmed.contains("://") {
        trimmed.to_string()
    } else {
        format!("https://{trimmed}")
    }
}

/// Appends the RFC 3986 percent-encoded form of `input` to `out`.
///
/// Every byte except the unreserved set (`ALPHA`, `DIGIT`, `-`, `.`, `_`, `~`)
/// is encoded as `%XX`.
pub fn percent_encode_rfc3986(input: &str, out: &mut String) {
    for byte in input.bytes() {
        match byte {
            b'a'..=b'z' | b'A'..=b'Z' | b'0'..=b'9' | b'-' | b'.' | b'_' | b'~' => {
                out.push(char::from(byte));
            }
            _ => {
                let _ = write!(out, "%{byte:02X}");
            }
        }
    }
}

/// Normalises a cryptocurrency amount without floating-point arithmetic.
///
/// Rejects negative values, explicit signs, scientific notation, and non-numeric
/// characters. Leading integer zeros and trailing fractional zeros are removed.
/// Returns `None` when `raw` is not a valid positive decimal amount.
#[must_use]
pub fn normalize_crypto_amount(raw: &str) -> Option<String> {
    let trimmed = raw.trim();
    if trimmed.is_empty() || trimmed.contains(['e', 'E']) {
        return None;
    }
    if trimmed.starts_with('-') || trimmed.starts_with('+') {
        return None;
    }

    let (integer_raw, fraction_raw) = match trimmed.split_once('.') {
        Some((integer, fraction)) => {
            if fraction.contains('.') {
                return None;
            }
            (integer, Some(fraction))
        }
        None => (trimmed, None),
    };

    if !integer_raw.bytes().all(|byte| byte.is_ascii_digit()) {
        return None;
    }

    let integer = match integer_raw.trim_start_matches('0') {
        "" => "0",
        stripped => stripped,
    };

    let Some(fraction_raw) = fraction_raw else {
        return Some(integer.to_string());
    };
    if !fraction_raw.bytes().all(|byte| byte.is_ascii_digit()) {
        return None;
    }

    let fraction = fraction_raw.trim_end_matches('0');
    if fraction.is_empty() {
        return Some(integer.to_string());
    }

    let mut amount = String::with_capacity(integer.len() + 1 + fraction.len());
    amount.push_str(integer);
    amount.push('.');
    amount.push_str(fraction);
    Some(amount)
}

/// Appends `value` to `out`, escaping Wi-Fi reserved characters.
///
/// Backslash, semicolon, comma, colon, and double quote are prefixed with a
/// backslash, as required by the ZXing barcode specification.
pub fn write_escaped_wifi(value: &str, out: &mut String) {
    for character in value.chars() {
        if matches!(character, '\\' | ';' | ',' | ':' | '"') {
            out.push('\\');
        }
        out.push(character);
    }
}

/// Returns `value` with Wi-Fi reserved characters escaped.
#[must_use]
pub fn escape_wifi_value(value: &str) -> String {
    let mut escaped = String::with_capacity(value.len() + 4);
    write_escaped_wifi(value, &mut escaped);
    escaped
}

/// Appends `value` to `out`, escaping vCard 3.0 reserved characters.
///
/// Backslash, semicolon, comma, and newlines are escaped; carriage returns are
/// dropped.
pub fn write_escaped_vcard(value: &str, out: &mut String) {
    for character in value.chars() {
        match character {
            '\\' => out.push_str("\\\\"),
            ';' => out.push_str("\\;"),
            ',' => out.push_str("\\,"),
            '\n' => out.push_str("\\n"),
            '\r' => {}
            other => out.push(other),
        }
    }
}

/// Returns `value` with vCard 3.0 reserved characters escaped.
#[must_use]
pub fn escape_vcard_value(value: &str) -> String {
    let mut escaped = String::with_capacity(value.len() + 4);
    write_escaped_vcard(value, &mut escaped);
    escaped
}

/// Appends `coordinate` with up to six decimal places and no trailing zeros.
fn write_clean_coordinate(coordinate: f64, out: &mut String) {
    if !coordinate.is_finite() {
        out.push('0');
        return;
    }

    let formatted = format!("{coordinate:.6}");
    if formatted.contains('.') {
        out.push_str(formatted.trim_end_matches('0').trim_end_matches('.'));
    } else {
        out.push_str(&formatted);
    }
}

/// Wraps a finished payload string in [`SanitizedPayload`].
fn from_content(content: String) -> SanitizedPayload {
    let bytes = content.len();
    SanitizedPayload {
        content,
        raw_bytes: bytes,
        cleaned_bytes: bytes,
        stripped_params_count: 0,
    }
}

/// Returns an empty [`SanitizedPayload`] for inputs that cannot be encoded.
fn empty_payload() -> SanitizedPayload {
    SanitizedPayload {
        content: String::new(),
        raw_bytes: 0,
        cleaned_bytes: 0,
        stripped_params_count: 0,
    }
}

impl Payload {
    /// Builds the sanitised payload string for this payload.
    #[must_use]
    pub fn sanitize(&self) -> SanitizedPayload {
        match self {
            Self::Url {
                url,
                strip_tracking,
            } => {
                let cleaned_url = clean_url(url);
                let cleaned = if *strip_tracking {
                    strip_tracking_query(&cleaned_url)
                } else {
                    CleanedUrl {
                        url: cleaned_url,
                        removed_count: 0,
                    }
                };
                let cleaned_bytes = cleaned.url.len();
                SanitizedPayload {
                    content: cleaned.url,
                    raw_bytes: url.trim().len(),
                    cleaned_bytes,
                    stripped_params_count: cleaned.removed_count,
                }
            }
            Self::Text { text } => {
                let content = text.trim();
                let length = content.len();
                SanitizedPayload {
                    content: content.to_string(),
                    raw_bytes: length,
                    cleaned_bytes: length,
                    stripped_params_count: 0,
                }
            }
            Self::Wifi {
                ssid,
                password,
                auth_type,
                hidden,
            } => {
                let ssid = ssid.trim();
                if ssid.is_empty() {
                    return empty_payload();
                }

                let auth = match auth_type {
                    WifiAuth::Wpa => "WPA",
                    WifiAuth::Wep => "WEP",
                    WifiAuth::Nopass => "nopass",
                };
                let password = password.trim();

                let mut content = String::with_capacity(32 + ssid.len() * 2 + password.len() * 2);
                content.push_str("WIFI:S:");
                write_escaped_wifi(ssid, &mut content);
                content.push_str(";T:");
                content.push_str(auth);
                content.push(';');
                if *auth_type != WifiAuth::Nopass && !password.is_empty() {
                    content.push_str("P:");
                    write_escaped_wifi(password, &mut content);
                    content.push(';');
                }
                if *hidden {
                    content.push_str("H:true;");
                }
                content.push(';');
                from_content(content)
            }
            Self::VCard {
                first_name,
                last_name,
                phone,
                email,
                organization,
                title,
                url,
                note,
            } => {
                let first_name = first_name.trim();
                let last_name = last_name.trim();
                let phone = phone.trim();
                let email = email.trim();
                let organization = organization.trim();
                let title = title.trim();
                let url = url.trim();
                let note = note.trim();

                let all_empty = first_name.is_empty()
                    && last_name.is_empty()
                    && phone.is_empty()
                    && email.is_empty()
                    && organization.is_empty()
                    && title.is_empty()
                    && url.is_empty()
                    && note.is_empty();
                if all_empty {
                    return empty_payload();
                }

                let estimated_capacity = 64
                    + first_name.len() * 2
                    + last_name.len() * 2
                    + phone.len() * 2
                    + email.len() * 2
                    + organization.len() * 2
                    + title.len() * 2
                    + url.len() * 2
                    + note.len() * 2;
                let mut content = String::with_capacity(estimated_capacity);
                content.push_str("BEGIN:VCARD\nVERSION:3.0\n");

                if !first_name.is_empty() || !last_name.is_empty() {
                    content.push_str("FN:");
                    if first_name.is_empty() {
                        write_escaped_vcard(last_name, &mut content);
                    } else {
                        write_escaped_vcard(first_name, &mut content);
                        if !last_name.is_empty() {
                            content.push(' ');
                            write_escaped_vcard(last_name, &mut content);
                        }
                    }
                    content.push('\n');

                    content.push_str("N:");
                    write_escaped_vcard(last_name, &mut content);
                    content.push(';');
                    write_escaped_vcard(first_name, &mut content);
                    content.push_str(";;;\n");
                }

                if !phone.is_empty() {
                    content.push_str("TEL;TYPE=CELL:");
                    write_escaped_vcard(phone, &mut content);
                    content.push('\n');
                }
                if !email.is_empty() {
                    content.push_str("EMAIL:");
                    write_escaped_vcard(email, &mut content);
                    content.push('\n');
                }
                if !organization.is_empty() {
                    content.push_str("ORG:");
                    write_escaped_vcard(organization, &mut content);
                    content.push('\n');
                }
                if !title.is_empty() {
                    content.push_str("TITLE:");
                    write_escaped_vcard(title, &mut content);
                    content.push('\n');
                }
                if !url.is_empty() {
                    content.push_str("URL:");
                    if !url.contains("://") {
                        content.push_str("https://");
                    }
                    write_escaped_vcard(url, &mut content);
                    content.push('\n');
                }
                if !note.is_empty() {
                    content.push_str("NOTE:");
                    write_escaped_vcard(note, &mut content);
                    content.push('\n');
                }

                content.push_str("END:VCARD");
                from_content(content)
            }
            Self::Email { to, subject, body } => {
                let to = to.trim();
                let subject = subject.trim();
                let body = body.trim();
                if to.is_empty() && subject.is_empty() && body.is_empty() {
                    return empty_payload();
                }

                let mut content =
                    String::with_capacity(16 + to.len() + subject.len() * 3 + body.len() * 3);
                content.push_str("mailto:");
                content.push_str(to);

                let mut has_query = false;
                if !subject.is_empty() {
                    content.push('?');
                    has_query = true;
                    content.push_str("subject=");
                    percent_encode_rfc3986(subject, &mut content);
                }
                if !body.is_empty() {
                    content.push(if has_query { '&' } else { '?' });
                    content.push_str("body=");
                    percent_encode_rfc3986(body, &mut content);
                }
                from_content(content)
            }
            Self::Phone { number } => {
                let number = number.trim();
                if number.is_empty() {
                    return empty_payload();
                }

                let mut content = String::with_capacity(number.len() + 4);
                content.push_str("tel:");

                let mut in_parameters = false;
                let mut has_digits = false;
                let mut is_first = true;
                for character in number.chars() {
                    if in_parameters {
                        // RFC 3966 parameters (for example `;ext=123`) are kept.
                        if !character.is_whitespace() {
                            content.push(character);
                        }
                    } else if character == ';' {
                        in_parameters = true;
                        content.push(';');
                    } else if character == '+' && is_first {
                        content.push('+');
                        is_first = false;
                    } else if character.is_ascii_digit() {
                        content.push(character);
                        has_digits = true;
                        is_first = false;
                    }
                    // Visual separators (spaces, hyphens, parentheses, dots) are dropped.
                }

                if !has_digits {
                    return empty_payload();
                }
                from_content(content)
            }
            Self::Geo {
                latitude,
                longitude,
                query,
            } => {
                let query = query.trim();
                if *latitude == 0.0 && *longitude == 0.0 && query.is_empty() {
                    return empty_payload();
                }

                let latitude = latitude.clamp(-90.0, 90.0);
                let longitude = longitude.clamp(-180.0, 180.0);

                let mut content = String::with_capacity(32 + query.len() * 3);
                content.push_str("geo:");
                write_clean_coordinate(latitude, &mut content);
                content.push(',');
                write_clean_coordinate(longitude, &mut content);
                if !query.is_empty() {
                    content.push_str("?q=");
                    percent_encode_rfc3986(query, &mut content);
                }
                from_content(content)
            }
            Self::Crypto {
                currency,
                address,
                amount,
                message,
                label,
            } => {
                let address = address.trim();
                if address.is_empty() {
                    return empty_payload();
                }

                let amount = normalize_crypto_amount(amount);
                let message = message.trim();
                let label = label.trim();

                let mut content = String::with_capacity(
                    32 + address.len()
                        + amount.as_ref().map_or(0, |amount| amount.len() + 10)
                        + message.len() * 3
                        + label.len() * 3,
                );
                content.push_str(currency.scheme());
                content.push(':');
                content.push_str(address);

                let mut has_query = false;
                if let Some(amount) = amount {
                    content.push('?');
                    has_query = true;
                    let key = match currency {
                        CryptoCurrency::Monero => "tx_amount",
                        _ => "amount",
                    };
                    content.push_str(key);
                    content.push('=');
                    content.push_str(&amount);
                }

                if !label.is_empty() {
                    content.push(if has_query { '&' } else { '?' });
                    has_query = true;
                    content.push_str("label=");
                    percent_encode_rfc3986(label, &mut content);
                }

                if !message.is_empty() {
                    content.push(if has_query { '&' } else { '?' });
                    let key = match currency {
                        CryptoCurrency::Monero => "tx_description",
                        _ => "message",
                    };
                    content.push_str(key);
                    content.push('=');
                    percent_encode_rfc3986(message, &mut content);
                }
                from_content(content)
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn tracking_parameters_are_removed() {
        let dirty =
            "https://example.com/page?utm_source=twitter&utm_medium=cpc&fbclid=12345&keep=me";
        let cleaned = strip_tracking_query(dirty);

        assert_eq!(cleaned.removed_count, 3, "three tracking parameters");
        assert!(cleaned.url.contains("keep=me"), "useful parameters survive");
        assert!(!cleaned.url.contains("utm_source"), "utm_source is removed");
        assert!(!cleaned.url.contains("fbclid"), "fbclid is removed");
    }

    #[test]
    fn cleaning_an_empty_url_yields_empty_content() {
        assert_eq!(clean_url(""), "");
        assert_eq!(
            Payload::Url {
                url: String::new(),
                strip_tracking: false,
            }
            .sanitize()
            .content,
            ""
        );
    }

    #[test]
    fn empty_payloads_do_not_produce_dummy_content() {
        let payloads = [
            Payload::Text {
                text: String::new(),
            },
            Payload::Wifi {
                ssid: String::new(),
                password: String::new(),
                auth_type: WifiAuth::Wpa,
                hidden: false,
            },
            Payload::Phone {
                number: String::new(),
            },
            Payload::Geo {
                latitude: 0.0,
                longitude: 0.0,
                query: String::new(),
            },
            Payload::Crypto {
                currency: CryptoCurrency::Bitcoin,
                address: String::new(),
                amount: String::new(),
                message: String::new(),
                label: String::new(),
            },
            Payload::VCard {
                first_name: String::new(),
                last_name: String::new(),
                phone: String::new(),
                email: String::new(),
                organization: String::new(),
                title: String::new(),
                url: String::new(),
                note: String::new(),
            },
            Payload::Email {
                to: String::new(),
                subject: String::new(),
                body: String::new(),
            },
        ];

        for payload in &payloads {
            let sanitized = payload.sanitize();
            assert_eq!(
                sanitized.content, "",
                "{payload:?} should not produce content"
            );
            assert_eq!(sanitized.cleaned_bytes, 0, "{payload:?} has no bytes");
        }
    }

    #[test]
    fn all_query_parameters_can_be_removed() {
        let dirty = "https://amazon.com/dp/B08X?ref_=123&utm_source=twitter#specs";
        assert_eq!(
            strip_all_query(dirty),
            "https://amazon.com/dp/B08X#specs",
            "fragment is preserved"
        );
    }

    #[test]
    fn query_metadata_is_reported() {
        let dirty = "https://amazon.com/dp/B08X?ref_=123&utm_source=twitter&id=456";
        let info = inspect_url_query(dirty).expect("URL has query parameters");

        assert_eq!(info.total_param_count, 3);
        assert_eq!(info.tracking_param_count, 2);
        assert_eq!(info.stripped_all_url, "https://amazon.com/dp/B08X");
        assert_eq!(
            info.stripped_tracking_url,
            "https://amazon.com/dp/B08X?id=456"
        );
    }

    #[test]
    fn wifi_payloads_escape_reserved_characters() {
        let wifi = Payload::Wifi {
            ssid: "My;Home\\Net:5G".to_string(),
            password: "p@ss;word\\123".to_string(),
            auth_type: WifiAuth::Wpa,
            hidden: true,
        };
        assert_eq!(
            wifi.sanitize().content,
            "WIFI:S:My\\;Home\\\\Net\\:5G;T:WPA;P:p@ss\\;word\\\\123;H:true;;"
        );

        let open_wifi = Payload::Wifi {
            ssid: "Guest Wi-Fi".to_string(),
            password: String::new(),
            auth_type: WifiAuth::Nopass,
            hidden: false,
        };
        assert_eq!(
            open_wifi.sanitize().content,
            "WIFI:S:Guest Wi-Fi;T:nopass;;"
        );
    }

    #[test]
    fn wifi_escape_helpers_match_writer_functions() {
        assert_eq!(escape_wifi_value("a;b"), "a\\;b");
        assert_eq!(escape_vcard_value("a;b,c\nd"), "a\\;b\\,c\\nd");
    }

    #[test]
    fn vcard_payloads_are_well_formed() {
        let vcard = Payload::VCard {
            first_name: "John".to_string(),
            last_name: "Doe;Jr.".to_string(),
            phone: "+123456789".to_string(),
            email: "john@example.com".to_string(),
            organization: "Acme, Inc.".to_string(),
            title: "Staff Engineer".to_string(),
            url: "example.com".to_string(),
            note: "VIP client".to_string(),
        };
        let content = vcard.sanitize().content;

        assert!(content.contains("BEGIN:VCARD"));
        assert!(content.contains("VERSION:3.0"));
        assert!(content.contains("FN:John Doe\\;Jr."));
        assert!(content.contains("N:Doe\\;Jr.;John;;;"));
        assert!(content.contains("TEL;TYPE=CELL:+123456789"));
        assert!(content.contains("EMAIL:john@example.com"));
        assert!(content.contains("ORG:Acme\\, Inc."));
        assert!(content.contains("TITLE:Staff Engineer"));
        assert!(content.contains("URL:https://example.com"));
        assert!(content.contains("NOTE:VIP client"));
        assert!(content.ends_with("END:VCARD"));
    }

    #[test]
    fn email_payloads_percent_encode_the_subject_and_body() {
        let email = Payload::Email {
            to: "hello@den1zz.dev".to_string(),
            subject: "QR Studio Feedback".to_string(),
            body: "Great tool!".to_string(),
        };
        assert_eq!(
            email.sanitize().content,
            "mailto:hello@den1zz.dev?subject=QR%20Studio%20Feedback&body=Great%20tool%21"
        );
    }

    #[test]
    fn phone_payloads_keep_digits_and_extensions() {
        let phone = Payload::Phone {
            number: "+1 (555) 019-2834".to_string(),
        };
        assert_eq!(phone.sanitize().content, "tel:+15550192834");

        let extension = Payload::Phone {
            number: "+1 (555) 019-2834;ext=101".to_string(),
        };
        assert_eq!(extension.sanitize().content, "tel:+15550192834;ext=101");

        let messy = Payload::Phone {
            number: "  (800) 555-0199  ".to_string(),
        };
        assert_eq!(messy.sanitize().content, "tel:8005550199");

        let punctuation_only = Payload::Phone {
            number: "  ()--..  ".to_string(),
        };
        assert_eq!(punctuation_only.sanitize().content, "");
    }

    #[test]
    fn geo_payloads_are_clamped_and_formatted() {
        let geo = Payload::Geo {
            latitude: 37.7749,
            longitude: -122.4194,
            query: "Golden Gate Bridge & Park".to_string(),
        };
        assert_eq!(
            geo.sanitize().content,
            "geo:37.7749,-122.4194?q=Golden%20Gate%20Bridge%20%26%20Park"
        );

        let clean = Payload::Geo {
            latitude: 40.7128,
            longitude: -74.006,
            query: String::new(),
        };
        assert_eq!(clean.sanitize().content, "geo:40.7128,-74.006");

        let overflow = Payload::Geo {
            latitude: 99.9,
            longitude: -200.0,
            query: String::new(),
        };
        assert_eq!(overflow.sanitize().content, "geo:90,-180");
    }

    #[test]
    fn crypto_payloads_use_currency_specific_parameter_names() {
        let bitcoin = Payload::Crypto {
            currency: CryptoCurrency::Bitcoin,
            address: "bc1qar0srrr7xfkvy5l643lydnw9re59gtzzwf5mdq".to_string(),
            amount: "0.05".to_string(),
            message: "Donation & Tip".to_string(),
            label: "Alice & Bob".to_string(),
        };
        assert_eq!(
            bitcoin.sanitize().content,
            "bitcoin:bc1qar0srrr7xfkvy5l643lydnw9re59gtzzwf5mdq?amount=0.05&label=Alice%20%26%20Bob&message=Donation%20%26%20Tip"
        );

        let monero = Payload::Crypto {
            currency: CryptoCurrency::Monero,
            address: "44AFFq5kSiGBoZ4NMDwYtN18obc8AemS33DBLWs3H7otXMV33".to_string(),
            amount: "1.5".to_string(),
            message: "Private Donation".to_string(),
            label: String::new(),
        };
        assert_eq!(
            monero.sanitize().content,
            "monero:44AFFq5kSiGBoZ4NMDwYtN18obc8AemS33DBLWs3H7otXMV33?tx_amount=1.5&tx_description=Private%20Donation"
        );
    }

    #[test]
    fn crypto_amounts_are_normalised_without_float_artifacts() {
        assert_eq!(normalize_crypto_amount("0.05"), Some("0.05".to_string()));
        assert_eq!(
            normalize_crypto_amount("0012.3400"),
            Some("12.34".to_string())
        );
        assert_eq!(normalize_crypto_amount("10.00"), Some("10".to_string()));
        assert_eq!(normalize_crypto_amount(".5"), Some("0.5".to_string()));
        assert_eq!(normalize_crypto_amount("5."), Some("5".to_string()));
        assert_eq!(normalize_crypto_amount("0"), Some("0".to_string()));
        assert_eq!(normalize_crypto_amount("0.00"), Some("0".to_string()));
        assert_eq!(
            normalize_crypto_amount("0.00000001"),
            Some("0.00000001".to_string())
        );

        assert_eq!(normalize_crypto_amount("-1.5"), None);
        assert_eq!(normalize_crypto_amount("+5.0"), None);
        assert_eq!(normalize_crypto_amount("1e-7"), None);
        assert_eq!(normalize_crypto_amount("1.2.3"), None);
        assert_eq!(normalize_crypto_amount("abc"), None);
        assert_eq!(normalize_crypto_amount(""), None);
    }
}
