use crate::{domain::request_model::RequestSavedModel, enums::auth_config::AuthConfig};

fn quote(value: &str) -> String {
    format!("'{}'", value.replace('\'', "'\\''"))
}

/// POSIX sh/bash/zsh-compatible command; intentionally not a Windows cmd formatter.
pub fn generate_curl_for_request(req: &RequestSavedModel) -> String {
    let mut args = vec!["curl".to_owned(), "-X".to_owned(), quote(&req.method)];
    let mut url = req.url.clone();
    let params: Vec<String> = req.params.iter().filter(|p| p.enabled && !p.key.is_empty())
        .map(|p| {
            // Encode query components without treating a user-supplied value as URL syntax.
            format!("{}={}", encode_component(&p.key), encode_component(&p.value))
        }).collect();
    if !params.is_empty() {
        url.push(if url.contains('?') { '&' } else { '?' });
        url.push_str(&params.join("&"));
    }
    args.push(quote(&url));
    for header in req.headers.iter().filter(|p| p.enabled && !p.key.trim().is_empty()) {
        args.push("-H".to_owned());
        args.push(quote(&format!("{}: {}", header.key, header.value)));
    }
    match &req.auth {
        AuthConfig::None => {}
        AuthConfig::Bearer { token } => {
            args.push("-H".into());
            args.push(quote(&format!("Authorization: Bearer {token}")));
        }
        AuthConfig::Basic { username, password } => {
            args.push("-u".into());
            args.push(quote(&format!("{username}:{password}")));
        }
        AuthConfig::ApiKey { key, value } => {
            args.push("-H".into());
            args.push(quote(&format!("{key}: {value}")));
        }
    }
    if !req.body.is_empty() {
        args.push("--data-raw".into());
        args.push(quote(&req.body));
    }
    args.join(" ")
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
    #[test]
    fn quotes_shell_single_quote() { assert_eq!(quote("a'b"), "'a'\\''b'"); }
    #[test]
    fn encodes_query() { assert_eq!(encode_component("a b&"), "a%20b%26"); }
}
