use super::LoudnessCache;

#[test]
fn aller_retour() {
    let c = LoudnessCache::in_memory().unwrap();
    c.set(42, -7.2, 0.98, "ebur128-offline");
    let got = c.get(42).unwrap();
    assert!((got.measured_lufs + 7.2).abs() < 0.001);
    assert_eq!(got.source, "ebur128-offline");
    assert!(c.has(42));
}

#[test]
fn gain_recalcule_pour_la_cible_courante() {
    let c = LoudnessCache::in_memory().unwrap();
    c.set(1, -6.4, 0.0, "ebur128");
    let m = c.get(1).unwrap();
    // Changer la cible ne rend plus l'entree fausse : elle est reinterpretee.
    assert!((m.gain_db(-14.0) + 7.6).abs() < 0.01);
    assert!((m.gain_db(-18.0) + 11.6).abs() < 0.01);
}

#[test]
fn mesure_de_silence_refusee() {
    let c = LoudnessCache::in_memory().unwrap();
    c.set(7, -60.0, 0.0, "ebur128");
    assert!(c.get(7).is_none());
    assert!(!c.has(7));
}

#[test]
fn piste_inconnue() {
    let c = LoudnessCache::in_memory().unwrap();
    assert!(c.get(999).is_none());
}
