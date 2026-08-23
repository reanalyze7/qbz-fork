use qbz_models::Artist;

use crate::favorites::fetch::FavLabel;
use crate::favorites::mapping::{ArtistCard, LabelCard};

pub(crate) fn map_artist(artist: Artist) -> ArtistCard {
    // albums_count is deliberately NOT shown (Tauri #169: Qobuz's count
    // includes compilations/tributes and is misleadingly high).
    ArtistCard {
        id: artist.id.to_string(),
        name: artist.name,
        image_url: artist
            .image
            .and_then(|img| img.best().cloned())
            .unwrap_or_default(),
    }
}

pub(crate) fn map_label(label: FavLabel) -> LabelCard {
    // Tauri's favorites label card says "{n} albums" (library.albumCount),
    // matching the sibling FavArtistCard's "{n} albums".
    let albums_line = match label.albums_count {
        Some(n) if n > 0 => format!("{} albums", n),
        _ => String::new(),
    };
    LabelCard {
        id: label.id.to_string(),
        name: label.name,
        albums_line,
        image_url: crate::label::extract_label_image(label.image.as_ref()),
    }
}
