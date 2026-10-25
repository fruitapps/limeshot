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

/// A class id as 32 hex digits in the SDK's portable order (`FUID::toString`), the same on
/// every platform and in `.vstpreset` files. On Windows a `TUID` holds a COM GUID, whose first
/// three fields are stored little-endian, so those eight bytes are reordered.
pub(crate) fn tuid_to_hex(id: &TUID) -> String {
    com_order(raw_bytes(id)).iter().map(|b| format!("{b:02X}")).collect()
}

/// The platform `TUID` for a portable hex class id (see [`tuid_to_hex`]).
pub(crate) fn hex_to_tuid(hex: &str) -> Option<TUID> {
    Some(com_order(hex_bytes(hex)?).map(|b| b as c_char))
}

/// The hex digits taken as the `TUID`'s bytes in memory order. Windows builds before the
/// portable order stored class ids this way, so lookups fall back to it.
pub(crate) fn raw_hex_to_tuid(hex: &str) -> Option<TUID> {
    Some(hex_bytes(hex)?.map(|b| b as c_char))
}

fn raw_bytes(id: &TUID) -> [u8; 16] {
    id.map(|b| b as u8)
}

fn hex_bytes(hex: &str) -> Option<[u8; 16]> {
    if hex.len() != 32 || !hex.is_ascii() {
        return None;
    }
    let mut id = [0u8; 16];
    for (i, b) in id.iter_mut().enumerate() {
        *b = u8::from_str_radix(&hex[2 * i..2 * i + 2], 16).ok()?;
    }
    Some(id)
}

/// Swap between portable and COM GUID byte order (its own inverse). Only Windows uses COM order.
fn com_order(id: [u8; 16]) -> [u8; 16] {
    if !cfg!(target_os = "windows") {
        return id;
    }
    let mut out = id;
    out[..4].reverse();
    out[4..6].reverse();
    out[6..8].reverse();
    out
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

    #[test]
    fn class_ids_use_the_portable_order_on_every_platform() {
        // `vst3::uid` lays the bytes out as this platform's SDK does (COM order on Windows).
        let id = vst3::uid(0x12345678, 0x9ABCDEF0, 0x0F1E2D3C, 0x4B5A6978);
        assert_eq!(tuid_to_hex(&id), "123456789ABCDEF00F1E2D3C4B5A6978");
        let raw = raw_hex_to_tuid(&tuid_to_hex(&id)).unwrap();
        assert_eq!(raw == id, !cfg!(target_os = "windows"));
    }
}
