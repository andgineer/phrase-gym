mod bytes;
mod mp3;
mod mp4;
mod ogg;
mod wav;

use phrase_gym_contracts::media::ReferenceMime;

use crate::error::CoreError;

pub const MAX_AUDIO_BYTES: usize = 12 * 1024 * 1024;

// Container checks apply to reference media only; learner recordings are never inspected.
pub fn validate(audio: &[u8], mime: ReferenceMime) -> Result<(), CoreError> {
    if audio.is_empty() || audio.len() > MAX_AUDIO_BYTES {
        return Err(CoreError::Invalid(
            "Reference audio must be nonempty and at most 12 MiB".to_owned(),
        ));
    }
    let valid = match mime {
        ReferenceMime::Wav => wav::valid(audio),
        ReferenceMime::Mpeg => mp3::valid(audio),
        ReferenceMime::Mp4 => mp4::valid(audio),
        ReferenceMime::Ogg => ogg::valid(audio),
    };
    if valid {
        Ok(())
    } else {
        Err(CoreError::Invalid(
            "Invalid audio or MIME type does not match its container".to_owned(),
        ))
    }
}

#[cfg(test)]
mod tests;
