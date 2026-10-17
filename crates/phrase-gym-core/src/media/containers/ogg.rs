use std::ops::Range;

use super::bytes::{u32_le, u64_le};

struct Page {
    begins_stream: bool,
    granule: u64,
    serial: u32,
    body: Range<usize>,
}

fn page(bytes: &[u8], at: usize) -> Option<Page> {
    if bytes.get(at..at + 4)? != b"OggS" || *bytes.get(at + 4)? != 0 {
        return None;
    }
    let segments = usize::from(*bytes.get(at + 26)?);
    let table = bytes.get(at + 27..at + 27 + segments)?;
    let start = at + 27 + segments;
    let end = start + table.iter().map(|&lace| usize::from(lace)).sum::<usize>();
    (end <= bytes.len()).then(|| Page {
        begins_stream: bytes[at + 5] & 0x02 != 0,
        granule: u64_le(bytes, at + 6).unwrap_or(0),
        serial: u32_le(bytes, at + 14).unwrap_or(0),
        body: start..end,
    })
}

// Vorbis in Ogg: identification and comment headers, then pages up to a positive end granule.
pub(super) fn valid(bytes: &[u8]) -> bool {
    let mut pages = Vec::new();
    let mut at = 0;
    while at < bytes.len() {
        let Some(found) = page(bytes, at) else {
            return false;
        };
        at = found.body.end;
        pages.push(found);
    }
    let Some(first) = pages.first().filter(|first| first.begins_stream) else {
        return false;
    };
    let ident = &bytes[first.body.clone()];
    let rate = u32_le(ident, 12).unwrap_or(0);
    let identified = ident.len() >= 30
        && ident.starts_with(b"\x01vorbis")
        && u32_le(ident, 7) == Some(0)
        && ident[11] > 0
        && rate > 0;
    let stream: Vec<&Page> = pages
        .iter()
        .filter(|page| page.serial == first.serial)
        .collect();
    let commented = stream
        .get(1)
        .is_some_and(|second| bytes[second.body.clone()].starts_with(b"\x03vorbis"));
    let last = stream
        .iter()
        .rev()
        .map(|page| page.granule)
        .find(|&granule| granule != u64::MAX)
        .unwrap_or(0);
    identified && commented && last > 0
}
