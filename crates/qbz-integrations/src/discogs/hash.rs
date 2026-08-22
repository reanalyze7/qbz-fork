//! Filename hashing for cached artwork.

use super::DiscogsClient;

impl DiscogsClient {
    /// Simple hash function for generating filenames
    pub(super) fn simple_hash(s: &str) -> u64 {
        let mut hash: u64 = 5381;
        for byte in s.bytes() {
            hash = hash.wrapping_mul(33).wrapping_add(byte as u64);
        }
        hash
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_hash() {
        let hash1 = DiscogsClient::simple_hash("Artist_Album");
        let hash2 = DiscogsClient::simple_hash("Artist_Album");
        assert_eq!(hash1, hash2);

        let hash3 = DiscogsClient::simple_hash("Different_Album");
        assert_ne!(hash1, hash3);
    }
}
