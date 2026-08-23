//! Writes into `TrackInfoState`/`AlbumInfoState` on the Slint event loop;
//! builds the paired-column credit model.

use slint::{ComponentHandle, ModelRc, SharedString, VecModel};

use crate::{
    AlbumCreditPerformer, AlbumCreditTrack, AlbumInfoState, AlbumState, AppWindow, InfoCreditPair,
    InfoCreditRow, TrackInfoState,
};

use super::types::{AlbumCreditsData, TrackInfoData};

pub(super) fn apply_track_info(window: &AppWindow, data: TrackInfoData) {
    let st = window.global::<TrackInfoState>();
    // Build the ordered cells, then pair them into 2-column rows so the modal
    // needs no dynamic grid placement.
    let cells: Vec<InfoCreditRow> = data
        .credits
        .into_iter()
        .map(|c| {
            let names_line = c.names.join(", ");
            let names: Vec<SharedString> = c.names.into_iter().map(SharedString::from).collect();
            InfoCreditRow {
                role: c.role.into(),
                role_raw: c.role_raw.into(),
                names: ModelRc::new(VecModel::from(names)),
                names_line: names_line.into(),
            }
        })
        .collect();
    // Two independent columns (even -> left, odd -> right) so the modal can
    // render natural vertical stacks instead of height-coupled paired rows.
    let credits_left: Vec<InfoCreditRow> = cells.iter().step_by(2).cloned().collect();
    let credits_right: Vec<InfoCreditRow> =
        cells.iter().skip(1).step_by(2).cloned().collect();

    let mut pairs: Vec<InfoCreditPair> = Vec::new();
    let mut iter = cells.into_iter();
    while let Some(left) = iter.next() {
        match iter.next() {
            Some(right) => pairs.push(InfoCreditPair {
                left,
                right,
                has_right: true,
            }),
            None => pairs.push(InfoCreditPair {
                left,
                right: InfoCreditRow {
                    role: SharedString::new(),
                    role_raw: SharedString::new(),
                    names: ModelRc::new(VecModel::from(Vec::<SharedString>::new())),
                    names_line: SharedString::new(),
                },
                has_right: false,
            }),
        }
    }
    st.set_title(data.title.into());
    st.set_album(data.album.into());
    st.set_artist(data.artist.into());
    st.set_artist_id(data.artist_id.into());
    st.set_duration(data.duration.into());
    st.set_quality(data.quality.into());
    st.set_isrc(data.isrc.into());
    st.set_label(data.label.into());
    st.set_label_id(data.label_id.into());
    st.set_copyright(data.copyright.into());
    st.set_credits(ModelRc::new(VecModel::from(pairs)));
    st.set_credits_left(ModelRc::new(VecModel::from(credits_left)));
    st.set_credits_right(ModelRc::new(VecModel::from(credits_right)));
}

pub(super) fn apply_album_credits(window: &AppWindow, data: AlbumCreditsData) {
    let st = window.global::<AlbumInfoState>();
    let tracks: Vec<AlbumCreditTrack> = data
        .tracks
        .into_iter()
        .map(|t| {
            let perfs: Vec<AlbumCreditPerformer> = t
                .performers
                .into_iter()
                .map(|p| AlbumCreditPerformer {
                    name: p.name.into(),
                    roles: p.roles.into(),
                    primary_role: p.primary_role.into(),
                })
                .collect();
            AlbumCreditTrack {
                id: t.id.into(),
                number: t.number.into(),
                title: t.title.into(),
                artist: t.artist.into(),
                has_credits: t.has_credits,
                performers: ModelRc::new(VecModel::from(perfs)),
                copyright: t.copyright.into(),
            }
        })
        .collect();
    // The modal opens from the album header, so the cover already lives in
    // AlbumState — reuse it instead of re-fetching the artwork.
    st.set_artwork(window.global::<AlbumState>().get_artwork());
    st.set_title(data.title.into());
    st.set_artist(data.artist.into());
    st.set_label(data.label.into());
    st.set_label_id(data.label_id.into());
    st.set_release_date(data.release_date.into());
    st.set_meta_line(data.meta_line.into());
    st.set_quality(data.quality.into());
    st.set_review(data.review.into());
    st.set_has_review(data.has_review);
    st.set_active_tab("credits".into());
    st.set_tracks(ModelRc::new(VecModel::from(tracks)));
}
