pub(super) fn from_pcm(pcm: &[u8], rate: u32) -> Result<Vec<u8>, String> {
    if pcm.is_empty()
        || !pcm.len().is_multiple_of(2)
        || pcm.len() as u64 > super::MAX_AUDIO_BYTES
        || pcm.len() as u64 > u64::from(rate) * 2 * 30
        || !pcm.chunks_exact(2).any(|s| s != [0, 0])
    {
        return Err(
            "Piper returned empty, silent, incomplete or oversized PCM; no reference accepted"
                .into(),
        );
    }
    let length = u32::try_from(pcm.len()).map_err(|_| "PCM length overflow")?;
    let mut wav = Vec::with_capacity(pcm.len() + 44);
    wav.extend_from_slice(b"RIFF");
    wav.extend_from_slice(&(length + 36).to_le_bytes());
    wav.extend_from_slice(b"WAVEfmt \x10\0\0\0\x01\0\x01\0");
    wav.extend_from_slice(&rate.to_le_bytes());
    wav.extend_from_slice(&(rate * 2).to_le_bytes());
    wav.extend_from_slice(b"\x02\0\x10\0data");
    wav.extend_from_slice(&length.to_le_bytes());
    wav.extend_from_slice(pcm);
    Ok(wav)
}
