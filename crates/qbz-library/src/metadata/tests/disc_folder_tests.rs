use crate::MetadataExtractor;

#[test]
fn test_is_disc_folder_true() {
    // Pure disc folders
    assert!(MetadataExtractor::is_disc_folder("Disc 1"));
    assert!(MetadataExtractor::is_disc_folder("disc 2"));
    assert!(MetadataExtractor::is_disc_folder("Disc1"));
    assert!(MetadataExtractor::is_disc_folder("disc01"));
    assert!(MetadataExtractor::is_disc_folder("CD 1"));
    assert!(MetadataExtractor::is_disc_folder("CD1"));
    assert!(MetadataExtractor::is_disc_folder("cd2"));
    assert!(MetadataExtractor::is_disc_folder("Disk 3"));
    assert!(MetadataExtractor::is_disc_folder("Bonus Disc"));
    assert!(MetadataExtractor::is_disc_folder("Bonus Disc 1"));
    assert!(MetadataExtractor::is_disc_folder("Extra CD 2"));
    assert!(MetadataExtractor::is_disc_folder("Side Disc 1"));
}

#[test]
fn test_is_disc_folder_false_album_names() {
    // Album names containing disc/cd keywords — NOT disc folders (issue #147)
    assert!(!MetadataExtractor::is_disc_folder(
        "100 Popular Classics, Disc 1"
    ));
    assert!(!MetadataExtractor::is_disc_folder(
        "100 Popular Classics_ Best Loved Works of the Great Composers, Disc 1"
    ));
    assert!(!MetadataExtractor::is_disc_folder("Relaxation Disc1"));
    assert!(!MetadataExtractor::is_disc_folder("Now 75 - CD1"));
    assert!(!MetadataExtractor::is_disc_folder(
        "Match of the Day - The Album CD1"
    ));
    assert!(!MetadataExtractor::is_disc_folder("20 Blues Greats"));
    assert!(!MetadataExtractor::is_disc_folder("The Beatles"));
    assert!(!MetadataExtractor::is_disc_folder("Abbey Road"));
}
