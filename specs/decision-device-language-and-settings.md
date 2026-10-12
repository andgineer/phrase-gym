# Device-local learning language and simple Settings

Accepted by the user on **2026-10-12** after five P2 review iterations. This decision supersedes server-persisted last-learning-language restoration and the ordered preference-write/cache-reconciliation requirements in earlier P2 records.

The URL owns the current learning language. Explicit /learn/{language} paths, including phrase and lesson bookmarks, take precedence on reload and Back/Forward. Each tab can keep its own explicit language. Native language continues to supply explanations; the saved UI mode chooses native language or that tab's current learning language.

Entering a supported language route remembers that language in browser localStorage, independently of catalog success or Settings visibility. Bare /, /learn and /learn/ routes restore a valid device value after setup; absent, invalid or inaccessible storage shows language choice. This is a best-effort convenience: storage failures do not block library navigation. Tabs share the device value but never rewrite each other's explicit URLs in response to it. A separate browser/device has its own fallback. This value is not progress or learning evidence.

Server Settings stores only native language and UI mode. Navigation makes no preference API writes. The obsolete last_learning_language field and PATCH endpoint are removed; migration 0002 removes that JSON field from existing settings without changing native language, UI mode or content/history. The legacy value is not copied to browsers.

Settings permits one pending save per tab. Disable Save, its editable fields and the Settings toggle until success/failure. Other navigation remains available. Success applies native/UI preferences and closes configuration; failure retains the draft and offers another save. There is no write queue, coalescing, language epoch or merged server/browser fallback. Separate tabs may save native/UI preferences independently; the server accepts each complete save and an existing tab reloads to read the current server value. Live synchronization is outside P2.

Keep essential guarantees: atomic package publication, stable learning-target identity, immutable revisions and reference-exposure retention before display. P3's grading evidence and durable learner recordings remain required. Simplifying this optional convenience does not relax those guarantees.

Add targeted behavioral coverage for invalid package references, media provenance, supported compressed containers, streaming body limits and the simplified preferences. Run the ordered full gate, then one bounded fresh Sol 6.1 review of this accepted design. Do not restart an open-ended general review loop. A further product change requires presenting a concrete proposal to the user.
