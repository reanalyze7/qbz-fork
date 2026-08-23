//! Artist portrait pipeline: generation guard, paint, and the capped
//! background Qobuz fetch for missing images.

mod fetch;
mod state;

pub use fetch::fetch_missing_artist_images;
pub use state::{artists_img_gen_current, set_artist_row_image};

pub(crate) use state::ARTISTS_IMG_GEN;
