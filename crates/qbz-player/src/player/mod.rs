//! Audio player module
//!
//! Handles audio playback with support for:
//! - HTTP streaming from Qobuz
//! - FLAC, MP3 decoding via symphonia
//! - Gapless playback
//! - Volume control
//! - Real-time position tracking via events
//!
//! Uses a dedicated audio thread since rodio's OutputStream is not Send.
//! Supports both rodio (PipeWire/Pulse) and direct ALSA (hw: devices).
//!
//! This module's own content is split across sibling files (see
//! refactor-plans/crates__qbz-player__src__player__mod.rs.md): data types,
//! device/stream helpers, `SharedState`, the audio thread (`audio_thread/`,
//! built around a `ThreadCtx` instead of one giant closure), and the public
//! `Player` API (`api/`). This file just wires them together and re-exports
//! the public surface.

mod playback_engine;
mod streaming_source;

pub use streaming_source::{
    max_initial_buffer_bytes, set_max_initial_buffer_bytes, BufferWriter, BufferedMediaSource,
    InMemorySource, IncrementalStreamingSource, StreamingConfig,
};

use rodio::buffer::SamplesBuffer;
use rodio::cpal::traits::{DeviceTrait, HostTrait};
use rodio::cpal::{
    BufferSize, SampleFormat, StreamConfig, SupportedBufferSize, SupportedStreamConfig,
};
use rodio::{Decoder, DeviceSinkBuilder, MixerDeviceSink, Source};
use std::io::{BufReader, Cursor, Read, Seek, SeekFrom};
use std::panic::{self, AssertUnwindSafe};
use std::sync::atomic::{AtomicBool, AtomicU32, AtomicU64, AtomicU8, Ordering};
use std::sync::mpsc::{self, Sender, SyncSender};
use std::sync::{Arc, Mutex};
use std::thread;
use std::time::{Duration, Instant};
use symphonia::core::audio::SampleBuffer;
use symphonia::core::codecs::DecoderOptions;
use symphonia::core::errors::Error as SymphoniaError;
use symphonia::core::formats::FormatOptions;
use symphonia::core::io::{MediaSource, MediaSourceStream};
use symphonia::core::meta::MetadataOptions;
use symphonia::core::probe::Hint;
use symphonia::default::{get_codecs, get_probe};

use playback_engine::PlaybackEngine;
use qbz_audio::{
    calculate_gain_factor, extract_replaygain, AnalyzerMessage, AnalyzerTap,
    AudioBackendType, AudioDiagnostic, AudioSettings, BackendConfig, BackendManager,
    BitPerfectMode, DiagnosticSource, DynamicAmplify, LoudnessCache, TappedSource,
    VisualizerTap,
};
use qbz_models::{AssetOrigin, ExternalStreamAsset, Quality, StreamQualityInfo};
use qbz_qobuz::QobuzClient;

// Small data types and free-function clusters, each in its own file (see
// module doc above). `use x::*` (not `pub use`) so every sibling file's
// `use super::*;` picks these up too — private `use` aliases are visible to
// descendant modules just like the items they name.
mod command;
mod cursor_source;
mod decode;
mod metadata;
mod offline_loudness;
use command::{AudioCommand, GaplessPending};
use cursor_source::{AudioSpecs, CursorMediaSource};
use decode::decode_with_fallback;
use metadata::{cached_quality_below_requested, extract_audio_metadata_full, is_isomp4};

mod backend_init;
mod device_probe;
mod device_stream;
mod stream_recreate;
mod stream_type;
use backend_init::try_init_stream_with_backend;
use device_stream::create_output_stream_with_config;
use stream_recreate::{evaluate_stream_recreate, StreamRecreateDecision};
use stream_type::{apply_engine_volume, cpal_device_name, StreamType};
#[cfg(target_os = "macos")]
use device_probe::coreaudio_shared_rate_mismatch;
#[cfg(test)]
use stream_recreate::compute_needs_new_stream;

mod shared_state;
mod shared_state_buffer;
mod shared_state_status;
mod shared_state_timer;
pub use shared_state::{PlaybackEvent, SharedState};

mod player_struct;
mod player_new;
pub use player_struct::Player;

mod dsd_error_report;
mod external_content_type;
mod playback_state;
use dsd_error_report::DsdErrorReport;
pub use external_content_type::external_content_type;
pub use playback_state::PlaybackState;

mod audio_thread;
mod api;

#[cfg(test)]
mod tests;
