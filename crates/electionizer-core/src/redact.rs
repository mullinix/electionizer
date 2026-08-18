/// Strip secrets (API keys in query strings, etc.) from error text before log/UI/DB.
pub fn redact_secrets(input: &str) -> String {
    let mut out = input.to_string();

    // api_key=... (URL query, form bodies, etc.)
    out = redact_param(&out, "api_key");
    out = redact_param(&out, "api-key");
    out = redact_param(&out, "apikey");
    out = redact_param(&out, "APIKey"); // FollowTheMoney
    out = redact_param(&out, "access_token");
    out = redact_param(&out, "token");

    // Common "Authorization: Bearer xxx"
    if let Some(idx) = out.to_ascii_lowercase().find("bearer ") {
        let start = idx + "bearer ".len();
        let rest = &out[start..];
        let end = rest
            .find(|c: char| c.is_whitespace() || c == '"' || c == '\'')
            .unwrap_or(rest.len());
        if end > 0 {
            out = format!("{}[REDACTED]{}", &out[..start], &rest[end..]);
        }
    }

    out
}

fn redact_param(input: &str, name: &str) -> String {
    // Match name=VALUE until & " ' space or end (case-insensitive name)
    let lower = input.to_ascii_lowercase();
    let needle = format!("{}=", name.to_ascii_lowercase());
    let mut out = String::with_capacity(input.len());
    let mut i = 0;
    let bytes = input.as_bytes();
    let lower_bytes = lower.as_bytes();
    let nlen = needle.len();

    while i < bytes.len() {
        if i + nlen <= bytes.len() && &lower_bytes[i..i + nlen] == needle.as_bytes() {
            out.push_str(&input[i..i + nlen]);
            out.push_str("[REDACTED]");
            i += nlen;
            while i < bytes.len() {
                let c = bytes[i] as char;
                if c == '&' || c == '"' || c == '\'' || c.is_whitespace() || c == ')' || c == ']'
                {
                    break;
                }
                i += 1;
            }
            continue;
        }
        out.push(bytes[i] as char);
        i += 1;
    }
    out
}

/// Safe source URL for FEC without embedding the API key.
pub fn fec_source_url_public(state: &str, office: &str, district: Option<u32>, cycle: i32) -> String {
    let mut u = format!(
        "https://api.open.fec.gov/v1/candidates/?state={state}&office={office}&cycle={cycle}&election_year={cycle}&candidate_status=C"
    );
    if let Some(d) = district {
        let dstr = if d == 0 { "00".into() } else { d.to_string() };
        u.push_str(&format!("&district={dstr}"));
    }
    u
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn redacts_api_key_in_url() {
        let s = "error sending request for url (https://api.open.fec.gov/v1/candidates/?api_key=TEST_SECRET_KEY_xyz&state=FL): operation timed out";
        let r = redact_secrets(s);
        assert!(!r.contains("TEST_SECRET_KEY_xyz"));
        assert!(r.contains("api_key=[REDACTED]"));
        assert!(r.contains("state=FL"));
    }

    #[test]
    fn redacts_bearer() {
        let r = redact_secrets("Authorization: Bearer supersecret123 stuff");
        assert!(!r.contains("supersecret"));
        assert!(r.contains("[REDACTED]"));
    }
}
