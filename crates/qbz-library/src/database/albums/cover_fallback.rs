use super::super::LibraryDatabase;

impl LibraryDatabase {
    /// Resolve a folder cover for an album that has no `artwork_path` in the
    /// index — e.g. an offline-cached (CMAF) album whose downloader wrote a
    /// `cover.jpg` into the track folder but didn't backfill `artwork_path`.
    /// Looks up a representative track for the metadata group, derives its
    /// containing folder (the path itself when it is a directory, as for CMAF
    /// bundles, else the parent dir), and returns `<folder>/cover.jpg` when
    /// that file exists. Frontend-agnostic (no `tauri::State`).
    pub fn resolve_album_cover_fallback(&self, group_key: &str) -> Option<String> {
        // Common on-disk cover filenames (the offline-cache writes cover.jpg;
        // ripped/local folders often use folder.jpg / front.*).
        const NAMES: [&str; 6] = [
            "cover.jpg",
            "cover.png",
            "folder.jpg",
            "Folder.jpg",
            "front.jpg",
            "front.png",
        ];
        let expr = crate::album_grouping::metadata_group_key_sql_expression();
        // Match by the metadata group key AND the raw folder key — the album
        // id depends on the Albums view's grouping mode (album|artist in
        // Metadata mode, the folder path in Folder mode). The OR keeps the
        // lookup correct under either.
        // Scan several tracks, not just one: a CMAF album keeps each track in
        // its own folder, and only some may carry a cover.jpg.
        let query = format!(
            "SELECT file_path FROM local_tracks WHERE ({expr}) = ?1 OR album_group_key = ?1 LIMIT 12"
        );
        let mut stmt = self.conn.prepare(&query).ok()?;
        let paths: Vec<String> = stmt
            .query_map(rusqlite::params![group_key], |row| row.get::<_, String>(0))
            .ok()?
            .filter_map(Result::ok)
            .collect();
        for fp in &paths {
            let p = std::path::Path::new(fp);
            // The track folder: the path itself for a CMAF bundle dir, else
            // the parent of the audio file.
            let Some(folder) = (if p.is_dir() {
                Some(p.to_path_buf())
            } else {
                p.parent().map(|x| x.to_path_buf())
            }) else {
                continue;
            };
            // Check the folder and its parent (covers multi-disc layouts where
            // the art sits one level up).
            let dirs = [Some(folder.clone()), folder.parent().map(|x| x.to_path_buf())];
            for dir in dirs.into_iter().flatten() {
                for name in NAMES {
                    let cover = dir.join(name);
                    if cover.is_file() {
                        return Some(cover.to_string_lossy().into_owned());
                    }
                }
            }
        }
        None
    }
}
