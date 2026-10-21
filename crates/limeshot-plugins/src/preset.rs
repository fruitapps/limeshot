//! VST3 preset files (`.vstpreset`, Steinberg's documented format) and the standard folders they
//! live in. FL Studio's Presets ◀ ▶ and presets menu list them after the plugin's own programs.
//!
//! Layout: `"VST3"`, version (i32), the class id as 32 ASCII hex digits, the offset of the chunk
//! list (i64); the chunk data; then `"List"`, a count (i32) and per chunk an id (4 bytes), an
//! offset and a size (i64 each). `Comp` holds the processor's state, `Cont` the controller's.
//! All integers are little-endian.

use std::path::{Path, PathBuf};

use crate::PluginError;

/// The largest preset file read (a plugin state is rarely more than a few MiB).
const MAX_PRESET_BYTES: u64 = 64 << 20;
const HEADER: usize = 4 + 4 + 32 + 8;

/// The contents of a `.vstpreset` file.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct VstPreset {
    /// 32 hexadecimal digits, as in the plugin's factory.
    pub class_id: String,
    pub component: Vec<u8>,
    pub controller: Vec<u8>,
}

impl VstPreset {
    pub fn parse(bytes: &[u8]) -> Result<Self, PluginError> {
        let bad = || PluginError::State;
        if bytes.len() < HEADER || &bytes[..4] != b"VST3" {
            return Err(bad());
        }
        let class_id = std::str::from_utf8(&bytes[8..40])
            .map_err(|_| bad())?
            .to_ascii_uppercase();
        let list = usize::try_from(i64_at(bytes, 40).ok_or_else(bad)?).map_err(|_| bad())?;
        if bytes.get(list..list + 4) != Some(b"List") {
            return Err(bad());
        }
        let count = usize::try_from(i32_at(bytes, list + 4).ok_or_else(bad)?).map_err(|_| bad())?;
        let mut preset = VstPreset {
            class_id,
            component: Vec::new(),
            controller: Vec::new(),
        };
        for i in 0..count.min(64) {
            let at = list + 8 + i * 20;
            let id = bytes.get(at..at + 4).ok_or_else(bad)?;
            let offset = usize::try_from(i64_at(bytes, at + 4).ok_or_else(bad)?).map_err(|_| bad())?;
            let size = usize::try_from(i64_at(bytes, at + 12).ok_or_else(bad)?).map_err(|_| bad())?;
            let data = bytes
                .get(offset..offset.checked_add(size).ok_or_else(bad)?)
                .ok_or_else(bad)?;
            match id {
                b"Comp" => preset.component = data.to_vec(),
                b"Cont" => preset.controller = data.to_vec(),
                _ => {}
            }
        }
        Ok(preset)
    }

    pub fn to_bytes(&self) -> Vec<u8> {
        let mut out = Vec::with_capacity(HEADER + self.component.len() + self.controller.len() + 48);
        out.extend_from_slice(b"VST3");
        out.extend_from_slice(&1i32.to_le_bytes());
        let mut id = self.class_id.as_bytes().to_vec();
        id.resize(32, b'0');
        out.extend_from_slice(&id);
        out.extend_from_slice(&0i64.to_le_bytes());
        let mut chunks = Vec::new();
        for (tag, data) in [(b"Comp", &self.component), (b"Cont", &self.controller)] {
            if tag == b"Cont" && data.is_empty() {
                continue;
            }
            chunks.push((tag, out.len(), data.len()));
            out.extend_from_slice(data);
        }
        let list = out.len() as i64;
        out[40..48].copy_from_slice(&list.to_le_bytes());
        out.extend_from_slice(b"List");
        out.extend_from_slice(&(chunks.len() as i32).to_le_bytes());
        for (tag, offset, size) in chunks {
            out.extend_from_slice(tag);
            out.extend_from_slice(&(offset as i64).to_le_bytes());
            out.extend_from_slice(&(size as i64).to_le_bytes());
        }
        out
    }

    pub fn read(path: &Path) -> Result<Self, PluginError> {
        let len = std::fs::metadata(path).map(|m| m.len()).unwrap_or(u64::MAX);
        if len > MAX_PRESET_BYTES {
            return Err(PluginError::State);
        }
        let bytes = std::fs::read(path).map_err(|_| PluginError::State)?;
        Self::parse(&bytes)
    }
}

fn i32_at(b: &[u8], at: usize) -> Option<i32> {
    Some(i32::from_le_bytes(b.get(at..at + 4)?.try_into().ok()?))
}

fn i64_at(b: &[u8], at: usize) -> Option<i64> {
    Some(i64::from_le_bytes(b.get(at..at + 8)?.try_into().ok()?))
}

/// The platform's VST3 preset folders, user folder first (the one presets are saved to).
pub fn preset_dirs() -> Vec<PathBuf> {
    let mut dirs = Vec::new();
    let home = std::env::var_os("HOME").map(PathBuf::from);
    if cfg!(target_os = "macos") {
        if let Some(h) = &home {
            dirs.push(h.join("Library/Audio/Presets"));
        }
        dirs.push(PathBuf::from("/Library/Audio/Presets"));
    } else if cfg!(windows) {
        if let Some(p) = std::env::var_os("USERPROFILE") {
            dirs.push(PathBuf::from(p).join(r"Documents\VST3 Presets"));
        }
        if let Some(p) = std::env::var_os("PROGRAMDATA") {
            dirs.push(PathBuf::from(p).join("VST3 Presets"));
        }
    } else {
        if let Some(h) = &home {
            dirs.push(h.join(".vst3/presets"));
        }
        dirs.push(PathBuf::from("/usr/share/vst3/presets"));
        dirs.push(PathBuf::from("/usr/local/share/vst3/presets"));
    }
    dirs
}

/// The folder a plugin's presets are saved to: `<user presets>/<vendor>/<plugin>`.
pub fn user_preset_dir(vendor: &str, name: &str) -> Option<PathBuf> {
    let clean = |s: &str| -> String {
        s.chars()
            .map(|c| if c == '/' || c == '\\' || c == ':' { '_' } else { c })
            .collect::<String>()
            .trim()
            .to_string()
    };
    let (vendor, name) = (clean(vendor), clean(name));
    if name.is_empty() {
        return None;
    }
    let base = preset_dirs().into_iter().next()?;
    Some(if vendor.is_empty() {
        base.join(name)
    } else {
        base.join(vendor).join(name)
    })
}

/// The `.vstpreset` files for class `class_id` in `<dir>/<vendor>/<name>` (and `<dir>/<name>`)
/// of every folder in `dirs`, sorted by name. Files made for another class are left out.
pub fn find_presets(dirs: &[PathBuf], vendor: &str, name: &str, class_id: &str) -> Vec<PathBuf> {
    let mut out = Vec::new();
    for d in dirs {
        for sub in [d.join(vendor).join(name), d.join(name)] {
            collect(&sub, 0, &mut out);
        }
    }
    out.sort_by_key(|p| p.file_stem().map(|s| s.to_string_lossy().to_lowercase()));
    out.dedup();
    out.retain(|p| preset_class(p).is_some_and(|c| c.eq_ignore_ascii_case(class_id)));
    out
}

fn collect(dir: &Path, depth: u32, out: &mut Vec<PathBuf>) {
    if depth > 4 || out.len() > 10_000 {
        return;
    }
    let Ok(entries) = std::fs::read_dir(dir) else {
        return;
    };
    for e in entries.flatten() {
        let p = e.path();
        if p.is_dir() {
            collect(&p, depth + 1, out);
        } else if p.extension().is_some_and(|x| x.eq_ignore_ascii_case("vstpreset")) {
            out.push(p);
        }
    }
}

/// The class id in a preset file's header, read without loading the rest.
fn preset_class(path: &Path) -> Option<String> {
    use std::io::Read as _;
    let mut head = [0u8; 40];
    std::fs::File::open(path).ok()?.read_exact(&mut head).ok()?;
    (&head[..4] == b"VST3").then(|| String::from_utf8_lossy(&head[8..40]).into_owned())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn a_preset_round_trips_and_garbage_is_refused() {
        let p = VstPreset {
            class_id: "0123456789ABCDEF0123456789ABCDEF".into(),
            component: b"processor".to_vec(),
            controller: b"edit".to_vec(),
        };
        let bytes = p.to_bytes();
        assert_eq!(VstPreset::parse(&bytes).unwrap(), p);
        assert!(VstPreset::parse(b"VST3").is_err());
        let mut cut = bytes.clone();
        cut.truncate(bytes.len() - 3);
        assert!(VstPreset::parse(&cut).is_err());
        let mut lying = bytes;
        lying[40..48].copy_from_slice(&i64::MAX.to_le_bytes());
        assert!(VstPreset::parse(&lying).is_err());
    }

    #[test]
    fn presets_are_found_by_folder_and_class() {
        let dir = std::env::temp_dir().join(format!("limeshot-presets-{}", std::process::id()));
        let plugin = dir.join("Lime").join("Gain");
        std::fs::create_dir_all(plugin.join("Factory")).unwrap();
        let mine = VstPreset {
            class_id: "AA".repeat(16),
            component: vec![1],
            controller: Vec::new(),
        };
        let other = VstPreset {
            class_id: "BB".repeat(16),
            ..mine.clone()
        };
        std::fs::write(plugin.join("b Loud.vstpreset"), mine.to_bytes()).unwrap();
        std::fs::write(plugin.join("Factory/a Soft.vstpreset"), mine.to_bytes()).unwrap();
        std::fs::write(plugin.join("Other.vstpreset"), other.to_bytes()).unwrap();
        std::fs::write(plugin.join("notes.txt"), b"x").unwrap();
        let found = find_presets(std::slice::from_ref(&dir), "Lime", "Gain", &"aa".repeat(16));
        let names: Vec<_> = found.iter().map(|p| p.file_stem().unwrap().to_string_lossy()).collect();
        assert_eq!(names, ["a Soft", "b Loud"]);
        std::fs::remove_dir_all(&dir).unwrap();
    }
}
