use super::parse::parse_mounts;
use super::*;

#[test]
fn nfs_variants_classify_network() {
    assert!(is_network_fs("nfs"));
    assert!(is_network_fs("nfs4"));
    assert!(is_network_fs("cifs"));
    assert!(is_network_fs("smb3"));
    assert!(is_network_fs("fuse.sshfs"));
    assert!(is_network_fs("fuse.rclone"));
}

#[test]
fn local_fs_does_not_classify() {
    assert!(!is_network_fs("ext4"));
    assert!(!is_network_fs("btrfs"));
    assert!(!is_network_fs("tmpfs"));
    assert!(!is_network_fs("fuse.gocryptfs"));
}

#[test]
fn best_fs_type_respects_path_boundaries() {
    let mounts = vec![
        ("/".to_string(), "ext4".to_string()),
        ("/mnt/music".to_string(), "nfs4".to_string()),
    ];
    assert_eq!(best_fs_type(&mounts, "/mnt/music"), Some("nfs4"));
    assert_eq!(best_fs_type(&mounts, "/mnt/music/Albums/x.flac"), Some("nfs4"));
    // A sibling dir sharing the string prefix must NOT inherit the
    // mount's fs type — it falls through to `/`.
    assert_eq!(best_fs_type(&mounts, "/mnt/music2/x.flac"), Some("ext4"));
}

#[test]
fn best_fs_type_longest_mount_wins() {
    let mounts = vec![
        ("/".to_string(), "ext4".to_string()),
        ("/mnt".to_string(), "xfs".to_string()),
        ("/mnt/nas".to_string(), "cifs".to_string()),
    ];
    assert_eq!(best_fs_type(&mounts, "/mnt/nas/music"), Some("cifs"));
    assert_eq!(best_fs_type(&mounts, "/mnt/local"), Some("xfs"));
    assert_eq!(best_fs_type(&mounts, "/home/user"), Some("ext4"));
}

#[test]
fn parse_mounts_reads_typical_entries() {
    let sample = "\
        /dev/sda1 / ext4 rw,relatime 0 0\n\
        tmpfs /run tmpfs rw,nosuid 0 0\n\
        nas:/music /mnt/music nfs4 rw,relatime 0 0\n\
    ";
    let parsed = parse_mounts(sample);
    assert_eq!(parsed.len(), 3);
    assert_eq!(parsed[2].0, "/mnt/music");
    assert_eq!(parsed[2].1, "nfs4");
}
