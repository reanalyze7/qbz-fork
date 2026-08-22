use super::QobuzClient;
use crate::endpoints::{self, paths};
use crate::error::Result;
use qbz_models::*;

impl QobuzClient {
    /// Create a new playlist
    pub async fn create_playlist(
        &self,
        name: &str,
        description: Option<&str>,
        is_public: bool,
    ) -> Result<Playlist> {
        let url = endpoints::build_url(paths::PLAYLIST_CREATE);

        let mut params = vec![
            ("name", name.to_string()),
            ("is_public", is_public.to_string()),
        ];
        if let Some(desc) = description {
            params.push(("description", desc.to_string()));
        }

        let params: Vec<(&str, String)> = params.iter().map(|(k, v)| (*k, v.clone())).collect();
        let response: Playlist = self
            .signed_get_auth(&url, "playlistcreate", &params)
            .await?
            .json()
            .await?;

        Ok(response)
    }

    /// Delete a playlist
    pub async fn delete_playlist(&self, playlist_id: u64) -> Result<()> {
        let url = endpoints::build_url(paths::PLAYLIST_DELETE);

        self.signed_get_auth(&url, "playlistdelete", &[("playlist_id", playlist_id.to_string())])
            .await?;

        Ok(())
    }

    /// Add tracks to a playlist
    pub async fn add_tracks_to_playlist(&self, playlist_id: u64, track_ids: &[u64]) -> Result<()> {
        let url = endpoints::build_url(paths::PLAYLIST_ADD_TRACKS);
        let track_ids_str = track_ids
            .iter()
            .map(|id| id.to_string())
            .collect::<Vec<_>>()
            .join(",");

        self.signed_get_auth(&url, "playlistaddTracks", &[
            ("playlist_id", playlist_id.to_string()),
            ("track_ids", track_ids_str),
        ])
        .await?;

        Ok(())
    }

    /// Remove tracks from a playlist
    pub async fn remove_tracks_from_playlist(
        &self,
        playlist_id: u64,
        playlist_track_ids: &[u64],
    ) -> Result<()> {
        let url = endpoints::build_url(paths::PLAYLIST_DELETE_TRACKS);
        let track_ids_str = playlist_track_ids
            .iter()
            .map(|id| id.to_string())
            .collect::<Vec<_>>()
            .join(",");

        self.signed_get_auth(&url, "playlistdeleteTracks", &[
            ("playlist_id", playlist_id.to_string()),
            ("playlist_track_ids", track_ids_str),
        ])
        .await?;

        Ok(())
    }

    /// Update playlist metadata
    pub async fn update_playlist(
        &self,
        playlist_id: u64,
        name: Option<&str>,
        description: Option<&str>,
        is_public: Option<bool>,
    ) -> Result<Playlist> {
        let url = endpoints::build_url(paths::PLAYLIST_UPDATE);

        let mut params = vec![("playlist_id", playlist_id.to_string())];
        if let Some(n) = name {
            params.push(("name", n.to_string()));
        }
        if let Some(d) = description {
            params.push(("description", d.to_string()));
        }
        if let Some(p) = is_public {
            params.push(("is_public", p.to_string()));
        }

        let params_ref: Vec<(&str, String)> = params.iter().map(|(k, v)| (*k, v.clone())).collect();
        let response: Playlist = self
            .signed_get_auth(&url, "playlistupdate", &params_ref)
            .await?
            .json()
            .await?;

        Ok(response)
    }
}
