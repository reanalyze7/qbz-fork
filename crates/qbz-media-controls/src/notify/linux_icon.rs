const PORTAL_ICON_MAX_EDGE: u32 = 512;
const PORTAL_ICON_MAX_BYTES: usize = 4 * 1024 * 1024;

/// Center-crop to a square, downscale to <=512px, re-encode PNG. Mirrors the
/// Tauri `v2_prepare_notification_icon_bytes`.
///
/// Decodes by CONTENT, never by extension: the shared disk-image cache names
/// files `{md5}.img` (no real image extension), and `image::open` resolves the
/// format from the path extension only — it returned `Unsupported` for every
/// cache hit, which is exactly the common online case.
pub(super) fn prepare_icon_bytes(path: &std::path::Path) -> Result<Vec<u8>, String> {
    use std::io::Cursor;

    let bytes =
        std::fs::read(path).map_err(|e| format!("Failed to read artwork {path:?}: {e}"))?;
    let source = image::load_from_memory(&bytes)
        .map_err(|e| format!("Failed to decode artwork {path:?}: {e}"))?;
    let (w, h) = (source.width(), source.height());
    let square = if w == h {
        source
    } else {
        let edge = w.min(h);
        source.crop_imm((w - edge) / 2, (h - edge) / 2, edge, edge)
    };
    let icon = if square.width() > PORTAL_ICON_MAX_EDGE {
        square.resize_exact(
            PORTAL_ICON_MAX_EDGE,
            PORTAL_ICON_MAX_EDGE,
            image::imageops::FilterType::Lanczos3,
        )
    } else {
        square
    };
    let mut buf = Cursor::new(Vec::new());
    icon.write_to(&mut buf, image::ImageFormat::Png)
        .map_err(|e| format!("Failed to encode notification PNG: {e}"))?;
    let bytes = buf.into_inner();
    if bytes.len() > PORTAL_ICON_MAX_BYTES {
        return Err(format!(
            "Notification icon too large after normalization: {} bytes (max {PORTAL_ICON_MAX_BYTES})",
            bytes.len()
        ));
    }
    Ok(bytes)
}
