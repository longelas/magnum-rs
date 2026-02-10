#![cfg(feature = "with_caf")]

use magnum::container::caf::OpusSourceCaf;
use magnum::error::OpusSourceError;
use std::fs::File;
use std::io::BufReader;
use std::path::PathBuf;

fn example_caf_path() -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("src/tests/Example.caf")
}

/// Opens Example.caf as Opus-in-CAF. Skips (returns None) if the file is missing or not
/// Opus CAF. Place an Opus-encoded CAF file at src/Example.caf for playback tests.
fn open_example_opus_caf() -> Option<OpusSourceCaf<BufReader<File>>> {
    let path = example_caf_path();
    let file = File::open(&path).ok()?;
    let reader = BufReader::new(file);
    match OpusSourceCaf::new(reader) {
        Ok(source) => Some(source),
        Err(OpusSourceError::InvalidContainerFormat) => {
            eprintln!("Skip: src/Example.caf not found or not a valid CAF file.");
            None
        }
        Err(OpusSourceError::InvalidAudioStream) => {
            eprintln!("Skip: src/Example.caf is not Opus-in-CAF. Use an Opus CAF file for tests.");
            None
        }
        Err(e) => panic!("Unexpected error opening Example.caf: {}", e),
    }
}

#[test]
fn open_example_caf() {
    let source = match open_example_opus_caf() {
        Some(s) => s,
        None => return,
    };
    assert_eq!(source.metadata.sample_rate, 48_000);
    assert!(source.metadata.channel_count >= 1 && source.metadata.channel_count <= 2);
}

#[test]
fn example_caf_playback_produces_samples() {
    let mut source = match open_example_opus_caf() {
        Some(s) => s,
        None => return,
    };

    let samples: Vec<f32> = source.by_ref().take(960).collect();
    assert_eq!(
        samples.len(),
        960,
        "Should yield 960 samples (20 ms at 48 kHz mono)"
    );
    assert!(
        samples.iter().any(|&s| s != 0.0),
        "At least some samples should be non-zero"
    );
    assert!(
        samples.iter().all(|&s| s >= -1.0 && s <= 1.0),
        "Decoded samples should be in [-1, 1]"
    );
}

#[test]
fn example_caf_playback_consumes_full_stream() {
    let source = match open_example_opus_caf() {
        Some(s) => s,
        None => return,
    };

    let total: usize = source.count();
    assert!(total > 0, "Stream should have at least one sample");
}

#[test]
fn playback_iterator_exhausts_then_returns_none() {
    let mut source = match open_example_opus_caf() {
        Some(s) => s,
        None => return,
    };
    let total: usize = source.by_ref().count();
    assert!(total > 0, "Stream should yield samples");
    assert!(
        Iterator::next(&mut source).is_none(),
        "After exhaustion, next() should be None"
    );
    assert!(
        Iterator::next(&mut source).is_none(),
        "Subsequent next() should stay None"
    );
}

#[test]
fn playback_multiple_chunks_large_read() {
    let mut source = match open_example_opus_caf() {
        Some(s) => s,
        None => return,
    };
    let take_n = 15_000;
    let samples: Vec<f32> = source.by_ref().take(take_n).collect();
    assert!(
        samples.len() >= 960,
        "Should yield at least one chunk (960 samples)"
    );
    assert!(
        samples.iter().all(|&s| s >= -1.0 && s <= 1.0),
        "All samples should be in [-1, 1]"
    );
    assert!(
        samples.iter().any(|&s| s != 0.0),
        "At least one sample should be non-zero"
    );
}

#[test]
fn playback_stereo_interleaved_if_two_channels() {
    let mut source = match open_example_opus_caf() {
        Some(s) => s,
        None => return,
    };
    if source.metadata.channel_count != 2 {
        return;
    }
    let samples: Vec<f32> = source.by_ref().take(200).collect();
    assert_eq!(samples.len(), 200);
    let left: Vec<f32> = samples.iter().step_by(2).copied().collect();
    let right: Vec<f32> = samples.iter().skip(1).step_by(2).copied().collect();
    assert_eq!(left.len(), 100);
    assert_eq!(right.len(), 100);
    let same = left
        .iter()
        .zip(right.iter())
        .filter(|(l, r)| l == r)
        .count();
    assert!(
        same < 100,
        "Stereo stream should have at least some L/R difference"
    );
}

#[cfg(feature = "with_rodio")]
#[test]
fn rodio_source_trait_caf() {
    use rodio::source::Source;
    let source = match open_example_opus_caf() {
        Some(s) => s,
        None => return,
    };
    assert_eq!(source.channels(), source.metadata.channel_count as u16);
    assert_eq!(source.sample_rate(), 48_000);
    assert!(
        source.current_frame_len().is_some(),
        "CAF should report frame length"
    );
    assert!(source.current_frame_len().unwrap() >= 1);
    assert!(source.total_duration().is_none());
}

#[cfg(feature = "with_rodio")]
#[test]
fn rodio_source_playback_caf() {
    let mut source = match open_example_opus_caf() {
        Some(s) => s,
        None => return,
    };
    let samples: Vec<f32> = source.by_ref().take(480).collect();
    assert_eq!(samples.len(), 480);
    assert!(samples.iter().all(|&s| s >= -1.0 && s <= 1.0));
}
