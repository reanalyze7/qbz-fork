//! Mosaic-cover artwork job building.

use slint::{ComponentHandle, Model};

use crate::artwork::{ArtworkJob, ArtworkTarget};
use crate::{AppWindow, MyQbzState};

use super::Grid;

/// Build mosaic-cover artwork jobs for every visible card of `grid`.
pub fn artwork_jobs(window: &AppWindow, grid: Grid) -> Vec<ArtworkJob> {
    let state = window.global::<MyQbzState>();
    let model = match grid {
        Grid::Mixtapes => state.get_mixtapes(),
        Grid::Collections => state.get_collections(),
    };
    let mut jobs = Vec::new();
    for index in 0..model.row_count() {
        let Some(card) = model.row_data(index) else { continue };
        let urls = [
            card.url1, card.url2, card.url3, card.url4, card.url5, card.url6, card.url7,
            card.url8, card.url9,
        ];
        for (slot, url) in urls.iter().enumerate() {
            if url.is_empty() {
                continue;
            }
            let target = match grid {
                Grid::Mixtapes => ArtworkTarget::MyQbzMixtapeCover { index, slot },
                Grid::Collections => ArtworkTarget::MyQbzCollectionCover { index, slot },
            };
            jobs.push(ArtworkJob {
                target,
                url: url.to_string(),
            });
        }
    }
    jobs
}
