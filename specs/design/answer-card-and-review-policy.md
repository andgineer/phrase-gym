# Proposal: answer card and adaptive review policy

**Latest accepted interaction, 2026-10-12:** follow the [unified practice-card decision](../decision-unified-practice-card.md). One card retains recording, replay, reference and colored/icon self-check controls; answer reveal replaces gaps, explanation expands inline beside Show answer. Remove the recording-mode switch, Practice again and supported-only Continue. The microphone/stop icon reflects the active audio action. This supersedes conflicting earlier flow details below.

2026-10-11. Requested research/design work; **proposal for discussion, not user acceptance of the exact controls or application implementation**. Based on the [evidence review](../research-answer-card-and-scheduling.md). Preserve already accepted whole-phrase drills, recording sequence, manual stop, all-recording retention, optional silent practice, compact presentation, on-request native-language explanation, prepared cues and one schedule per target.

**Accepted user constraint:** choose the best-supported practical approach and let the learner use it normally. The user explicitly rejected being asked to validate learning effectiveness. Do not add experiments, validation screens, special assessment sessions, policy A/B tests, calibration tasks or a requirement to prove that the method works for them. Ordinary answer ratings drive automatic scheduling. Software correctness checks are an engineering responsibility.

## Objective

Maximize delayed, correct retrieval of useful language per minute of sustainable practice and support ready spoken use. Specify what is being predicted: recalling the target under the declared cue/task conditions, not a probability of success in any conversation.

## What the research changes in the design

The conclusion is that the design needs changes, not that the existing screens already implement the best-supported approach. The table below makes the recommendations concrete. Evidence supports the learning principles; the exact controls and implementation defaults are our design choices. These are current recommendations for discussion, not user acceptance or implemented features.

| Area | Earlier design/proposal | Current recommendation and learning purpose |
|---|---|---|
| Answer card | Got it / Try again mixes recall accuracy with the desire to practise pronunciation again. | **Not yet / With effort / Recalled** rates the original attempt. **Practice again** is separate. Accurate self-assessment and comparison make the schedule respond to retrieval rather than familiarity after seeing the answer. |
| Intervals | A fixed 1/3/7/14/30-day ladder, a one-day reset after every lapse and a permanent 30-day ceiling. | **FSRS with 0.90 desired retention**, adapting from ordinary results and elapsed time. Mature and difficult targets receive different intervals. Spacing and adaptive vocabulary-practice research motivate this; the exact library/default is an engineering recommendation. |
| Failed attempts | Retry after two other cards/end of session; elapsed spacing and completion criteria were insufficiently defined. | Give the correct model, allow supported practice, then retry after intervening material and time. Aim for one successful separated retrieval; cap the repair loop. Later scheduled sessions continue retrieval after initial success. This applies spacing and successive relearning without requiring repeated immediate echoes. |
| Alternate masks | Rotate stored variants primarily to show a different version; require every chunk at each interval level before advancing. | Prefer a clear, comparably supported cue covering **under-practised or previously difficult hidden chunks**. Keep cumulative coverage without the per-level gate. For word targets, the word itself must be hidden to count as word-form retrieval. This turns variation into purposeful retrieval practice; the selector is a design inference. |
| Use beyond one memorized example | Prepared dialogues and frame variations existed, but selection was not integrated into ordinary review. | Revisit the same target in suitable **different prepared situations or dialogue turns**, sharing one schedule. For fixed expressions the wording stays fixed; word/frame targets may use reviewed realizations. This matches practice more closely to intended use without introducing free conversation or ambiguous prompts. |
| Meaning and repair | Show & explain was available; return behaviour could amount to rereading and immediately repeating. | Keep its concise meaning/use and optional chunk explanation. After help, offer model-supported speaking and a later retrieval opportunity. If failures persist, revisit meaning or cue quality rather than cycle indefinitely. No mandatory introduction for already-known material. |
| Silent and helped responses | Silent scheduling was unresolved; practice actions and successful retrieval were not fully separated. | Deliberate silent recall may use the memory ratings, with modality retained. Reading, a displayed target and immediate model-supported repetition do not become independent-recall successes. A later successful take never overwrites the first attempt. This preserves meaningful scheduling evidence. |

Already suitable and retained: attempting the whole phrase before reveal, informative model comparison, optional actual speaking, adequate identifying cues, on-request native-language explanation and one schedule per target. Home search and recording storage support usability/history; the research does not establish them as learning interventions.

## Answer card

Keep the complete target phrase prominent after the original learner take → reference comparison. Keep **My recording**, **Reference** and Show & explain. The user rejected Model as an audio-button label: reference audio may be actor/course-supplied or generated. Localize controls using the UI preference while explanations retain the native language; see the [language/audio decision](../decision-languages-and-audio-labels.md). Use one concise prompt, **How did you recall it?**, and three result buttons:

| Visible result | Meaning for the original attempt, before feedback | Proposed scheduler mapping |
|---|---|---|
| **Not yet** | Could not retrieve the target, or omitted/changed required material beyond allowed variants. Recognizing it only after reveal is not recall. | `Again` / 1 for a valid attempted retrieval. |
| **With effort** | Retrieved the required target correctly using only the planned cue, but with substantial retrieval difficulty. | `Hard` / 2; still a successful retrieval. |
| **Recalled** | Retrieved the target correctly using only the planned cue, without substantial difficulty. Ordinary hesitation is allowed. | `Good` / 3; easy successes also use this button initially. |

The recommended three-button set omits a separate Easy grade to reduce fine subjective distinctions. Research supports distinguishing actual recall from failure; it does not prescribe the number of buttons. This is our design choice. The official scheduler accepts the supplied subset of ratings. Make clear that With effort means correct recall, never partial recall or an answer obtained from feedback. Any automatic parameter fitting uses the ratings actually supplied, not an assumed four-button distribution.

Keep criteria in a collapsible **What counts?** help area rather than repeating long instructions on every attempt. Do not display prospective day intervals under the buttons before the rating; the learner should report performance, not choose a preferred deadline. One result tap commits the observation and advances explicitly. Offer Undo for a mistaken tap with event correction/replay, not an extra review. No mandatory second confidence question, numerical score or response countdown.

**Separate actions:** Practice again is a request for extra practice, not a failure grade. Replaying audio is not a new attempt. Before a rerecording, preserve/rate the original attempt; later success cannot replace it. Pronunciation/flow practice can follow correct recall without declaring the word forgotten. A noticeable accent alone is not an error in recalling the target. Serious word-form errors and comprehension problems require appropriate repair, not just a speed score.

**Accepted simplification, 2026-10-12:** remove **Other answer / problem** and its submenu entirely, without a replacement skip/report action. An explicitly allowed equivalent remains a correct answer under What counts?; technical capture failures still create no automatic memory grade. See [manual feedback](../decision-p1-manual-feedback.md).

The full-phrase help control is **Show answer**, replacing the misleading Additional hint label. It reveals the whole phrase and records full-phrase exposure; subtler hints are not required for version one. Remove the separate I couldn’t recall it action. Present Show & explain outside the drill card as supplementary learning materials, visually distinct from answer and rating controls.

## Assistance and mode states

- **Normal graded attempt:** only the planned, reviewed cue was available before the response. Comparison afterward does not invalidate that original response. Grade that response, not the later echo.
- **Help before any attempt:** opening Show & explain is an exposure, not evidence that the learner tried and failed. The card says **Practised with help** and offers Continue / Practice again, without asking for an independent-recall success. Keep a later retrieval check pending. The user removed the separate I couldn’t recall it shortcut on 2026-10-12. Do not create a failure simply from opening the answer or learning materials.
- **Repair after a rated failure:** keep the original Not yet. Further recordings are supported practice; subsequent success in that repair episode does not create repeated long-term promotions.
- **Silent attempt:** if the learner deliberately recalled the answer before reveal, the same three outcomes can provide self-reported memory evidence, labeled silent. This is a proposed resolution of the previously open silent-scheduling question. It never verifies pronunciation or spoken fluency. Mere reading/recognition remains exposure only. Retain the response mode in normal history; do not ask the learner to run comparisons between modes.
- **Full target shown:** using a displayed word in a sentence does not demonstrate retrieval of that word. A model-visible take is practice, with every recording retained.
- **Search/list exposure:** keep accepted lookup behavior: no review grade or due-date promotion just from looking up or listing phrases. Record exposure only for content actually presented; preloaded hidden text is not exposure. An immediate follow-up drill is supported. A later, separated attempt can be assessed again; prior exposure is not a permanent prohibition on grading.

## Scheduler: preferred candidate

Recommend a versioned **FSRS-based ReviewPolicy**, with one target-level memory state and one due schedule shared across lesson links, masks and dialogue turns. This replaces the earlier fixed ladder as the assistant's recommended design, not as an already accepted user choice. Do not offer competing algorithms or an evaluation baseline in the learning UI.

Use **0.90 desired retention** as the app's default for the declared retrieval task. It means scheduling toward an estimated 90% chance of successful recall at the due time. It is not a fluency score. This is the practical default selected from the software's documented behavior and the broader research on efficient adaptive practice; the learner does not need to evaluate or tune it. Use maintained library parameters and pin the package/model/policy versions. Ordinary future success/failure ratings automatically change the target's memory estimate and interval.

Conceptually, update memory state from the observed grade and actual elapsed time, then choose the interval at which modeled recall reaches the retention target. Let the maintained library compute this rather than inventing an exponential approximation or hand-tuning FSRS coefficients. Preserve the policy version and state before/after the observation so scheduling decisions and corrections are reproducible.

### Concrete session rules

1. **First encounter in the app:** allow a diagnostic retrieval attempt because the material may already be known. If unfamiliar or misunderstood, show meaning/model, practise, and arrange a separated retrieval check. Import or reading alone creates no successful grade. Proposed fallback after study without a grade: a check the next day, recorded as a learning check rather than fabricated recall evidence.
2. **Due retrieval:** show a reviewed cue and collect one of the three outcomes. A valid first attempt supplies the scheduler observation. Record planned support, hidden chunks, target/cue revision, time, response mode and assessment source.
3. **After Not yet:** show/correct the target, optionally repeat with the model, then offer a retrieval retry after intervening material. Recommended default: roughly 5–10 minutes when the session permits, after at least two other targets; otherwise use a later session. Do not make the learner wait on a timer or immediately loop the card until it feels familiar.
4. **Repair limit:** aim for one correct separated retry. Default cap: two extra retrieval attempts per target per session, then continue later and flag persistent difficulty for content/meaning/cue repair. These are pragmatic defaults chosen to apply spaced retrieval without endless drilling, not parameters the learner must test. An immediate model-visible recording is not that separated retrieval.
5. **Long-term update:** the first valid result in the review group remains authoritative. Store all repairs, with their support and timestamps, without treating each as an independent spaced success. Configure library short learning/relearning steps explicitly to avoid a second hidden queue duplicating the app's repair loop. Recommendation: disable those library steps in this adapter and own the repair queue separately.
6. **Return across days:** after success, use the computed adaptive interval; after a lapse, use the shorter interval computed from prior history. Do not universally erase a mature target's history, reset every failure to one day, or cap every target permanently at 30 days. A never-assessed item can use the next-day learning check above. Scheduling uses real elapsed time after missed days; it does not reset the whole course.
7. **Session selection:** default to due material across lessons, respecting a sustainable time budget and throttling new material when due work grows. Recommend that selected-lesson Start session also uses that lesson's due items, with Practice all as optional extra practice. This resolves the earlier open question only if the user accepts it. Early extra practice/exposure must be logged; it is not free of memory effects merely because it does not promote the schedule.

### Eligibility and engineering responsibilities

Only record a retrieval grade when the target was actually tested under an adequate cue. Required target words shown before the response, model-help recordings, technical failures, unresolved ambiguous prompts and skipped items are not independent-recall observations. A planned partial cue can still yield a valid **cued-retrieval** observation; never relabel it wholly independent speech.

FSRS does not natively understand our cue chunks, pronunciation or arbitrary search exposure. The app must classify these events correctly before giving it a recall rating. Preserve exposures/repairs even when they do not change the schedule, and keep cue support comparable where possible. Do not silently remove intervening exposures and train on a history that falsely appears to be uninterrupted independent testing. First-version recommendation: use the library's population parameters, automatically updating each target's memory state from normal reviews. Per-user parameter refitting is unnecessary for the first version and must never become a learner task. Check rating mapping, elapsed-time handling, deduplication and repair behavior with ordinary software tests. No learner-facing validation feature follows from these engineering requirements.

## Cue coverage and spoken transfer

- Keep prepared variations, but choose the next cue to exercise under-practiced or weak meaningful chunks, rather than maximize cosmetic variety. Where possible keep cue support comparable across scored attempts; record changes in support/task difficulty.
- For word/frame targets, hide the actual target form when testing retrieval; allow permitted surrounding variations. For fixed expressions, track which important chunks have been retrieved across delayed checks.
- Replace the earlier unvalidated requirement to cover every chunk at *every interval level* with persistent coverage evidence and targeted selection. Coverage is not another independent schedule. Never claim full-phrase readiness when a crucial chunk is always supplied.
- Include different **prepared** situations/dialogue turns in ordinary practice of the same target. Retain enough identifying context and allow valid alternatives; a broad situation cannot demand an arbitrary exact sentence. These are learning exercises, not separate assessment sessions, open-ended role-play or an automatic move to zero hints.
- Keep the explanation concise: natural meaning, when to use the phrase, and an optional breakdown of a troublesome chunk or construction. After help, allow speaking with the model and arrange a separated retrieval opportunity. Persistent failure calls for returning to meaning or fixing a poor cue, not merely accumulating more immediate repetitions. This refines the accepted help route; it does not add a compulsory introduction.
- Treat spoken practice as additional evidence on the same target. Silent practice may strengthen memory; when the learner enables speaking again, ordinary spoken drills resume. Do not claim fluency from silent responses or add a mandatory speaking examination.
- Do not grade speed from cue-to-microphone-tap time: it includes reading, deliberation and device operation. Any future speech timing needs actual voice onset, phrase-length/context adjustment and validation. For now use the learner's retrieval report plus optional pronunciation practice.

## What the learner does

Start a session, attempt the phrase, compare, and choose one result. The app schedules the next encounter. Explanation and extra speaking practice remain available as needed. No experiment setup, control group, special 7/28-day exam, calibration report, algorithm comparison or effectiveness-validation task is part of the product. The earlier proposal for those activities was rejected by the user and is withdrawn.

## Changes from earlier proposals and discussion status

Recommended changes are: explicit three-way recall grading; separate extra practice from failure; FSRS with 0.90 default retention and automatic target-state updates; conditional lapse handling instead of a universal reset; no permanent 30-day cap; persistent chunk coverage instead of per-level coverage gates; explicit silent-memory grading and separate spoken evidence; varied prepared situations during normal practice. No learner-run validation is required.

Already accepted product decisions stay intact. The user has requested a design based on research, but has not yet selected these labels, parameters, scheduler or policy details. Save subsequent acceptance/corrections explicitly before implementation.

## Discussion illustration

The focused answer-card sketch is `/Users/andrei_sorokin2/.codex/visualizations/2026/10/11/01a12a6f-c63e-7d32-a9c9-be0bbc8da9ae/answer-card-research.html`. Its preview selector demonstrates ordinary spoken, silent and model-supported states; it is outside the depicted app and is not a proposed validation control. The sketch holds a selected result so its consequences and Undo can be inspected; the recommended app flow combines the grade tap with advancing. Audio and recording are simulated, and no real scheduler or learning database is running. Syntax/control and state checks are engineering checks of the illustration, not learning-effectiveness evidence. Browser/device rendering has not been verified.
