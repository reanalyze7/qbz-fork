//! Local-library row derivation for the cortinilla's "on this device"
//! sections: caps, grouping helpers, and the append/fetch entry points.

mod append;
mod caps;
mod derive;
mod load;

pub use append::{append_immersive_local_albums, append_local_sections};
pub use caps::LocalCaps;
pub use load::load_cortinilla_local;
