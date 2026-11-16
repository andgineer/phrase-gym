# Phone-first interface

Stage plan for [stage 4](implementation-plan.md#4-phone-first-interface). The learner uses Phrase Gym as a home-screen web app on a phone. This stage reshapes the retained Vue client around a phone screen: one navigation bar, Coach as an ordinary page, a practice card whose controls never move, and the basics a home-screen app needs. Scheduling, selection, recording, Coach preparation and stored data do not change; the server gains two small reads and one wording change.

The learner approved the layout on a design canvas: <https://claude.ai/artifact/6qXvuc8FRBB7KhemJnqGX6>. It is private to the learner's claude.ai account; a session signed in as the learner can open it with the Artifact tool's read action. This plan is the authoritative description. The canvas's phrases, counts and facts panel are illustrative sample data.

## Screens

### Every page

- Top row: **Phrase Gym ◌** in the current brand style on the left, linking to the Practice home; the installed app version (for example `v0.2.0`) on the right in small muted text. There is no page-title row: the highlighted tab names the page. Each page keeps a visually hidden `h1` with its name for assistive technology.
- Under it one segmented bar of four equal tabs: **Practice · Library · Coach · Settings**, at least 44 px tall. The current page's tab is raised (card background, light shadow) and carries `aria-current="page"`; the others are plain. Coach looks like every other tab.
- The header, its border, the three nav buttons and the header Coach field are removed.
- At 390 px width nothing scrolls horizontally and side gutters are 16 px. Wider screens show the same column centred at the current reading width.
- Tapping any tab while a card is open leaves the card exactly as the header buttons do today: any recording is finished and kept, uploads flush, the client card closes. The server's regular view remains, so **Start** resumes it.

### Practice home

- The **English** eyebrow, then the Start card, then the existing practice-facts panel with its content unchanged.
- While something is waiting, the Start card shows two large numbers, **due** and **new**: exactly what Start will offer today for the selected lesson, meaning phrases eligible under Daily's selection, capped by today's remaining allowance including any Extra extension, excluding phrases skipped in the current traversal. The lesson selector sits at the card's top right as a compact chip naming the selection (**All lessons**); opening it keeps the existing lesson-field behaviour (list, typing narrows, leaving keeps the selection). Below, one full-width primary **Start** button opens or resumes Daily exactly as the Daily button does now. There is no **Today** label, and Extra is hidden.
- When nothing is waiting, the numbers give way to the existing finished wording (**Nothing waiting today** or **Today's practice is complete**, with **Next due: …** when known). Start stays in place, greyed and disabled. **Extra** appears beside it as the primary button, but only when Extra would find phrases: some eligible due or new phrase in the selection lies beyond today's allowance. When nothing is left at all, only the greyed Start and the message show.
- An empty library says **Add your first phrase with Coach to begin.**, with Coach linking to the Coach page.
- Errors and the Extra retry stay below the card as now.

### Coach page

- One card: the label **Phrase or word**, a wrapping four-row box with 17 px text, the helper line **Add the sentence you met it in after the phrase, if you have it.**, a full-width **Send** button disabled while the box is empty, then the existing confirmations (**Saved for Coach** / **Already waiting for Coach** / **Already in your Library**) and errors.
- Below the card, one muted line: **Coach prepares a card later. It appears in Library under Added with Coach.**
- What Coach receives does not change. Enter sends, and line breaks become spaces before sending, so the request text keeps today's single-line shape and the frozen Coach prompts need no new measurement.
- No list of waiting requests: the browser does not track preparation. Failure notices stay in the practice facts.

### Library

- The search field comes first, full width with a search icon. Below it, the lesson field as the same chip while not searching, or **Searching all lessons.** while searching. Results are unchanged.
- A search that finds nothing shows **Not in your Library yet.** and one full-width button **Send “…” to Coach** with the typed text, a plus icon and the subline **Coach prepares a card for Added with Coach.** It submits the trimmed search text through the same Coach request and shows the same confirmation in its place. When a non-empty search has results, a quiet link after them, **Not it? Send “…” to Coach**, does the same.
- Phrase details keep their current layout inside the new shell.

### Practice card, both sides

- Under the bar, a progress row: today's position as now (**3 / 20 due** or **… new**) with a thin progress bar. Extra cards prefix **Extra ·**; phrase-specific cards show **Practice**. On the back a **⋯** button (accessible name **More actions**) at the row's right end opens a small menu whose **Delete phrase…** opens the existing delete confirmation. The front reserves the same space.
- Inside the card, the control rows come first, then a divider, then the text. Nothing above the controls changes height, so a hint, the recording history, the longer explanation or a status message only pushes content below the controls.
- Row 1 holds equal 48 px buttons: three on the front, four on the back. Row 2 holds the microphone. Both rows occupy the same positions on both sides.
- The microphone is a full-width button, 64 px tall with a 16 px radius, filled green, showing only a 30 px microphone icon when idle. While recording it turns red and shows a stop square with **Recording · tap to stop**. While waiting for microphone access, or during playback (when it stops playback), it shows the stop square with matching short text. Accessible names stay as today. Capture state no longer appears as a separate line.
- Front row 1: **Hint** (bulb icon) and **New cue** (circular-arrow icon) in one tinted style, pale green fill with green text, then **Reveal** (eye icon) filled green. Reveal replaces Show. New cue keeps its slot, disabled when no other cue exists, so Reveal never moves.
- Front text: the cue situation at 18 px, the partner line in italics, then **Hint: …** once a hint is shown.
- Back row 1: **Ref**, **Mine**, **Compare**, then the icon-only history button (history icon and chevron, accessible name **Recording history**, `aria-expanded`). Mine and Compare keep their slots, disabled until a take exists.
- Recording history is collapsed on each new card view. Open, its framed list appears directly under the microphone with today's content: newest first, each row the recording date and a **Mine** play button, **More** for older pages, **No saved recordings yet.** when empty.
- Back text: the phrase heading (Georgia serif, 30 px), the short explanation, the **Used for** line, then **More explanation** as a text link, as today. The longer explanation opens below the link, so the link itself stays put.
- Messages (capture and save errors, pending uploads with **Retry save**, practice errors with **Retry**) appear between the controls and the text.
- On the back, the ratings (**Again / Hard / Good / Easy** with return spans above the labels) and **Skip** below them are docked at the bottom of the screen; the content between the controls and the ratings scrolls.
- No **Stop** button: tapping Practice, or any tab, returns to the Practice home with the leave behaviour Stop had. Card screens have no **English** eyebrow.

### Settings

- Existing content inside the new shell.

### Home-screen app

- A web manifest: name and short name **Phrase Gym**, start URL `/`, display `standalone`, background and theme colour `#f7f4ec`, PNG icons at 192 and 512 px (one maskable), and a **Coach** shortcut to `/#coach`. Android offers it on a long press; iOS ignores shortcuts.
- `index.html`: the manifest link, a 180 px `apple-touch-icon`, `apple-mobile-web-app-capable` and its title, and `viewport-fit=cover`. It currently references `/icon.svg`, which does not exist; add the icon or remove the reference.
- The shell pads with the safe-area insets so content clears the notch and the home indicator in standalone mode.
- Every text input uses at least 16 px so iOS Safari does not zoom on focus.
- `#coach`, `#library` and `#settings` select the page on load, and tab changes replace the hash without adding history entries, so a reload stays on the page.
- No service worker or offline cache: the app needs its server under the [online-only decision](../decision-online-only.md).
- Dark mode: every new element gets the existing dark palette.

## Server changes

1. **Today's waiting counts.** Due and new counts that Start will offer for a lesson (none selected means all lessons), and whether Extra would find anything. Compute them in a focused module from the same pieces Daily selection uses: the due and new phrase queries in `practice_selection.py`, `daily_progress` allowances, and the current traversal's skipped IDs. The screen must never promise a card that Daily will not open, and the client must not duplicate selection rules. Expose them through a small read endpoint with a response model (suggested: `GET /api/practice/today?lesson_id=`). The client refreshes them where the entry already refreshes counts: page return, after practice, lesson change and learning-day change.
2. **Finished wording.** Drop the mode name: **Today's practice is complete.** replaces **Today's Daily practice is complete.** The Extra wording stays.
3. **Version.** The health response gains the installed package version from `phrase_gym.__about__`, with a response model so the generated client type includes it. FastAPI's own `version` uses the same value instead of `1.0.0`. The client reads it once at startup.
4. **Manifest type.** Serve the manifest as `application/manifest+json`, registering the type if Starlette's guess differs.

Regenerate the OpenAPI document and client types.

## Spec and documentation updates

Make each update in the same commit as the code it describes.

- [Architecture](../architecture.md), first paragraph: the brand row and tab bar replace the compact header as the established shell.
- [Core trainer](../decision-core-trainer.md): under phone-sized controls, add the card rules (controls above the text in fixed positions on both sides, a full-width microphone in the same place on both sides, Hint / New cue / Reveal, docked ratings). Replace **Show** with **Reveal** throughout. State that the entry button opening Daily is labelled **Start**; specs keep the mode name Daily.
- [Practice and engagement](../decision-transfer-and-engagement.md#practice-entry-and-todays-facts): rewrite the entry paragraph for the Start card, the waiting counts, the Start and Extra states, the lesson chip, the tab bar, the empty library pointing to the Coach page, leaving a card through the tabs instead of Stop, and the progress line text.
- [External preparation](../decision-external-preparation-agent.md#coach-page-typed-phrases-and-words): the Coach page replaces the header field (retitle the section and update its links); sending from a Library search with no match; opening Coach leaves a card and Start resumes it; Enter sends and line breaks become spaces.
- [Recording and comparison](../decision-recording-and-comparison.md): the grouping bullet takes the new back-side rows, the history position, the microphone's shared position on both sides and capture state inside the microphone.
- [Implementation plan](implementation-plan.md): the Coach stage's learner outcome names the Coach page. When this stage is accepted, delete it and this plan.
- [README](../../README.md) and the [user guide](../../docs/src/en/index.md): Start, Reveal, the Coach page, the tab bar, no Stop.

## Implementation notes

- Keep handwritten files under 250 lines. Extract the shell (brand row, tab bar, page selection) from `App.vue`. Split `StudyCard.vue` into front controls, back controls and text blocks. Split `SavedHistory.vue` so the card back owns the toggle in row 1 and a panel component renders the list under the microphone; phrase details keep using both together. A card-menu component owns **⋯**. A Coach page component replaces `CoachField.vue`; a Start panel replaces the buttons in `PracticeEntry.vue`.
- Reuse the existing composables. The practice reducer stays pure, and generated API types remain the client contract.
- Keep existing accessible names where the control survives: **Record a take**, **Stop recording**, **Recording history**, **Phrase or word for Coach**.
- The menu closes on Escape and on an outside tap and never opens a native dialog.

## Checkpoints

Each checkpoint is one coherent deliverable under the [handout](orchestration-handout.md), with the full gate and CI.

2. **Practice home and card:** the waiting-counts read, the Start and Extra states, the finished wording, the front and back card layout, the microphone, Reveal, the history placement, **⋯** Delete, the docked ratings and removal of Stop, with their spec and documentation updates.
3. **Home-screen app:** manifest, icons, safe areas, input sizes, Apple meta tags, the Coach shortcut and the dark-mode pass.

## Tests

- Update the existing browser specs for the changed controls: Daily to Start, Show to Reveal (including `helpers.ts`), the header Coach field to the Coach tab, the header buttons to tabs, Stop to the Practice tab (`practice-recovery.spec.ts`, `extra-package.ts`) and the moved status lines.
- Add a browser spec at 390 × 844 in both engines against the real server and built client:
  - the microphone's bounding box is identical on the front and back of the same card;
  - opening Hint, Recording history and More explanation leaves every control's bounding box unchanged;
  - Practice, Library, Coach, Settings and both card sides have no horizontal overflow;
  - Start alone while phrases wait; greyed Start with Extra once the allowance is used and phrases remain; greyed Start without Extra when nothing remains;
  - the Coach page submits and shows each of the three confirmations; a Library search with no match sends;
  - the shown version equals the server's;
  - `/#coach` opens the Coach page.
- Python: waiting counts (allowance cap, Extra extension, lesson filter, skipped IDs, zero limits), Extra availability, the finished wording, the health version and the manifest content type.
- Client unit tests for new pure logic, such as the Start and Extra decision.
- Browser contexts stay muted and the host stays silent, as AGENTS.md requires.

## Acceptance

Software acceptance is the full gate and CI on each pushed checkpoint. The learner separately checks, attended, on the iPhone: adding to the home screen, opening in standalone mode, spacing around the notch and home indicator, reaching the microphone, Reveal and the ratings with one thumb, and the keyboard on the Coach page. Report device acceptance separately; it does not block the software checkpoints.
