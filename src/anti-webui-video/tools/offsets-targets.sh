#!/bin/sh
# Prints, for each system, where the fields `LAYOUTS` in src/ffi.rs asks for are, from the headers
# of an FFmpeg, so that a row can be checked to hold everywhere. Only clang is needed: the C library
# of each system is stood in for by what the headers need of it, which has nothing to do with the
# numbers. Each line should be the same as the others and as the row.
#
#   tools/offsets-targets.sh [<include dir of FFmpeg>]    (/usr/include by default)
set -eu
include=${1:-/usr/include}
work=$(mktemp -d)
trap 'rm -rf "$work"' EXIT
mkdir -p "$work/inc" "$work/stub"
cp -r "$include/libavcodec" "$include/libavformat" "$include/libavutil" "$work/inc/"
for h in errno.h math.h stdlib.h string.h time.h; do : > "$work/stub/$h"; done
printf 'typedef struct FILE FILE;\n#define SEEK_SET 0\n#define SEEK_CUR 1\n#define SEEK_END 2\n' \
    > "$work/stub/stdio.h"
printf '#include <stdint.h>\n#define PRId64 "lld"\n#define PRIu64 "llu"\n#define PRIx64 "llx"\n#define PRIX64 "llX"\n#define PRId32 "d"\n#define PRIu32 "u"\n#define PRIx32 "x"\n' \
    > "$work/stub/inttypes.h"
# In the order of the fields of `Layout`, then the sizes of `AVChannelLayout` and a codec id.
cat > "$work/numbers.c" <<'EOF'
#include <stddef.h>
#include <libavcodec/avcodec.h>
#include <libavformat/avformat.h>
#include <libavutil/frame.h>
#define O(t, f) offsetof(t, f)
const unsigned long long numbers[] = {
    O(AVFormatContext, streams), O(AVFormatContext, start_time), O(AVFormatContext, duration),
    O(AVFormatContext, interrupt_callback),
    O(AVStream, codecpar), O(AVStream, time_base), O(AVStream, start_time), O(AVStream, avg_frame_rate),
    O(AVCodecParameters, codec_id), O(AVCodecParameters, coded_side_data),
    O(AVCodecParameters, nb_coded_side_data), O(AVCodecParameters, width), O(AVCodecParameters, height),
    O(AVCodecContext, get_format), O(AVCodecContext, hw_device_ctx),
    O(AVFrame, data), O(AVFrame, linesize), O(AVFrame, extended_data), O(AVFrame, width),
    O(AVFrame, height), O(AVFrame, nb_samples), O(AVFrame, format), O(AVFrame, sample_rate),
    O(AVFrame, color_range), O(AVFrame, color_trc), O(AVFrame, colorspace),
    O(AVFrame, best_effort_timestamp), O(AVFrame, ch_layout), O(AVFrame, duration),
    O(AVPacket, size), O(AVPacket, stream_index), O(AVPacketSideData, data), O(AVPacketSideData, size),
    sizeof(AVChannelLayout), sizeof(enum AVCodecID),
};
EOF
for target in x86_64-unknown-linux-gnu x86_64-w64-windows-gnu x86_64-pc-windows-msvc \
    arm64-apple-macos x86_64-apple-macos; do
    numbers=$(clang --target="$target" -ffreestanding -nostdlibinc -isystem "$work/stub" \
        -I"$work/inc" -w -S -emit-llvm -o - "$work/numbers.c" |
        sed -n 's/^@numbers = .*\[\(i64 .*\)\].*/\1/p' | sed 's/i64 //g')
    printf '%-26s %s\n' "$target" "$numbers"
done
