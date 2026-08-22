/// Walk ISO BMFF boxes and find the first UUID box matching `target_uuid`.
/// Returns `(payload_start, box_end)` where payload_start is after the 16-byte UUID.
pub(super) fn find_uuid_box(data: &[u8], target_uuid: &[u8; 16]) -> Option<(usize, usize)> {
    let mut pos = 0;
    while pos + 8 <= data.len() {
        let size = read_box_size(data, pos);
        if size < 8 || pos + size > data.len() {
            break;
        }
        if &data[pos + 4..pos + 8] == b"uuid" && pos + 24 <= data.len() {
            if &data[pos + 8..pos + 24] == target_uuid.as_ref() {
                // payload_start = box_start + 8 (header) + 16 (uuid) = box_start + 24
                return Some((pos + 24, pos + size));
            }
        }
        pos += size;
    }
    None
}

/// Walk ISO BMFF boxes and find the `mdat` box.
/// Returns `(data_start, box_end)` where data_start = box_start + 8.
#[allow(dead_code)]
pub(super) fn find_mdat_box(data: &[u8]) -> Option<(usize, usize)> {
    let mut pos = 0;
    while pos + 8 <= data.len() {
        let size = read_box_size(data, pos);
        if size < 8 || pos + size > data.len() {
            break;
        }
        if &data[pos + 4..pos + 8] == b"mdat" {
            return Some((pos + 8, pos + size));
        }
        pos += size;
    }
    None
}

pub(super) fn read_box_size(data: &[u8], pos: usize) -> usize {
    if pos + 8 > data.len() {
        return 0;
    }
    let s = u32::from_be_bytes([data[pos], data[pos + 1], data[pos + 2], data[pos + 3]]);
    match s {
        0 => data.len() - pos,
        1..=7 => 0,
        s => s as usize,
    }
}
