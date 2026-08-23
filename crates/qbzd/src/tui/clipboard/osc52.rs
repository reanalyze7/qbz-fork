/// Standard base64 with padding (no external crate — keeps the slim `qbzd`
/// dependency set, and the payload builder must stay pure/testable).
pub(super) fn base64(input: &[u8]) -> String {
    const T: &[u8; 64] = b"ABCDEFGHIJKLMNOPQRSTUVWXYZabcdefghijklmnopqrstuvwxyz0123456789+/";
    let mut out = String::with_capacity(input.len().div_ceil(3) * 4);
    for chunk in input.chunks(3) {
        let b0 = chunk[0] as u32;
        let b1 = *chunk.get(1).unwrap_or(&0) as u32;
        let b2 = *chunk.get(2).unwrap_or(&0) as u32;
        let n = (b0 << 16) | (b1 << 8) | b2;
        out.push(T[((n >> 18) & 63) as usize] as char);
        out.push(T[((n >> 12) & 63) as usize] as char);
        out.push(if chunk.len() > 1 { T[((n >> 6) & 63) as usize] as char } else { '=' });
        out.push(if chunk.len() > 2 { T[(n & 63) as usize] as char } else { '=' });
    }
    out
}

/// Build the OSC 52 clipboard-set escape for `data`. When `tmux` is true, wrap
/// it in tmux's DCS passthrough (`\ePtmux;…\e\\`) with every inner ESC doubled,
/// so the sequence reaches the outer terminal instead of being swallowed by
/// tmux. Base64 keeps arbitrary bytes chunk-safe on the wire.
pub fn osc52_payload(data: &str, tmux: bool) -> String {
    let b64 = base64(data.as_bytes());
    let seq = format!("\x1b]52;c;{b64}\x07");
    if tmux {
        let inner = seq.replace('\x1b', "\x1b\x1b");
        format!("\x1bPtmux;{inner}\x1b\\")
    } else {
        seq
    }
}

/// OSC 52's payload cap, applied to the base64-encoded blob (post-inflation,
/// which is what actually crosses the wire). Terminals differ wildly in what
/// they accept for a clipboard-set escape — many silently truncate (or just
/// drop) anything past a few tens of KB — so past this ceiling `copy` skips
/// the tier outright rather than risk a truncated, silently-wrong paste.
/// 100 KB is generous but bounded. Kept as a pure fn so the threshold
/// decision is unit-tested without a tty.
const OSC52_MAX_B64_LEN: usize = 100 * 1024;

/// Whether a base64-encoded OSC 52 payload of `b64_len` bytes is small enough
/// to attempt over OSC 52 (`> 100 KB` post-base64 skips the tier).
pub fn osc52_fits(b64_len: usize) -> bool {
    b64_len <= OSC52_MAX_B64_LEN
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn base64_matches_known_vectors() {
        assert_eq!(base64(b""), "");
        assert_eq!(base64(b"f"), "Zg==");
        assert_eq!(base64(b"fo"), "Zm8=");
        assert_eq!(base64(b"foo"), "Zm9v");
        assert_eq!(base64(b"foob"), "Zm9vYg==");
        assert_eq!(base64(b"fooba"), "Zm9vYmE=");
        assert_eq!(base64(b"foobar"), "Zm9vYmFy");
    }

    #[test]
    fn osc52_payload_wraps_base64_in_the_escape() {
        let p = osc52_payload("foobar", false);
        assert_eq!(p, "\x1b]52;c;Zm9vYmFy\x07");
    }

    #[test]
    fn osc52_payload_tmux_passthrough_doubles_esc() {
        let p = osc52_payload("foobar", true);
        // DCS passthrough envelope, ST terminator, inner ESC doubled.
        assert!(p.starts_with("\x1bPtmux;"));
        assert!(p.ends_with("\x1b\\"));
        // The inner OSC introducer's ESC is doubled inside the envelope
        // (`;` from the `Ptmux;` prefix, then the doubled ESC, then the OSC).
        assert!(p.contains(";\x1b\x1b]52;c;Zm9vYmFy\x07"));
        // Exactly the envelope's two structural ESCs remain single: the leading
        // `\x1bP` and the trailing `\x1b\\`; every ESC in between is doubled, so
        // no single ESC is adjacent to a non-ESC in the inner body.
        assert_eq!(p.matches('\x1b').count(), 4); // \x1bP + \x1b\x1b + \x1b\\
        // The exact wrapped payload.
        assert_eq!(p, "\x1bPtmux;\x1b\x1b]52;c;Zm9vYmFy\x07\x1b\\");
    }

    #[test]
    fn osc52_fits_thresholds_at_100kb_post_base64() {
        assert!(osc52_fits(0));
        assert!(osc52_fits(100 * 1024)); // exactly the cap still fits
        assert!(!osc52_fits(100 * 1024 + 1)); // one byte over skips the tier
    }
}
