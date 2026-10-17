use std::ops::Range;

use super::bytes::{u32_be, u64_be};

fn boxes(bytes: &[u8], range: Range<usize>) -> Option<Vec<([u8; 4], Range<usize>)>> {
    let mut found = Vec::new();
    let mut at = range.start;
    while at < range.end {
        let size = u32_be(bytes, at)?;
        let kind: [u8; 4] = bytes.get(at + 4..at + 8)?.try_into().ok()?;
        let (header, total) = match size {
            0 => (8, range.end - at),
            1 => (16, usize::try_from(u64_be(bytes, at + 8)?).ok()?),
            size => (8, size as usize),
        };
        let end = at.checked_add(total)?;
        if total < header || end > range.end {
            return None;
        }
        found.push((kind, at + header..end));
        at = end;
    }
    Some(found)
}

fn child(bytes: &[u8], parent: &Range<usize>, kind: [u8; 4]) -> Option<Range<usize>> {
    boxes(bytes, parent.clone())?
        .into_iter()
        .find(|(found, _)| *found == kind)
        .map(|(_, body)| body)
}

fn sound_duration(bytes: &[u8], track: &Range<usize>) -> Option<bool> {
    let media = child(bytes, track, *b"mdia")?;
    let handler = child(bytes, &media, *b"hdlr")?;
    if bytes.get(handler.start + 8..handler.start + 12)? != b"soun" {
        return Some(false);
    }
    let header = child(bytes, &media, *b"mdhd")?;
    let at = header.start;
    let (timescale, duration) = match *bytes.get(at)? {
        0 => (u32_be(bytes, at + 12)?, u64::from(u32_be(bytes, at + 16)?)),
        1 => (u32_be(bytes, at + 20)?, u64_be(bytes, at + 24)?),
        _ => return None,
    };
    Some(timescale > 0 && duration > 0)
}

// An MP4/M4A file is accepted when its box tree parses and a sound track declares a duration.
pub(super) fn valid(bytes: &[u8]) -> bool {
    let Some(top) = boxes(bytes, 0..bytes.len()) else {
        return false;
    };
    let Some((_, movie)) = top.into_iter().find(|(kind, _)| kind == b"moov") else {
        return false;
    };
    boxes(bytes, movie).is_some_and(|tracks| {
        tracks
            .iter()
            .filter(|(kind, _)| kind == b"trak")
            .any(|(_, track)| sound_duration(bytes, track) == Some(true))
    })
}
