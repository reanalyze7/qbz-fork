use super::*;

#[test]
fn slug_index_roundtrip_for_p1() {
    for (i, id) in dropdown_themes().into_iter().enumerate() {
        assert_eq!(index_for_id(id), i as i32);
        assert_eq!(id_for_index(i as i32), id);
        assert_eq!(id_for_slug(id.slug()), id);
    }
}

#[test]
fn unknown_slug_falls_back_to_oled() {
    assert_eq!(id_for_slug("nope"), ThemeId::Oled);
    assert_eq!(id_for_slug(""), ThemeId::Oled);
}

#[test]
fn out_of_range_index_falls_back_to_default() {
    assert_eq!(id_for_index(9999), qbz_theme::default_theme_id());
    assert_eq!(id_for_index(-1), qbz_theme::default_theme_id());
}

#[test]
fn auto_then_custom_are_the_last_two_entries() {
    // Auto is appended first, Custom right after it.
    assert_eq!(auto_index(), dropdown_themes().len() as i32);
    assert_eq!(custom_index(), auto_index() + 1);
    assert!(is_auto_index(auto_index()));
    assert!(is_custom_index(custom_index()));
    assert!(!is_auto_index(custom_index()));
    assert!(!is_custom_index(auto_index()));
    // The labels list is registry rows + Auto + Custom, in that order.
    let labels = dropdown_labels();
    assert_eq!(labels.len(), dropdown_themes().len() + 2);
    assert_eq!(labels[auto_index() as usize], AUTO_LABEL);
    assert_eq!(labels[custom_index() as usize], CUSTOM_LABEL);
}

#[test]
fn synthetic_slugs_map_to_appended_indices() {
    assert_eq!(selected_index_for_slug(AUTO_SLUG), auto_index());
    assert_eq!(selected_index_for_slug(CUSTOM_SLUG), custom_index());
    // A real slug still resolves through the registry.
    assert_eq!(selected_index_for_slug("oled"), index_for_id(ThemeId::Oled));
}

#[test]
fn filter_all_matches_the_unfiltered_list() {
    // The `All` filter is a pure passthrough of the existing behaviour.
    assert_eq!(filtered_dropdown_themes(FILTER_ALL), dropdown_themes());
    assert_eq!(filtered_dropdown_labels(FILTER_ALL), dropdown_labels());
    assert_eq!(filtered_auto_index(FILTER_ALL), auto_index());
    assert_eq!(filtered_custom_index(FILTER_ALL), custom_index());
    assert_eq!(
        filtered_selected_index_for_slug("oled", FILTER_ALL),
        selected_index_for_slug("oled")
    );
}

#[test]
fn dark_and_light_partition_the_themes_by_luminance() {
    let all = filtered_dropdown_themes(FILTER_ALL);
    let dark = filtered_dropdown_themes(FILTER_DARK);
    let light = filtered_dropdown_themes(FILTER_LIGHT);
    // Every theme lands in exactly one subset; together they cover `All`.
    assert_eq!(dark.len() + light.len(), all.len());
    assert!(dark.iter().all(|&id| !qbz_theme::is_light(id)));
    assert!(light.iter().all(|&id| qbz_theme::is_light(id)));
    // OLED (dark) is in Dark, absent from Light.
    assert!(dark.contains(&ThemeId::Oled));
    assert!(!light.contains(&ThemeId::Oled));
}

#[test]
fn narrowed_lists_omit_auto_and_custom() {
    for filter in [FILTER_DARK, FILTER_LIGHT] {
        let labels = filtered_dropdown_labels(filter);
        assert_eq!(labels.len(), filtered_dropdown_themes(filter).len());
        assert!(!labels.iter().any(|l| l == AUTO_LABEL || l == CUSTOM_LABEL));
        assert_eq!(filtered_auto_index(filter), -1);
        assert_eq!(filtered_custom_index(filter), -1);
        // Synthetic slugs have no row here.
        assert_eq!(filtered_selected_index_for_slug(AUTO_SLUG, filter), -1);
        assert_eq!(filtered_selected_index_for_slug(CUSTOM_SLUG, filter), -1);
    }
}

#[test]
fn filtered_index_roundtrips_and_reports_absent_as_minus_one() {
    // A theme filtered out of the active view reports index -1.
    assert_eq!(
        filtered_selected_index_for_slug("oled", FILTER_LIGHT),
        -1
    );
    // Within a subset, slug->index->id round-trips.
    for filter in [FILTER_DARK, FILTER_LIGHT] {
        for (i, id) in filtered_dropdown_themes(filter).into_iter().enumerate() {
            assert_eq!(
                filtered_selected_index_for_slug(id.slug(), filter),
                i as i32
            );
            assert_eq!(filtered_id_for_index(i as i32, filter), id);
        }
    }
}
