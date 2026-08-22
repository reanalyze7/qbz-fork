use super::*;

pub(crate) fn decode_with_symphonia(data: &[u8]) -> Result<AudioSpecs, String> {
    let source = Box::new(CursorMediaSource::new(data.to_vec())) as Box<dyn MediaSource>;
    let mss = MediaSourceStream::new(source, Default::default());

    let mut hint = Hint::new();
    hint.with_extension("m4a");

    let format_opts = FormatOptions {
        enable_gapless: true,
        ..Default::default()
    };
    let metadata_opts: MetadataOptions = Default::default();
    let mut probed = get_probe()
        .format(&hint, mss, &format_opts, &metadata_opts)
        .map_err(|err| format!("Symphonia probe failed: {}", err))?;

    let track = probed
        .format
        .default_track()
        .ok_or_else(|| "Symphonia: no supported audio tracks".to_string())?;
    let track_id = track.id;
    let codec_params = track.codec_params.clone();

    let mut decoder = get_codecs()
        .make(&codec_params, &DecoderOptions::default())
        .map_err(|err| format!("Symphonia decoder init failed: {}", err))?;

    let mut sample_rate = 0;
    let mut channels = 0u16;
    let mut samples: Vec<f32> = Vec::new();

    loop {
        let packet = match probed.format.next_packet() {
            Ok(packet) => packet,
            Err(SymphoniaError::IoError(_)) => break,
            Err(err) => return Err(format!("Symphonia read error: {}", err)),
        };

        if packet.track_id() != track_id {
            continue;
        }

        match decoder.decode(&packet) {
            Ok(audio_buf) => {
                let spec = *audio_buf.spec();
                if sample_rate == 0 {
                    sample_rate = spec.rate;
                    channels = spec.channels.count() as u16;
                }

                let mut sample_buf = SampleBuffer::<f32>::new(audio_buf.frames() as u64, spec);
                sample_buf.copy_interleaved_ref(audio_buf);
                samples.extend_from_slice(sample_buf.samples());
            }
            Err(SymphoniaError::DecodeError(_)) => continue,
            Err(SymphoniaError::ResetRequired) => {
                decoder.reset();
                continue;
            }
            Err(err) => return Err(format!("Symphonia decode error: {}", err)),
        }
    }

    if samples.is_empty() || sample_rate == 0 || channels == 0 {
        return Err("Symphonia decode produced no audio".to_string());
    }

    Ok(AudioSpecs {
        samples: SamplesBuffer::new(
            std::num::NonZero::new(channels).unwrap(),
            std::num::NonZero::new(sample_rate).unwrap(),
            samples,
        ),
        sample_rate,
        channels,
    })
}

pub(crate) fn decode_with_fallback(data: &[u8]) -> Result<Box<dyn Source<Item = f32> + Send>, String> {
    if is_isomp4(data) {
        return decode_with_symphonia(data).map(|specs| {
            log::info!("Decoded audio using symphonia fallback (isomp4)");
            Box::new(specs.samples) as Box<dyn Source<Item = f32> + Send>
        });
    }

    let primary = panic::catch_unwind(AssertUnwindSafe(|| {
        Decoder::new(BufReader::new(Cursor::new(data.to_vec())))
    }));

    match primary {
        Ok(Ok(decoder)) => return Ok(Box::new(decoder)),
        Ok(Err(err)) => {
            log::warn!("Primary decode failed, attempting mp4 fallback: {}", err);
        }
        Err(_) => {
            log::warn!("Primary decode panicked, attempting mp4 fallback");
        }
    }

    // Try mp4 fallback (rodio 0.22 removed Mp4Type hint)
    {
        let attempt = panic::catch_unwind(AssertUnwindSafe(|| {
            Decoder::new_mp4(BufReader::new(Cursor::new(data.to_vec())))
        }));

        match attempt {
            Ok(Ok(decoder)) => {
                log::info!("Decoded audio using mp4 fallback");
                return Ok(Box::new(decoder));
            }
            Ok(Err(err)) => {
                log::warn!("mp4 fallback failed: {}", err);
            }
            Err(_) => {
                log::warn!("mp4 fallback panicked");
            }
        }
    }

    match decode_with_symphonia(data) {
        Ok(specs) => {
            log::info!("Decoded audio using symphonia fallback");
            Ok(Box::new(specs.samples))
        }
        Err(err) => Err(err),
    }
}
