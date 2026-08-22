use super::PluralRule;

#[test]
fn parses_nplurals_and_en_rule_index() {
    let rule = PluralRule::parse("nplurals=2; plural=(n != 1);");
    assert_eq!(rule.nplurals(), 2);
    // English / Spanish / German / Portuguese: 1 is singular.
    assert_eq!(rule.index(0), 1);
    assert_eq!(rule.index(1), 0);
    assert_eq!(rule.index(2), 1);
    assert_eq!(rule.index(5), 1);
}

#[test]
fn parses_fr_greater_than_one_rule_index() {
    let rule = PluralRule::parse("nplurals=2; plural=(n > 1);");
    assert_eq!(rule.nplurals(), 2);
    // French: 0 and 1 are singular form.
    assert_eq!(rule.index(0), 0);
    assert_eq!(rule.index(1), 0);
    assert_eq!(rule.index(2), 1);
    assert_eq!(rule.index(10), 1);
}

#[test]
fn parses_japanese_single_rule() {
    let rule = PluralRule::parse("nplurals=1; plural=0;");
    assert_eq!(rule.nplurals(), 1);
    // One form for every count.
    assert_eq!(rule.index(0), 0);
    assert_eq!(rule.index(1), 0);
    assert_eq!(rule.index(2), 0);
    assert_eq!(rule.index(100), 0);
}

#[test]
fn parses_russian_three_form_rule() {
    let rule = PluralRule::parse(
        "nplurals=3; plural=(n%10==1 && n%100!=11 ? 0 : n%10>=2 && n%10<=4 && (n%100<12 || n%100>14) ? 1 : 2);",
    );
    assert_eq!(rule.nplurals(), 3);
    // one: 1, 21, 31 (but not 11)
    assert_eq!(rule.index(1), 0);
    assert_eq!(rule.index(21), 0);
    assert_eq!(rule.index(11), 2);
    // few: 2-4, 22-24 (but not 12-14)
    assert_eq!(rule.index(2), 1);
    assert_eq!(rule.index(4), 1);
    assert_eq!(rule.index(23), 1);
    assert_eq!(rule.index(12), 2);
    // many: 0, 5-20, 25-30
    assert_eq!(rule.index(0), 2);
    assert_eq!(rule.index(5), 2);
    assert_eq!(rule.index(100), 2);
}

#[test]
fn unknown_expression_falls_back_to_default() {
    let rule = PluralRule::parse("garbage header with no plural expr");
    assert_eq!(rule.nplurals(), 2);
    assert_eq!(rule.index(1), 0);
    assert_eq!(rule.index(3), 1);
}
