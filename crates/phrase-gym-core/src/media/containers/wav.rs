use super::bytes::{u16_le, u32_le};

const PCM: u16 = 1;
const EXTENSIBLE: u16 = 0xFFFE;

pub(super) fn valid(bytes: &[u8]) -> bool {
    if bytes.get(0..4) != Some(b"RIFF") || bytes.get(8..12) != Some(b"WAVE") {
        return false;
    }
    let mut format = None;
    let mut at = 12;
    while let (Some(id), Some(size)) = (bytes.get(at..at + 4), u32_le(bytes, at + 4)) {
        let start = at + 8;
        let size = size as usize;
        match id {
            b"fmt " => format = parse_format(bytes.get(start..start + size)),
            b"data" => {
                let Some((channels, width)) = format else {
                    return false;
                };
                let frame = usize::from(channels) * usize::from(width);
                let available = bytes.len().saturating_sub(start);
                return frame > 0 && size >= frame && available >= size - size % frame;
            }
            _ => {}
        }
        at = start.saturating_add(size).saturating_add(size % 2);
    }
    false
}

fn parse_format(chunk: Option<&[u8]>) -> Option<(u16, u16)> {
    let chunk = chunk?;
    let tag = u16_le(chunk, 0)?;
    let channels = u16_le(chunk, 2)?;
    let bits = u16_le(chunk, 14)?;
    let pcm = tag == PCM || (tag == EXTENSIBLE && u16_le(chunk, 24)? == PCM);
    (pcm && channels > 0 && bits > 0).then(|| (channels, bits.div_ceil(8)))
}
