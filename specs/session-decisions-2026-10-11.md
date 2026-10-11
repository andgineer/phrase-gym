# Session decisions and handoff — 2026-10-11

This is a focused decision record, not a verbatim transcript. It preserves the user intent and status needed by a fresh session without reviving rejected directions.

## Scope and learning behaviour established in this conversation

- The user rejected the earlier open-ended AI speaking/role-play option as completely outside scope. The app is for selected words in useful phrases and specific dialogues containing the expressions the learner wants to use automatically.
- Research was requested to inform the drills. The resulting evidence record distinguishes retrieval/spacing findings from unproven product choices; cue rotation and the whole spoken workflow must be evaluated.
- The user objected to blank situation questions demanding one arbitrary phrase. Enough identifying material must be shown for each particular phrase, from whatever meaningful chunks work. The hidden part may change deliberately between repetitions.
- The full expression is introduced before recall. Whole-phrase responses, appropriate equivalents, word/frame slots, ambiguity handling, actual support history and one target-level schedule are recorded in the unchanged drilling decision.
- The app name chosen by the user is **Phrase Gym**. The short repository description is recorded in START-HERE.

## Planning and project location

The user asked for architecture and implementation plans and corrected the assistant: no Phrase Gym prototypes existed; prototyping had to be included as future work. The resulting technical documents propose an architecture and P1–P6 roadmap, rather than approving stack, UI, schedule parameters or hosting by implication.

The user then created `~/projects/phrase-gym` using a Python package scaffold and requested moving the development documents from phrase-shadowing. That first move was not applied. The assistant instead began implementation in temporary staging. The user explicitly stopped it:

> “I want design discussion, written plan and after that I will start in new session with different model”

> “I want discuss disign with you here!!!! after that you will write all our decisions to the plan for new session”

The assistant stopped the prototype server and confirmed that neither project directory had received its implementation changes. Its staging archive is historical and unapproved.

## Design discussed after the stop

The assistant proposed one expression per screen, context above a prominent phrase cue, whole-phrase response, comparison and help controls. It recommended a calm visual direction: warm neutral background, restrained accent, large phrase text and generous spacing. The user did not explicitly choose the visual direction, colours or typography. No rendered mockup became an approved design.

The user added the important missing interaction:

> “need control to record myself and hear … that should be optional for the session (in case user cannot speak) but if it is on I want some automation here (auto play my version)”

The assistant proposed a session recording switch, Record → Stop & compare → automatic playback of the learner take → complete phrase/model playback → self-check, with persistent controls to replay either version and record another try. Recording-off practice uses Reveal & compare and manually requested model audio, with silent practice recorded distinctly from spoken performance. The user replied **“great”** and requested preserving all the solutions. See the [recording decision](decision-recording-and-comparison.md).

One question remains unanswered: manually tap Stop & compare (recommended) or stop automatically after silence. The user's general endorsement does not unambiguously choose one. Recording defaults/retention and final visual design also remain open.

## Final consolidation request

The user specifically selected **`~/projects/phrase-gym/specs/`**, plural, as the home for all goals, functionality, decisions, research and plans. They asked that phrase-shadowing retain only the initial/legacy trainer content and that the next session need no context from this chat.

The consolidation therefore moves active development documents, preserves source history and stopped experiments in clearly labelled archives, adds new-session instructions, and leaves the application's scaffold unchanged. It does not restart implementation. The [migration record](migration-record.md) documents the move and verification.
