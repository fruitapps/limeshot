//! Hosted plugins as project data (FL Studio's Fruity Wrapper): which plugin a channel or insert
//! slot runs, and the plugin's saved state. Loading and processing live in `limeshot-plugins`.

use std::sync::Arc;

use serde::{Deserialize, Deserializer, Serialize, Serializer};

/// The largest plugin state kept from a file (a hostile file could claim more).
pub const MAX_STATE_BYTES: usize = 64 << 20;

/// A plugin and its state.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct PluginRef {
    /// "VST3" (the only format so far).
    pub format: String,
    /// The bundle, as the scan found it.
    pub path: String,
    /// The class inside the bundle (32 hexadecimal digits for VST3).
    pub class_id: String,
    pub name: String,
    #[serde(default)]
    pub vendor: String,
    /// The plugin's own state (opaque), saved base64-encoded.
    #[serde(default, skip_serializing_if = "PluginState::is_empty")]
    pub state: PluginState,
    #[serde(flatten)]
    pub extra: crate::Extra,
}

impl PluginRef {
    pub fn vst3(path: impl Into<String>, class_id: impl Into<String>, name: impl Into<String>) -> Self {
        Self {
            format: "VST3".into(),
            path: path.into(),
            class_id: class_id.into(),
            name: name.into(),
            vendor: String::new(),
            state: PluginState::default(),
            extra: crate::Extra::new(),
        }
    }

    pub fn sanitize(&mut self) {
        for s in [
            &mut self.format,
            &mut self.path,
            &mut self.class_id,
            &mut self.name,
            &mut self.vendor,
        ] {
            if s.len() > 4096 {
                let mut end = 4096;
                while !s.is_char_boundary(end) {
                    end -= 1;
                }
                s.truncate(end);
            }
        }
        if self.state.len() > MAX_STATE_BYTES {
            self.state = PluginState::default();
        }
    }
}

/// Plugin state bytes. Shared, so undo snapshots of a project do not copy them.
#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct PluginState(Arc<[u8]>);

impl PluginState {
    pub fn new(bytes: Vec<u8>) -> Self {
        Self(bytes.into())
    }

    pub fn bytes(&self) -> &[u8] {
        &self.0
    }

    pub fn len(&self) -> usize {
        self.0.len()
    }

    pub fn is_empty(&self) -> bool {
        self.0.is_empty()
    }
}

impl Serialize for PluginState {
    fn serialize<S: Serializer>(&self, s: S) -> Result<S::Ok, S::Error> {
        s.serialize_str(&base64_encode(&self.0))
    }
}

/// An oversized or undecodable state loads as empty: the plugin then starts from its defaults
/// instead of the whole project failing to open.
impl<'de> Deserialize<'de> for PluginState {
    fn deserialize<D: Deserializer<'de>>(d: D) -> Result<Self, D::Error> {
        let s = String::deserialize(d)?;
        if s.len() / 4 * 3 > MAX_STATE_BYTES {
            return Ok(PluginState::default());
        }
        Ok(base64_decode(&s).map(PluginState::new).unwrap_or_default())
    }
}

const B64: &[u8; 64] = b"ABCDEFGHIJKLMNOPQRSTUVWXYZabcdefghijklmnopqrstuvwxyz0123456789+/";

/// Standard base64 with padding (RFC 4648).
pub fn base64_encode(bytes: &[u8]) -> String {
    let mut out = String::with_capacity(bytes.len().div_ceil(3) * 4);
    for chunk in bytes.chunks(3) {
        let b = [chunk[0], *chunk.get(1).unwrap_or(&0), *chunk.get(2).unwrap_or(&0)];
        let n = (u32::from(b[0]) << 16) | (u32::from(b[1]) << 8) | u32::from(b[2]);
        for i in 0..4 {
            if i <= chunk.len() {
                out.push(char::from(B64[(n >> (18 - 6 * i) & 63) as usize]));
            } else {
                out.push('=');
            }
        }
    }
    out
}

/// Decode standard base64; whitespace is ignored, anything else invalid gives `None`.
pub fn base64_decode(s: &str) -> Option<Vec<u8>> {
    let digits: Vec<u8> = s.bytes().filter(|b| !b.is_ascii_whitespace()).collect();
    if !digits.len().is_multiple_of(4) {
        return None;
    }
    let value = |c: u8| -> Option<u32> {
        Some(match c {
            b'A'..=b'Z' => c - b'A',
            b'a'..=b'z' => c - b'a' + 26,
            b'0'..=b'9' => c - b'0' + 52,
            b'+' => 62,
            b'/' => 63,
            _ => return None,
        } as u32)
    };
    let mut out = Vec::with_capacity(digits.len() / 4 * 3);
    let groups = digits.len() / 4;
    for (g, quad) in digits.chunks(4).enumerate() {
        let pad = quad.iter().rev().take_while(|&&c| c == b'=').count();
        if pad > 2 || (pad > 0 && g + 1 != groups) || quad[..4 - pad].contains(&b'=') {
            return None;
        }
        let mut n = 0u32;
        for &c in &quad[..4 - pad] {
            n = (n << 6) | value(c)?;
        }
        n <<= 6 * pad as u32;
        let bytes = [(n >> 16) as u8, (n >> 8) as u8, n as u8];
        out.extend_from_slice(&bytes[..3 - pad]);
    }
    Some(out)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn base64_matches_rfc_4648_vectors() {
        for (raw, enc) in [
            ("", ""),
            ("f", "Zg=="),
            ("fo", "Zm8="),
            ("foo", "Zm9v"),
            ("foob", "Zm9vYg=="),
            ("fooba", "Zm9vYmE="),
            ("foobar", "Zm9vYmFy"),
        ] {
            assert_eq!(base64_encode(raw.as_bytes()), enc);
            assert_eq!(base64_decode(enc).unwrap(), raw.as_bytes());
        }
        let all: Vec<u8> = (0..=255).collect();
        assert_eq!(base64_decode(&base64_encode(&all)).unwrap(), all);
        for bad in ["Zg=", "Z===", "Zg==Zg==", "Zm9v!", "=Zm9"] {
            assert_eq!(base64_decode(bad), None, "{bad}");
        }
    }

    #[test]
    fn a_plugin_ref_round_trips_through_json() {
        let mut p = PluginRef::vst3("/x/Y.vst3", "00112233445566778899AABBCCDDEEFF", "Y");
        p.state = PluginState::new(vec![0, 1, 2, 250]);
        let json = serde_json::to_string(&p).unwrap();
        assert!(json.contains("\"state\":\"AAEC+g==\""), "{json}");
        let back: PluginRef = serde_json::from_str(&json).unwrap();
        assert_eq!(back, p);
        let bad = json.replace("AAEC+g==", "not base64");
        assert!(serde_json::from_str::<PluginRef>(&bad).unwrap().state.is_empty());
    }
}
