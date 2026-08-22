use super::*;

#[test]
fn test_parse_single_performer() {
    let result = parse_performers("John Coltrane, Saxophone");
    assert_eq!(result.len(), 1);
    assert_eq!(result[0].name, "John Coltrane");
    assert_eq!(result[0].roles, vec!["Saxophone"]);
}

#[test]
fn test_parse_multiple_performers() {
    let result = parse_performers(
        "John Coltrane, Saxophone, MainArtist - McCoy Tyner, Piano - Elvin Jones, Drums",
    );
    assert_eq!(result.len(), 3);
    assert_eq!(result[0].name, "John Coltrane");
    assert_eq!(result[0].roles, vec!["Saxophone", "MainArtist"]);
    assert_eq!(result[1].name, "McCoy Tyner");
    assert_eq!(result[1].roles, vec!["Piano"]);
    assert_eq!(result[2].name, "Elvin Jones");
    assert_eq!(result[2].roles, vec!["Drums"]);
}

#[test]
fn test_parse_empty_string() {
    let result = parse_performers("");
    assert!(result.is_empty());
}

#[test]
fn test_parse_performer_no_roles() {
    let result = parse_performers("John Coltrane");
    assert_eq!(result.len(), 1);
    assert_eq!(result[0].name, "John Coltrane");
    assert!(result[0].roles.is_empty());
}

#[test]
fn test_group_by_role() {
    let performers = vec![
        Performer::new("John".to_string(), vec!["Saxophone".to_string()]),
        Performer::new(
            "Jane".to_string(),
            vec!["Saxophone".to_string(), "Vocals".to_string()],
        ),
    ];
    let grouped = group_by_role(&performers);
    assert_eq!(grouped.get("Saxophone").unwrap().len(), 2);
    assert_eq!(grouped.get("Vocals").unwrap().len(), 1);
}
