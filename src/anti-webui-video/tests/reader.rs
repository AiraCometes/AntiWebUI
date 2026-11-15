//! Frames at the times asked for, through a real FFmpeg, when the system has one this knows.
//!
//! `counting.mp4` is 4 s at 30 fps, 64×48, a key frame each second and B-frames, and the luma of
//! frame `n` is `16 + 2n`:
//!
//! ```sh
//! ffmpeg -f lavfi -i "color=black:s=64x48:r=30:d=4,format=yuv420p,geq=lum='16+N*2':cb=128:cr=128" \
//!   -c:v libx264 -qp 4 -g 30 -bf 2 -b_strategy 0 -pix_fmt yuv420p counting.mp4
//! ```

use anti_webui_core::video::{VideoBackend, VideoPixels, VideoSettings};
use anti_webui_video::{DecodedFrame, FfmpegBackend, VideoReader};

fn open() -> Option<VideoReader> {
    if let Err(e) = FfmpegBackend.availability(&VideoSettings::default()) {
        eprintln!("skipped: {e}");
        return None;
    }
    let source = format!("{}/tests/data/counting.mp4", env!("CARGO_MANIFEST_DIR"));
    Some(VideoReader::open(&source, &VideoSettings::default(), None).expect("it opens"))
}

/// Which frame of `counting.mp4` this is, by its luma.
// Without dmabufs, pictures are only ever in memory.
#[cfg_attr(not(unix), allow(irrefutable_let_patterns))]
fn number(frame: &DecodedFrame) -> i32 {
    let VideoPixels::Memory(planes) = &frame.frame.pixels else {
        panic!("in memory")
    };
    let (w, h) = planes[0].size;
    let y = planes[0].bytes()[(h / 2 * planes[0].stride + w / 2) as usize];
    ((y as f32 - 16.0) / 2.0).round() as i32
}

#[test]
fn it_says_what_the_video_is() {
    let Some(reader) = open() else { return };
    let info = reader.info();
    assert_eq!(info.size, (64, 48));
    assert_eq!(info.frame_rate, Some(30.0));
    assert!((info.duration.expect("it says") - 4.0).abs() < 0.05);
    assert!(!info.has_audio);
}

#[test]
fn frames_come_in_order() {
    let Some(mut reader) = open() else { return };
    let mut n = 0;
    while let Some(frame) = reader.next_frame().expect("it decodes") {
        assert_eq!(number(&frame), n);
        assert!(
            (frame.start - n as f64 / 30.0).abs() < 1e-3,
            "{}",
            frame.start
        );
        n += 1;
    }
    assert_eq!(n, 120);
}

#[test]
fn any_frame_is_the_one_at_its_time_whatever_came_before() {
    let Some(mut reader) = open() else { return };
    // Forward, back over key frames, within a group of pictures, the same again, and the ends.
    let wanted = [
        0, 1, 2, 29, 30, 31, 95, 94, 3, 3, 60, 59, 119, 61, 0, 45, 46, 50, 10,
    ];
    for n in wanted {
        // Halfway into the frame, as an editor asking for a time between frames would.
        let at = (n as f64 + 0.5) / 30.0;
        let frame = reader
            .frame_at(at)
            .expect("it decodes")
            .expect("there is one");
        assert_eq!(number(&frame), n, "at {at}");
    }
    // Past the end is the last one; before the start, the first.
    let last = reader
        .frame_at(10.0)
        .expect("it decodes")
        .expect("there is one");
    assert_eq!(number(&last), 119);
    let first = reader
        .frame_at(-1.0)
        .expect("it decodes")
        .expect("there is one");
    assert_eq!(number(&first), 0);
}
