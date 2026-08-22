use jack::{AudioOut, Client, ClientOptions, PortFlags};
use ringbuf::traits::{Observer, Producer, Split};
use ringbuf::{HeapProd, HeapRb};
use std::sync::atomic::{AtomicU64, Ordering};
use std::sync::{Arc, Mutex};

use super::process::JackProcess;
use super::{MAX_NFRAMES, RING_CAPACITY_FRAMES};

/// An active JACK client plus the producer side of the audio ring buffer.
///
/// Mirrors `AlsaDirectStream`: the player feeds interleaved stereo f32 via
/// [`write_f32`](Self::write_f32). Dropping this deactivates + closes the JACK
/// client (ports disappear).
pub struct JackStream {
    /// Activated async client; its `Drop` deactivates + unregisters the ports.
    _async_client: jack::AsyncClient<(), JackProcess>,
    producer: Mutex<HeapProd<f32>>,
    sample_rate: u32,
    channels: u16,
    underruns: Arc<AtomicU64>,
}

impl JackStream {
    /// Open the JACK client, register stable stereo ports, activate, and
    /// auto-connect to the system's physical playback.
    pub fn new(channels: u16) -> Result<Self, String> {
        let (client, _status) = Client::new("qbz", ClientOptions::NO_START_SERVER)
            .map_err(|e| format!("JACK client open failed (is a JACK/pipewire-jack server running?): {e}"))?;

        let sample_rate = client.sample_rate() as u32;

        let out_l = client
            .register_port("out_FL", AudioOut::default())
            .map_err(|e| format!("JACK register out_FL failed: {e}"))?;
        let out_r = client
            .register_port("out_FR", AudioOut::default())
            .map_err(|e| format!("JACK register out_FR failed: {e}"))?;

        let rb = HeapRb::<f32>::new(RING_CAPACITY_FRAMES * 2);
        let (producer, consumer) = rb.split();

        let underruns = Arc::new(AtomicU64::new(0));
        let process = JackProcess {
            consumer,
            out_l,
            out_r,
            scratch: vec![0.0f32; MAX_NFRAMES * 2],
            underruns: underruns.clone(),
        };

        let async_client = client
            .activate_async((), process)
            .map_err(|e| format!("JACK activate failed: {e}"))?;

        // Auto-connect qbz:out_FL/FR to the first two physical playback ports so
        // audio reaches the hardware without a manual patchbay. A user may
        // disconnect + re-patch freely (the ports are stable).
        {
            let c = async_client.as_client();
            let playback = c.ports(None, None, PortFlags::IS_INPUT | PortFlags::IS_PHYSICAL);
            if playback.len() >= 2 {
                if let Err(e) = c.connect_ports_by_name("qbz:out_FL", &playback[0]) {
                    log::warn!("[JACK] auto-connect out_FL -> {} failed: {e}", playback[0]);
                }
                if let Err(e) = c.connect_ports_by_name("qbz:out_FR", &playback[1]) {
                    log::warn!("[JACK] auto-connect out_FR -> {} failed: {e}", playback[1]);
                }
                log::info!(
                    "[JACK] auto-connected qbz:out_FL/FR -> {} / {}",
                    playback[0], playback[1]
                );
            } else {
                log::warn!(
                    "[JACK] no physical playback ports to auto-connect ({} found); patch qbz:out_FL/FR manually",
                    playback.len()
                );
            }
        }

        log::info!(
            "[JACK] client 'qbz' active at {} Hz — ports qbz:out_FL / qbz:out_FR (NOT bit-perfect: resampled to the graph rate)",
            sample_rate
        );

        Ok(Self {
            _async_client: async_client,
            producer: Mutex::new(producer),
            sample_rate,
            channels,
            underruns,
        })
    }

    /// The JACK graph sample rate. The player resamples each track to this rate
    /// before feeding. (Runtime graph-rate changes are a slice-3.2 item; today
    /// this is read once at open.)
    pub fn sample_rate(&self) -> u32 {
        self.sample_rate
    }

    pub fn channels(&self) -> u16 {
        self.channels
    }

    /// Push interleaved stereo f32 (already at the graph rate) into the ring.
    /// Called from the player's feeder thread. Returns the number of *frames*
    /// accepted (whole stereo frames only); fewer than requested means the ring
    /// is full and the feeder should retry (it paces against real time).
    pub fn write_f32(&self, samples: &[f32]) -> usize {
        let mut p = self.producer.lock().unwrap();
        // Only push whole stereo frames so the L/R interleave never shifts.
        let n = samples.len().min(p.vacant_len()) & !1;
        if n == 0 {
            return 0;
        }
        let pushed = p.push_slice(&samples[..n]);
        pushed / 2
    }

    /// Total underrun events since open (diagnostic).
    #[allow(dead_code)]
    pub fn underruns(&self) -> u64 {
        self.underruns.load(Ordering::Relaxed)
    }
}
