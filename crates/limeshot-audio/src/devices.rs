//! Output device discovery and configuration.

use cpal::traits::{DeviceTrait, HostTrait};
use serde::{Deserialize, Serialize};

/// User choices for the audio output; `None` means "the system default".
#[derive(Debug, Clone, Default, PartialEq, Eq, Serialize, Deserialize)]
#[serde(default)]
pub struct AudioConfig {
    pub device: Option<String>,
    pub sample_rate: Option<u32>,
    pub buffer_size: Option<u32>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct DeviceInfo {
    pub name: String,
    pub is_default: bool,
    pub sample_rates: Vec<u32>,
    pub buffer_min: Option<u32>,
    pub buffer_max: Option<u32>,
}

const STANDARD_RATES: [u32; 6] = [22_050, 44_100, 48_000, 88_200, 96_000, 192_000];

pub(crate) fn supported_format(f: cpal::SampleFormat) -> bool {
    matches!(f, cpal::SampleFormat::F32 | cpal::SampleFormat::I16 | cpal::SampleFormat::I32 | cpal::SampleFormat::U16)
}

/// Every output device of the default host.
pub fn list_output_devices() -> Vec<DeviceInfo> {
    let host = cpal::default_host();
    let default = host.default_output_device().and_then(|d| d.name().ok());
    let Ok(devices) = host.output_devices() else { return Vec::new() };
    devices
        .filter_map(|d| {
            let name = d.name().ok()?;
            let ranges: Vec<_> = d.supported_output_configs().ok()?.filter(|c| supported_format(c.sample_format())).collect();
            let sample_rates = STANDARD_RATES
                .iter()
                .copied()
                .filter(|r| ranges.iter().any(|c| c.min_sample_rate().0 <= *r && *r <= c.max_sample_rate().0))
                .collect();
            let (mut lo, mut hi) = (None::<u32>, None::<u32>);
            for c in &ranges {
                if let cpal::SupportedBufferSize::Range { min, max } = c.buffer_size() {
                    lo = Some(lo.map_or(*min, |v| v.min(*min)));
                    hi = Some(hi.map_or(*max, |v| v.max(*max)));
                }
            }
            Some(DeviceInfo { is_default: default.as_deref() == Some(name.as_str()), name, sample_rates, buffer_min: lo, buffer_max: hi })
        })
        .collect()
}

/// The named output device, or the default one.
pub(crate) fn find_output(host: &cpal::Host, name: Option<&str>) -> Option<cpal::Device> {
    if let Some(want) = name
        && let Ok(mut devices) = host.output_devices()
        && let Some(d) = devices.find(|d| d.name().ok().as_deref() == Some(want))
    {
        return Some(d);
    }
    host.default_output_device()
}

/// A fixed buffer size when the request is within what the device supports.
pub(crate) fn buffer_size(want: Option<u32>, supported: &cpal::SupportedBufferSize) -> cpal::BufferSize {
    match (want, supported) {
        (Some(n), cpal::SupportedBufferSize::Range { min, max }) if (*min..=*max).contains(&n) => cpal::BufferSize::Fixed(n),
        (Some(n), cpal::SupportedBufferSize::Unknown) if (32..=8192).contains(&n) => cpal::BufferSize::Fixed(n),
        _ => cpal::BufferSize::Default,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn buffer_size_requests_are_validated() {
        let r = cpal::SupportedBufferSize::Range { min: 64, max: 2048 };
        assert_eq!(buffer_size(Some(256), &r), cpal::BufferSize::Fixed(256));
        assert_eq!(buffer_size(Some(16), &r), cpal::BufferSize::Default);
        assert_eq!(buffer_size(None, &r), cpal::BufferSize::Default);
        assert_eq!(buffer_size(Some(1 << 20), &cpal::SupportedBufferSize::Unknown), cpal::BufferSize::Default);
    }
}
