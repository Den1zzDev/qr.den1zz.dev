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

const TRACKING_PARAMS: &[&str] = &[
    "fbclid", "gclid", "gbraid", "wbraid", "mc_cid", "mc_eid", "_ga", "_gl",
    "msclkid", "twclid", "ref", "source", "igshid", "ysclid", "sc_cid",
    "ttclid", "rdid", "dclid", "_hsenc", "_hsmi", "vero_id", "vero_conv",
    "nr_email_referer", "si",
];

pub fn is_tracking_param(name: &str) -> bool {
    let lower = name.to_ascii_lowercase();
    if lower.starts_with("utm_") {
        return true;
    }
    TRACKING_PARAMS.contains(&lower.as_str())
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

    match Url::parse(&url_string) {
        Ok(mut parsed) => {
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

            (parsed.to_string(), removed_count)
        }
        Err(_) => (url_string, 0),
    }
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
                let content = if text.is_empty() { "QR".to_string() } else { text.clone() };
                let len = content.as_bytes().len();
                SanitizedPayload {
                    content,
                    raw_bytes: len,
                    cleaned_bytes: len,
                    stripped_params_count: 0,
                }
            }
            Payload::Wifi { ssid, password, auth_type, hidden } => {
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
            Payload::VCard { name, phone, email, organization } => {
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
                let encoded_subject = url::form_urlencoded::byte_serialize(subject.as_bytes()).collect::<String>();
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
    fn test_wifi_payload() {
        let wifi = Payload::Wifi {
            ssid: "OfficeGuest".into(),
            password: "secretpassword".into(),
            auth_type: WifiAuth::Wpa,
            hidden: false,
        };
        let sanitized = wifi.sanitize();
        assert_eq!(sanitized.content, "WIFI:S:OfficeGuest;T:WPA;P:secretpassword;H:false;;");
    }
}
