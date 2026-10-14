use serde::{Deserialize, Serialize};
use std::fmt::Write as _;
use url::Url;

#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq)]
pub enum CryptoCurrency {
    Bitcoin,
    Ethereum,
    Solana,
    Monero,
    Litecoin,
}

impl CryptoCurrency {
    pub const ALL: [CryptoCurrency; 5] = [
        CryptoCurrency::Bitcoin,
        CryptoCurrency::Ethereum,
        CryptoCurrency::Solana,
        CryptoCurrency::Monero,
        CryptoCurrency::Litecoin,
    ];

    pub fn label(&self) -> &'static str {
        match self {
            CryptoCurrency::Bitcoin => "Bitcoin (BTC)",
            CryptoCurrency::Ethereum => "Ethereum (ETH)",
            CryptoCurrency::Solana => "Solana (SOL)",
            CryptoCurrency::Monero => "Monero (XMR)",
            CryptoCurrency::Litecoin => "Litecoin (LTC)",
        }
    }

    pub fn scheme(&self) -> &'static str {
        match self {
            CryptoCurrency::Bitcoin => "bitcoin",
            CryptoCurrency::Ethereum => "ethereum",
            CryptoCurrency::Solana => "solana",
            CryptoCurrency::Monero => "monero",
            CryptoCurrency::Litecoin => "litecoin",
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[serde(tag = "type", rename_all = "lowercase")]
pub enum Payload {
    Url {
        url: String,
        strip_tracking: bool,
    },
    Text {
        text: String,
    },
    Wifi {
        ssid: String,
        password: String,
        auth_type: WifiAuth,
        #[serde(default)]
        hidden: bool,
    },
    VCard {
        #[serde(default)]
        first_name: String,
        #[serde(default)]
        last_name: String,
        #[serde(default)]
        phone: String,
        #[serde(default)]
        email: String,
        #[serde(default)]
        organization: String,
        #[serde(default)]
        title: String,
        #[serde(default)]
        url: String,
        #[serde(default)]
        note: String,
    },
    Email {
        to: String,
        #[serde(default)]
        subject: String,
        #[serde(default)]
        body: String,
    },
    Phone {
        number: String,
    },
    Geo {
        latitude: f64,
        longitude: f64,
        #[serde(default)]
        query: String,
    },
    Crypto {
        currency: CryptoCurrency,
        address: String,
        #[serde(default)]
        amount: String,
        #[serde(default)]
        message: String,
        #[serde(default)]
        label: String,
    },
}

#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "UPPERCASE")]
pub enum WifiAuth {
    Wpa,
    Wep,
    Nopass,
}

impl WifiAuth {
    pub const ALL: [WifiAuth; 3] = [WifiAuth::Wpa, WifiAuth::Wep, WifiAuth::Nopass];

    pub fn label(&self) -> &'static str {
        match self {
            WifiAuth::Wpa => "WPA / WPA2 / WPA3",
            WifiAuth::Wep => "WEP (Legacy)",
            WifiAuth::Nopass => "None (Open)",
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct SanitizedPayload {
    pub content: String,
    pub raw_bytes: usize,
    pub cleaned_bytes: usize,
    pub stripped_params_count: usize,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct UrlQueryInfo {
    pub has_query: bool,
    pub total_param_count: usize,
    pub tracking_param_count: usize,
    pub stripped_tracking_url: String,
    pub stripped_all_url: String,
    pub original_len: usize,
    pub tracking_cleaned_len: usize,
    pub all_cleaned_len: usize,
}

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

const TRACKING_PREFIXES: &[&str] = &["utm_", "pd_rd_", "pf_rd_", "at_", "itm_"];

pub fn is_tracking_param(name: &str) -> bool {
    let lower = name.to_ascii_lowercase();
    for prefix in TRACKING_PREFIXES {
        if lower.starts_with(prefix) {
            return true;
        }
    }
    TRACKING_EXACT.contains(&lower.as_str())
}

pub fn strip_all_query(raw: &str) -> String {
    let trimmed = raw.trim();
    if let Some(q_idx) = trimmed.find('?') {
        let before_query = &trimmed[..q_idx];
        if let Some(hash_idx) = trimmed[q_idx..].find('#') {
            let fragment = &trimmed[q_idx + hash_idx..];
            format!("{before_query}{fragment}")
        } else {
            before_query.to_string()
        }
    } else {
        trimmed.to_string()
    }
}

pub fn strip_tracking_query(raw: &str) -> (String, usize) {
    let trimmed = raw.trim();
    if !trimmed.contains('?') {
        return (trimmed.to_string(), 0);
    }

    let has_scheme = trimmed.contains("://");
    let to_parse = if !has_scheme {
        format!("https://{trimmed}")
    } else {
        trimmed.to_string()
    };

    if let Ok(mut parsed) = Url::parse(&to_parse) {
        let pairs: Vec<(String, String)> = parsed.query_pairs().into_owned().collect();
        let mut removed_count = 0;
        let filtered: Vec<(String, String)> = pairs
            .into_iter()
            .filter(|(k, _)| {
                if is_tracking_param(k) {
                    removed_count += 1;
                    false
                } else {
                    true
                }
            })
            .collect();

        if removed_count == 0 {
            return (trimmed.to_string(), 0);
        }

        if filtered.is_empty() {
            parsed.set_query(None);
        } else {
            let mut query_builder = String::new();
            for (i, (k, v)) in filtered.iter().enumerate() {
                if i > 0 {
                    query_builder.push('&');
                }
                query_builder.push_str(k);
                if !v.is_empty() {
                    query_builder.push('=');
                    query_builder.push_str(v);
                }
            }
            parsed.set_query(Some(&query_builder));
        }

        let res_str = parsed.to_string();
        let final_str = if !has_scheme && res_str.starts_with("https://") {
            res_str.trim_start_matches("https://").to_string()
        } else {
            res_str
        };
        (final_str, removed_count)
    } else {
        (trimmed.to_string(), 0)
    }
}

pub fn inspect_url_query(raw: &str) -> Option<UrlQueryInfo> {
    let trimmed = raw.trim();
    if !trimmed.contains('?') {
        return None;
    }

    let (stripped_tracking, tracking_count) = strip_tracking_query(trimmed);
    let stripped_all = strip_all_query(trimmed);

    let has_scheme = trimmed.contains("://");
    let to_parse = if !has_scheme {
        format!("https://{trimmed}")
    } else {
        trimmed.to_string()
    };

    let total_params = Url::parse(&to_parse)
        .map(|u| u.query_pairs().count())
        .unwrap_or(0);

    if total_params == 0 {
        return None;
    }

    Some(UrlQueryInfo {
        has_query: true,
        total_param_count: total_params,
        tracking_param_count: tracking_count,
        stripped_tracking_url: stripped_tracking.clone(),
        stripped_all_url: stripped_all.clone(),
        original_len: trimmed.len(),
        tracking_cleaned_len: stripped_tracking.len(),
        all_cleaned_len: stripped_all.len(),
    })
}

pub fn clean_url(raw: &str, strip_tracking: bool) -> (String, usize) {
    let trimmed = raw.trim();
    if trimmed.is_empty() {
        return (String::new(), 0);
    }

    let url_string = if !trimmed.contains("://") {
        format!("https://{trimmed}")
    } else {
        trimmed.to_string()
    };

    if !strip_tracking {
        return (url_string, 0);
    }

    strip_tracking_query(&url_string)
}

/// Zero-allocation RFC 3986 percent-encoding appending to an existing String.
/// Strictly encodes all characters except unreserved: ALPHA / DIGIT / "-" / "." / "_" / "~".
pub fn percent_encode_rfc3986(input: &str, out: &mut String) {
    for b in input.bytes() {
        match b {
            b'a'..=b'z' | b'A'..=b'Z' | b'0'..=b'9' | b'-' | b'.' | b'_' | b'~' => {
                out.push(b as char);
            }
            _ => {
                let _ = write!(out, "%{:02X}", b);
            }
        }
    }
}

/// Validates and normalizes cryptocurrency decimal strings cleanly.
/// Rejects negative amounts, signs (+/-), scientific notation (e.g. 1e-7), and non-numeric chars.
/// Strips leading zeros (preserving "0." for decimals, "0" for zero) and trailing fractional zeros.
/// Avoids floating-point math completely to eliminate IEEE-754 precision artifacts.
pub fn normalize_crypto_amount(raw: &str) -> Option<String> {
    let trimmed = raw.trim();
    if trimmed.is_empty() {
        return None;
    }

    // Reject signs or scientific notation characters
    if trimmed.starts_with('-') || trimmed.starts_with('+') || trimmed.contains(['e', 'E']) {
        return None;
    }

    let parts: Vec<&str> = trimmed.split('.').collect();
    if parts.len() > 2 {
        return None; // Multiple decimal points
    }

    let int_part_raw = parts[0];
    if !int_part_raw.is_empty() && !int_part_raw.bytes().all(|b| b.is_ascii_digit()) {
        return None;
    }

    // Strip leading zeros from int_part: e.g. "007" -> "7", "00" -> "0", "" -> "0"
    let int_part_stripped = int_part_raw.trim_start_matches('0');
    let int_part = if int_part_stripped.is_empty() {
        "0"
    } else {
        int_part_stripped
    };

    if parts.len() == 1 {
        Some(int_part.to_string())
    } else {
        let frac_part_raw = parts[1];
        if !frac_part_raw.bytes().all(|b| b.is_ascii_digit()) {
            return None;
        }

        // Strip trailing zeros from fractional part: e.g. "500" -> "5", "000" -> ""
        let frac_part_stripped = frac_part_raw.trim_end_matches('0');
        if frac_part_stripped.is_empty() {
            Some(int_part.to_string())
        } else {
            let mut out = String::with_capacity(int_part.len() + 1 + frac_part_stripped.len());
            out.push_str(int_part);
            out.push('.');
            out.push_str(frac_part_stripped);
            Some(out)
        }
    }
}

/// Escapes reserved characters in Wi-Fi strings per ZXing barcode specification:
/// backslash (\), semicolon (;), comma (,), colon (:), double quote (").
pub fn write_escaped_wifi(value: &str, out: &mut String) {
    for ch in value.chars() {
        if matches!(ch, '\\' | ';' | ',' | ':' | '"') {
            out.push('\\');
        }
        out.push(ch);
    }
}

pub fn escape_wifi_value(value: &str) -> String {
    let mut out = String::with_capacity(value.len() + 4);
    write_escaped_wifi(value, &mut out);
    out
}

/// Escapes reserved characters in vCard 3.0 fields:
/// backslash (\), semicolon (;), comma (,), and newlines (\n).
pub fn write_escaped_vcard(value: &str, out: &mut String) {
    for ch in value.chars() {
        match ch {
            '\\' => out.push_str("\\\\"),
            ';' => out.push_str("\\;"),
            ',' => out.push_str("\\,"),
            '\n' => out.push_str("\\n"),
            '\r' => {}
            other => out.push(other),
        }
    }
}

pub fn escape_vcard_value(value: &str) -> String {
    let mut out = String::with_capacity(value.len() + 4);
    write_escaped_vcard(value, &mut out);
    out
}

/// Formats a coordinate cleanly with up to 6 decimal places, stripping trailing fractional zeros.
fn write_clean_coord(coord: f64, out: &mut String) {
    if coord.is_nan() || coord.is_infinite() {
        out.push('0');
        return;
    }
    let s = format!("{coord:.6}");
    if s.contains('.') {
        let trimmed = s.trim_end_matches('0').trim_end_matches('.');
        out.push_str(trimmed);
    } else {
        out.push_str(&s);
    }
}

impl Payload {
    pub fn sanitize(&self) -> SanitizedPayload {
        match self {
            Payload::Url {
                url,
                strip_tracking,
            } => {
                let trimmed = url.trim();
                let raw_len = trimmed.len();
                let (cleaned, stripped_count) = clean_url(url, *strip_tracking);
                let cleaned_len = cleaned.len();
                SanitizedPayload {
                    content: cleaned,
                    raw_bytes: raw_len,
                    cleaned_bytes: cleaned_len,
                    stripped_params_count: stripped_count,
                }
            }
            Payload::Text { text } => {
                let trimmed = text.trim();
                let len = trimmed.len();
                SanitizedPayload {
                    content: trimmed.to_string(),
                    raw_bytes: len,
                    cleaned_bytes: len,
                    stripped_params_count: 0,
                }
            }
            Payload::Wifi {
                ssid,
                password,
                auth_type,
                hidden,
            } => {
                let ssid_trim = ssid.trim();
                if ssid_trim.is_empty() {
                    return SanitizedPayload {
                        content: String::new(),
                        raw_bytes: 0,
                        cleaned_bytes: 0,
                        stripped_params_count: 0,
                    };
                }

                let t_val = match auth_type {
                    WifiAuth::Wpa => "WPA",
                    WifiAuth::Wep => "WEP",
                    WifiAuth::Nopass => "nopass",
                };

                let pwd_trim = password.trim();
                let mut content =
                    String::with_capacity(32 + ssid_trim.len() * 2 + pwd_trim.len() * 2);
                content.push_str("WIFI:S:");
                write_escaped_wifi(ssid_trim, &mut content);
                content.push_str(";T:");
                content.push_str(t_val);
                content.push(';');
                if *auth_type != WifiAuth::Nopass && !pwd_trim.is_empty() {
                    content.push_str("P:");
                    write_escaped_wifi(pwd_trim, &mut content);
                    content.push(';');
                }
                if *hidden {
                    content.push_str("H:true;");
                }
                content.push(';');

                let len = content.len();
                SanitizedPayload {
                    content,
                    raw_bytes: len,
                    cleaned_bytes: len,
                    stripped_params_count: 0,
                }
            }
            Payload::VCard {
                first_name,
                last_name,
                phone,
                email,
                organization,
                title,
                url,
                note,
            } => {
                let fn_trim = first_name.trim();
                let ln_trim = last_name.trim();
                let p_trim = phone.trim();
                let e_trim = email.trim();
                let o_trim = organization.trim();
                let t_trim = title.trim();
                let u_trim = url.trim();
                let n_trim = note.trim();

                if fn_trim.is_empty()
                    && ln_trim.is_empty()
                    && p_trim.is_empty()
                    && e_trim.is_empty()
                    && o_trim.is_empty()
                    && t_trim.is_empty()
                    && u_trim.is_empty()
                    && n_trim.is_empty()
                {
                    return SanitizedPayload {
                        content: String::new(),
                        raw_bytes: 0,
                        cleaned_bytes: 0,
                        stripped_params_count: 0,
                    };
                }

                let est_cap = 64
                    + fn_trim.len() * 2
                    + ln_trim.len() * 2
                    + p_trim.len() * 2
                    + e_trim.len() * 2
                    + o_trim.len() * 2
                    + t_trim.len() * 2
                    + u_trim.len() * 2
                    + n_trim.len() * 2;
                let mut content = String::with_capacity(est_cap);
                content.push_str("BEGIN:VCARD\nVERSION:3.0\n");

                if !fn_trim.is_empty() || !ln_trim.is_empty() {
                    content.push_str("FN:");
                    if !fn_trim.is_empty() && !ln_trim.is_empty() {
                        write_escaped_vcard(fn_trim, &mut content);
                        content.push(' ');
                        write_escaped_vcard(ln_trim, &mut content);
                    } else if !fn_trim.is_empty() {
                        write_escaped_vcard(fn_trim, &mut content);
                    } else {
                        write_escaped_vcard(ln_trim, &mut content);
                    }
                    content.push('\n');

                    content.push_str("N:");
                    write_escaped_vcard(ln_trim, &mut content);
                    content.push(';');
                    write_escaped_vcard(fn_trim, &mut content);
                    content.push_str(";;;\n");
                }

                if !p_trim.is_empty() {
                    content.push_str("TEL;TYPE=CELL:");
                    write_escaped_vcard(p_trim, &mut content);
                    content.push('\n');
                }

                if !e_trim.is_empty() {
                    content.push_str("EMAIL:");
                    write_escaped_vcard(e_trim, &mut content);
                    content.push('\n');
                }

                if !o_trim.is_empty() {
                    content.push_str("ORG:");
                    write_escaped_vcard(o_trim, &mut content);
                    content.push('\n');
                }

                if !t_trim.is_empty() {
                    content.push_str("TITLE:");
                    write_escaped_vcard(t_trim, &mut content);
                    content.push('\n');
                }

                if !u_trim.is_empty() {
                    content.push_str("URL:");
                    if !u_trim.contains("://") {
                        content.push_str("https://");
                    }
                    write_escaped_vcard(u_trim, &mut content);
                    content.push('\n');
                }

                if !n_trim.is_empty() {
                    content.push_str("NOTE:");
                    write_escaped_vcard(n_trim, &mut content);
                    content.push('\n');
                }

                content.push_str("END:VCARD");
                let len = content.len();
                SanitizedPayload {
                    content,
                    raw_bytes: len,
                    cleaned_bytes: len,
                    stripped_params_count: 0,
                }
            }
            Payload::Email { to, subject, body } => {
                let to_trim = to.trim();
                let sub_trim = subject.trim();
                let body_trim = body.trim();

                if to_trim.is_empty() && sub_trim.is_empty() && body_trim.is_empty() {
                    return SanitizedPayload {
                        content: String::new(),
                        raw_bytes: 0,
                        cleaned_bytes: 0,
                        stripped_params_count: 0,
                    };
                }

                let mut content = String::with_capacity(
                    16 + to_trim.len() + sub_trim.len() * 3 + body_trim.len() * 3,
                );
                content.push_str("mailto:");
                content.push_str(to_trim);

                let mut has_query = false;
                if !sub_trim.is_empty() {
                    content.push('?');
                    has_query = true;
                    content.push_str("subject=");
                    percent_encode_rfc3986(sub_trim, &mut content);
                }
                if !body_trim.is_empty() {
                    if has_query {
                        content.push('&');
                    } else {
                        content.push('?');
                    }
                    content.push_str("body=");
                    percent_encode_rfc3986(body_trim, &mut content);
                }

                let len = content.len();
                SanitizedPayload {
                    content,
                    raw_bytes: len,
                    cleaned_bytes: len,
                    stripped_params_count: 0,
                }
            }
            Payload::Phone { number } => {
                let trimmed = number.trim();
                if trimmed.is_empty() {
                    return SanitizedPayload {
                        content: String::new(),
                        raw_bytes: 0,
                        cleaned_bytes: 0,
                        stripped_params_count: 0,
                    };
                }

                let mut content = String::with_capacity(trimmed.len() + 4);
                content.push_str("tel:");

                let mut in_params = false;
                let mut has_digits = false;
                let mut is_first = true;

                for ch in trimmed.chars() {
                    if in_params {
                        // In RFC 3966 parameters (e.g. ;ext=123), preserve parameter characters
                        if !ch.is_whitespace() {
                            content.push(ch);
                        }
                    } else if ch == ';' {
                        in_params = true;
                        content.push(';');
                    } else if ch == '+' && is_first {
                        content.push('+');
                        is_first = false;
                    } else if ch.is_ascii_digit() {
                        content.push(ch);
                        has_digits = true;
                        is_first = false;
                    }
                    // Strip visual separators: whitespace, hyphens, parentheses, dots
                }

                if !has_digits {
                    return SanitizedPayload {
                        content: String::new(),
                        raw_bytes: 0,
                        cleaned_bytes: 0,
                        stripped_params_count: 0,
                    };
                }

                let len = content.len();
                SanitizedPayload {
                    content,
                    raw_bytes: len,
                    cleaned_bytes: len,
                    stripped_params_count: 0,
                }
            }
            Payload::Geo {
                latitude,
                longitude,
                query,
            } => {
                let q_trim = query.trim();
                if *latitude == 0.0 && *longitude == 0.0 && q_trim.is_empty() {
                    return SanitizedPayload {
                        content: String::new(),
                        raw_bytes: 0,
                        cleaned_bytes: 0,
                        stripped_params_count: 0,
                    };
                }

                let lat = latitude.clamp(-90.0, 90.0);
                let lng = longitude.clamp(-180.0, 180.0);

                let mut content = String::with_capacity(32 + q_trim.len() * 3);
                content.push_str("geo:");
                write_clean_coord(lat, &mut content);
                content.push(',');
                write_clean_coord(lng, &mut content);

                if !q_trim.is_empty() {
                    content.push_str("?q=");
                    percent_encode_rfc3986(q_trim, &mut content);
                }

                let len = content.len();
                SanitizedPayload {
                    content,
                    raw_bytes: len,
                    cleaned_bytes: len,
                    stripped_params_count: 0,
                }
            }
            Payload::Crypto {
                currency,
                address,
                amount,
                message,
                label,
            } => {
                let addr_trim = address.trim();
                if addr_trim.is_empty() {
                    return SanitizedPayload {
                        content: String::new(),
                        raw_bytes: 0,
                        cleaned_bytes: 0,
                        stripped_params_count: 0,
                    };
                }

                let scheme = currency.scheme();
                let amt_normalized = normalize_crypto_amount(amount);
                let msg_trim = message.trim();
                let lbl_trim = label.trim();

                let mut content = String::with_capacity(
                    32 + addr_trim.len()
                        + amt_normalized.as_ref().map_or(0, |a| a.len() + 10)
                        + msg_trim.len() * 3
                        + lbl_trim.len() * 3,
                );
                content.push_str(scheme);
                content.push(':');
                content.push_str(addr_trim);

                let mut has_query = false;

                if let Some(amt) = amt_normalized {
                    content.push('?');
                    has_query = true;
                    let key = match currency {
                        CryptoCurrency::Monero => "tx_amount",
                        _ => "amount",
                    };
                    content.push_str(key);
                    content.push('=');
                    content.push_str(&amt);
                }

                if !lbl_trim.is_empty() {
                    if has_query {
                        content.push('&');
                    } else {
                        content.push('?');
                        has_query = true;
                    }
                    content.push_str("label=");
                    percent_encode_rfc3986(lbl_trim, &mut content);
                }

                if !msg_trim.is_empty() {
                    if has_query {
                        content.push('&');
                    } else {
                        content.push('?');
                    }
                    let key = match currency {
                        CryptoCurrency::Monero => "tx_description",
                        _ => "message",
                    };
                    content.push_str(key);
                    content.push('=');
                    percent_encode_rfc3986(msg_trim, &mut content);
                }

                let len = content.len();
                SanitizedPayload {
                    content,
                    raw_bytes: len,
                    cleaned_bytes: len,
                    stripped_params_count: 0,
                }
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_tracking_cleaner() {
        let dirty =
            "https://example.com/page?utm_source=twitter&utm_medium=cpc&fbclid=12345&keep=me";
        let (cleaned, count) = clean_url(dirty, true);
        assert_eq!(count, 3);
        assert!(cleaned.contains("keep=me"));
        assert!(!cleaned.contains("utm_source"));
        assert!(!cleaned.contains("fbclid"));
    }

    #[test]
    fn test_clean_url_empty_default() {
        let (cleaned, count) = clean_url("", false);
        assert_eq!(cleaned, "");
        assert_eq!(count, 0);
    }

    #[test]
    fn test_empty_payloads_do_not_produce_dummy_content() {
        let empty_url = Payload::Url {
            url: String::new(),
            strip_tracking: false,
        }
        .sanitize();
        assert_eq!(empty_url.content, "");
        assert_eq!(empty_url.cleaned_bytes, 0);

        let empty_text = Payload::Text {
            text: String::new(),
        }
        .sanitize();
        assert_eq!(empty_text.content, "");
        assert_eq!(empty_text.cleaned_bytes, 0);

        let empty_wifi = Payload::Wifi {
            ssid: String::new(),
            password: String::new(),
            auth_type: WifiAuth::Wpa,
            hidden: false,
        }
        .sanitize();
        assert_eq!(empty_wifi.content, "");
        assert_eq!(empty_wifi.cleaned_bytes, 0);

        let empty_phone = Payload::Phone {
            number: String::new(),
        }
        .sanitize();
        assert_eq!(empty_phone.content, "");
        assert_eq!(empty_phone.cleaned_bytes, 0);

        let empty_geo = Payload::Geo {
            latitude: 0.0,
            longitude: 0.0,
            query: String::new(),
        }
        .sanitize();
        assert_eq!(empty_geo.content, "");
        assert_eq!(empty_geo.cleaned_bytes, 0);

        let empty_crypto = Payload::Crypto {
            currency: CryptoCurrency::Bitcoin,
            address: String::new(),
            amount: String::new(),
            message: String::new(),
            label: String::new(),
        }
        .sanitize();
        assert_eq!(empty_crypto.content, "");
        assert_eq!(empty_crypto.cleaned_bytes, 0);

        let empty_vcard = Payload::VCard {
            first_name: String::new(),
            last_name: String::new(),
            phone: String::new(),
            email: String::new(),
            organization: String::new(),
            title: String::new(),
            url: String::new(),
            note: String::new(),
        }
        .sanitize();
        assert_eq!(empty_vcard.content, "");
        assert_eq!(empty_vcard.cleaned_bytes, 0);

        let empty_email = Payload::Email {
            to: String::new(),
            subject: String::new(),
            body: String::new(),
        }
        .sanitize();
        assert_eq!(empty_email.content, "");
        assert_eq!(empty_email.cleaned_bytes, 0);
    }

    #[test]
    fn test_strip_all_query() {
        let dirty = "https://amazon.com/dp/B08X?ref_=123&utm_source=twitter#specs";
        let cleaned = strip_all_query(dirty);
        assert_eq!(cleaned, "https://amazon.com/dp/B08X#specs");
    }

    #[test]
    fn test_inspect_url_query() {
        let dirty = "https://amazon.com/dp/B08X?ref_=123&utm_source=twitter&id=456";
        let info = inspect_url_query(dirty).expect("Should find query info");
        assert_eq!(info.total_param_count, 3);
        assert_eq!(info.tracking_param_count, 2);
        assert_eq!(info.stripped_all_url, "https://amazon.com/dp/B08X");
        assert_eq!(
            info.stripped_tracking_url,
            "https://amazon.com/dp/B08X?id=456"
        );
    }

    #[test]
    fn test_wifi_payload_formatting_and_escaping() {
        let wifi = Payload::Wifi {
            ssid: "My;Home\\Net:5G".to_string(),
            password: "p@ss;word\\123".to_string(),
            auth_type: WifiAuth::Wpa,
            hidden: true,
        };
        let sanitized = wifi.sanitize();
        assert_eq!(
            sanitized.content,
            "WIFI:S:My\\;Home\\\\Net\\:5G;T:WPA;P:p@ss\\;word\\\\123;H:true;;"
        );

        let open_wifi = Payload::Wifi {
            ssid: "Guest Wi-Fi".to_string(),
            password: "".to_string(),
            auth_type: WifiAuth::Nopass,
            hidden: false,
        };
        let sanitized_open = open_wifi.sanitize();
        assert_eq!(sanitized_open.content, "WIFI:S:Guest Wi-Fi;T:nopass;;");
    }

    #[test]
    fn test_vcard_payload_formatting() {
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
        let sanitized = vcard.sanitize();
        assert!(sanitized.content.contains("BEGIN:VCARD"));
        assert!(sanitized.content.contains("VERSION:3.0"));
        assert!(sanitized.content.contains("FN:John Doe\\;Jr."));
        assert!(sanitized.content.contains("N:Doe\\;Jr.;John;;;"));
        assert!(sanitized.content.contains("TEL;TYPE=CELL:+123456789"));
        assert!(sanitized.content.contains("EMAIL:john@example.com"));
        assert!(sanitized.content.contains("ORG:Acme\\, Inc."));
        assert!(sanitized.content.contains("TITLE:Staff Engineer"));
        assert!(sanitized.content.contains("URL:https://example.com"));
        assert!(sanitized.content.contains("NOTE:VIP client"));
        assert!(sanitized.content.ends_with("END:VCARD"));
    }

    #[test]
    fn test_email_payload_formatting() {
        let email = Payload::Email {
            to: "hello@den1zz.dev".to_string(),
            subject: "QR Studio Feedback".to_string(),
            body: "Great tool!".to_string(),
        };
        let sanitized = email.sanitize();
        assert_eq!(
            sanitized.content,
            "mailto:hello@den1zz.dev?subject=QR%20Studio%20Feedback&body=Great%20tool%21"
        );
    }

    #[test]
    fn test_phone_payload_formatting() {
        let phone = Payload::Phone {
            number: "+1 (555) 019-2834".to_string(),
        };
        let sanitized = phone.sanitize();
        assert_eq!(sanitized.content, "tel:+15550192834");

        // RFC 3966 extension parameter preserved
        let phone_ext = Payload::Phone {
            number: "+1 (555) 019-2834;ext=101".to_string(),
        };
        assert_eq!(phone_ext.sanitize().content, "tel:+15550192834;ext=101");

        // Messy formatting and non-digit cleanup
        let messy = Payload::Phone {
            number: "  (800) 555-0199  ".to_string(),
        };
        assert_eq!(messy.sanitize().content, "tel:8005550199");

        // Only punctuation yields empty payload
        let invalid = Payload::Phone {
            number: "  ()--..  ".to_string(),
        };
        assert_eq!(invalid.sanitize().content, "");
    }

    #[test]
    fn test_geo_payload_formatting() {
        let geo = Payload::Geo {
            latitude: 37.7749,
            longitude: -122.4194,
            query: "Golden Gate Bridge & Park".to_string(),
        };
        let sanitized = geo.sanitize();
        assert_eq!(
            sanitized.content,
            "geo:37.7749,-122.4194?q=Golden%20Gate%20Bridge%20%26%20Park"
        );

        let geo_clean = Payload::Geo {
            latitude: 40.712800,
            longitude: -74.006000,
            query: String::new(),
        };
        assert_eq!(geo_clean.sanitize().content, "geo:40.7128,-74.006");

        // Latitude / Longitude boundary clamping
        let geo_overflow = Payload::Geo {
            latitude: 99.9,
            longitude: -200.0,
            query: String::new(),
        };
        assert_eq!(geo_overflow.sanitize().content, "geo:90,-180");
    }

    #[test]
    fn test_crypto_payload_formatting() {
        let btc = Payload::Crypto {
            currency: CryptoCurrency::Bitcoin,
            address: "bc1qar0srrr7xfkvy5l643lydnw9re59gtzzwf5mdq".to_string(),
            amount: "0.05".to_string(),
            message: "Donation & Tip".to_string(),
            label: "Alice & Bob".to_string(),
        };
        let sanitized = btc.sanitize();
        assert_eq!(
            sanitized.content,
            "bitcoin:bc1qar0srrr7xfkvy5l643lydnw9re59gtzzwf5mdq?amount=0.05&label=Alice%20%26%20Bob&message=Donation%20%26%20Tip"
        );

        let eth = Payload::Crypto {
            currency: CryptoCurrency::Ethereum,
            address: "0xd8dA6BF26964aF9D7eEd9e03E53415D37aA96045".to_string(),
            amount: "10.00".to_string(),
            message: String::new(),
            label: String::new(),
        };
        assert_eq!(
            eth.sanitize().content,
            "ethereum:0xd8dA6BF26964aF9D7eEd9e03E53415D37aA96045?amount=10"
        );

        let sol = Payload::Crypto {
            currency: CryptoCurrency::Solana,
            address: "7xKXtg2CW87d97TXJSDpbD5jBkheTqA83TZRuJosgAsU".to_string(),
            amount: ".5".to_string(),
            message: "Solana Pay #1".to_string(),
            label: "Store".to_string(),
        };
        assert_eq!(
            sol.sanitize().content,
            "solana:7xKXtg2CW87d97TXJSDpbD5jBkheTqA83TZRuJosgAsU?amount=0.5&label=Store&message=Solana%20Pay%20%231"
        );

        let ltc = Payload::Crypto {
            currency: CryptoCurrency::Litecoin,
            address: "LQTpS3VaYTbvqgZ2qQk4nEvbZ4D2gYd8h3".to_string(),
            amount: "0012.3400".to_string(),
            message: String::new(),
            label: String::new(),
        };
        assert_eq!(
            ltc.sanitize().content,
            "litecoin:LQTpS3VaYTbvqgZ2qQk4nEvbZ4D2gYd8h3?amount=12.34"
        );

        let xmr = Payload::Crypto {
            currency: CryptoCurrency::Monero,
            address: "44AFFq5kSiGBoZ4NMDwYtN18obc8AemS33DBLWs3H7otXMV33".to_string(),
            amount: "1.5".to_string(),
            message: "Private Donation".to_string(),
            label: String::new(),
        };
        assert_eq!(
            xmr.sanitize().content,
            "monero:44AFFq5kSiGBoZ4NMDwYtN18obc8AemS33DBLWs3H7otXMV33?tx_amount=1.5&tx_description=Private%20Donation"
        );
    }

    #[test]
    fn test_crypto_amount_normalization_safety() {
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

        // Invalid / unsafe inputs rejected
        assert_eq!(normalize_crypto_amount("-1.5"), None);
        assert_eq!(normalize_crypto_amount("+5.0"), None);
        assert_eq!(normalize_crypto_amount("1e-7"), None);
        assert_eq!(normalize_crypto_amount("1.2.3"), None);
        assert_eq!(normalize_crypto_amount("abc"), None);
        assert_eq!(normalize_crypto_amount(""), None);
    }
}
