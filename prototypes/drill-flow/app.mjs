import * as db from "/shared/storage.mjs";
import { AudioFlow } from "/shared/audio.mjs";
import {
  search,
  selectCue,
  noteCoverage,
  makeSession,
  snapshotAttempt,
  finalize,
  next,
  undo,
  RETRY_MS,
} from "./core.mjs";
import { dictionaries, languageNames, criteria } from "./i18n.mjs";

const pack = await (await fetch("/drill-flow/fixtures.json")).json();
const app = document.querySelector("#app");
const mic =
  '<svg viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="1.8" aria-hidden="true"><rect x="9" y="2" width="6" height="13" rx="3"/><path d="M5 10v2a7 7 0 0 0 14 0v-2M12 19v3M8 22h8"/></svg>';
const stop =
  '<svg viewBox="0 0 24 24" fill="currentColor" aria-hidden="true"><rect x="5" y="5" width="14" height="14" rx="2"/></svg>';
const esc = (value) =>
  String(value ?? "").replace(
    /[&<>"']/g,
    (c) =>
      ({ "&": "&amp;", "<": "&lt;", ">": "&gt;", '"': "&quot;", "'": "&#39;" })[
        c
      ],
  );
const button = (action, label, cls = "", attrs = "") =>
  `<button class="${cls}" data-action="${action}" ${attrs}>${label}</button>`;
let prefs = (await db.get("settings", "preferences")) || {
  id: "preferences",
  native: null,
  learningUI: false,
  lastLanguage: "en",
};
let state = (await db.get("settings", "history")) || {
  id: "history",
  coverage: structuredClone(pack.coverageSeed),
  reviewed: {},
  exposures: [],
};
let sessions = Object.fromEntries(
  (await db.all("sessions")).map((s) => [s.language, s]),
);
let route,
  language,
  query = "",
  lessonFilter = "",
  selectedLesson = "",
  policy = "due",
  showSettings = false;
let error = "",
  message = "",
  lastUndo = JSON.parse(sessionStorage.getItem("pg-undo") || "null"),
  lastTake = null,
  clockOffset = 0,
  lookupRecording = true;
const tabId = sessionStorage.getItem("pg-tab") || crypto.randomUUID();
sessionStorage.setItem("pg-tab", tabId);
const now = () => Date.now() + clockOffset;
const session = () => sessions[language];
const t = (key) =>
  dictionaries[prefs.learningUI ? language : prefs.native || "en"][key] || key;
const activeItem = () => session()?.queue[session().cursor];
const targetFor = (id) => pack.targets.find((target) => target.id === id);
const activeTarget = () => targetFor(activeItem()?.targetId);
const isDrill = () => route?.kind === "session";
const targetHere = () =>
  route?.kind === "phrase" ? targetFor(route.id) : activeTarget();
const busy = () => !!audio.recorder || audio.requesting || autoComparePending;
let returnRoute = JSON.parse(sessionStorage.getItem("pg-return") || "null");
let autoComparePending = false;
let comparisonGeneration = 0;
const updateHistory = async (change) => {
  state = await db.update("settings", "history", (stored) => {
    const current = stored || {
      id: "history",
      coverage: structuredClone(pack.coverageSeed),
      reviewed: {},
      exposures: [],
    };
    change(current);
    return current;
  });
};
const updatePreferences = async (changes) => {
  prefs = await db.update("settings", "preferences", (stored) => ({
    ...prefs,
    ...stored,
    ...changes,
  }));
};
const audio = new AudioFlow((phase, problem) => {
  if (isDrill() && ["permission", "recording"].includes(phase))
    session().phase = phase;
  if (phase === "storage-error") error = t("storageFailed");
  if (problem) error = problem.message;
  render();
});

function parseRoute() {
  const parts = location.pathname.split("/").filter(Boolean);
  if (parts[0] === "learn" && parts[1] && !["en", "sr"].includes(parts[1])) {
    error = `Learning language “${parts[1]}” has no prototype content. Choose English or Serbian.`;
  }
  language = ["en", "sr"].includes(parts[1]) ? parts[1] : prefs.lastLanguage;
  if (!["en", "sr"].includes(language)) language = "en";
  route = { kind: parts[2] || "home", id: parts[3] };
  if (!["home", "session", "phrase", "lesson"].includes(route.kind))
    route.kind = "home";
  const params = new URLSearchParams(location.search);
  query = params.get("q") || "";
  selectedLesson = params.get("lesson") || "";
  if (parts[0] !== "learn" || !["en", "sr"].includes(parts[1]))
    history.replaceState({}, "", `/learn/${language}`);
}
async function persist() {
  try {
    sessionStorage.setItem("pg-return", JSON.stringify(returnRoute));
    sessionStorage.setItem("pg-undo", JSON.stringify(lastUndo));
    const owner = JSON.parse(localStorage.getItem("pg-owner") || "null");
    if (session() && owner?.tabId === tabId)
      await db.put("sessions", session());
  } catch (problem) {
    error = t("storageFailed");
    throw problem;
  }
}
async function expose(target, kind) {
  const entry = { targetId: target.id, kind, at: now() };
  await updateHistory((current) => current.exposures.push(entry));
  if (session() && activeTarget()?.id === target.id && !session().attempt)
    session().help.push(kind);
  await db.event("exposure", entry);
  await persist();
}
function recentExposure(target) {
  return state.exposures
    .filter((e) => e.targetId === target.id && now() - e.at < RETRY_MS)
    .map((e) => e.kind);
}
async function navigate(path, replace = false) {
  const previousPhase = session()?.phase;
  ++comparisonGeneration;
  autoComparePending = false;
  await audio.cancel();
  if (
    session() &&
    ["permission", "recording", "own", "reference"].includes(session().phase)
  )
    session().phase = session().attempt?.finalized
      ? "practice"
      : ["own", "reference"].includes(previousPhase)
        ? "check"
        : "cue";
  if (
    ["permission", "recording"].includes(previousPhase) &&
    !session()?.attempt?.finalized
  ) {
    session().attempt = null;
  }
  await persist();
  history[replace ? "replaceState" : "pushState"]({}, "", path);
  parseRoute();
  error = "";
  message = "";
  await prepare();
  render(true);
}
function homePath() {
  const params = new URLSearchParams();
  if (query) params.set("q", query);
  if (selectedLesson) params.set("lesson", selectedLesson);
  return `/learn/${language}${params.size ? "?" + params : ""}`;
}
async function prepare() {
  const s = session();
  const current = targetHere();
  const takes = current
    ? (await db.all("takes")).filter(
        (take) =>
          take.targetId === current.id &&
          (route.kind === "phrase" || take.sessionId === s?.id),
      )
    : [];
  const replayTakes =
    route.kind === "session" && s?.attempt && !s.attempt.finalized
      ? takes.filter(
          (take) =>
            take.attemptId === s.attempt.id || take.id === s.attempt.takeId,
        )
      : takes;
  lastTake = replayTakes.sort((a, b) => b.created - a.created)[0] || null;
  if (route.kind === "session" && s && !s.finished) {
    const target = activeTarget();
    if (!s.cueId) {
      await updateHistory((current) => {
        const cue = selectCue(target, current.coverage[target.id]);
        s.cueId = cue.id;
        current.coverage[target.id] = noteCoverage(
          target,
          cue,
          current.coverage[target.id],
        );
      });
      await persist();
    }
    if (
      ["permission", "recording", "own", "reference"].includes(s.phase) &&
      !busy()
    ) {
      if (s.attempt?.finalized) s.phase = "practice";
      else if (
        lastTake?.status === "complete" &&
        lastTake.attemptId === s.attempt?.id
      )
        s.phase = "check";
      else {
        s.phase = "cue";
        s.attempt = null;
      }
      error =
        "Interrupted capture/playback. Any recovered bytes remain an incomplete take. Start a fresh response.";
      await persist();
    }
  }
  if (route.kind === "phrase") {
    const target = targetHere();
    if (target?.language === language) await expose(target, "explanation");
  }
  if (prefs.native && route.kind === "home")
    await exposeList(search(pack, language, query), "search-result");
  if (prefs.native && route.kind === "lesson")
    await exposeList(
      pack.targets.filter(
        (target) => target.lesson === route.id && target.language === language,
      ),
      "lesson-list",
    );
}
async function exposeList(targets, kind) {
  if (!targets.length) return;
  const entries = targets.map((target) => ({
    targetId: target.id,
    kind,
    at: now(),
  }));
  await updateHistory((current) => current.exposures.push(...entries));
  await Promise.all(entries.map((entry) => db.event("exposure", entry)));
}
function settings() {
  if (!showSettings) return "";
  return `<section class="settings"><h2>${t("settings")}</h2><label>${t("native")}<select id="native">${Object.entries(
    languageNames,
  )
    .map(
      ([code, name]) =>
        `<option value="${code}" ${prefs.native === code ? "selected" : ""}>${name}</option>`,
    )
    .join(
      "",
    )}</select></label><label class="check"><input type="checkbox" id="ui-mode" ${prefs.learningUI ? "checked" : ""}>${t("learningUI")}</label><a href="/audio-probe/">${t("report")}</a></section>`;
}
function modeSwitch(checked) {
  return `<label class="check"><input type="checkbox" id="recording-mode" ${checked ? "checked" : ""} ${busy() ? "disabled" : ""}>${t("recording")}: ${checked ? t("on") : t("off")}</label>`;
}
function home() {
  const lessons = pack.lessons.filter(
    (l) =>
      l.language === language &&
      l.title.toLowerCase().includes(lessonFilter.toLowerCase()),
  );
  const matches = search(pack, language, query);
  const s = session();
  return `<p class="eyebrow">${t("preview")}</p><h1>Phrase Gym</h1><div class="block"><label for="lesson-filter">${t("lesson")}</label><input id="lesson-filter" placeholder="${t("filterLessons")}" value="${esc(lessonFilter)}" type="search" aria-controls="lessons"><select id="lessons" aria-label="${t("lesson")}" style="margin-top:8px"><option value="">${t("allLessons")}</option>${lessons.map((l) => `<option value="${l.id}" ${selectedLesson === l.id ? "selected" : ""}>${esc(l.title)}</option>`).join("")}</select>${lessonFilter && !lessons.length ? `<p>${t("noMatches")}</p>` : ""}${selectedLesson ? policyOptions() : ""}</div>${modeSwitch(s && !s.finished ? s.recording : lookupRecording)}<div class="row">${s && !s.finished ? button("resume", t("resume"), "primary") : button("start", t("start"), "primary")}${s && !s.finished ? button("finish", t("finish")) : ""}</div><div class="block rule"><label for="phrase-search">${t("findPhrase")}</label><input id="phrase-search" type="search" value="${esc(query)}" placeholder="${t("searchHint")}"><div class="results" id="search-results">${searchResults(matches)}</div></div><details><summary>${t("dialogue")}</summary><div class="stack">${pack.dialogues
    .filter((d) => d.language === language)
    .map((d) => button("dialogue", esc(d.title), "", `data-id="${d.id}"`))
    .join("")}</div></details>${engineering()}`;
}
function searchResults(matches) {
  return query.trim()
    ? matches.length
      ? matches
          .map(
            (target) =>
              `<button data-action="phrase" data-id="${target.id}">${esc(target.text)}<small>${esc(pack.lessons.find((l) => l.id === target.lesson).title)}</small></button>`,
          )
          .join("")
      : `<p class="muted">${t("noMatches")}</p>`
    : "";
}
function policyOptions() {
  return `<fieldset><legend>${t("duePolicy")}</legend><label class="check"><input type="radio" name="policy" value="due" ${policy === "due" ? "checked" : ""}>${t("dueOnly")}</label><label class="check"><input type="radio" name="policy" value="all" ${policy === "all" ? "checked" : ""}>${t("practiceAll")}</label></fieldset>`;
}
function lessonView() {
  const lesson = pack.lessons.find(
    (l) => l.id === route.id && l.language === language,
  );
  if (!lesson) return `<h1>${t("noMatches")}</h1>`;
  return `${button("back", t("back"), "subtle")}<p class="eyebrow">${t("lesson")}</p><h1>${esc(lesson.title)}</h1><div class="results">${pack.targets
    .filter((target) => target.lesson === lesson.id)
    .map(
      (target) =>
        `<button data-action="phrase" data-id="${target.id}">${esc(target.text)}</button>`,
    )
    .join(
      "",
    )}</div>${policyOptions()}${modeSwitch(lookupRecording)}${button("lesson-start", t("lessonDrill"), "primary")}`;
}
function captureControls(recording, supported = false) {
  if (audio.requesting)
    return `<p role="status">${t("permission")}</p>${button("cancel-capture", t("cancel"))}`;
  if (audio.recorder)
    return `<p role="status">${t("capturing")}</p>${button("stop-recording", stop + t("stopRecording"), "primary")}`;
  if (autoComparePending)
    return `<p role="status">${t(session()?.phase === "reference" ? "referencePlayback" : "ownPlayback")}</p>${button("stop-audio", t("stopAudio"))}`;
  if (recording)
    return `<div class="center">${button("record", mic, "mic", `aria-label="${t("startRecording")}"`)}</div>`;
  return supported ? "" : button("reveal", t("reveal"), "primary");
}
function replayControls() {
  const disabled =
    audio.recorder || audio.requesting || autoComparePending ? "disabled" : "";
  return `<div class="row">${lastTake ? button("play-own", "▶ " + t("myRecording"), "", disabled) : ""}${button("play-reference", "▶ " + t("reference"), "", 'aria-label="' + t("reference") + '" ' + disabled)}${button("stop-audio", t("stopAudio"), "subtle")}</div>`;
}
function explanation() {
  const target = targetHere();
  if (!target || target.language !== language)
    return `<h1>${t("noMatches")}</h1>`;
  const fromDrill = returnRoute?.kind === "session";
  const recording = fromDrill ? session().recording : lookupRecording;
  return `${button("back", fromDrill ? t("returnDrill") : t("back"), "subtle")}<p class="eyebrow">${t("explain")}</p><div class="card"><p class="cue" lang="${language}">${esc(target.text)}</p>${modeSwitch(recording)}${captureControls(recording, true)}${replayControls()}<div class="rule"><h2>${t("meaning")}</h2><p lang="${prefs.native}">${esc(target.meaning[prefs.native])}</p><details><summary>${t("breakdown")}</summary><p lang="${language}">${target.chunks.map(esc).join(" / ")}</p><h3>${t("allowed")}</h3><p lang="${language}">${target.allowed.map(esc).join("<br>")}</p></details></div><p class="notice muted">${t("lookupExposure")}</p></div>${button("lesson", esc(pack.lessons.find((l) => l.id === target.lesson).title), "", `data-id="${target.lesson}"`)}`;
}
function dialogueContext(s, item) {
  if (!item.dialogueId) return "";
  const dialogue = pack.dialogues.find((d) => d.id === item.dialogueId);
  let start = item.turn - 1;
  while (start >= 0 && dialogue.turns[start].speaker !== "learner") start--;
  const lines = dialogue.turns
    .slice(start + 1, item.turn)
    .filter((turn) => turn.text);
  return `<p class="eyebrow">${esc(dialogue.title)}</p><div class="status" lang="${language}">${lines.map((line) => `<p>${esc(line.text)}</p>`).join("")}</div>${s.isolated ? button("return-exchange", t("returnExchange")) : button("isolate", t("isolate"), "subtle")}`;
}
function drill() {
  const s = session();
  if (!s) return `${button("home", t("home"))}<h1>${t("empty")}</h1>`;
  if (s.finished) return results();
  const item = activeItem(),
    target = activeTarget(),
    cue = target.cues.find((c) => c.id === s.cueId) || target.cues[0];
  const shown =
    ["check", "practice", "reference"].includes(s.phase) ||
    s.captureSupported ||
    s.hint;
  const content = shown
    ? esc(target.text)
    : target.chunks
        .map((chunk, index) =>
          cue.hidden.includes(index)
            ? '<span class="gap">[…]</span>'
            : esc(chunk),
        )
        .join(" ");
  let actions;
  if (s.phase === "check") {
    const supported = !s.attempt?.eligible;
    actions = `${replayControls()}<p class="notice muted">${supported ? t("support") : s.attempt.modality === "silent-recall" ? t("silent") : t("original")}</p>${s.practiceRequested ? `<p role="status">${t("chooseResult")}</p>` : ""}${supported ? button("supported-continue", t("supportedContinue"), "primary") : ratings()}<div class="row block">${button("practice", t("practice"))}${button("explain", t("explain"))}</div>${gradingCriteria()}`;
  } else if (s.phase === "practice") {
    actions = `<p class="status">${t("savedGrade")} · ${t("practiceGrade")}</p>${captureControls(s.recording, true)}${replayControls()}<div class="row block">${button("continue-practice", s.isolated ? t("returnExchange") : t("continue"), "primary")}${button("explain", t("explain"))}</div>`;
  } else {
    actions = `${captureControls(s.recording)}<div class="row block">${button("explain", t("explain"), "subtle", busy() ? "disabled" : "")}${button("hint", t("showAnswer"), "subtle", busy() ? "disabled" : "")}${button("missed", t("missedBeforeHelp"), "subtle", busy() ? "disabled" : "")}</div>${s.help.length ? `<p class="notice muted">${t("support")}</p>` : ""}`;
  }
  return `<div class="row spread">${button("pause", "‹ " + t("pause"), "subtle")}<small>${Math.min(s.cursor + 1, s.queue.length)} / ${s.queue.length}</small></div><progress value="${s.cursor}" max="${s.queue.length}"></progress>${modeSwitch(s.recording)}${dialogueContext(s, item)}<div class="card"><p class="context" lang="${language}">${esc(cue.context)}</p><p class="cue" lang="${language}">${content}</p>${actions}</div>${engineering()}`;
}
function ratings() {
  return `<div class="ratings">${[1, 2, 3].map((rating, index) => button("grade", t(["notYet", "effort", "recalled"][index]), rating === 3 ? "primary" : "", `data-rating="${rating}"`)).join("")}</div>`;
}
function gradingCriteria() {
  return `<details><summary>${t("counts")}</summary><p>${criteria[prefs.learningUI ? language : prefs.native]}</p></details>`;
}
function results() {
  const s = session(),
    pending = s.repairs.filter((r) => !r.done);
  const grouped = [
    ...new Set(s.grades.filter((g) => !g.undone).map((g) => g.targetId)),
  ];
  return `<p class="eyebrow">${t("preview")}</p><h1>${s.queue.length ? t("sessionDone") : t("empty")}</h1><div class="results">${grouped
    .map((id) => {
      const original = s.grades.find((g) => g.targetId === id && !g.undone);
      return `<p lang="${language}">${esc(targetFor(id).text)}<br><small>${original.result ? t(["notYet", "effort", "recalled"][original.result - 1]) : t("support")}</small></p>`;
    })
    .join(
      "",
    )}</div>${pending.length ? `<p class="status">${t("pendingRetries")}</p>` : ""}<div class="row">${button("home", t("home"), "primary")}${button("finish", t("newSession"))}</div>`;
}
function engineering() {
  return `<details><summary>Prototype diagnostics</summary><p>Fixture due flags and review receipts only; FSRS is implemented in P3. Visual design and review-policy parameters are proposals. Recording starts on; microphone capture starts only when requested. The local keyword index demonstrates concept lookup; it is not an evaluated semantic-search system.</p><div class="row">${button("clock", "Advance probe clock +5 min")}${button("download", t("download"))}${button("export", "Export metadata (no audio)")}${button("reset", t("reset"), "danger")}</div><pre>${esc(JSON.stringify({ fixture: pack.revision, clockOffset, coverage: state.coverage[activeTarget()?.id], repairs: session()?.repairs }, null, 2))}</pre></details>`;
}
function render(focus = false) {
  const ui = prefs.learningUI ? language : prefs.native || "en";
  document.documentElement.lang = ui;
  document.title = `Phrase Gym: ${languageNames[language]}`;
  const header = `<header><a class="brand" href="/learn/${language}" data-action="home">Phrase Gym</a><div class="header-tools"><select id="language" aria-label="${t("language")}">${["en", "sr"].map((code) => `<option value="${code}" ${code === language ? "selected" : ""}>${languageNames[code]}</option>`).join("")}</select>${button("settings", "⚙", "", `aria-label="${t("settings")}"`)}</div></header>`;
  let body;
  if (!prefs.native) {
    body = `<p class="eyebrow">Phrase Gym</p><h1>${t("chooseNative")}</h1><label>${t("native")}<select id="first-native"><option value="">—</option>${Object.entries(
      languageNames,
    )
      .map(([code, name]) => `<option value="${code}">${name}</option>`)
      .join(
        "",
      )}</select></label><label class="check"><input type="checkbox" id="first-ui-mode" ${prefs.learningUI ? "checked" : ""}>${t("learningUI")}</label>${button("onboard", t("save"), "primary")}`;
  } else
    body =
      route.kind === "phrase"
        ? explanation()
        : route.kind === "lesson"
          ? lessonView()
          : route.kind === "session"
            ? drill()
            : home();
  app.innerHTML =
    header +
    `<main>${settings()}${audio.pending && !audio.recorder ? `<div class="error" role="alert">${t("storageFailed")}${button("retry-storage", t("retryStorage"))}</div>` : ""}${error ? `<div class="error" role="alert">${esc(error)}</div>` : ""}${message ? `<p class="status" role="status">${esc(message)}</p>` : ""}${lastUndo?.language === language ? `<div class="status row">${t("savedGrade")}${button("undo", t("undo"))}</div>` : ""}${body}</main><footer><span>P1 · ${pack.revision}</span><a href="/audio-probe/">${t("report")}</a></footer>`;
  if (focus) {
    const h = app.querySelector("h1");
    if (h) {
      h.tabIndex = -1;
      h.focus();
    }
  }
}
async function claim() {
  const owner = JSON.parse(localStorage.getItem("pg-owner") || "null");
  if (owner && owner.tabId !== tabId && Date.now() - owner.at < 30000) {
    if (
      !confirm(
        "A prototype session is open in another tab. Take over and pause that tab?",
      )
    )
      return false;
  }
  localStorage.setItem(
    "pg-owner",
    JSON.stringify({ tabId, language, at: Date.now() }),
  );
  return true;
}
async function startSession(dialogueId = null, lessonOverride = null) {
  if (!(await claim())) return;
  const fresh = await db.get("settings", "history");
  if (fresh) state = fresh;
  const recording = document.querySelector("#recording-mode")?.checked ?? true;
  sessions[language] = makeSession(
    pack,
    language,
    lessonOverride ?? selectedLesson,
    selectedLesson || lessonOverride ? policy : "due",
    state.reviewed,
    now(),
    dialogueId,
  );
  session().recording = recording;
  lastTake = null;
  lastUndo = null;
  await navigate(`/learn/${language}/session`);
}
async function beginAttempt() {
  const s = session();
  state = (await db.get("settings", "history")) || state;
  if (!s.attempt)
    s.attempt = snapshotAttempt(
      s,
      activeItem(),
      activeTarget(),
      now(),
      recentExposure(activeTarget()),
    );
  await persist();
}
async function grade(result, reason = null) {
  if (!session().attempt) await beginAttempt();
  const before = structuredClone(session());
  const s = structuredClone(session());
  const saved = finalize(s, s.attempt, result, now());
  saved.reason = reason;
  let undoState = null;
  if (s.practiceRequested || s.isolated) {
    s.phase = "practice";
  } else {
    next(s, now());
    undoState = { eventId: saved.id, before, language };
  }
  state = await db.commitSessionEvent(
    s,
    { type: "result", at: now(), ...saved },
    state,
  );
  sessions[language] = s;
  lastUndo = undoState;
  await persist();
  await prepare();
  render(true);
}
async function record() {
  ++comparisonGeneration;
  const target = targetHere();
  const s = session();
  const supported =
    route.kind === "phrase" || s?.phase === "practice" || s?.hint;
  if (isDrill() && !supported) await beginAttempt();
  const metadata = {
    targetId: target.id,
    targetRevision: target.revision,
    sessionId: s?.id || null,
    attemptId: !supported ? s?.attempt?.id : null,
    cueId: !supported ? s?.cueId : null,
    support: supported
      ? [
          "full-phrase",
          ...(route.kind === "phrase" ? ["explanation"] : []),
          ...(s?.help || []),
        ]
      : [...(s?.attempt?.support || [])],
    modality: supported ? "supported-practice" : "recorded-recall",
    surface: route.kind,
    language,
  };
  if (isDrill()) s.captureSupported = supported;
  try {
    await audio.start(metadata);
  } catch (problem) {
    if (isDrill()) {
      s.phase = s.attempt?.finalized ? "practice" : "cue";
      if (!s.attempt?.finalized) s.attempt = null;
      await persist();
    }
    throw problem;
  }
  await persist();
  render();
}
async function compare() {
  const generation = ++comparisonGeneration;
  const target = targetHere(),
    s = session(),
    surface = route.kind;
  const current = () =>
    generation === comparisonGeneration &&
    route.kind === surface &&
    targetHere()?.id === target.id &&
    session() === s;
  const take = await audio.stop(true);
  if (!take || !current()) return;
  lastTake = take;
  if (isDrill() && s.attempt && take.attemptId === s.attempt.id)
    s.attempt.takeId = take.id;
  autoComparePending = true;
  if (!take.bytes || take.status !== "complete") {
    autoComparePending = false;
    error =
      "Capture was empty or interrupted. Retry or switch recording off; no recall result was saved.";
    if (isDrill()) {
      session().phase = "cue";
      session().attempt = null;
    }
    await persist();
    render();
    return;
  }
  const supported = take.modality === "supported-practice";
  if (isDrill()) s.phase = "own";
  render();
  try {
    await audio.play(take.blob, "own");
    if (!current()) return;
    if (isDrill()) s.phase = "reference";
    await expose(target, "answer-reveal");
    if (!current()) return;
    render();
    const reference = await audio.reference(target);
    if (!current()) return;
    await audio.play(reference, "reference");
  } catch (problem) {
    if (current())
      error = `${problem.message} ${t("reference")} / ${t("myRecording")}`;
  } finally {
    if (current()) {
      autoComparePending = false;
      if (isDrill())
        s.phase = supported && s.attempt?.finalized ? "practice" : "check";
      await expose(target, "answer-reveal");
      await persist();
      render();
    }
  }
}
async function dispatch(action, el) {
  error = "";
  message = "";
  if (action === "onboard") {
    const code = document.querySelector("#first-native").value;
    if (!code) return;
    await updatePreferences({
      native: code,
      learningUI: document.querySelector("#first-ui-mode").checked,
    });
    await prepare();
    await persist();
    render();
    return;
  }
  if (action === "settings") {
    showSettings = !showSettings;
    render();
    return;
  }
  if (action === "home" || action === "pause") {
    await navigate(homePath());
    return;
  }
  if (action === "start") {
    await startSession();
    return;
  }
  if (action === "dialogue") {
    await startSession(el.dataset.id);
    return;
  }
  if (action === "resume") {
    if (await claim()) {
      sessions[language] = await db.get("sessions", language);
      await navigate(`/learn/${language}/session`);
    }
    return;
  }
  if (action === "finish") {
    if (session()) session().finished = true;
    lastUndo = null;
    await navigate(homePath());
    return;
  }
  if (action === "lesson-start") {
    selectedLesson = route.id;
    await startSession(null, route.id);
    return;
  }
  if (action === "phrase" || action === "explain") {
    returnRoute = { ...route, path: location.pathname + location.search };
    await navigate(
      `/learn/${language}/phrase/${el.dataset.id || activeTarget().id}`,
    );
    return;
  }
  if (action === "lesson") {
    returnRoute = { ...route, path: location.pathname + location.search };
    await navigate(`/learn/${language}/lesson/${el.dataset.id}`);
    return;
  }
  if (action === "back") {
    await navigate(returnRoute?.path || homePath());
    returnRoute = null;
    await persist();
    return;
  }
  if (action === "hint") {
    session().hint = true;
    await expose(activeTarget(), "full-phrase");
    message = t("support");
    await persist();
    render();
    return;
  }
  if (action === "missed") {
    await beginAttempt();
    session().attempt.modality = "self-reported-missed";
    session().practiceRequested = true;
    await grade(1);
    returnRoute = { kind: "session", path: `/learn/${language}/session` };
    await navigate(`/learn/${language}/phrase/${activeTarget().id}`);
    return;
  }
  if (action === "reveal") {
    await beginAttempt();
    session().phase = "check";
    await expose(activeTarget(), "answer-reveal");
    await persist();
    render();
    return;
  }
  if (action === "record") {
    await record();
    return;
  }
  if (action === "stop-recording") {
    await compare();
    return;
  }
  if (action === "cancel-capture") {
    await audio.cancel();
    if (isDrill()) {
      session().phase = "cue";
      session().attempt = null;
    }
    await persist();
    render();
    return;
  }
  if (action === "stop-audio") {
    ++comparisonGeneration;
    autoComparePending = false;
    audio.stopPlayback();
    if (isDrill() && ["own", "reference"].includes(session().phase)) {
      session().phase = session().attempt?.finalized ? "practice" : "check";
      await expose(activeTarget(), "answer-reveal");
      await persist();
    }
    render();
    return;
  }
  if (action === "play-own") {
    if (busy()) return;
    if (lastTake) await audio.play(lastTake.blob, "own");
    render();
    return;
  }
  if (action === "play-reference") {
    if (busy()) return;
    const generation = comparisonGeneration,
      target = targetHere();
    await expose(target, "reference-audio");
    const reference = await audio.reference(target);
    if (
      generation !== comparisonGeneration ||
      targetHere()?.id !== target.id ||
      busy()
    )
      return;
    await audio.play(reference, "reference");
    render();
    return;
  }
  if (action === "grade") {
    await grade(Number(el.dataset.rating));
    return;
  }
  if (action === "supported-continue") {
    await grade(null);
    return;
  }
  if (action === "practice") {
    if (session().attempt?.result) session().phase = "practice";
    else if (!session().attempt?.eligible) {
      session().practiceRequested = true;
      await grade(null);
      return;
    } else session().practiceRequested = true;
    await persist();
    render();
    return;
  }
  if (action === "continue-practice") {
    session().isolated = false;
    next(session(), now());
    await persist();
    await prepare();
    render(true);
    return;
  }
  if (action === "isolate") {
    session().isolated = true;
    render();
    return;
  }
  if (action === "return-exchange") {
    session().isolated = false;
    render();
    return;
  }
  if (action === "undo") {
    if (lastUndo?.language !== language) return;
    ++comparisonGeneration;
    autoComparePending = false;
    await audio.cancel();
    const current = structuredClone(session()),
      previous = lastUndo.before;
    const event = current.grades.find((g) => g.id === lastUndo.eventId);
    undo(current, lastUndo.eventId);
    current.attempt.takeId = event?.takeId;
    current.queue = previous.queue;
    current.cursor = previous.cursor;
    current.completed = previous.completed;
    current.repairs = previous.repairs;
    state = await db.commitSessionEvent(
      current,
      {
        id: crypto.randomUUID(),
        type: "undo",
        at: now(),
        eventId: lastUndo.eventId,
        targetId: event.targetId,
      },
      state,
    );
    sessions[language] = current;
    lastUndo = null;
    await persist();
    await prepare();
    render();
    return;
  }
  if (action === "retry-storage") {
    await audio.retryStorage();
    await persist();
    message = t("savedLocal");
    render();
    return;
  }
  if (action === "clock") {
    clockOffset += RETRY_MS;
    render();
    return;
  }
  if (action === "download") {
    await download();
    return;
  }
  if (action === "export") {
    const payload = {
      fixture: pack.revision,
      capabilities: AudioFlow.capabilities(),
      events: await db.all("events"),
      sessions: await db.all("sessions"),
      takes: (await db.all("takes")).map(({ blob, ...meta }) => meta),
    };
    exportJSON(payload);
    return;
  }
  if (action === "reset") {
    if (!confirm(t("confirmReset"))) return;
    await audio.cancel();
    for (const store of [
      "settings",
      "sessions",
      "takes",
      "events",
      "references",
    ])
      await db.clear(store);
    localStorage.removeItem("pg-owner");
    location.assign(`/learn/${language}`);
  }
}
function exportJSON(payload) {
  const url = URL.createObjectURL(
    new Blob([JSON.stringify(payload, null, 2)], { type: "application/json" }),
  );
  const a = document.createElement("a");
  a.href = url;
  a.download = "phrase-gym-p1-report.json";
  a.click();
  setTimeout(() => URL.revokeObjectURL(url), 1000);
}
async function download() {
  if (!navigator.serviceWorker)
    throw new Error("Offline download requires a secure origin.");
  const registration = await navigator.serviceWorker.ready;
  const cache = await caches.open("phrase-gym-p1-v3");
  for (const target of pack.targets.filter(
    (t) => t.language === language && t.audio.url,
  )) {
    const response = await fetch(target.audio.url);
    if (!response.ok) throw new Error("An asset could not be downloaded.");
    await cache.put(target.audio.url, response);
  }
  if (!registration.active)
    throw new Error("Offline shell not ready; reload once.");
  message = t("downloaded");
  render();
}
let working = 0;
app.addEventListener("click", (e) => {
  const el = e.target.closest("[data-action]");
  if (!el) return;
  e.preventDefault();
  working++;
  document.documentElement.dataset.working = "true";
  dispatch(el.dataset.action, el)
    .catch((problem) => {
      error = problem.message;
      render();
    })
    .finally(() => {
      working--;
      document.documentElement.dataset.working = String(working > 0);
    });
});
app.addEventListener("input", (e) => {
  if (e.target.id === "phrase-search") {
    query = e.target.value;
    document.querySelector("#search-results").innerHTML = searchResults(
      search(pack, language, query),
    );
    history.replaceState({}, "", homePath());
    working++;
    document.documentElement.dataset.working = "true";
    exposeList(search(pack, language, query), "search-result")
      .catch((problem) => {
        error = problem.message;
        render();
      })
      .finally(() => {
        working--;
        document.documentElement.dataset.working = String(working > 0);
      });
  }
  if (e.target.id === "lesson-filter") {
    lessonFilter = e.target.value;
    const select = document.querySelector("#lessons");
    const lessons = pack.lessons.filter(
      (l) =>
        l.language === language &&
        l.title.toLowerCase().includes(lessonFilter.toLowerCase()),
    );
    if (selectedLesson && !lessons.some((l) => l.id === selectedLesson)) {
      selectedLesson = "";
      history.replaceState({}, "", homePath());
    }
    select.innerHTML =
      `<option value="">${t("allLessons")}</option>` +
      lessons
        .map(
          (l) =>
            `<option value="${l.id}" ${selectedLesson === l.id ? "selected" : ""}>${esc(l.title)}</option>`,
        )
        .join("");
  }
});
app.addEventListener("change", async (e) => {
  try {
    if (e.target.id === "language") {
      await updatePreferences({ lastLanguage: e.target.value });
      await navigate(`/learn/${e.target.value}`);
    }
    if (e.target.id === "native") {
      await updatePreferences({ native: e.target.value });
      await persist();
      render();
    }
    if (e.target.id === "ui-mode") {
      await updatePreferences({ learningUI: e.target.checked });
      await persist();
      render();
    }
    if (e.target.id === "lessons") {
      selectedLesson = e.target.value;
      history.replaceState({}, "", homePath());
      render();
    }
    if (e.target.name === "policy") {
      policy = e.target.value;
    }
    if (e.target.id === "recording-mode") {
      if (
        session() &&
        (isDrill() ||
          returnRoute?.kind === "session" ||
          (route.kind === "home" && !session().finished))
      )
        session().recording = e.target.checked;
      else lookupRecording = e.target.checked;
      await db.event("recording-mode", {
        language,
        enabled: e.target.checked,
        sessionId: session()?.id,
      });
      await persist();
      render();
    }
  } catch (problem) {
    error = problem.message;
    render();
  }
});
window.addEventListener("popstate", async () => {
  ++comparisonGeneration;
  autoComparePending = false;
  await audio.cancel();
  parseRoute();
  await prepare();
  render();
});
window.addEventListener("storage", (e) => {
  if (e.key === "pg-owner" && isDrill()) {
    const owner = JSON.parse(e.newValue || "null");
    if (owner && owner.tabId !== tabId) {
      audio.cancel();
      navigate(homePath()).then(() => {
        message =
          "This session was taken over in another tab. It is paused here.";
        render();
      });
    }
  }
});
setInterval(() => {
  const owner = JSON.parse(localStorage.getItem("pg-owner") || "null");
  if (owner?.tabId === tabId && isDrill())
    localStorage.setItem(
      "pg-owner",
      JSON.stringify({ ...owner, at: Date.now() }),
    );
}, 10000);
document.addEventListener("visibilitychange", async () => {
  if (document.hidden && (audio.recorder || audio.requesting)) {
    const take = await audio.cancel();
    if (isDrill()) {
      session().phase = "cue";
      session().attempt = null;
    }
    await db.event("interruption", { takeId: take?.id, kind: "background" });
    await persist();
    error = "Capture interrupted. Recovered audio is retained as incomplete.";
    render();
  }
});
window.addEventListener("pagehide", () => {
  audio.cancel();
});
if ("serviceWorker" in navigator)
  navigator.serviceWorker.register("/sw.js").catch((problem) => {
    error = problem.message;
    render();
  });
parseRoute();
await prepare();
render();
