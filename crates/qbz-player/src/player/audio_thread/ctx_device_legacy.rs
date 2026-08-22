use super::super::*;
use super::ctx::ThreadCtx;
use super::ctx_device::is_device_valid;

impl ThreadCtx {
    /// Legacy CPAL device lookup by name, falling back to the default
    /// output device when not found or invalid.
    pub(super) fn find_legacy_device(&self, name: &Option<String>) -> Option<rodio::cpal::Device> {
        if let Some(ref name) = name {
            log::info!("Looking for audio device: {}", name);
            let found = self.host.output_devices().ok().and_then(|mut devices| {
                devices.find(|d| cpal_device_name(d).as_deref() == Some(name.as_str()))
            });

            match found {
                Some(d) if is_device_valid(&d) => {
                    log::info!("Found and validated device: {}", name);
                    Some(d)
                }
                Some(_) => {
                    log::warn!(
                        "Device '{}' found but has no valid output configs, using default",
                        name
                    );
                    self.host.default_output_device()
                }
                None => {
                    log::warn!("Device '{}' not found, using default", name);
                    self.host.default_output_device()
                }
            }
        } else {
            log::info!("Using default audio device");
            self.host.default_output_device()
        }
    }
}
