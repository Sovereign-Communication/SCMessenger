// Identity envelope ("scm.message.identity.v1") — shared builder + parser.
//
// Every outbound chat message travels wrapped in an identity envelope so
// receivers auto-learn the sender's nickname and route hints. Android/iOS
// build this JSON inline (see MeshRepository.kt `encodeMeshMessagePayload`);
// this module is the canonical Rust implementation used by the CLI, kept
// byte-shape compatible with the mobile field set:
//
// {"schema":"scm.message.identity.v1","kind":"text","text":"<msg>",
//  "sender":{"identity_id","public_key","device_id","nickname",
//            "libp2p_peer_id","listeners","external_addresses",
//            "connection_hints"}}

use serde_json::{json, Value};

pub const IDENTITY_ENVELOPE_SCHEMA: &str = "scm.message.identity.v1";

/// Caps mirroring the Android encoder (`take(3)` / `take(6)` / nickname 64).
const MAX_LISTENERS: usize = 3;
const MAX_EXTERNAL_ADDRESSES: usize = 3;
const MAX_CONNECTION_HINTS: usize = 6;
const MAX_NICKNAME_LEN: usize = 64;

/// Sender identity + route hints carried by every identity envelope.
#[derive(Debug, Clone, Default)]
pub struct EnvelopeSenderHints {
    pub identity_id: String,
    pub public_key: String,
    pub device_id: String,
    pub nickname: String,
    pub libp2p_peer_id: String,
    pub listeners: Vec<String>,
    pub external_addresses: Vec<String>,
}

/// Parsed inbound identity envelope.
#[derive(Debug, Clone, Default)]
pub struct DecodedIdentityEnvelope {
    pub kind: String,
    pub text: String,
    pub identity_id: Option<String>,
    pub public_key: Option<String>,
    pub device_id: Option<String>,
    pub nickname: Option<String>,
    pub libp2p_peer_id: Option<String>,
    pub listeners: Vec<String>,
    pub external_addresses: Vec<String>,
    pub connection_hints: Vec<String>,
}

fn trim_take(value: &str, max: usize) -> String {
    value.trim().chars().take(max).collect()
}

fn normalize_nickname(value: &str) -> Option<String> {
    let trimmed = value.trim();
    if trimmed.is_empty() {
        None
    } else {
        Some(trimmed.to_string())
    }
}

fn clean_hints(values: &[String]) -> Vec<String> {
    let mut out = Vec::with_capacity(values.len());
    for v in values {
        let t = v.trim();
        if !t.is_empty() && !out.iter().any(|e: &String| e == t) {
            out.push(t.to_string());
        }
    }
    out
}

/// Build the wire form of an identity envelope. Falls back to bare text if
/// serialization ever fails (additive wrapping must never eat a message).
#[allow(clippy::disallowed_methods)] // json! macro expansion contains serde_json internals
pub fn build_identity_envelope(kind: &str, text: &str, sender: &EnvelopeSenderHints) -> String {
    let mut listeners = clean_hints(&sender.listeners);
    listeners.truncate(MAX_LISTENERS);
    let mut external_addresses = clean_hints(&sender.external_addresses);
    external_addresses.truncate(MAX_EXTERNAL_ADDRESSES);

    let mut connection_hints = clean_hints(&listeners);
    connection_hints.extend(clean_hints(&external_addresses));
    connection_hints.truncate(MAX_CONNECTION_HINTS);

    json!({
        "schema": IDENTITY_ENVELOPE_SCHEMA,
        "kind": kind,
        "text": text,
        "sender": {
            "identity_id": sender.identity_id,
            "public_key": sender.public_key,
            "device_id": sender.device_id,
            "nickname": trim_take(&sender.nickname, MAX_NICKNAME_LEN),
            "libp2p_peer_id": sender.libp2p_peer_id,
            "listeners": listeners,
            "external_addresses": external_addresses,
            "connection_hints": connection_hints,
        },
    })
    .to_string()
}

/// Detect and parse an identity envelope from raw decrypted message text.
/// Returns `None` for bare text (legacy peers) or malformed payloads so
/// callers can fall through to legacy display paths unchanged.
pub fn parse_identity_envelope(raw: &str) -> Option<DecodedIdentityEnvelope> {
    let trimmed = raw.trim();
    if !trimmed.starts_with('{') {
        return None;
    }
    let value: Value = serde_json::from_str(trimmed).ok()?;
    if value.get("schema")?.as_str()? != IDENTITY_ENVELOPE_SCHEMA {
        return None;
    }

    let string_field = |key: &str| -> Option<String> {
        value
            .get(key)
            .and_then(Value::as_str)
            .map(str::trim)
            .filter(|s| !s.is_empty())
            .map(str::to_string)
    };

    let sender = value.get("sender");
    let sender_str = |key: &str| -> Option<String> {
        sender
            .and_then(|s| s.get(key))
            .and_then(Value::as_str)
            .map(str::trim)
            .filter(|s| !s.is_empty())
            .map(str::to_string)
    };
    let sender_array = |key: &str| -> Vec<String> {
        sender
            .and_then(|s| s.get(key))
            .and_then(Value::as_array)
            .map(|arr| {
                arr.iter()
                    .filter_map(Value::as_str)
                    .map(str::trim)
                    .filter(|s| !s.is_empty())
                    .map(str::to_string)
                    .collect()
            })
            .unwrap_or_default()
    };

    Some(DecodedIdentityEnvelope {
        kind: string_field("kind").unwrap_or_else(|| "text".to_string()),
        text: value
            .get("text")
            .and_then(Value::as_str)
            .unwrap_or(raw)
            .to_string(),
        identity_id: sender_str("identity_id"),
        public_key: sender_str("public_key"),
        device_id: sender_str("device_id"),
        nickname: sender_str("nickname").and_then(|n| normalize_nickname(&n)),
        libp2p_peer_id: sender_str("libp2p_peer_id"),
        listeners: sender_array("listeners"),
        external_addresses: sender_array("external_addresses"),
        connection_hints: sender_array("connection_hints"),
    })
}

/// Placeholder nicknames ("peer-xxxxxx") generated receiver-side must never
/// win over a sender-provided name.
pub fn is_synthetic_fallback_nickname(value: Option<&str>) -> bool {
    value
        .map(str::trim)
        .filter(|v| !v.is_empty())
        .map(|v| v.to_lowercase().starts_with("peer-"))
        .unwrap_or(false)
}

/// Mirrors Android's `selectAuthoritativeNickname`: synthetic placeholders are
/// demoted, real names beat placeholders, and a missing incoming hint keeps
/// whatever exists today.
pub fn select_authoritative_nickname(
    incoming: Option<&str>,
    existing: Option<&str>,
) -> Option<String> {
    let incoming_norm = incoming.and_then(normalize_nickname);
    let existing_norm = existing.and_then(normalize_nickname);

    let incoming_synthetic = is_synthetic_fallback_nickname(incoming_norm.as_deref());
    let existing_synthetic = is_synthetic_fallback_nickname(existing_norm.as_deref());

    match (
        incoming_norm.clone(),
        existing_norm.clone(),
        incoming_synthetic,
        existing_synthetic,
    ) {
        (None, Some(_), _, true) => None,
        (None, existing, _, _) => existing,
        (Some(_), None, true, _) => None,
        (Some(_), Some(_), true, true) => None,
        // Placeholder incoming never overwrites a real existing name.
        (Some(_), existing, true, false) => existing,
        (Some(incoming), _, false, true) => Some(incoming),
        (Some(incoming), _, false, false) => Some(incoming),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn sample_hints() -> EnvelopeSenderHints {
        EnvelopeSenderHints {
            identity_id: "a43772fe4343079a56d05b7816d38d0db0144dcbb906b4572d98a784ce4a279a"
                .to_string(),
            public_key: "c0a682eff9128f4e9d1511c39b1e35526d9ceb4d93429a630c0649cacf16b9a5"
                .to_string(),
            device_id: "device-uuid-1".to_string(),
            nickname: "Claude-Windows-Driver".to_string(),
            libp2p_peer_id: "12D3KooWNnPi9wqUJ7Jypj6g4jHmW2PUTmynUs9sJY1h6SQbjLrG".to_string(),
            listeners: vec![
                "/ip4/192.168.0.121/tcp/9090".to_string(),
                "/ip6/::1/tcp/9090/ws".to_string(),
            ],
            external_addresses: vec!["/ip4/192.168.0.121/tcp/9090".to_string()],
        }
    }

    #[test]
    fn test_build_identity_envelope_fields_and_roundtrip() {
        let built = build_identity_envelope("text", "hello rig", &sample_hints());

        let decoded = parse_identity_envelope(&built).expect("envelope must round-trip via parser");
        assert_eq!(decoded.kind, "text");
        assert_eq!(decoded.text, "hello rig");
        assert_eq!(decoded.nickname.as_deref(), Some("Claude-Windows-Driver"));
        assert_eq!(
            decoded.identity_id.as_deref(),
            Some("a43772fe4343079a56d05b7816d38d0db0144dcbb906b4572d98a784ce4a279a")
        );
        assert_eq!(
            decoded.public_key.as_deref(),
            Some("c0a682eff9128f4e9d1511c39b1e35526d9ceb4d93429a630c0649cacf16b9a5")
        );
        assert_eq!(decoded.device_id.as_deref(), Some("device-uuid-1"));
        assert_eq!(
            decoded.libp2p_peer_id.as_deref(),
            Some("12D3KooWNnPi9wqUJ7Jypj6g4jHmW2PUTmynUs9sJY1h6SQbjLrG")
        );
        assert!(decoded
            .listeners
            .contains(&"/ip4/192.168.0.121/tcp/9090".to_string()));
        // connection_hints = distinct(listeners + external), capped at 6
        assert!(!decoded.connection_hints.is_empty());
        assert!(decoded.connection_hints.len() <= MAX_CONNECTION_HINTS);

        // Valid compact JSON with all required keys.
        let value: Value = serde_json::from_str(&built).expect("valid JSON");
        for key in ["schema", "kind", "text", "sender"] {
            assert!(value.get(key).is_some(), "missing key {key}");
        }
        let sender = value.get("sender").unwrap();
        for key in [
            "identity_id",
            "public_key",
            "device_id",
            "nickname",
            "libp2p_peer_id",
            "listeners",
            "external_addresses",
            "connection_hints",
        ] {
            assert!(sender.get(key).is_some(), "missing sender key {key}");
        }
        assert_eq!(
            value.get("schema").unwrap().as_str().unwrap(),
            IDENTITY_ENVELOPE_SCHEMA
        );
    }

    #[test]
    fn test_parse_rejects_bare_text_and_foreign_schemas() {
        assert!(parse_identity_envelope("plain hello").is_none());
        assert!(parse_identity_envelope("{\"schema\":\"other.v1\",\"text\":\"x\"}").is_none());
        assert!(parse_identity_envelope("{\"not\":\"json\"").is_none());
        // Delivery-receipt style JSON without our schema must not be eaten.
        assert!(parse_identity_envelope(
            "{\"message_id\":\"abc\",\"status\":\"delivered\",\"timestamp\":123}"
        )
        .is_none());
    }

    #[test]
    fn test_hint_caps_and_dedup() {
        let mut hints = sample_hints();
        hints.listeners = (0..8).map(|i| format!("/ip4/10.0.0.{i}/tcp/{i}")).collect();
        hints.external_addresses = vec![
            "/ip4/10.0.0.99/tcp/1".to_string(),
            "/ip4/10.0.0.99/tcp/1".to_string(),
        ];

        let built = build_identity_envelope("text", "caps", &hints);
        let value: Value = serde_json::from_str(&built).unwrap();
        let sender = value.get("sender").unwrap();
        assert_eq!(
            sender.get("listeners").unwrap().as_array().unwrap().len(),
            MAX_LISTENERS
        );
        assert_eq!(
            sender
                .get("external_addresses")
                .unwrap()
                .as_array()
                .unwrap()
                .len(),
            2 - 1
        );
        assert_eq!(
            sender
                .get("connection_hints")
                .unwrap()
                .as_array()
                .unwrap()
                .len(),
            // distinct(capped listeners=3 + deduped externals=1)
            4
        );
    }

    #[test]
    fn test_nickname_long_is_trimmed() {
        let mut hints = sample_hints();
        hints.nickname = "x".repeat(200);
        let built = build_identity_envelope("text", "t", &hints);
        let decoded = parse_identity_envelope(&built).unwrap();
        assert_eq!(decoded.nickname.unwrap().chars().count(), MAX_NICKNAME_LEN);
    }

    #[test]
    fn test_select_authoritative_nickname_semantics() {
        // Real incoming beats placeholder existing.
        assert_eq!(
            select_authoritative_nickname(Some("Lucas Light"), Some("peer-abcdef")),
            Some("Lucas Light".to_string())
        );
        // Placeholder never overwrites a real name.
        assert_eq!(
            select_authoritative_nickname(Some("peer-abcdef"), Some("Lucas Light")),
            Some("Lucas Light".to_string())
        );
        // Fill blanks.
        assert_eq!(
            select_authoritative_nickname(Some("Lucas Light"), None),
            Some("Lucas Light".to_string())
        );
        // Nothing authoritative anywhere.
        assert_eq!(select_authoritative_nickname(None, Some("peer-x")), None);
        assert_eq!(select_authoritative_nickname(Some("peer-x"), None), None);
        assert_eq!(
            select_authoritative_nickname(Some("peer-x"), Some("peer-y")),
            None
        );
        // Blank strings treated as absent.
        assert_eq!(
            select_authoritative_nickname(Some("  "), Some("Existing")),
            Some("Existing".to_string())
        );
    }
}
