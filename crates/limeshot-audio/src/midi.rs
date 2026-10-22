//! MIDI input from hardware and virtual ports (`midir`): FL Studio's MIDI settings pick an
//! input device, whose notes then play the selected channel. Messages are parsed on midir's
//! thread and handed to the UI thread over a channel; the UI forwards notes to the engine.

use std::sync::mpsc::{Receiver, channel};

/// A MIDI message LimeShot uses. The MIDI channel is ignored (FL Studio's "omni" input).
#[derive(Debug, Clone, Copy, PartialEq)]
pub enum MidiEvent {
    /// Velocity 0–1 (a note-on with velocity 0 is a note-off).
    NoteOn {
        key: u8,
        velocity: f32,
    },
    NoteOff {
        key: u8,
    },
    /// Controller number and value 0–1.
    Control {
        cc: u8,
        value: f32,
    },
    /// −1–1.
    PitchBend(f32),
}

/// Parse one MIDI message (running status is resolved by the OS drivers midir uses).
pub fn parse(bytes: &[u8]) -> Option<MidiEvent> {
    let (&status, data) = bytes.split_first()?;
    let d = |i: usize| data.get(i).copied().filter(|b| *b < 0x80);
    match status & 0xF0 {
        0x90 => {
            let (key, vel) = (d(0)?, d(1)?);
            Some(if vel == 0 {
                MidiEvent::NoteOff { key }
            } else {
                MidiEvent::NoteOn {
                    key,
                    velocity: f32::from(vel) / 127.0,
                }
            })
        }
        0x80 => Some(MidiEvent::NoteOff { key: d(0)? }),
        0xB0 => Some(MidiEvent::Control {
            cc: d(0)?,
            value: f32::from(d(1)?) / 127.0,
        }),
        0xE0 => {
            let v = i32::from(d(0)?) | (i32::from(d(1)?) << 7);
            Some(MidiEvent::PitchBend(((v - 8192) as f32 / 8192.0).clamp(-1.0, 1.0)))
        }
        _ => None,
    }
}

/// The names of the MIDI input ports on this computer.
pub fn input_ports() -> Vec<String> {
    let Ok(input) = midir::MidiInput::new("LimeShot") else {
        return Vec::new();
    };
    input.ports().iter().filter_map(|p| input.port_name(p).ok()).collect()
}

/// An open MIDI input port.
pub struct MidiIn {
    name: String,
    events: Receiver<MidiEvent>,
    _conn: midir::MidiInputConnection<()>,
}

impl MidiIn {
    /// Open the input port called `name`; `wake` is called (on midir's thread) after each
    /// message, so the UI can repaint and pick it up at once.
    pub fn open(name: &str, wake: impl Fn() + Send + 'static) -> Result<Self, String> {
        let mut input = midir::MidiInput::new("LimeShot").map_err(|e| e.to_string())?;
        input.ignore(midir::Ignore::All);
        let port = input
            .ports()
            .into_iter()
            .find(|p| input.port_name(p).is_ok_and(|n| n == name))
            .ok_or_else(|| format!("MIDI input \"{name}\" is not connected"))?;
        let (tx, rx) = channel();
        let conn = input
            .connect(
                &port,
                "LimeShot input",
                move |_, bytes, _| {
                    if let Some(e) = parse(bytes)
                        && tx.send(e).is_ok()
                    {
                        wake();
                    }
                },
                (),
            )
            .map_err(|e| e.to_string())?;
        Ok(MidiIn {
            name: name.to_string(),
            events: rx,
            _conn: conn,
        })
    }

    pub fn name(&self) -> &str {
        &self.name
    }

    /// The messages received since the last call.
    pub fn drain(&self) -> Vec<MidiEvent> {
        self.events.try_iter().collect()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn parses_notes_controllers_and_bends_on_any_channel() {
        assert_eq!(
            parse(&[0x90, 60, 127]),
            Some(MidiEvent::NoteOn { key: 60, velocity: 1.0 })
        );
        assert_eq!(
            parse(&[0x93, 61, 0]),
            Some(MidiEvent::NoteOff { key: 61 }),
            "velocity 0, channel 4"
        );
        assert_eq!(parse(&[0x8F, 62, 40]), Some(MidiEvent::NoteOff { key: 62 }));
        assert_eq!(parse(&[0xB0, 1, 127]), Some(MidiEvent::Control { cc: 1, value: 1.0 }));
        assert_eq!(parse(&[0xE0, 0, 64]), Some(MidiEvent::PitchBend(0.0)));
        assert_eq!(parse(&[0xE0, 0, 0]), Some(MidiEvent::PitchBend(-1.0)));
        // Truncated, data bytes with the top bit set, clock and sysex are ignored.
        assert_eq!(parse(&[0x90, 60]), None);
        assert_eq!(parse(&[0x90, 0x80, 1]), None);
        assert_eq!(parse(&[0xF8]), None);
        assert_eq!(parse(&[]), None);
    }

    /// A virtual port stands in for a keyboard: what it sends arrives parsed through
    /// `MidiIn`. Skipped where the system has no MIDI service.
    #[cfg(unix)]
    #[test]
    fn a_virtual_port_delivers_notes() {
        use midir::os::unix::VirtualOutput;
        let Ok(out) = midir::MidiOutput::new("LimeShot test") else {
            return;
        };
        let name = format!("LimeShot test {}", std::process::id());
        let Ok(mut port) = out.create_virtual(&name) else {
            return;
        };
        let woken = std::sync::Arc::new(std::sync::atomic::AtomicUsize::new(0));
        let w = woken.clone();
        let input = match MidiIn::open(&name, move || {
            w.fetch_add(1, std::sync::atomic::Ordering::Relaxed);
        }) {
            Ok(i) => i,
            // Some systems list virtual ports only after a moment, or not to the same process.
            Err(e) => {
                eprintln!("skipped: {e}");
                return;
            }
        };
        assert!(input_ports().contains(&name));
        port.send(&[0x90, 60, 100]).unwrap();
        port.send(&[0x80, 60, 0]).unwrap();
        let start = std::time::Instant::now();
        let mut got = Vec::new();
        while got.len() < 2 && start.elapsed().as_secs() < 3 {
            got.extend(input.drain());
            std::thread::sleep(std::time::Duration::from_millis(5));
        }
        assert_eq!(
            got,
            vec![
                MidiEvent::NoteOn {
                    key: 60,
                    velocity: 100.0 / 127.0
                },
                MidiEvent::NoteOff { key: 60 }
            ]
        );
        assert!(woken.load(std::sync::atomic::Ordering::Relaxed) >= 2);
    }
}
