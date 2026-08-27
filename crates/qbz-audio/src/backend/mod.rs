//! Audio backend abstraction
//!
//! Provides a unified interface for different audio backends (PipeWire, ALSA, PulseAudio)
//! allowing users to choose their preferred audio stack.
//!
//! Split by responsibility: cross-backend data types (`types`, `error`), the
//! `AudioBackend` trait contract (`trait_def`), the `BackendManager` factory
//! (`manager`, `manager_detect`), and the concrete `CpalDefaultBackend`
//! ("System") implementation (`cpal_default*`).

mod cpal_default;
mod cpal_default_enum;
mod cpal_default_stream;
mod device_config;
mod error;
mod manager;
mod manager_detect;
mod trait_def;
mod types;

pub use cpal_default::CpalDefaultBackend;
pub use device_config::{AudioDevice, BackendConfig, BackendResult};
pub use error::{AlsaDirectError, BitPerfectMode};
pub use manager::BackendManager;
pub use trait_def::AudioBackend;
pub use types::{AlsaPlugin, AudioBackendType};
