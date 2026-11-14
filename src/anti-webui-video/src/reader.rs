//! Frames of a video one by one, or the one at a time, for programs that decide what shows when:
//! an editor that goes to a frame and back, or what writes a video out. Nothing here keeps time or
//! plays sound. See "フレーム単位の読み出し" in `docs/design/24-video.md`.

use crate::ffi::*;
use crate::player::{
    Decoder, Input, Packet, Scaler, StreamInfo, VAAPI_PIX_FMT, alloc_frame, color_of,
    hardware_decoder, memory_frame, next_id, open_decoder, open_input, packet_stream, thread_count,
};
use anti_webui_core::video::{VideoError, VideoFrame, VideoSettings};
use std::ffi::c_int;
use std::path::{Path, PathBuf};
use std::sync::Arc;
use std::sync::atomic::{AtomicBool, Ordering};

/// How far ahead [`VideoReader::frame_at`] reads on instead of going to a key frame.
const READ_ON: f64 = 2.0;
/// How many pictures in a row may fail before it gives up.
const MAX_FAILURES: u32 = 32;
/// Times this close are the same.
const EPSILON: f64 = 1e-6;

/// What a video is.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct VideoInfo {
    /// In pixels, after turning it as it says.
    pub size: (u32, u32),
    /// In seconds, if the file says.
    pub duration: Option<f64>,
    /// Frames a second on average, if the file says.
    pub frame_rate: Option<f64>,
    pub has_audio: bool,
}

/// A picture and when it shows, in seconds from the start of the video.
#[derive(Clone, Debug)]
pub struct DecodedFrame {
    pub start: f64,
    /// How long it shows, as far as is known.
    pub duration: Option<f64>,
    pub frame: Arc<VideoFrame>,
}

/// Reads the pictures of a video in order, from wherever it is told to go. What it gives is in
/// memory and can be kept for as long as is wanted: it doesn't hold up the decoder.
pub struct VideoReader {
    // The decoder goes before what it reads from.
    decoder: Decoder,
    input: Input,
    ffmpeg: Arc<Ffmpeg>,
    video: StreamInfo,
    scaler: Option<Scaler>,
    render_node: Option<PathBuf>,
    info: VideoInfo,
    /// When the stream starts, in its own time.
    origin: f64,
    stream: u64,
    alive: Arc<()>,
    stop: Arc<AtomicBool>,
    /// The decoder has been told that there is nothing more.
    drained: bool,
    ended: bool,
    /// Where it last went.
    sought: f64,
    last: Option<(f64, f64)>,
    decoded_any: bool,
    failures: u32,
    /// The last one [`frame_at`](Self::frame_at) gave, and the one after it, if it has read that.
    shown: Option<DecodedFrame>,
    peeked: Option<DecodedFrame>,
}

// SAFETY: what FFmpeg has of it is used by one thread at a time, and FFmpeg doesn't mind which.
unsafe impl Send for VideoReader {}

impl VideoReader {
    /// Opens the video in `source`, a path or what FFmpeg can read. It decodes on the GPU of
    /// `render_node` when that can, and otherwise on the CPU.
    pub fn open(
        source: &str,
        settings: &VideoSettings,
        render_node: Option<&Path>,
    ) -> Result<Self, VideoError> {
        let ffmpeg = ffmpeg(settings)?;
        VAAPI_PIX_FMT.store(ffmpeg.vaapi_pix_fmt, Ordering::Relaxed);
        let stop = Arc::new(AtomicBool::new(false));
        let input = open_input(&ffmpeg, &stop, source)?;
        let Some(video) = input.video else {
            return Err(VideoError::UnsupportedCodec(format!(
                "{source}: no pictures in it that can be decoded"
            )));
        };
        let l = ffmpeg.layout;
        // SAFETY: the input is open, and the layout says where the fields are.
        let (size, duration, frame_rate, origin) = unsafe {
            let (w, h): (c_int, c_int) =
                (read(video.par, l.par_width), read(video.par, l.par_height));
            let size = if video.rotation % 180 == 0 {
                (w as u32, h as u32)
            } else {
                (h as u32, w as u32)
            };
            let duration: i64 = read(input.raw, l.format_duration);
            let duration = (duration != AV_NOPTS_VALUE && duration > 0)
                .then(|| duration as f64 / AV_TIME_BASE);
            let rate: Rational = read(video.stream, l.stream_avg_frame_rate);
            let rate = rate.as_f64();
            let start: i64 = read(video.stream, l.stream_start_time);
            let origin = if start != AV_NOPTS_VALUE {
                start as f64 * video.time_base
            } else {
                let start: i64 = read(input.raw, l.format_start_time);
                if start == AV_NOPTS_VALUE {
                    0.0
                } else {
                    start as f64 / AV_TIME_BASE
                }
            };
            (
                size,
                duration,
                (rate.is_finite() && rate > 0.0).then_some(rate),
                origin,
            )
        };
        let render_node = render_node
            .filter(|_| !settings.software_only)
            .map(Path::to_path_buf);
        let decoder = open_video_decoder(&ffmpeg, &video, render_node.as_deref())?;
        Ok(Self {
            decoder,
            info: VideoInfo {
                size,
                duration,
                frame_rate,
                has_audio: input.audio.is_some(),
            },
            input,
            ffmpeg,
            video,
            scaler: None,
            render_node,
            origin,
            stream: next_id(),
            alive: Arc::new(()),
            stop,
            drained: false,
            ended: false,
            sought: 0.0,
            last: None,
            decoded_any: false,
            failures: 0,
            shown: None,
            peeked: None,
        })
    }

    pub fn info(&self) -> VideoInfo {
        self.info
    }

    /// Whether it decodes on the GPU.
    pub fn is_hardware(&self) -> bool {
        self.decoder.hardware
    }

    /// Goes to the last key frame at or before `seconds`: [`next_frame`](Self::next_frame) goes on from there.
    /// Some files only know roughly where their key frames are, so that can be after `seconds`.
    pub fn seek(&mut self, seconds: f64) -> Result<(), VideoError> {
        let target = ((seconds + self.origin) / self.video.time_base).floor() as i64;
        // SAFETY: the input and the decoder are open.
        let code = unsafe {
            let code = (self.ffmpeg.f.av_seek_frame)(
                self.input.raw,
                self.video.index,
                target,
                AVSEEK_FLAG_BACKWARD,
            );
            (self.ffmpeg.f.avcodec_flush_buffers)(self.decoder.raw);
            code
        };
        self.sought = seconds;
        self.drained = false;
        self.ended = false;
        self.last = None;
        self.shown = None;
        self.peeked = None;
        if code < 0 {
            return Err(VideoError::Source(format!(
                "can't go to {seconds} s: {}",
                self.ffmpeg.error(code)
            )));
        }
        Ok(())
    }

    /// The next picture in the order they show, or `None` after the last.
    pub fn next_frame(&mut self) -> Result<Option<DecodedFrame>, VideoError> {
        if let Some(peeked) = self.peeked.take() {
            return Ok(Some(peeked));
        }
        loop {
            if self.ended {
                return Ok(None);
            }
            let frame = alloc_frame(&self.ffmpeg);
            // SAFETY: an open decoder and an empty frame.
            let code = unsafe { (self.ffmpeg.f.avcodec_receive_frame)(self.decoder.raw, frame) };
            if code >= 0 {
                self.decoded_any = true;
                self.failures = 0;
                if let Some(decoded) = self.deliver(frame) {
                    return Ok(Some(decoded));
                }
                continue;
            }
            let mut frame = frame;
            // SAFETY: from `av_frame_alloc`.
            unsafe { (self.ffmpeg.f.av_frame_free)(&mut frame) };
            if code == AVERROR_EOF || (code == AVERROR_EAGAIN && self.drained) {
                self.ended = true;
                return Ok(None);
            }
            if code != AVERROR_EAGAIN {
                self.failed(code)?;
                continue;
            }
            let packet = self.read_packet()?;
            let raw = packet
                .as_ref()
                .map_or(std::ptr::null(), |p| p.raw.cast_const());
            self.drained = packet.is_none();
            // SAFETY: an open decoder, and a packet or nothing at the end.
            let code = unsafe { (self.ffmpeg.f.avcodec_send_packet)(self.decoder.raw, raw) };
            if code < 0 && code != AVERROR_EOF && code != AVERROR_EAGAIN {
                self.failed(code)?;
            }
        }
    }

    /// The picture that shows at `seconds`: the last one that starts by then, or the first if
    /// none does. Reads on from the last one it gave when that is close behind, and otherwise
    /// goes to the key frame before it. `None` if there are no pictures at all.
    pub fn frame_at(&mut self, seconds: f64) -> Result<Option<DecodedFrame>, VideoError> {
        let reads_on = self
            .shown
            .as_ref()
            .is_some_and(|s| s.start <= seconds + EPSILON && seconds - s.start < READ_ON);
        if !reads_on {
            self.seek(seconds)?;
        }
        let mut back = 1.0;
        loop {
            let shown = self.shown.take();
            match self.next_frame()? {
                Some(f) if f.start <= seconds + EPSILON => self.shown = Some(f),
                next => {
                    if shown.is_some() {
                        self.peeked = next;
                        self.shown = shown;
                        return Ok(self.shown.clone());
                    }
                    // It came down after the time, or past the end: from further back, unless it
                    // was from the start already.
                    if self.sought <= 0.0 {
                        self.shown = next;
                        return Ok(self.shown.clone());
                    }
                    let from = (seconds - back).max(0.0);
                    back *= 2.0;
                    self.seek(from)?;
                }
            }
        }
    }

    /// Reads the next packet of the pictures, or `None` at the end.
    fn read_packet(&mut self) -> Result<Option<Packet>, VideoError> {
        loop {
            // SAFETY: nothing to it.
            let raw = unsafe { (self.ffmpeg.f.av_packet_alloc)() };
            if raw.is_null() {
                return Err(VideoError::Source("out of memory".into()));
            }
            let packet = Packet {
                ffmpeg: self.ffmpeg.clone(),
                raw,
                size: 0,
            };
            // SAFETY: the input is open and the packet is empty.
            let code = unsafe { (self.ffmpeg.f.av_read_frame)(self.input.raw, packet.raw) };
            if code == AVERROR_EOF {
                return Ok(None);
            }
            if code < 0 {
                return Err(VideoError::Source(self.ffmpeg.error(code)));
            }
            if packet_stream(&packet, self.ffmpeg.layout) == self.video.index {
                return Ok(Some(packet));
            }
        }
    }

    /// A picture that couldn't be decoded. On the GPU before anything came out, that is a sign
    /// that the GPU can't do this one, and the CPU does it from where it went.
    fn failed(&mut self, code: c_int) -> Result<(), VideoError> {
        if self.decoder.hardware && !self.decoded_any {
            self.decoder = open_video_decoder(&self.ffmpeg, &self.video, None)?;
            self.render_node = None;
            return self.seek(self.sought);
        }
        self.failures += 1;
        if self.failures > MAX_FAILURES {
            return Err(VideoError::Decode(self.ffmpeg.error(code)));
        }
        Ok(())
    }

    /// The picture in a frame just decoded, in memory. Takes care of `frame`.
    fn deliver(&mut self, frame: *mut Opaque) -> Option<DecodedFrame> {
        let ffmpeg = self.ffmpeg.clone();
        let (f, l) = (&ffmpeg.f, ffmpeg.layout);
        let tb = self.video.time_base;
        // SAFETY: a frame just decoded, and the layout says where the fields are.
        let (pts, length, format) = unsafe {
            let pts: i64 = read(frame, l.frame_best_effort);
            let length: i64 = read(frame, l.frame_duration);
            let format: c_int = read(frame, l.frame_format);
            (pts, length, format)
        };
        let duration = if length > 0 {
            Some(length as f64 * tb)
        } else {
            self.info.frame_rate.map(|r| 1.0 / r)
        };
        let start = match (pts, self.last) {
            (AV_NOPTS_VALUE, Some((start, length))) => start + length,
            (AV_NOPTS_VALUE, None) => self.sought,
            _ => pts as f64 * tb - self.origin,
        };
        self.last = Some((start, duration.unwrap_or(0.0)));
        let mut frame = frame;
        if format == ffmpeg.vaapi_pix_fmt {
            let copy = alloc_frame(&ffmpeg);
            // SAFETY: a picture on the GPU, and an empty frame to copy it to.
            let code = unsafe { (f.av_hwframe_transfer_data)(copy, frame, 0) };
            // SAFETY: from `av_frame_alloc`.
            unsafe { (f.av_frame_free)(&mut frame) };
            frame = copy;
            if code < 0 {
                // SAFETY: from `av_frame_alloc`.
                unsafe { (f.av_frame_free)(&mut frame) };
                return None;
            }
        }
        // SAFETY: a frame in memory.
        let (format, w, h) = unsafe {
            let format: c_int = read(frame, l.frame_format);
            let w: c_int = read(frame, l.frame_width);
            let h: c_int = read(frame, l.frame_height);
            (format, w as u32, h as u32)
        };
        let name = ffmpeg.pix_fmt_name(format);
        // SAFETY: a decoded picture.
        let color = unsafe { color_of(&ffmpeg, frame, &name, h) };
        let (pixels, keep) = memory_frame(&ffmpeg, frame, &mut self.scaler, Vec::new())?;
        Some(DecodedFrame {
            start,
            duration,
            frame: Arc::new(VideoFrame {
                id: next_id(),
                stream: self.stream,
                alive: Arc::downgrade(&self.alive),
                size: (w, h),
                pixels,
                color,
                rotation: self.video.rotation,
                keep: Box::new(keep),
            }),
        })
    }
}

impl Drop for VideoReader {
    fn drop(&mut self) {
        self.stop.store(true, Ordering::Relaxed);
    }
}

fn open_video_decoder(
    ffmpeg: &Arc<Ffmpeg>,
    video: &StreamInfo,
    render_node: Option<&Path>,
) -> Result<Decoder, VideoError> {
    if let Some(decoder) = render_node.and_then(|node| hardware_decoder(ffmpeg, video, node)) {
        return Ok(decoder);
    }
    open_decoder(ffmpeg, video, video.decoder, None, thread_count())
}
