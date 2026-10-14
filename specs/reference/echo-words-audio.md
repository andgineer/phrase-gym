# Echo-words audio: inspected implementation and Phrase Gym guidance

Inspected **2026-10-11** at the user's request. This records the local repository's code, configuration and documented findings; no synthesis service was called, audio listened to, or echo-words files changed during this inspection. The engine choices below are an existing implementation reference, not a fresh comparison of all currently available voices. Phrase Gym adoption details remain recommendations.

## What echo-words uses

The [example language configuration](../../../echo-words/languages.example.toml), [audio implementation](../../../echo-words/src/echo_words/audio.py) and [TTS decision](../../../echo-words/spec/decision-tts.md) agree on a language-specific chain:

| Language | Configured generation choice | Important handling |
|---|---|---|
| English | Piper `en_US-lessac-medium`; English Edge voice as fallback | US English in the example. Configured default Edge voices are `en-US-AriaNeural` or `en-GB-SoniaNeural`, according to the accent setting. |
| German | Piper `de_DE-thorsten-medium` | An Edge fallback requires an explicitly configured German voice. The example file supplies none; the English global default is deliberately ineligible. |
| Serbian | Edge TTS `sr-RS-SophieNeural` | Convert Serbian Latin spelling to Cyrillic for speech input only; retain the original displayed text. |

For single words with a configured Commons prefix, echo-words first attempts a human recording. Multi-word phrases bypass that lookup and go to the configured synthesis path. Its function accepts whole phrases/text even though its argument is named `word`; this is not a reason to synthesize a phrase word by word. It generates an MP3 once and reuses it by language/text.

The Serbian preparation is explicit in `audio.py`: `_speech_text` selects a script converter for `sr-RS` voices; `_serbian_cyrillic` maps letters and digraphs including `lj`, `nj` and `dž`. The local TTS decision records pronunciation problems when these voices were given Latin input. Its tests assert the transformed input, including `haljina` → `хаљина`, and leave English unchanged. Those tests check code behaviour, not acoustic quality.

The same decision records a critical rejected voice: the Piper `serbski_institut` voice listed under a Serbian locale was identified there as **Lower Sorbian**, not Serbian. Do not select that voice just from its locale/name. The pinned [voice catalogue](../../../echo-words/src/echo_words/voices.py) actually installs only the English Lessac and German Thorsten voices, with SHA-256 verification.

The implementation never falls back to an unrelated language's default voice. It uses temporary files and atomic publication, bounded voice loading and a cache of two loaded Piper voices. That cache size and the preference for small local engines were chosen for echo-words' 1 GB server alongside Anki. They are host constraints, not a universal ranking of speech quality or a Phrase Gym hosting decision.

## Recommended application to Phrase Gym

1. **Keep supplied course/actor audio first.** Import the complete phrase or correct dialogue turn with its text and source. Reuse suitable existing audio from echo-words only when it speaks the exact intended phrase/turn, not merely its dictionary headword. Do not replace a professional recording just to standardize the engine.
2. **Generate missing reference clips during preparation.** Use the language-specific choices above as the implementation baseline, preserving an explicitly chosen accent and dialogue speaker. The English example is US English; do not silently substitute it for a requested British voice. Keep engine/voice choices in preparation configuration, not routine drill controls.
3. **Prepare Serbian speech input separately.** Keep the user's displayed Latin or Cyrillic text unchanged; store the speech input and conversion version with the audio. Check full-phrase conversion, digraph boundaries, names and embedded foreign words during implementation rather than assume that blind replacement is correct for every text.
4. **Speak complete phrases/turns.** Preserve punctuation and natural phrase delivery; do not assemble whole-phrase reference audio from isolated word recordings. Partial-cue audio, if used, is a separate reviewed asset and must not reveal the hidden answer.
5. **Store once and play locally during drills.** Keep engine, voice, language/locale, accent, exact speech text, source/revision and preparation settings with the asset. Include these in generation identity so a voice/text change cannot silently reuse stale audio. Preserve course originals; any normalized playback version remains linked to its original.
6. **Use only a matching-language fallback.** If unavailable, retain a clear pending/failed audio job with bounded retry and a retry action. Existing recordings continue to play during an online TTS outage. Do not replace missing reference audio with speech from a different language or silently treat missing audio as a memory failure.

The UI label remains **Reference**, regardless of actor or synthesis provenance. Explanations/UI language never select the reference voice: the phrase's learning language and intended accent do.

## Details to adapt rather than copy

- Echo-words' cache key is language plus text, excluding voice/settings. Phrase Gym's versioned media/jobs must include the preparation configuration and keep explicit provenance; do not infer human origin from sample rate as echo-words' inspection helper does.
- Its Commons lookup is a single-word pronunciation feature, not a source of complete phrase recordings. Phrase Gym's first source is supplied course/lesson audio. Do not add a Commons lookup to every phrase import.
- The [observed-defects record](../../../echo-words/spec/plan/observed-defects.md) documents missing audio with no ordinary per-item retry. Phrase Gym already plans durable jobs: make failure and recovery explicit instead of inheriting that gap.
- Do not import echo-words' Anki lifecycle, free-only budget rule, ten-second answer deadline or specific machine sizing as Phrase Gym requirements. Reuse the audio approach through a small preparation adapter, without depending on echo-words' application runtime/settings.
- Engineering acceptance should cover complete English/German/Serbian phrases, script handling, correct text/voice, playable nonempty output, consistent playback volume, same-language fallback and retry without replacing good existing assets. Listening/content checks are implementation work, not learner-run efficacy experiments. No new quality or device pass is claimed by this source inspection.

Useful source locations: `audio.py:75` (cache/source chain), `audio.py:335` (Piper), `audio.py:404` (Edge and Serbian input), `audio.py:480` (cache identity), `config.py:21` (English accent defaults), `tests/test_audio.py:696` (voice failure and language/script tests), and `spec/functional-description.md:472` (audio contract). The paths above are navigation aids; the findings are written here so the handoff remains understandable without the sibling checkout.

## P5 implementation measurements — 2026-10-14

Phrase Gym now has its own bounded Piper subprocess and Edge adapter; echo-words remains
read-only. The English/German native models and configurations were downloaded to disposable
storage using the inspected pinned catalogue and verified byte-for-byte:

| Voice | Model SHA-256 | Configuration SHA-256 |
|---|---|---|
| `en_US-lessac-medium` | `5efe09e69902187827af646e1a6e9d269dee769f9877d17b16b1b46eeaaf019f` | `efe19c417bed055f2d69908248c6ba650fa135bc868b0e6abb3da181dab690a0` |
| `de_DE-thorsten-medium` | `7e64762d8e5118bb578f2eea6207e1a35a8e0c30595010b666f983fc87bb7819` | `974adee790533adb273a1ac88f49027d2a1b8f0f2cf4905954a4791e79264e85` |

Pinned base URL: `https://huggingface.co/rhasspy/piper-voices/resolve/v1.0.0/`.
Append `en/en_US/lessac/medium/en_US-lessac-medium.onnx` or
`de/de_DE/thorsten/medium/de_DE-thorsten-medium.onnx`, and `.json` for its configuration.

Real native full-phrase outputs and real English/German/Serbian Edge outputs are retained in
[the P5 audio evidence](../plan/evidence/p5-audio/) through the main P5 evidence/runbook links.
Native cache repeats preserved the exact asset ID. Container/duration checks are positive;
no agent or automated check attests human listening, pronunciation/accent quality, or
actual-device acceptance. Original actor/course clips remain preferred and unmodified.
