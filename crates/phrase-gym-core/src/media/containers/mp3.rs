use super::bytes::u32_be;

const SEARCH_WINDOW: usize = 64 * 1024;

struct Frame {
    length: usize,
    samples: u64,
    side_info: usize,
}

const BITRATES: [[u32; 14]; 5] = [
    [
        32, 64, 96, 128, 160, 192, 224, 256, 288, 320, 352, 384, 416, 448,
    ],
    [
        32, 48, 56, 64, 80, 96, 112, 128, 160, 192, 224, 256, 320, 384,
    ],
    [
        32, 40, 48, 56, 64, 80, 96, 112, 128, 160, 192, 224, 256, 320,
    ],
    [
        32, 48, 56, 64, 80, 96, 112, 128, 144, 160, 176, 192, 224, 256,
    ],
    [8, 16, 24, 32, 40, 48, 56, 64, 80, 96, 112, 128, 144, 160],
];

fn frame(bytes: &[u8], at: usize) -> Option<Frame> {
    let header = bytes.get(at..at + 4)?;
    if header[0] != 0xFF || header[1] & 0xE0 != 0xE0 {
        return None;
    }
    let version = (header[1] >> 3) & 3;
    let layer = (header[1] >> 1) & 3;
    let bitrate_index = usize::from(header[2] >> 4);
    let rate_index = usize::from((header[2] >> 2) & 3);
    if version == 1 || layer == 0 || !(1..=14).contains(&bitrate_index) || rate_index == 3 {
        return None;
    }
    let mpeg1 = version == 3;
    let table = match (mpeg1, layer) {
        (true, 3) => 0,
        (true, 2) => 1,
        (true, _) => 2,
        (false, 3) => 3,
        (false, _) => 4,
    };
    let bitrate = u64::from(BITRATES[table][bitrate_index - 1]) * 1000;
    let base_rate = [44_100, 48_000, 32_000][rate_index];
    let sample_rate = match version {
        3 => base_rate,
        2 => base_rate / 2,
        _ => base_rate / 4,
    };
    let padding = u64::from((header[2] >> 1) & 1);
    let (length, samples) = match (layer, mpeg1) {
        (3, _) => ((12 * bitrate / sample_rate + padding) * 4, 384),
        (2, _) | (_, true) => (144 * bitrate / sample_rate + padding, 1152),
        (_, false) => (72 * bitrate / sample_rate + padding, 576),
    };
    let mono = header[3] >> 6 == 3;
    let side_info = match (mpeg1, mono) {
        (true, true) | (false, false) => 17,
        (true, false) => 32,
        (false, true) => 9,
    };
    Some(Frame {
        length: usize::try_from(length).ok()?,
        samples,
        side_info,
    })
}

fn after_id3(bytes: &[u8]) -> usize {
    if bytes.get(0..3) != Some(b"ID3") || bytes.len() < 10 {
        return 0;
    }
    let size = bytes[6..10].iter().fold(0usize, |total, byte| {
        (total << 7) | usize::from(byte & 0x7F)
    });
    let footer = if bytes[5] & 0x10 == 0 { 0 } else { 10 };
    10 + size + footer
}

// The first frame must be confirmed by a following frame (or end the file) so random bytes that
// happen to contain a sync word are not accepted as MPEG audio.
pub(super) fn valid(bytes: &[u8]) -> bool {
    let start = after_id3(bytes);
    let end = bytes.len().min(start.saturating_add(SEARCH_WINDOW));
    (start..end).any(|at| {
        let Some(first) = frame(bytes, at) else {
            return false;
        };
        let next = at + first.length;
        let confirmed = next == bytes.len()
            || bytes.get(next..next + 3) == Some(b"TAG")
            || frame(bytes, next).is_some();
        first.length > 4 && next <= bytes.len() && confirmed && playable(bytes, at, &first)
    })
}

fn playable(bytes: &[u8], at: usize, first: &Frame) -> bool {
    let tag = at + 4 + first.side_info;
    let summary = bytes.get(tag..tag + 4);
    if summary == Some(b"Xing") || summary == Some(b"Info") {
        let flags = u32_be(bytes, tag + 4).unwrap_or(0);
        if flags & 1 == 1 {
            let frames = u64::from(u32_be(bytes, tag + 8).unwrap_or(0));
            return frames * first.samples > 0;
        }
    }
    true
}
