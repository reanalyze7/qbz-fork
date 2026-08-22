use super::*;

#[test]
fn detect_provider_table() {
    // Spotify URI / URL / embed forms + query strip
    assert_eq!(
        detect_provider("spotify:playlist:37i9dQZF1DXcBWIGoYBM5M").unwrap(),
        ProviderKind::Spotify {
            playlist_id: "37i9dQZF1DXcBWIGoYBM5M".to_string()
        }
    );
    assert_eq!(
        detect_provider("https://open.spotify.com/playlist/37i9dQZF1DXcBWIGoYBM5M?si=abc")
            .unwrap(),
        ProviderKind::Spotify {
            playlist_id: "37i9dQZF1DXcBWIGoYBM5M".to_string()
        }
    );
    assert_eq!(
        detect_provider("https://open.spotify.com/embed/playlist/37i9dQZF1DXcBWIGoYBM5M")
            .unwrap(),
        ProviderKind::Spotify {
            playlist_id: "37i9dQZF1DXcBWIGoYBM5M".to_string()
        }
    );

    // Apple storefront + pl. ids
    assert_eq!(
        detect_provider("https://music.apple.com/us/playlist/top-100-global/pl.d25f5d1181894928af76c85c967f8f31")
            .unwrap(),
        ProviderKind::AppleMusic {
            storefront: "us".to_string(),
            playlist_id: "pl.d25f5d1181894928af76c85c967f8f31".to_string()
        }
    );

    // Tidal
    assert_eq!(
        detect_provider(
            "https://tidal.com/browse/playlist/1b418bb8-90a7-4f87-901d-707993838346"
        )
        .unwrap(),
        ProviderKind::Tidal {
            playlist_id: "1b418bb8-90a7-4f87-901d-707993838346".to_string()
        }
    );

    // Deezer
    assert_eq!(
        detect_provider("https://www.deezer.com/en/playlist/1234567890").unwrap(),
        ProviderKind::Deezer {
            playlist_id: "1234567890".to_string()
        }
    );

    // Rejects
    assert!(detect_provider("https://example.com/playlist/1").is_err());
    assert!(detect_provider("https://open.spotify.com/track/abc").is_err());
    assert!(detect_provider("").is_err());
}

#[test]
fn detect_music_resource_song_link_and_none() {
    assert_eq!(
        detect_music_resource("https://song.link/i/1440857781"),
        Some(MusicResource::SongLink {
            url: "https://song.link/i/1440857781".to_string()
        })
    );
    assert_eq!(detect_music_resource(""), None);
    assert_eq!(detect_music_resource("https://example.com/whatever"), None);
}

#[test]
fn detect_music_resource_playlist_routes_to_importer() {
    assert_eq!(
        detect_music_resource("https://open.spotify.com/playlist/37i9dQZF1DXcBWIGoYBM5M"),
        Some(MusicResource::Playlist {
            provider: MusicProvider::Spotify
        })
    );
}
