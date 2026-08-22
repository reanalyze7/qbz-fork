//! Synthetic DSF/DFF file builders used by the integration tests.

use std::io::Write;
use std::path::PathBuf;

pub fn tmp(name: &str) -> PathBuf {
    PathBuf::from(env!("CARGO_TARGET_TMPDIR")).join(name)
}

/// Minimal valid DSF: `groups` block-groups of 0x69 silence.
pub fn write_dsf(
    name: &str,
    channels: u32,
    rate: u32,
    groups: usize,
    metadata: Option<&[u8]>,
) -> PathBuf {
    let block_size = 4096u32;
    let bytes_per_ch = groups * block_size as usize;
    let sample_count = (bytes_per_ch as u64) * 8;
    let data_len = (bytes_per_ch * channels as usize) as u64;
    // DSD chunk (28) + fmt chunk (52, header included) + data header (12).
    let file_len_without_meta = 28 + 52 + 12 + data_len;
    let metadata_ptr = if metadata.is_some() { file_len_without_meta } else { 0 };

    let mut f = Vec::new();
    f.extend_from_slice(b"DSD ");
    f.extend_from_slice(&28u64.to_le_bytes());
    f.extend_from_slice(&(file_len_without_meta + metadata.map_or(0, |m| m.len() as u64)).to_le_bytes());
    f.extend_from_slice(&metadata_ptr.to_le_bytes());
    f.extend_from_slice(b"fmt ");
    f.extend_from_slice(&52u64.to_le_bytes());
    f.extend_from_slice(&1u32.to_le_bytes()); // version
    f.extend_from_slice(&0u32.to_le_bytes()); // format id = DSD raw
    f.extend_from_slice(&2u32.to_le_bytes()); // channel type (stereo)
    f.extend_from_slice(&channels.to_le_bytes());
    f.extend_from_slice(&rate.to_le_bytes());
    f.extend_from_slice(&1u32.to_le_bytes()); // bits per sample: 1 = LSB first
    f.extend_from_slice(&sample_count.to_le_bytes());
    f.extend_from_slice(&block_size.to_le_bytes());
    f.extend_from_slice(&0u32.to_le_bytes()); // reserved
    f.extend_from_slice(b"data");
    f.extend_from_slice(&(12 + data_len).to_le_bytes());
    for _ in 0..groups {
        for _ in 0..channels {
            f.extend_from_slice(&vec![0x69u8; block_size as usize]);
        }
    }
    if let Some(m) = metadata {
        f.extend_from_slice(m);
    }
    let path = tmp(name);
    std::fs::File::create(&path).unwrap().write_all(&f).unwrap();
    path
}

/// Minimal DFF: FRM8 { FVER, PROP(SND){ FS, CHNL, CMPR }, DSD data }.
pub fn write_dff(name: &str, channels: u16, rate: u32, data_bytes_total: usize, cmpr: &[u8; 4]) -> PathBuf {
    let mut prop = Vec::new();
    prop.extend_from_slice(b"SND ");
    prop.extend_from_slice(b"FS  ");
    prop.extend_from_slice(&4u64.to_be_bytes());
    prop.extend_from_slice(&rate.to_be_bytes());
    prop.extend_from_slice(b"CHNL");
    let chnl_len = 2 + 4 * channels as u64;
    prop.extend_from_slice(&chnl_len.to_be_bytes());
    prop.extend_from_slice(&channels.to_be_bytes());
    for _ in 0..channels {
        prop.extend_from_slice(b"SLFT");
    }
    if chnl_len % 2 == 1 {
        prop.push(0);
    }
    prop.extend_from_slice(b"CMPR");
    // Compression subchunk: 4-byte ID + pascal-ish name (we write just the ID
    // + a 1-byte count 0, padded).
    prop.extend_from_slice(&5u64.to_be_bytes());
    prop.extend_from_slice(cmpr);
    prop.push(0);
    prop.push(0); // even padding

    let mut f = Vec::new();
    f.extend_from_slice(b"FRM8");
    f.extend_from_slice(&0u64.to_be_bytes()); // form size (unused by reader)
    f.extend_from_slice(b"DSD ");
    f.extend_from_slice(b"FVER");
    f.extend_from_slice(&4u64.to_be_bytes());
    f.extend_from_slice(&[1, 5, 0, 0]);
    f.extend_from_slice(b"PROP");
    f.extend_from_slice(&(prop.len() as u64).to_be_bytes());
    f.extend_from_slice(&prop);
    f.extend_from_slice(b"DSD ");
    f.extend_from_slice(&(data_bytes_total as u64).to_be_bytes());
    f.extend_from_slice(&vec![0x69u8; data_bytes_total]);
    let path = tmp(name);
    std::fs::File::create(&path).unwrap().write_all(&f).unwrap();
    path
}
