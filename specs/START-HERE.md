# Phrase Gym decisions

Build a small personal trainer around research-informed spoken retrieval, the retained Vue recorder/player and a Python backend. Specs describe the intended product; code and operational docs describe what currently runs. Existing prototype behavior is not automatically a requirement.

Read in this order:

1. [Research basis and limits](research/README.md): which findings support the learning loop, and what remains an inference.
2. [Content and active-use goal](decision-targeted-drilling.md): useful phrases first, word senses next.
3. [Observable learning loop](decision-core-trainer.md): study, later retrieval, feedback, honest rating and scheduled return.
4. [Recording and saved replay](decision-recording-and-comparison.md): preserve the working Vue audio behavior.
5. [Preparation and speech](decision-external-preparation-agent.md): external course batches first; internal echo-words preparation later.
6. [Language choices](decision-personal-preferences.md) and [local availability](decision-online-only.md).
7. [Small application architecture](architecture.md): Python, Vue, SQLite and stopped-app backup/restore.
8. [Delivery ladder](plan/implementation-plan.md) and [unverified acceptance](open-problems.md).

[Agent instructions](../AGENTS.md) and the [implementation handout](plan/orchestration-handout.md) govern delivery. [README](../README.md) distinguishes the runnable prototype from this target. Keep decisions here, operational commands in docs, and only remaining work in the plan.
