use super::super::*;

/// Wrap a decoded source with diagnostic capture, normalization, and the
/// visualizer tap.
///
/// Pipeline order (normalization ON):
///   Diagnostic (raw) -> AnalyzerTap -> DynamicAmplify -> Visualizer
/// Pipeline order (normalization OFF - bit-perfect):
///   Diagnostic (raw) -> Visualizer
///
/// A free function (not a `ThreadCtx` method) on purpose: several call
/// sites hold a `&mut` borrow of `ctx.current_engine` at the same time (via
/// `.as_mut()`), and a `&self` method here would borrow the whole struct
/// and conflict with it. Taking the specific fields it needs keeps the
/// borrows disjoint.
pub(crate) fn wrap_source(
    diagnostic: &AudioDiagnostic,
    viz_tap: &Option<VisualizerTap>,
    analyzer_tx: &SyncSender<AnalyzerMessage>,
    analyzer_enabled: &Arc<AtomicBool>,
    source: Box<dyn Source<Item = f32> + Send>,
    normalization_gain: Option<f32>,
    gain_atomic: Option<Arc<AtomicU32>>,
) -> Box<dyn Source<Item = f32> + Send> {
    // Diagnostic tap (innermost - captures raw decoded samples)
    let source: Box<dyn Source<Item = f32> + Send> =
        Box::new(DiagnosticSource::new(source, diagnostic.clone()));

    // Normalization: dynamic (Phase 2) > static (Phase 1 fallback) > none (bit-perfect)
    let source: Box<dyn Source<Item = f32> + Send> = if let Some(gain_atomic) = gain_atomic {
        let initial_gain = normalization_gain.unwrap_or(1.0);
        log::info!(
            "Audio thread: dynamic normalization enabled (initial gain {:.4})",
            initial_gain
        );
        analyzer_enabled.store(true, Ordering::SeqCst);
        let source: Box<dyn Source<Item = f32> + Send> = Box::new(AnalyzerTap::new(
            source,
            analyzer_tx.clone(),
            analyzer_enabled.clone(),
        ));
        Box::new(DynamicAmplify::new(source, gain_atomic, initial_gain))
    } else if let Some(gain) = normalization_gain {
        log::info!(
            "Audio thread: applying static normalization gain factor {:.4}",
            gain
        );
        Box::new(source.amplify(gain))
    } else {
        source
    };

    // Visualizer tap (outermost)
    if let Some(ref tap) = viz_tap {
        Box::new(TappedSource::new(
            source,
            tap.ring_buffer.clone(),
            tap.enabled.clone(),
        ))
    } else {
        source
    }
}
