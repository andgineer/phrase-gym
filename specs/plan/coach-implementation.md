# Coach implementation plan

The learner pastes a word or phrase, optionally with its source sentence, or pastes the sentence itself. Coach prepares the card automatically. Start with English, then German with Russian help.

The [preparation decision](../decision-external-preparation-agent.md#preparation-by-provider) fixes the models. Paid subscriptions keep `claude-opus-5-5` and `gpt-6.1-sol` at medium effort: prepare, then at most two review→revise rounds. Gemini free recognizes the input in one `gemini-3.8-flash-low` call, and the program builds basic practice. Token savings come from how Coach calls the strong models. Never move writing or checking to weaker models; branch `coach-plan-trial` shows that this loses card quality.

Steps 1–3 run in order. Step 4 runs only if step 2 misses its token target.

## 1. Add German with Russian help

The learner studies German now. The [language choices](../decision-personal-preferences.md) set German at A1 with **Translation in native language** on, so its meaning layer (cue glosses and short help) is Russian, like the detailed help. This step brings German through the whole loop: Settings, Coach, cards, reference audio, Library and Practice.

Main is English-only today:

- Settings hold one card language and one help language, both typed `Literal["en"]` (`src/phrase_gym/server/contracts.py`).
- Import packages accept only `en` (`src/phrase_gym/server/import_contracts.py`).
- Phrases store no learning language.
- Coach and Library sends carry the page's learning language, which the server accepts only as `en`. Each request snapshots the native language and a translation choice that is always captured as off (`src/phrase_gym/server/coach_requests.py`). Every page sends `en` (`pageLanguage` in `frontend/src/composables/useCoachSubmission.ts`).
- `src/phrase_gym/coach/speech.py` rejects any Piper voice that is not `en-US`.
- `prepare.md`, `card-format.md` and `content-rules.md` say "English" and "B2" throughout.

**Settings and data.**

- `de` becomes a learning language.
- Settings replace the card and help languages with one native language for the profile and a **Translation in native language** choice per learning language. They are seeded from the learner's table: English off and German on.
- The Settings page edits the native language and each language's translation choice. It has no active-language control.
- Settings are one JSON document, so they need no upgrade step. A stored card or help language is ignored, and the next save drops it.
- Each phrase records its learning language. A schema step adds it and marks existing phrases English.
- External packages replace `card_language` and `help_language` with one declared meaning-layer language. It must equal the native language when the learning language's choice is on, and the learning language when it is off: a German package declares `ru`, and an English package declares `en`. A mismatch reports what to change, as today. Bump the package `version` to 3, since older formats are rejected. Update the schema table in `docs/includes/agent-add-phrases.md` in the same commit; its detailed-help row already says native language.

**Separate languages, language buttons and addresses.** Follow [separate learning languages](../decision-personal-preferences.md#separate-learning-languages).

- **Language buttons.** In `frontend/src/App.vue`, replace the Practice start screen's `<p class="eyebrow">English</p>` with one button per supported learning language, at most three, in the same place. Highlight the selected one with `aria-current`. Build them from the existing eyebrow and tab styles: AGENTS.md requires adapting the retained shell, not redesigning it. Study cards show no buttons, as they show no eyebrow today.
- **Addresses.** Navigation is hash-based today (`pageFromHash` and `show` in `frontend/src/composables/useAppNavigation.ts`), so add the language as the first hash segment:

  | Address | Opens |
  |---|---|
  | `#en`, `#de` | Practice in that language |
  | `#de/library`, `#de/coach`, `#de/settings` | The other tabs, keeping the language |
  | `/`, `#library`, `#coach`, `#settings` | English, so old bookmarks keep working |
  | An unknown or unsupported language code | English Practice |

  A hash keeps one served page, so the server needs no fallback routes. Tabs and the brand link keep the current language segment, and a language button switches to `#<lang>`. All of them use `history.replaceState`, as tabs do today, and add no history entries. An open card keeps its language's Practice address.
- **Server scope.** Today's counts, Daily, Extra, phrase-specific Practice, practice facts, the Library search and lessons take the page's learning language, for example as a `language` query parameter. Coach and Library sends take it from the address, and the request snapshot reads the language's translation choice from Settings. Regenerate the OpenAPI types.
- **Data.**
  - Each `practice_log` row records its learning language, through the same schema step as phrases, with existing rows marked English.
  - Allowances count each language's completions separately under the shared limits.
  - Extra targets are keyed by date and language.
  - Each language keeps its own current view, so English and German Daily resume independently.
- **Practice facts.** **Today**, **Total** and **In long-term memory** describe the selected language. **Progress**, the streak and the weekly target count every language. The Coach inbox keeps listing each result with its language.

**Coach judges German.**

- Recognition and the paid prompts receive the learning language, the native language, the translation choice and the stated level, instead of the fixed English and B2 wording.
- Coach writes the meaning layer in the native language when the choice is on and in the learning language when it is off. Detailed help is always in the native language. Today `card-format.md` asks for English detailed help, so English cards' detailed help becomes Russian with this step.
- Reword `content-rules.md` and `card-format.md` to speak of the learning language, the meaning-layer language and the native language. Put German notes in a section used only for German. `content-rules.md` is also part of the external-agent guide, so the guide gets the same rules.
  - **Nouns:** a noun is practised with its article, and Russian help mentions gender and plural when relevant.
  - **Capitalisation is part of the word:** **Morgen** (утро) and **morgen** (завтра) are both valid and are never corrected into each other. A lowercase noun after an article, such as **das auto**, is an obvious typo.
  - **Separable verbs** split in a main clause (**Ich rufe dich an**). A cue therefore uses a frame where the verb stays whole, such as **Ich muss dich morgen __________** for **anrufen**. A sentence gap still covers one contiguous span only.
- Paid-route German cards follow the [word-sense rules](../decision-targeted-drilling.md#words-after-the-phrase-mvp) for beginner German. The cue gives the Russian meaning of a short chunk and asks for the German chunk. Glosses, short help and detailed help are in Russian. Recall hints keep German wording fragments, and the revealed sentence and the answer stay German.
- Gemini free German cards are basic cards. The instruction and inbox reasons use the native-language messages Coach already renders. The fresh reviewer also checks the Russian texts. A bare noun stays as typed, because basic practice claims nothing beyond the wording. A sentence gap works as for English. A separable verb split in its sentence falls back to the partial form of the target.
  - The revealed **Corrected from “…”.** line on a corrected card follows the language settings like the instruction; the server still renders it in English.
  - A partial-form card's instruction asks for the whole word or phrase, such as «Произнесите слово или фразу целиком». The 2026-11-25 blind review found that «Произнесите пропущенное.» can read as "say only the missing letters".

**Reference audio.**

- Add one German Piper voice. The private voice file named by `PHRASE_GYM_PIPER_CONFIG` names one voice per learning language. The locale check compares the voice with the card's learning language (`de-DE` for German) instead of the fixed `en-US`.
- Pick a candidate from Piper's German voices (for example `de_DE-thorsten-medium`). Pin its runtime and hashes as for English, and extend `docs/src/en/piper-voice.md` with the German voice.
- A missing German voice is a service failure with a clear reason, never a language rejection, and English keeps working.
- Automated tests keep the silent fake child.

**Acceptance check: software.** Through the real server and both browser engines:

- the learner presses the German button on the Practice start screen and submits **Tisch** from Coach and from a Library search;
- the card appears in Library and Practice with a Russian instruction or cue, and Reveal shows the German wording with its German reference audio;
- English cards keep working;
- an external German package with Russian help is accepted, and a mismatched one is rejected with what to change;
- the schema upgrade carries an existing database forward, with every phrase marked English;
- **Morgen** and **morgen** are separate submissions;
- a missing German voice gives a retryable service failure;
- opening `#de`, `#de/library` and `#de/coach` directly, as a bookmark would, shows German pages with German highlighted, and a reload keeps them;
- `/`, `#library`, `#coach` and `#settings` open the English pages;
- switching with the language buttons changes the address without adding a history entry, and Start shows that language's due and new counts;
- a send from `#de/coach` or a `#de/library` search creates a German request, and a send from the `#en` pages creates an English one;
- the Library and Daily show only the selected language's cards;
- Settings shows no active-language control.

Server tests on a simulated clock check that:

- finishing English Daily at its limits leaves German's allowance whole;
- Extra in German leaves English's targets unchanged;
- each language's Daily resumes its own view;
- **Today** and **Total** are per language;
- a German-only practice day extends the streak and counts toward the weekly target.

**Acceptance check: real providers.** Fix a held-out German set before editing any prompt, and never use it during prompt work. The proposed set:

| Input | What it tests |
|---|---|
| `Tisch` | Bare noun that needs its article (**der Tisch**) |
| `See` | Bare noun whose gender depends on its sense (**der See**, **die See**) |
| `aufstehen` | Bare separable verb |
| `anrufen — Ich rufe dich morgen an.` | Separable verb split in its sentence |
| `sich Sorgen machen — Mach dir keine Sorgen, das klappt schon.` | Phrase with its sentence, inflected |
| `Krankenhuas` | Obvious typo, corrected to **Krankenhaus** |
| `morgen` | Valid lowercase word, kept and not changed to **Morgen** |
| `das auto` | Capitalisation typo, corrected to **das Auto** |
| `Wie geht's?` | Valid colloquial variant, kept |
| `I am very tired` | English input, rejected with a Russian reason |

Run each input once on Claude (`claude-opus-5-5`), Codex (`gpt-6.1-sol`) and Gemini free under the [measurement rules](#measurement-and-qualification). The fresh blind reviewer judges the German and the Russian, and adds a Russian-help score to the rubric. The run must show:

- zero material errors on every route:
  - a wrong article, gender, case, plural or meaning anywhere is a material error;
  - so is changing a valid form, such as **morgen** to **Morgen**;
  - on Gemini free, a bare noun without its article is not an error;
- accurate, natural Russian help on both paid routes, with every reviewer complaint quoted and resolved;
- the English input rejected on all three routes.

Report each input's outcome, cue, score, elapsed time, reported usage and the model actually launched per route, including paid-route token totals. These are the first German numbers and carry no target.

Rewording the shared rules changes the English prompts. So also rerun the 12 English baseline inputs on both paid routes. Their meaning layer must stay English and their detailed help must be Russian. Their quality must not fall below the 2026-11-25 review, and their token sums become the step 2 baseline.

**Acceptance check: attended listening.** The learner listens to the German reference voice on a few of these cards. Report this separately from the software checks and the semantic review. Automated tests stay silent.

## 2. Send paid prompts directly

The Claude and Codex command templates in `src/phrase_gym/coach/agents/config.py` pass only `"Read your task from {prompt_file} and execute it."`. The model then spends extra rounds reading the file, and every round resends the whole context:

- Claude takes about 3 turns per call, including the structured-output turn.
- Codex reads the file with two shell commands and then answers, so it makes 3 model requests.

On 2026-11-25, means per call were about 22k tokens on main Claude (3.2 calls per request) and about 57k on main Codex. The CLI's own system prompt and tool definitions add about 7k tokens on Claude, and much more on Codex. Coach's prompt is about 6k tokens. The 2026-11-21 benchmark (`.phrase-gym-coach/coach-call-experiment-20261121/metrics.tsv`, transport `inline argument`) ran a Codex call in one model request with about 18.7k input tokens for a 1.8 KB prompt.

Change prompt delivery for Claude and Codex only. Keep models, effort, the prepare/review/revise flow, every prompt template and `content-rules.md` as step 1 left them. Also keep each CLI's tool and permission flags unchanged in this step, so the measurement isolates delivery; step 3 changes them.

- **Claude:** put the rendered prompt as the positional argument after `--`. The variadic `--allowed-tools` and `--disallowed-tools` would otherwise swallow it.
- **Codex:** put the rendered prompt as the positional `PROMPT` of `codex exec`.
- **Never stdin.** `claude -p` returns empty output above a few KB of piped stdin, and `codex exec -` hangs on a non-TTY stdin. Keep child stdin at `/dev/null`.
- **Pass the prompt as its own argv element**, not through template substitution or shell parsing. On POSIX, append it to the argv list. On Windows, quote it by `subprocess.list2cmdline` rules. Prompts contain quotes, newlines, `$`, backticks, non-ASCII text and learner input.
- **Length fallback.** For a call that would exceed these limits, fall back to today's file bootstrap and record which delivery the call used in its attempt trail.

  | Platform | Use direct delivery only when | Why |
  |---|---|---|
  | Windows | The resolved executable is a native `.exe` and the rendered command line is at most 30,000 UTF-16 units | `CreateProcess` accepts at most 32,767 UTF-16 units. An npm `.cmd`/`.bat` shim runs through `cmd.exe`, which allows only 8,191 characters and reinterprets `%`, `^`, `&`, `"` and newlines. |
  | Linux and macOS | The prompt is at most 100 KiB of UTF-8 | Linux allows one argument at most 128 KiB (`MAX_ARG_STRLEN`). |

  For scale, today's prepare prompt is about 23 KB, of which `content-rules.md` is 20 KB and `card-format.md` 2 KB. Review and revise prompts add the draft and findings, and the Claude command also carries the 2 KB JSON schema inline.
- **Gemini free** (`agy`) keeps news-recap’s file bootstrap unless a measurement shows a benefit.

**Acceptance check: software.** Fake-CLI tests cover the following:

- The argv carries the exact prompt, including quotes, newlines, Cyrillic and a leading `#`, and stdin receives nothing.
- Windows rendering round-trips through `list2cmdline` parsing.
- An over-limit prompt and a `.cmd` shim each select the file fallback.
- The Gemini command is unchanged.

**Acceptance check: measurement.** Rerun the main Claude and main Codex arms on the 12 baseline inputs under the [measurement rules](#measurement-and-qualification). The token baseline is step 1's English rerun. For reference, the 2026-11-25 sums in `.phrase-gym-coach/compare-20261125` (`metrics.tsv`, `summary.md`) were 846,541 on Claude and 1,829,415 on Codex, which would give targets of 592,578 and 1,280,590. The target is per route:

- total tokens at most 70% of the baseline sum;
- no fewer full cards and no lower average cue quality than the baseline outputs scored in the same blind batch (2026-11-25: 10 of 12 on both routes, average cue quality 4.8 on Claude and 4.2 on Codex);
- zero material errors, with the German and gibberish inputs still rejected.

Run the German set from step 1 once on both paid routes as well. It must still have zero material errors, and its tokens are reported without a target.

Commit when quality holds and tokens fall. Whether step 4 runs depends on the 70% target. If quality drops, do not ship. Investigate instead: delivery alone should not change content.

## 3. Choose the Coach provider in Settings

This is not implemented anywhere yet, including the trial branch. Follow the [routing decision](../architecture.md#coach-provider-selection).

Add **Coach provider** to Settings with these choices: **Autodetect** (the default), **Codex subscription**, **Claude subscription**, **Gemini subscription** and **Gemini free**. The Gemini subscription choice appears only once its preparation design is qualified. Store the choice in the settings JSON document, so no schema step is needed. Coach reads the choice through the Phrase Gym API when it claims work.

- **Autodetect** tries the Codex subscription, then the Gemini subscription when qualified and available, then the Claude subscription, then Gemini free.
- **A pinned subscription** falls back directly to Gemini free, and the pin stays saved for the next request.
- **CLI flags** `--cli codex`, `--cli claude` and `--cli gemini` override Settings for the Coach process.
- **Availability checks** stay bounded. Read the signed-in CLI's access status first, then use the bounded **Reply OK** probe on a subscription-only model with its exact ID and effort. Cache the result per provider for the session, and invalidate it after an account or configuration change or an actual access failure.

Keep model and effort explicit on every call, and keep news-recap’s free Flash discovery and pacing.

Use the learner's global CLI environment and remove the Coach-only tool and permission restrictions: Claude's `--tools`, `--allowed-tools`, `--disallowed-tools` and `--permission-mode manual`, and Codex's `phrase-gym-coach` permission profile. Run each call from an attempt directory outside the repository, as news-recap does, so no project instructions load. Then rerun the step 2 token measurement. If tokens rise, report the rise to the user with options and a recommendation. Do not keep restrictions on your own.

**Acceptance check.** Fake adapters demonstrate the following:

- Autodetect order and pin persistence;
- status checks or bounded probes, with cache invalidation;
- free model replacement, quota exhaustion and interruption of a review or revision;
- moving to the next permitted route under the same request ID after a service failure;
- an input rejection ending the request;
- exhausting all eligible routes, which gives one retryable service failure in the Coach inbox;
- recovery publishing at most once.

The attempt trail records the requested and actual route, model, effort and reason for any switch, without account identifiers. In the real browser, the selected provider persists across reloads, and a fresh database shows Autodetect.

## 4. Save more tokens only if step 2 misses the target

Run this step only if step 2 kept quality but missed the 70% target. Try one change at a time on top of step 2, in this order. Measure each under the [measurement rules](#measurement-and-qualification) against the same baseline, and judge it by the step 2 criteria. Keep only changes that hold quality, and stop once the target is met. A kept change updates the [preparation decision](../decision-external-preparation-agent.md#preparation-by-provider) in the same commit.

1. **Rules file only for the writing call.** Give `content-rules.md` (20 KB) only to the prepare call. Review and revise keep `card-format.md` and the rules' **Inspect every card before submission** section. If the review then misses problems, give revise the full rules too.
2. **Limit hidden thinking** where the CLI allows it while effort stays explicit. For Claude, use the `MAX_THINKING_TOKENS` environment variable; never set `CLAUDE_CODE_DISABLE_ADAPTIVE_THINKING` alone, because it moves reasoning into stdout. Codex's only control is `model_reasoning_effort`, so measure `low` on the same model.
3. **Fewer review rounds.** Allow one review→revise round instead of two, and always keep the review.

Never move to weaker models.

## Measurement and qualification

- **Inputs.** For English, use the 12 held-out inputs listed in `.phrase-gym-coach/compare-20261125/harness.py`. For German, use step 1's fixed held-out set. Run one request per input per route, so every result pairs with its baseline. Reuse that `harness.py` and `analyze.py`. Write each new run to its own dated folder under `.phrase-gym-coach/`.
- **Tokens.** Total tokens = uncached input + cache read + cache write + output, with reasoning or thinking counted inside output. Codex usage comes from `--json` events, which the harness adds without changing the prompt, model or effort. Claude usage comes from its JSON envelope. Count every retry, and report sums, per-call means, calls per request and elapsed time.
- **Blind review.** A fresh reviewer, in a new session with an exact model ID and effort, judges the outputs blind, with arm identities kept in a separate key. The reviewer must not have written the code under test. Use the rubric of `semantic-review.json`: material error with a quote, cue quality from 1 to 5, help quality and a note. Rescore the baseline outputs in the same blind batch so that both arms use one scale. Inspect every output yourself as well.
- **Reporting.** Report semantic review, software checks, token numbers and attended audio acceptance separately. Every implementation step passes the ordered [repository gate](../../AGENTS.md#done-gate) and CI.
- **Authorization.** Real calls use subscription or free access only, under the [standing authorization](implementation-plan.md#standing-authorization-for-the-local-english-demo), with no API keys and no paid fallback.
- **Other languages.** Keep [later language rollout](implementation-plan.md#5-extend-only-the-content-the-learner-next-needs), such as Serbian, separate from these English and German checks.
