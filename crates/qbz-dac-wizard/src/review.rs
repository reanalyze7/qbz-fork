//! Slice 10: review-and-apply (per-DAC config generation).

mod conf;
mod state;

pub use state::{apply_configs, checked_dacs, gen_configs_blocking, toggle_config, DacConfigData};
