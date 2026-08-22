use super::cosmic::*;
use super::parse::*;
use super::*;

#[test]
fn detect_de_does_not_panic() {
    let _de = detect_desktop_environment();
}

#[test]
fn parse_gsettings_uri_variants() {
    assert_eq!(
        parse_gsettings_uri("'file:///home/user/wallpaper.jpg'"),
        Some("/home/user/wallpaper.jpg".to_string())
    );
    assert_eq!(
        parse_gsettings_uri("'file:///home/user/my%20wallpaper.png'"),
        Some("/home/user/my wallpaper.png".to_string())
    );
}

#[test]
fn parse_file_uri_only_file_scheme() {
    assert_eq!(
        parse_file_uri("file:///home/user/pic.jpg"),
        Some("/home/user/pic.jpg".to_string())
    );
    assert_eq!(parse_file_uri("/just/a/path"), None);
}

#[test]
fn parse_rgb_csv_ok() {
    assert_eq!(parse_rgb_csv("66,133,244").unwrap(), PaletteColor::new(66, 133, 244));
    assert_eq!(
        parse_rgb_csv(" 66 , 133 , 244 ").unwrap(),
        PaletteColor::new(66, 133, 244)
    );
}

#[test]
fn parse_cosmic_color_float() {
    let color = parse_cosmic_color("(0.26, 0.52, 0.96, 1.0)").unwrap();
    assert_eq!(color.r, 66);
    assert_eq!(color.g, 133);
    assert_eq!(color.b, 245);
}

#[test]
fn is_image_path_matches() {
    assert!(is_image_path("/home/user/wall.jpg"));
    assert!(is_image_path("/home/user/wall.PNG"));
    assert!(is_image_path("/home/user/wall.webp"));
    assert!(!is_image_path("/home/user/wall.mp4"));
}
