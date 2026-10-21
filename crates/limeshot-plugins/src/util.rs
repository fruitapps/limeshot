//! String and id conversions between Rust and the VST3 ABI.

use std::ffi::c_char;

use vst3::Steinberg::{TUID, Vst::TChar};

/// A NUL-terminated byte array (`char8[N]`) as a string.
pub(crate) fn from_cstr(src: &[c_char]) -> String {
    let bytes: Vec<u8> = src.iter().take_while(|&&c| c != 0).map(|&c| c as u8).collect();
    String::from_utf8_lossy(&bytes).into_owned()
}

/// A NUL-terminated UTF-16 array (`String128`) as a string.
pub(crate) fn from_wstr(src: &[TChar]) -> String {
    let len = src.iter().position(|&c| c == 0).unwrap_or(src.len());
    String::from_utf16_lossy(&src[..len])
}

/// Copy `src` into a fixed `char8` array, truncated and always NUL-terminated.
pub(crate) fn copy_cstr(src: &str, dst: &mut [c_char]) {
    let n = src.len().min(dst.len().saturating_sub(1));
    for (d, s) in dst.iter_mut().zip(src.bytes().take(n)) {
        *d = s as c_char;
    }
    if let Some(end) = dst.get_mut(n) {
        *end = 0;
    }
}

/// Copy `src` into a fixed UTF-16 array, truncated and always NUL-terminated.
pub(crate) fn copy_wstr(src: &str, dst: &mut [TChar]) {
    let mut n = 0;
    let cap = dst.len().saturating_sub(1);
    for (d, s) in dst.iter_mut().take(cap).zip(src.encode_utf16()) {
        *d = s;
        n += 1;
    }
    if let Some(end) = dst.get_mut(n) {
        *end = 0;
    }
}

pub(crate) fn tuid_to_hex(id: &TUID) -> String {
    id.iter().map(|b| format!("{:02X}", *b as u8)).collect()
}

pub(crate) fn hex_to_tuid(hex: &str) -> Option<TUID> {
    if hex.len() != 32 || !hex.is_ascii() {
        return None;
    }
    let mut id: TUID = [0; 16];
    for (i, b) in id.iter_mut().enumerate() {
        *b = u8::from_str_radix(&hex[2 * i..2 * i + 2], 16).ok()? as c_char;
    }
    Some(id)
}

/// The interface id of `I` as a `TUID`, for comparing with ids a plugin passes in.
pub(crate) fn iid<I: vst3::Interface>() -> TUID {
    I::IID.map(|b| b as c_char)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn strings_round_trip_and_truncate() {
        let mut c = [1 as c_char; 5];
        copy_cstr("abcdefg", &mut c);
        assert_eq!(from_cstr(&c), "abcd");
        let mut w = [1 as TChar; 4];
        copy_wstr("Gäin", &mut w);
        assert_eq!(from_wstr(&w), "Gäi");
    }

    #[test]
    fn class_ids_round_trip() {
        let id = vst3::uid(0x12345678, 0x9ABCDEF0, 0x0F1E2D3C, 0x4B5A6978);
        let hex = tuid_to_hex(&id);
        assert_eq!(hex.len(), 32);
        assert_eq!(hex_to_tuid(&hex), Some(id));
        assert_eq!(hex_to_tuid("zz"), None);
    }
}
