use crate::{domain::request_model::RequestSavedModel, enums::auth_config::AuthConfig};

/// Quote one argument for POSIX shells (bash, zsh, sh). This is NOT Windows CMD.
fn quote(value: &str) -> String {
    format!("'{}'", value.replace('\'', "'\\''"))
}

/// Builds an editable, readable cURL command. Values are quoted as arguments,
/// not interpolated into shell syntax. Auth values are sensitive: do not log.
pub fn generate_curl_for_request(req: &RequestSavedModel) -> String {
    let mut url = req.url.clone();
    let params: Vec<String> = req.params.iter()
        .filter(|p| p.enabled && !p.key.trim().is_empty())
        .map(|p| format!("{}={}", encode_component(&p.key), encode_component(&p.value)))
        .collect();
    if !params.is_empty() {
        // Query comes before #fragment, even when the saved URL has a fragment.
        let (before_fragment, fragment) = match url.split_once('#') {
            Some((before, frag)) => (before.to_string(), Some(frag.to_owned())),
            None => (url.clone(), None),
        };
        url = before_fragment;
        url.push(if url.contains('?') { '&' } else { '?' });
        url.push_str(&params.join("&"));
        if let Some(fragment) = fragment {
            url.push('#');
            url.push_str(&fragment);
        }
    }

    let mut lines = vec![format!("curl -X {} {}", quote(&req.method), quote(&url))];
    for header in req.headers.iter().filter(|p| p.enabled && !p.key.trim().is_empty()) {
        lines.push(format!("-H {}", quote(&format!("{}: {}", header.key, header.value))));
    }
    match &req.auth {
        AuthConfig::None => {}
        AuthConfig::Bearer { token } => {
            lines.push(format!("-H {}", quote(&format!("Authorization: Bearer {token}"))));
        }
        AuthConfig::Basic { username, password } => {
            lines.push(format!("-u {}", quote(&format!("{username}:{password}"))));
        }
        AuthConfig::ApiKey { key, value } => {
            lines.push(format!("-H {}", quote(&format!("{key}: {value}"))));
        }
    }
    if !req.body.is_empty() {
        lines.push(format!("--data-raw {}", quote(&req.body)));
    }
    lines.join(" \\\n  ")
}

fn encode_component(s: &str) -> String {
    let mut encoded = String::new();
    for b in s.bytes() {
        if b.is_ascii_alphanumeric() || matches!(b, b'-' | b'_' | b'.' | b'~') {
            encoded.push(b as char);
        } else {
            encoded.push_str(&format!("%{b:02X}"));
        }
    }
    encoded
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::{domain::request_model::KeyValuePair, enums::auth_config::AuthConfig};

    #[test]
    fn quotes_shell_single_quote() {
        assert_eq!(quote("a'b"), "'a'\\''b'");
    }

    #[test]
    fn encodes_query() {
        assert_eq!(encode_component("a b&"), "a%20b%26");
    }

    #[test]
    fn renders_query_before_fragment_and_escapes_body() {
        let req = RequestSavedModel {
            id: Some(4), folder_id: None, name: "Test".into(),
            method: "POST".into(), url: "https://api.example.test/items#section".into(),
            headers: vec![],
            params: vec![KeyValuePair { key: "q".into(), value: "a b".into(), enabled: true }],
            auth: AuthConfig::None, body: "it's safe".into(), response: None,
        };
        let curl = generate_curl_for_request(&req);
        assert!(curl.contains("items?q=a%20b#section"));
        assert!(curl.contains("--data-raw 'it'\\''s safe'"));
        assert!(curl.contains(" \\\n  --data-raw"));
    }
}
