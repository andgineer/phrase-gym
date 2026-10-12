# Synthetic container fixtures

These are 100 ms, 440 Hz tones generated locally for MP3, Vorbis Ogg and AAC MP4 container validation and original-byte delivery tests. They contain no speech, learner recordings or provider output. Tests read the checked-in bytes and do not require FFmpeg.

Generation (FFmpeg 9.0.1; encoder metadata can vary across versions):

```bash
ffmpeg -f lavfi -i 'sine=frequency=440:duration=0.1:sample_rate=16000' -map_metadata -1 -c:a libmp3lame tone.mp3
ffmpeg -f lavfi -i 'sine=frequency=440:duration=0.1:sample_rate=16000' -map_metadata -1 -ac 2 -c:a vorbis -strict -2 tone.ogg
ffmpeg -f lavfi -i 'sine=frequency=440:duration=0.1:sample_rate=16000' -map_metadata -1 -c:a aac tone.m4a
```

These fixtures establish container acceptance, not speech quality or actual-device playback.
