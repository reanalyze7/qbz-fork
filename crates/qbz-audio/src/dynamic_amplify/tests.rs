use super::*;
use rodio::buffer::SamplesBuffer;

fn source() -> SamplesBuffer {
    SamplesBuffer::new(
        std::num::NonZero::new(2u16).unwrap(),
        std::num::NonZero::new(48000u32).unwrap(),
        vec![1.0f32; 16],
    )
}

#[test]
fn gain_connu_applique_des_le_premier_echantillon() {
    let atomic = Arc::new(AtomicU32::new(0.5f32.to_bits()));
    let mut amp = DynamicAmplify::new(source(), atomic, 1.0);
    assert_eq!(amp.next(), Some(0.5));
}

#[test]
fn gain_inconnu_garde_le_gain_initial() {
    let atomic = Arc::new(AtomicU32::new(0.0f32.to_bits()));
    let mut amp = DynamicAmplify::new(source(), atomic, 1.0);
    assert_eq!(amp.next(), Some(1.0));
}

#[test]
fn changement_en_cours_de_route_passe_par_la_rampe() {
    let atomic = Arc::new(AtomicU32::new(1.0f32.to_bits()));
    let mut amp = DynamicAmplify::new(source(), atomic.clone(), 1.0);
    assert_eq!(amp.next(), Some(1.0));
    atomic.store(0.5f32.to_bits(), Ordering::Relaxed);
    let next = amp.next().unwrap();
    assert!(next < 1.0 && next > 0.5, "rampe progressive, pas un saut");
}
