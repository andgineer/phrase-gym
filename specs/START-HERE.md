# Phrase Gym decisions

Build a small adult personal trainer to bring useful expressions from courses and videos into the learner's own speech. Start with research-informed spoken retrieval, the retained Vue recorder/player and a Python backend; follow with freer speaking and sustainable practice. Specs describe the intended product; code and operational docs describe what currently runs. Existing prototype behavior is not automatically a requirement.

Read in this order:

1. [Research basis and limits](research/README.md): which findings support the learning loop, and what remains an inference.
2. [Content and active-use goal](decision-targeted-drilling.md): useful phrase cards and later word senses.
3. [Observable learning loop](decision-core-trainer.md): study, later retrieval, feedback, honest rating and scheduled return.
4. [Transfer and adult engagement](decision-transfer-and-engagement.md): supported-to-unscripted speaking, real opportunities and forgiving motivation after the MVP.
5. [Recording and saved replay](decision-recording-and-comparison.md): preserve the working Vue audio behavior.
6. [Preparation and speech](decision-external-preparation-agent.md): external course batches first; internal echo-words preparation later.
7. [Language choices](decision-personal-preferences.md) and [local availability](decision-online-only.md).
8. [Small application architecture](architecture.md): Python, Vue, SQLite and stopped-app backup/restore.
9. [Delivery ladder](plan/implementation-plan.md) and [unverified acceptance](open-problems.md).

[Agent instructions](../AGENTS.md) and the [implementation handout](plan/orchestration-handout.md) govern delivery. [README](../README.md) distinguishes the runnable prototype from this target. Keep decisions here, operational commands in docs, and only remaining work in the plan.
