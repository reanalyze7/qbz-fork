//! Locates the track-list array inside Apple Music's `serialized-server-data`
//! JSON blob, which nests it at an unpredictable depth.

use serde_json::Value;

pub(super) fn find_track_items(data: &Value) -> Option<Vec<&Value>> {
    match data {
        Value::Object(map) => {
            if map.get("itemKind").and_then(|v| v.as_str()) == Some("trackLockup") {
                let items = map.get("items").and_then(|v| v.as_array())?;
                if !items.is_empty() {
                    return Some(items.iter().collect());
                }
            }

            for value in map.values() {
                if let Some(found) = find_track_items(value) {
                    return Some(found);
                }
            }
        }
        Value::Array(list) => {
            for value in list {
                if let Some(found) = find_track_items(value) {
                    return Some(found);
                }
            }
        }
        _ => {}
    }

    None
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn find_track_items_locates_track_lockup_anywhere() {
        let data: Value = serde_json::from_str(
            r#"[{"sections":[{"itemKind":"trackLockup","items":[
                {"title":"Hey Jude","artistName":"The Beatles","duration":431333},
                {"title":"Let It Be","artistName":"The Beatles","duration":243026}
            ]}]}]"#,
        )
        .unwrap();
        let items = find_track_items(&data).expect("found");
        assert_eq!(items.len(), 2);
        assert_eq!(items[0]["title"].as_str(), Some("Hey Jude"));
        assert_eq!(items[1]["artistName"].as_str(), Some("The Beatles"));
    }

    #[test]
    fn find_track_items_ignores_empty_lockups() {
        let data: Value = serde_json::from_str(r#"{"itemKind":"trackLockup","items":[]}"#).unwrap();
        assert!(find_track_items(&data).is_none());
    }
}
