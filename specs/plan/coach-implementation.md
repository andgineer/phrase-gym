# Coach implementation plan

The learner pastes a word or phrase, optionally with its source sentence, or pastes the sentence itself. Coach prepares English and German cards automatically, German with Russian help.

The [preparation decision](../decision-external-preparation-agent.md#preparation-by-provider) fixes the models. Paid subscriptions keep `claude-opus-5-5` and `gpt-6.1-sol` at medium effort: prepare, then at most two review→revise rounds. Gemini free recognizes the input in one `gemini-3.8-flash-low` call, and the program builds basic practice. Token savings come from how Coach calls the strong models. Never move writing or checking to weaker models; branch `coach-plan-trial` shows that this loses card quality.

Steps 1 and 2 run in order. Step 3 runs only if step 1 misses its token target.

## 1. Send paid prompts directly

The Claude and Codex command templates in `src/phrase_gym/coach/agents/config.py` pass only `"Read your task from {prompt_file} and execute it."`. The model then spends extra rounds reading the file, and every round resends the whole context:

- Claude takes about 3 turns per call, including the structured-output turn.
- Codex reads the file with two shell commands and then answers, so it makes 3 model requests.

On 2026-11-25, means per call were about 22k tokens on main Claude (3.2 calls per request) and about 57k on main Codex. The CLI's own system prompt and tool definitions add about 7k tokens on Claude, and much more on Codex. Coach's prompt is about 6k tokens. The 2026-11-21 benchmark (`.phrase-gym-coach/coach-call-experiment-20261121/metrics.tsv`, transport `inline argument`) ran a Codex call in one model request with about 18.7k input tokens for a 1.8 KB prompt.

Change prompt delivery for Claude and Codex only. Keep models, effort, the prepare/review/revise flow, every prompt template and the shared rules unchanged. Also keep each CLI's tool and permission flags unchanged in this step, so the measurement isolates delivery; step 2 changes them.

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

**Acceptance check: measurement.** Rerun the main Claude and main Codex arms on the 12 baseline inputs under the [measurement rules](#measurement-and-qualification). The token baseline is the 2026-11-26 English rerun with the language-neutral rules in `.phrase-gym-coach/german-step-20261126` (`metrics.tsv`, `summary.md`), whose summed totals per route are the baseline sums. The target is per route:

- total tokens at most 70% of the baseline sum;
- no fewer full cards and no lower average cue quality than the baseline outputs scored in the same blind batch (the 2026-11-25 main flow had 10 of 12 on both routes, average cue quality 4.8 on Claude and 4.2 on Codex; the German-step blind review rescored those outputs beside the rerun);
- zero material errors, with the German and gibberish inputs still rejected.

Run the held-out German set (`.phrase-gym-coach/german-step-20261126/held-out-set.json`) once on both paid routes as well. It must still have zero material errors, and its tokens are reported without a target.

Commit when quality holds and tokens fall. Whether step 3 runs depends on the 70% target. If quality drops, do not ship. Investigate instead: delivery alone should not change content.

## 2. Choose the Coach provider in Settings

This is not implemented anywhere yet, including the trial branch. Follow the [routing decision](../architecture.md#coach-provider-selection).

Add **Coach provider** to Settings with these choices: **Autodetect** (the default), **Codex subscription**, **Claude subscription**, **Gemini subscription** and **Gemini free**. The Gemini subscription choice appears only once its preparation design is qualified. Store the choice in the settings JSON document, so no schema step is needed. Coach reads the choice through the Phrase Gym API when it claims work.

- **Autodetect** tries the Codex subscription, then the Gemini subscription when qualified and available, then the Claude subscription, then Gemini free.
- **A pinned subscription** falls back directly to Gemini free, and the pin stays saved for the next request.
- **CLI flags** `--cli codex`, `--cli claude` and `--cli gemini` override Settings for the Coach process.
- **Availability checks** stay bounded. Read the signed-in CLI's access status first, then use the bounded **Reply OK** probe on a subscription-only model with its exact ID and effort. Cache the result per provider for the session, and invalidate it after an account or configuration change or an actual access failure.

Keep model and effort explicit on every call, and keep news-recap’s free Flash discovery and pacing.

Use the learner's global CLI environment and remove the Coach-only tool and permission restrictions: Claude's `--tools`, `--allowed-tools`, `--disallowed-tools` and `--permission-mode manual`, and Codex's `phrase-gym-coach` permission profile. Run each call from an attempt directory outside the repository, as news-recap does, so no project instructions load. Then rerun the step 1 token measurement. If tokens rise, report the rise to the user with options and a recommendation. Do not keep restrictions on your own.

**Acceptance check.** Fake adapters demonstrate the following:

- Autodetect order and pin persistence;
- status checks or bounded probes, with cache invalidation;
- free model replacement, quota exhaustion and interruption of a review or revision;
- moving to the next permitted route under the same request ID after a service failure;
- an input rejection ending the request;
- exhausting all eligible routes, which gives one retryable service failure in the Coach inbox;
- recovery publishing at most once.

The attempt trail records the requested and actual route, model, effort and reason for any switch, without account identifiers. In the real browser, the selected provider persists across reloads, and a fresh database shows Autodetect.

## 3. Save more tokens only if step 1 misses the target

Run this step only if step 1 kept quality but missed the 70% target. Try one change at a time on top of step 1, in this order. Measure each under the [measurement rules](#measurement-and-qualification) against the same baseline, and judge it by the step 1 criteria. Keep only changes that hold quality, and stop once the target is met. A kept change updates the [preparation decision](../decision-external-preparation-agent.md#preparation-by-provider) in the same commit.

1. **Rules file only for the writing call.** Give `content-rules.md` (20 KB) only to the prepare call. Review and revise keep `card-format.md` and the rules' **Inspect every card before submission** section. If the review then misses problems, give revise the full rules too.
2. **Limit hidden thinking** where the CLI allows it while effort stays explicit. For Claude, use the `MAX_THINKING_TOKENS` environment variable; never set `CLAUDE_CODE_DISABLE_ADAPTIVE_THINKING` alone, because it moves reasoning into stdout. Codex's only control is `model_reasoning_effort`, so measure `low` on the same model.
3. **Fewer review rounds.** Allow one review→revise round instead of two, and always keep the review.

Never move to weaker models.

## Measurement and qualification

- **Inputs.** For English, use the 12 held-out inputs listed in `.phrase-gym-coach/compare-20261125/harness.py`. For German, use the held-out set fixed on 2026-11-26 in `.phrase-gym-coach/german-step-20261126/held-out-set.json`; never use it while editing prompts. Run one request per input per route, so every result pairs with its baseline. Reuse `harness.py`, `gemini_harness.py` and `analyze.py` from `.phrase-gym-coach/german-step-20261126`, which run the current Coach interface. Write each new run to its own dated folder under `.phrase-gym-coach/`.
- **Tokens.** Total tokens = uncached input + cache read + cache write + output, with reasoning or thinking counted inside output. Codex usage comes from `--json` events, which the harness adds without changing the prompt, model or effort. Claude usage comes from its JSON envelope. Count every retry, and report sums, per-call means, calls per request and elapsed time.
- **Blind review.** A fresh reviewer, in a new session with an exact model ID and effort, judges the outputs blind, with arm identities kept in a separate key. The reviewer must not have written the code under test. Use the rubric of `semantic-review.json`: material error with a quote, cue quality from 1 to 5, help quality and a note. Rescore the baseline outputs in the same blind batch so that both arms use one scale. Inspect every output yourself as well.
- **Reporting.** Report semantic review, software checks, token numbers and attended audio acceptance separately. Every implementation step passes the ordered [repository gate](../../AGENTS.md#done-gate) and CI.
- **Authorization.** Real calls use subscription or free access only, under the [standing authorization](implementation-plan.md#standing-authorization-for-the-local-english-demo), with no API keys and no paid fallback.
- **Other languages.** Keep [later language rollout](implementation-plan.md#5-extend-only-the-content-the-learner-next-needs), such as Serbian, separate from these English and German checks.
