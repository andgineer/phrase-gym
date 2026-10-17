use phrase_gym_contracts::media::ReferenceMime;

use super::{MAX_AUDIO_BYTES, validate};

const WAV: &[u8] = include_bytes!("../../../../../fixtures/preservation/media/silence-100ms.wav");
const TRUNCATED_WAV: &[u8] =
    include_bytes!("../../../../../fixtures/preservation/media/truncated.wav");
const MP3: &[u8] = include_bytes!("../../../../../fixtures/preservation/media/tone.mp3");
const OGG: &[u8] = include_bytes!("../../../../../fixtures/preservation/media/tone.ogg");
const M4A: &[u8] = include_bytes!("../../../../../fixtures/preservation/media/tone.m4a");

const ALL: [ReferenceMime; 4] = [
    ReferenceMime::Wav,
    ReferenceMime::Mpeg,
    ReferenceMime::Ogg,
    ReferenceMime::Mp4,
];

#[test]
fn each_container_is_accepted_only_under_its_own_mime() {
    for (bytes, own) in [
        (WAV, ReferenceMime::Wav),
        (MP3, ReferenceMime::Mpeg),
        (OGG, ReferenceMime::Ogg),
        (M4A, ReferenceMime::Mp4),
    ] {
        for mime in ALL {
            assert_eq!(
                validate(bytes, mime).is_ok(),
                mime == own,
                "{own:?} as {mime:?}"
            );
        }
    }
}

#[test]
fn empty_oversized_truncated_and_non_audio_bytes_are_rejected() {
    for mime in ALL {
        assert!(validate(b"", mime).is_err());
        assert!(validate(b"not-audio", mime).is_err());
    }
    assert!(validate(TRUNCATED_WAV, ReferenceMime::Wav).is_err());
    assert!(validate(&OGG[..OGG.len() - 10], ReferenceMime::Ogg).is_err());
    assert!(validate(&M4A[..M4A.len() - 10], ReferenceMime::Mp4).is_err());
    assert!(validate(&MP3[..40], ReferenceMime::Mpeg).is_err());
    let mut oversized = WAV.to_vec();
    oversized.resize(MAX_AUDIO_BYTES + 1, 0);
    assert!(validate(&oversized, ReferenceMime::Wav).is_err());
}

#[test]
fn wav_needs_pcm_frames_and_a_format_before_its_data() {
    let mut no_frames = WAV[..44].to_vec();
    no_frames[40..44].copy_from_slice(&0u32.to_le_bytes());
    assert!(validate(&no_frames, ReferenceMime::Wav).is_err());
    let mut compressed = WAV.to_vec();
    compressed[20..22].copy_from_slice(&3u16.to_le_bytes());
    assert!(validate(&compressed, ReferenceMime::Wav).is_err());
}
