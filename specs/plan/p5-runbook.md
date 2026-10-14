# P5 preparation and source intake runbook

Use disposable data for evaluation; preserve `.data/vault-test` progress and recordings.
Preparation is an operator workflow. Serving practice never opens a provider connection.
The configured worker runs separately, one leased job at a time. No automatic paid escalation.

## Configure and enqueue

Create an ignored/private JSON configuration outside the served client:

```json
{
  "provider": {"keys_file": "/private/path/provider.env", "wait_seconds": 60},
  "speech": {
    "timeout_seconds": 60,
    "voices": {
      "en": [{"engine": "edge", "voice": "en-US-AriaNeural", "locale": "en-US", "accent": "US"}],
      "de": [{"engine": "edge", "voice": "de-DE-KatjaNeural", "locale": "de-DE", "accent": "DE"}],
      "sr": [{"engine": "edge", "voice": "sr-RS-SophieNeural", "locale": "sr-RS", "accent": "RS"}]
    }
  }
}
```

Use explicitly matching locale/accent fallbacks. For Piper supply `engine: "piper"`,
`voice: "en_US-lessac-medium"` or `"de_DE-thorsten-medium"`, `model_path`, and both
`model_sha256` and `config_sha256`. Its configuration must be beside the model as
`<model_path>.json`, and its actual language code must match `locale`. The pinned
model/config URLs and checksums are recorded in the [audio reference](../reference/echo-words-audio.md).
Piper runs in a bounded subprocess; a timeout kills it before fallback. Do not use
`serbski_institut` for Serbian. Serbian Latin becomes Cyrillic for speech input only;
proper names, foreign words and exceptional digraph boundaries still require listening review.

```sh
uv run phrase-gym --data-dir /private/tmp/phrase-gym-intake prepare providers /private/path/preparation.json
uv run phrase-gym --data-dir /private/tmp/phrase-gym-intake prepare enqueue source.json /private/path/preparation.json
uv run phrase-gym --data-dir /private/tmp/phrase-gym-intake prepare work /private/path/preparation.json --limit 1
uv run phrase-gym --data-dir /private/tmp/phrase-gym-intake prepare inspect JOB_UUID
```

`SourcePackage` accepts stable namespace/source IDs, selected meaning key/expression/kind,
complete source text, purpose, explicit missing or supplied context, neighbouring competing
expressions, learning/native languages, optional verified reference IDs, lessons and complete
dialogues. See the [public fixture sources](evidence/p5-fixtures/sources-v2.json) for executable
examples. Context changes in prepared cues remain ordinary practice; no transfer-test quota.
The provider cannot rewrite source text or target identity. Existing actor/course media stays first.
The worker synthesizes only missing complete phrase/partner-turn audio; partial-cue audio is not
generated and cannot leak an answer. Supplied cue audio remains governed by the importer checks.

The API accepts `POST /api/v1/preparation`, job status/retry, and durable handoffs. Set
`PHRASE_GYM_PREPARATION_CONFIG` when serving to bind intake to the same explicit worker
configuration. Without it, intake receipts are durable but unconfigured jobs cannot be run by
an unrelated configured worker. Existing published drills continue normally.

## Review, correct and publish

`awaiting_review` is withheld, never learner-ready. Inspect every raw generation, model
advisory review, exact visible mask, native help, concepts, equivalent/frame boundaries,
complete dialogue bindings and supplied/generated audio. Model approval is advisory.
The fresh semantic report does not attest human listening or approve these samples.

For operator corrections, save the corrected `PreparationDraft` to a local file and run:

```sh
uv run phrase-gym --data-dir /private/tmp/phrase-gym-intake prepare revise JOB_UUID corrected-draft.json --reviewer "Operator name" --notes "Concrete corrections"
uv run phrase-gym --data-dir /private/tmp/phrase-gym-intake prepare inspect JOB_UUID
```

Raw provider answers and the previous draft remain retained. Corrected results invalidate old
review hashes; the earlier advisory review is explicitly marked as applying to the original.
Approve only after actual initial human content review and listening. A `HumanReview` file
contains exact `input_hash` and `draft_hash` from inspect, reviewer, notes, an explicit cue-ID
list for **every** source phrase (empty means supported-study-only), and true confirmations
for help, meaning/search/equivalents and audio review. Do not use test-only approvals on real data.

```sh
uv run phrase-gym --data-dir /private/tmp/phrase-gym-intake prepare publish JOB_UUID human-review.json
```

Publication updates content/job state in one transaction, is idempotent, and refuses a source
revision changed during preparation. No target schedule, take or earlier session manifest is
rewritten. New native-language help merges with earlier reviewed help. Missing reliable cues
keep supported lookup available and retrieval preparation incomplete. Multiple cues must hide
different material with comparable required-chunk counts so the current selector reaches them.
One reliable variant is supported without manufacturing a quota.

## Failure and restart

Jobs retain input, hashes, raw answers, progress checkpoints and structured error codes.
Leases are fenced by tokens and expiration; each checkpoint renews the live lease. Restart
reclaims expired work. Bounded automatic retries stop after three attempts; manual retry is explicit:

```sh
uv run phrase-gym --data-dir /private/tmp/phrase-gym-intake prepare retry JOB_UUID
uv run phrase-gym --data-dir /private/tmp/phrase-gym-intake prepare retry JOB_UUID --regenerate
```

Ordinary retry reuses completed model/audio stages; malformed model answers need `--regenerate`,
which archives previous results before requesting new material. A changed provider/voice/prompt
configuration requires a new enqueue (new input identity). Changing the selected meaning requires
a new source ID, not reuse of another target's schedule. No provider failure grades forgetting.

## Existing 44-phrase local pack

Determine its actual namespace from the existing import package/operator records. Export is
read-only to content/progress/audio; no assumptions about target numbering or meaning are made.

```sh
uv run phrase-gym --data-dir .data/vault-test export-existing ACTUAL_NAMESPACE ru /private/tmp/vault-preparation.json
uv run phrase-gym --data-dir .data/vault-test prepare enqueue /private/tmp/vault-preparation.json /private/path/preparation.json
uv run phrase-gym --data-dir .data/vault-test prepare work /private/path/preparation.json --limit 1
```

This exports stable identities, exact texts, preserved reference IDs, lessons and the latest
complete dialogues with their exact turn/audio bindings. Review the
source and add explicit source context/competing expressions before enqueue if useful. Existing
published material remains unchanged through generation/review; publish only after operator
review. Earlier dialogue revisions stay retained; current complete dialogue bindings are exported
for reviewed regeneration. For a new regeneration increment `source_revision` deliberately.
The implementation session did not alter this private pack or spend provider budget on its text.

## Read-only vault adapter

A `VaultManifest` lives outside the vault. Each selection binds an explicit relative note to a
stable external `source_id`, `meaning_key`, `recall_expression`, purpose and either
`source_audio_name` (the existing lesson format) or a stable Markdown `block_id`.
The actual existing format is parsed from front-matter `id`, `## фразы` numbered phrases,
indented Obsidian audio embeds/situations, and `## диалоги` speaker turns with optional bold
phrase text and voice comments. Reorder/rename a note by updating only its external note path;
keep source IDs. Identical surfaces with different meanings retain distinct mappings.
Missing/ambiguous attachment/block matches fail instead of fuzzy matching.

Each supplied asset needs exact text/language/role/origin/source/performer or engine provenance
and a relative attachment path/MIME type. `include_dialogues: true` preserves complete parsed
exchanges; only exact phrase/attachment matches become target turns. The optional `assets`
map supplies verified dialogue clips. Other missing partner media is prepared by the worker.
Provide explicit stable dialogue IDs via `dialogues` when their identity must survive title/order edits.

```sh
uv run phrase-gym --data-dir /private/tmp/phrase-gym-intake export-vault /selected/vault/root manifest.json
uv run phrase-gym --data-dir /private/tmp/phrase-gym-intake export-vault /selected/vault/root manifest.json --upload-assets --output /private/tmp/vault-source.json
```

Dry-run reports selected/valid entries, duplicate surfaces, missing assets and ambiguous mappings;
and immediately reports create/update/unchanged source mappings or changed-objective conflicts;
ordinary reviewed publication additionally reports prepared-content changes. Source paths and
symlinks must stay inside the selected root. Output must stay outside it. No watcher, download,
course enrolment or source mutation exists. Original bytes are uploaded unchanged.

## Manual echo-words contract demonstration

```sh
uv run python scripts/demo_handoff.py /private/tmp/phrase-gym-envelope.json
uv run phrase-gym --data-dir /private/tmp/phrase-gym-intake prepare receive /private/tmp/phrase-gym-envelope.json /private/path/preparation.json
uv run phrase-gym --data-dir /private/tmp/phrase-gym-intake prepare receive /private/tmp/phrase-gym-envelope.json /private/path/preparation.json
```

The public demonstration writes one portable selected-sense envelope with a fixed export UUID.
Both receives acknowledge the same durable preparation ID, including receiver restart. Changing
the sense/content under that UUID is a conflict. Selected sense IDs must match phrase meaning keys;
the selected meaning is explicitly carried to preparation. Acknowledgement means received,
not ready. Uploaded media IDs are exact-text/language/role checked at publication; unrestricted
URL fetching is absent. Use queued/received/preparing/ready/failed states in a future sender.

The real echo-words selection-time outbox, destination UI/action and unavailable-receiver retry
are **external sender rollout work** requiring separate source-write authorization. This session
kept echo-words/Anki/vault sources read-only; it did not dual-send, migrate or switch existing output.

## Publication fences and chunk evidence after review round 1

The intake snapshot covers every target, lesson and dialogue the package can publish, including
absent identities. Lesson snapshots include title/language, ordered membership and member target
revisions. Dialogue snapshots include the latest complete payload, revision and target bindings.
A concurrent reviewed lesson/dialogue edit or create blocks the pending package instead of
restoring old content. Jobs with the earlier target-only snapshot are withheld; enqueue an
explicit new source revision rather than silently rebasing their original approval.

Generated/operator-corrected chunk IDs are reconciled before the review hash is produced.
Exact text, required-material status, frame-slot values and occurrence distinguish material;
all retained target revisions and history IDs participate. Verified unchanged material keeps
its durable ID even when the provider renames or swaps labels. Changed material gets a distinct
ID; historically ambiguous IDs remain retained but are not reused for current coverage.
Inspect `chunk_id_mapping` alongside the normalized draft/package. Raw provider answers remain
unchanged. Publication refuses any identity normalization that would silently alter approved
content. Earlier session manifests and late events keep their original material IDs and evidence.

A truncated JSON fence is a normal retained `draft_conformance_failure`; use explicit regeneration
to request new candidates. The vault adapter rejects a runtime data directory inside the selected
vault before opening SQLite or creating media/staging directories, including resolved symlink,
case and default-directory aliases. Both runtime/export paths also compare existing ancestor
inode identity. Keep both runtime storage and export output outside the read-only source.
