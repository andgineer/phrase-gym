# P1 review and proposed production starting point

**Latest accepted interaction, 2026-10-12:** follow the [unified practice-card decision](../decision-unified-practice-card.md). One card retains recording, replay, reference and colored/icon self-check controls; answer reveal replaces gaps, explanation expands inline beside Show answer. Remove the recording-mode switch, Practice again and supported-only Continue. The microphone/stop icon reflects the active audio action. This supersedes conflicting earlier flow details below.

2026-10-12. The user requested implementation of P1. A new disposable prototype now exists under `prototypes/`; the archived attempt was not used. [Evidence](evidence/p1-2026-10-12.md) separates observed software behaviour from missing device/content/user evidence. P2 has not begun.

## Concrete proposals available for review

| Choice | Implemented P1 proposal | What the learner experiences |
|---|---|---|
| Card and help | One situation, meaningful stored mask, visible Show & explain; Show answer reveals the complete phrase | Familiar material drills directly; forgotten material gets explicit meaning help without a fabricated failure |
| Grading | Not yet / With effort / Recalled; one tap saves/advances; Undo; extra practice first retains the original result | Recall difficulty is separate from pronunciation practice; capture failure creates no automatic grade; the problem submenu is removed |
| Recording default | On, accepted by the user on 2026-10-12; manual start/stop | Microphone access starts only on the learner’s explicit action; turning recording off enables silent practice |
| Lesson policy | Due-only initially, explicit Practice all choice when a lesson is selected | The open due-only/all choice is visible; choosing a lesson alone does not start a session |
| Language navigation | `/learn/en`, `/learn/sr`; first-launch and Settings toggle initially uses native-language controls | Bookmarks/tabs keep independent learning languages; explanations stay native; switching pauses and preserves the session |
| Session/empty queue | Small fixture queue, no forced wait; one summary entry per target; explicit empty-due state | Repairs defer when too little intervening work/time remains; the learner can browse or voluntarily practise all |
| Retry/coverage | Five-minute lower bound plus two distinct intervening targets; cap two retries; under-covered chunks first, tie-break against last cue | A later prepared mask can differ without creating another learning target |
| First content | Twelve manual authored fixtures, two prepared dialogues; English/Serbian views, three explanation/UI languages | Inspect fixed wording, word recall, construction use, frame inflection, contraction and long-phrase cases |
| Audio/provider | English macOS Samantha for disposable checks; import checked actor/generated clips; Serbian missing until supplied | Existing actor audio wins; no wrong-language fallback; generation is absent from the drill's runtime |
| Platform/foundation | Continue the existing Vue/TypeScript PWA + Python/FastAPI/SQLite recommendation only after the P1 gate | Current browser code is disposable; production adapters, FSRS and transactional recording BLOB storage remain P2/P3 work |

Except for the [accepted manual feedback](../decision-p1-manual-feedback.md), these are implementer recommendations made concrete by the prototype, not newly accepted user decisions. Manual stop, all-take retention, optional recording on Show & explain, language roles and the unified session entry were already agreed and remain intact.

## Review still needed

- User walkthrough/corrections for the compact layout, help placement, labels, lesson policy and language/resume presentation. Record accepted corrections in the relevant decision documents; do not infer visual approval from passing browser tests.
- Actual iPhone Safari and installed-home-screen checks, including permission, real spoken capture, ordered automatic playback, speaker/headphones, interruptions, offline writes and normal restart.
- A phone-reachable private HTTPS origin trusted by that phone. The local TLS smoke check used a temporary untrusted development certificate with a test-browser exception, not installed phone trust.
- Supplied actor/course audio and a prepared Serbian clip, checked against complete target wording. English assets decoded in software; no listening-quality or accent benchmark is claimed. Serbian wording/translations need native-speaker confirmation.
- Real retrieval quality on representative source queries, beyond the prototype's hand-authored index; source rollout/provider checks remain P5.

No delayed effectiveness trial, special exam or learner calibration is requested. These are usability, software, content and device checks. Keep the P1 exit gate open and do not begin P2 until the review/device limitations are resolved or explicitly accepted as release dependencies under the plan.

## Follow-up review and user direction

Three fresh Sol 6.1 reviews and corrections are recorded in [the review-loop report](evidence/p1-review-loop-2026-10-12.md). The next recommended P1 step is the user’s manual walkthrough, with a pause before a fourth automated review. The user rejects macOS-only audio dependence and wants the echo-words Piper/Edge approach and llmbroker for preparation; existing Samantha assets are disposable test provenance. Future code should be committed directly to `main`. Production source under `src/phrase_gym` remains a P2 scope decision.
