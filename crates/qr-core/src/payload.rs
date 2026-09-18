use serde::{Deserialize, Serialize};
use url::Url;

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
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
        hidden: bool,
    },
    VCard {
        name: String,
        phone: String,
        email: String,
        organization: String,
    },
    Email {
        to: String,
        subject: String,
    },
}

#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "UPPERCASE")]
pub enum WifiAuth {
    Wpa,
    Wep,
    Nopass,
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
    "fbclid", "gclid", "gbraid", "wbraid", "mc_cid", "mc_eid", "_ga", "_gl",
    "msclkid", "twclid", "ref", "ref_", "ref_src", "ref_url", "source", "source_id",
    "igshid", "ysclid", "sc_cid", "ttclid", "rdid", "dclid", "_hsenc", "_hsmi",
    "hsctatracking", "vero_id", "vero_conv", "nr_email_referer", "si", "feature",
    "tag", "spm", "scm", "qid", "sr", "dchild", "ved", "ei", "usg",
];

const TRACKING_PREFIXES: &[&str] = &[
    "utm_", "pd_rd_", "pf_rd_", "at_", "itm_",
];

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

    let has_scheme = trimmed.contains("://");
    let to_parse = if !has_scheme {
        format!("https://{trimmed}")
    } else {
        trimmed.to_string()
    };

    let (stripped_tracking, tracking_count) = strip_tracking_query(trimmed);
    let stripped_all = strip_all_query(trimmed);

    let total_count = match Url::parse(&to_parse) {
        Ok(parsed) => parsed.query_pairs().count(),
        Err(_) => {
            if let Some(q_idx) = trimmed.find('?') {
                let q_part = match trimmed.find('#') {
                    Some(h_idx) if h_idx > q_idx => &trimmed[q_idx + 1..h_idx],
                    _ => &trimmed[q_idx + 1..],
                };
                if q_part.is_empty() {
                    0
                } else {
                    q_part.split('&').count()
                }
            } else {
                0
            }
        }
    };

    if total_count == 0 {
        return None;
    }

    Some(UrlQueryInfo {
        has_query: true,
        total_param_count: total_count,
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
        return ("https://qr.den1zz.dev".to_string(), 0);
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

impl Payload {
    pub fn sanitize(&self) -> SanitizedPayload {
        match self {
            Payload::Url { url, strip_tracking } => {
                let raw_bytes = url.as_bytes().len();
                let (cleaned, stripped_count) = clean_url(url, *strip_tracking);
                let cleaned_bytes = cleaned.as_bytes().len();
                SanitizedPayload {
                    content: cleaned,
                    raw_bytes,
                    cleaned_bytes,
                    stripped_params_count: stripped_count,
                }
            }
            Payload::Text { text } => {
                let content = if text.is_empty() {
                    "https://qr.den1zz.dev".to_string()
                } else {
                    text.clone()
                };
                let len = content.as_bytes().len();
                SanitizedPayload {
                    content,
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
                let auth_str = match auth_type {
                    WifiAuth::Wpa => "WPA",
                    WifiAuth::Wep => "WEP",
                    WifiAuth::Nopass => "nopass",
                };
                let content = format!(
                    "WIFI:S:{ssid};T:{auth_str};P:{password};H:{};;",
                    if *hidden { "true" } else { "false" }
                );
                let len = content.as_bytes().len();
                SanitizedPayload {
                    content,
                    raw_bytes: len,
                    cleaned_bytes: len,
                    stripped_params_count: 0,
                }
            }
            Payload::VCard {
                name,
                phone,
                email,
                organization,
            } => {
                let content = format!(
                    "BEGIN:VCARD\nVERSION:3.0\nFN:{name}\nTEL:{phone}\nEMAIL:{email}\nORG:{organization}\nEND:VCARD"
                );
                let len = content.as_bytes().len();
                SanitizedPayload {
                    content,
                    raw_bytes: len,
                    cleaned_bytes: len,
                    stripped_params_count: 0,
                }
            }
            Payload::Email { to, subject } => {
                let encoded_subject =
                    url::form_urlencoded::byte_serialize(subject.as_bytes()).collect::<String>();
                let content = format!("mailto:{to}?subject={encoded_subject}");
                let len = content.as_bytes().len();
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
        let dirty = "https://example.com/page?utm_source=twitter&utm_medium=cpc&fbclid=12345&keep=me";
        let (cleaned, count) = clean_url(dirty, true);
        assert_eq!(count, 3);
        assert!(cleaned.contains("keep=me"));
        assert!(!cleaned.contains("utm_source"));
        assert!(!cleaned.contains("fbclid"));
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
        assert_eq!(info.stripped_tracking_url, "https://amazon.com/dp/B08X?id=456");
    }
}
