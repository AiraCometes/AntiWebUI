//! Sound from the times asked for, through a real FFmpeg, when the system has one this knows.
//!
//! `ramp.flac` is 3 s at 48 kHz in two channels, and says which sample it is: sample `n` is
//! `(n % 1000) * 16 / 32768` on the left and `(n / 1000) * 16 / 32768` on the right.
//!
//! ```sh
//! ffmpeg -f lavfi -i "aevalsrc=exprs='mod(n\,1000)*16/32768|floor(n/1000)*16/32768':s=48000:d=3" \
//!   -c:a flac ramp.flac
//! ```

use anti_webui_core::video::{VideoBackend, VideoSettings};
use anti_webui_video::{AudioReader, FfmpegBackend};

const RATE: f64 = 48000.0;

fn open(rate: u32) -> Option<AudioReader> {
    if let Err(e) = FfmpegBackend.availability(&VideoSettings::default()) {
        eprintln!("skipped: {e}");
        return None;
    }
    let source = format!("{}/tests/data/ramp.flac", env!("CARGO_MANIFEST_DIR"));
    Some(AudioReader::open(&source, &VideoSettings::default(), rate).expect("it opens"))
}

/// Which sample of `ramp.flac` the pair at `i` is.
fn number(samples: &[f32], i: usize) -> i64 {
    let unit = 16.0 / 32768.0;
    let low = (samples[i * 2] / unit).round() as i64;
    let high = (samples[i * 2 + 1] / unit).round() as i64;
    high * 1000 + low
}

#[test]
fn it_says_how_long_it_is() {
    let Some(reader) = open(48000) else { return };
    assert!((reader.info().duration.expect("it says") - 3.0).abs() < 0.01);
}

#[test]
fn it_reads_every_sample_in_order() {
    let Some(mut reader) = open(48000) else {
        return;
    };
    let mut n = 0i64;
    while let Some(chunk) = reader.next_chunk().expect("it decodes") {
        assert!(
            (chunk.start - n as f64 / RATE).abs() < 1e-9,
            "at {n}: {}",
            chunk.start
        );
        for i in 0..chunk.samples.len() / 2 {
            assert_eq!(number(&chunk.samples, i), n);
            n += 1;
        }
    }
    assert_eq!(n, 3 * 48000);
}

#[test]
fn what_comes_after_going_somewhere_starts_where_it_says() {
    let Some(mut reader) = open(48000) else {
        return;
    };
    for at in [2.5, 0.7, 1.234, 0.0, 2.99] {
        reader.seek(at).expect("it goes there");
        let chunk = reader
            .next_chunk()
            .expect("it decodes")
            .expect("there is more");
        assert!(chunk.start <= at + 1e-9, "{at}: from {}", chunk.start);
        let first = number(&chunk.samples, 0);
        assert!(
            (first as f64 / RATE - chunk.start).abs() < 1e-9,
            "{at}: {first} at {}",
            chunk.start
        );
        let last = chunk.samples.len() / 2 - 1;
        assert_eq!(number(&chunk.samples, last), first + last as i64);
    }
}

#[test]
fn it_comes_out_at_the_rate_asked_for() {
    let Some(mut reader) = open(44100) else {
        return;
    };
    let mut count = 0;
    let mut end = 0.0;
    while let Some(chunk) = reader.next_chunk().expect("it decodes") {
        count += chunk.samples.len() / 2;
        end = chunk.start + (chunk.samples.len() / 2) as f64 / 44100.0;
    }
    assert!((count as i64 - 3 * 44100).abs() < 64, "{count}");
    assert!((end - 3.0).abs() < 0.002, "{end}");
}
