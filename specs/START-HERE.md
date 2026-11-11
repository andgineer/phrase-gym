# Phrase Gym decisions

Build a small adult personal trainer to bring useful expressions from courses and videos into the learner's own speech. Research-informed spoken retrieval, the retained Vue recorder/player, a Python backend, forgiving engagement and the Coach field with its preparation path for typed phrases and words are in place; Coach's Claude preparation still needs a semantic review of held-out outputs and a real end-to-end run, then further languages follow. Freer speaking needs its own activity design first. Specs describe the intended product; code and operational docs describe what currently runs. Existing prototype behavior is not automatically a requirement.

Read in this order:

1. [Research basis and limits](research/README.md): which findings support the learning loop, and what remains an inference.
2. [Content and active-use goal](decision-targeted-drilling.md): useful phrase cards and later word senses.
3. [Observable learning loop](decision-core-trainer.md): retrieval from the first attempt, feedback, honest rating and scheduled return.
4. [Extra practice and adult engagement](decision-transfer-and-engagement.md): repeatable bounded phrase recall and forgiving motivation after the MVP.
5. [Recording and saved replay](decision-recording-and-comparison.md): preserve the working Vue audio behavior.
6. [Preparation and speech](decision-external-preparation-agent.md): external-agent course batches and Coach preparation of typed phrases and words.
7. [Language choices](decision-personal-preferences.md) and [local availability](decision-online-only.md).
8. [Small application architecture](architecture.md): lightweight Phrase Gym, separate Coach, SQLite and stopped-app backup/restore.
9. [Delivery ladder](plan/implementation-plan.md) and [unverified acceptance](open-problems.md).

[Agent instructions](../AGENTS.md) and the [implementation handout](plan/orchestration-handout.md) govern delivery. [README](../README.md) documents the runnable application and launch workflow. Keep decisions and [development operations](development.md) here, user and external-agent guidance in `docs/src/`, and only remaining work in the plan.
