import * as db from "/shared/storage.mjs";
import { AudioFlow, referenceRecord } from "/shared/audio.mjs";
const $ = (id) => document.getElementById(id);
const pack = await (await fetch("/drill-flow/fixtures.json")).json();
const capabilities = AudioFlow.capabilities();
$("capabilities").textContent = JSON.stringify(capabilities, null, 2);
for (const target of pack.targets) {
  const option = new Option(`${target.language} · ${target.text}`, target.id);
  $("target").add(option);
}
const target = () => pack.targets.find((t) => t.id === $("target").value);
const status = (text) => {
  $("status").textContent = text;
};
let lastTake, originalCue, inspectURL;
let pendingComparison = false;
let playbackGeneration = 0;
const flow = new AudioFlow((phase, error) => {
  status(
    error?.message ||
      {
        permission: "Waiting for microphone permission",
        recording: "Recording…",
        captured: "Take captured",
        empty: "Empty capture; no recall grade",
        own: "Playing my recording",
        reference: "Playing reference",
        "storage-error": "Local storage failed. Keep this tab open and retry.",
      }[phase] ||
      phase,
  );
  $("retry").hidden = !flow.pending;
  controls();
});
function controls() {
  const active = !!flow.recorder;
  const waiting = !!flow.requesting;
  const locked = active || waiting || pendingComparison;
  $("record").hidden = !$("mode").checked || locked;
  $("stop-recording").hidden = !active;
  $("cancel").hidden = !waiting;
  $("reveal").hidden = $("mode").checked;
  $("mode").disabled = locked;
  $("target").disabled = locked;
  $("visible").disabled = locked;
  $("reference").disabled = locked;
  $("own").disabled = !lastTake || locked;
  $("import").disabled = locked;
  $("download").disabled = locked;
}
async function update() {
  ++playbackGeneration;
  await flow.cancel();
  lastTake = null;
  $("own").disabled = true;
  $("inspect").hidden = true;
  if (inspectURL) URL.revokeObjectURL(inspectURL);
  const t = target();
  originalCue = t.chunks
    .map((c, i) => (t.cues[0].hidden.includes(i) ? "[…]" : c))
    .join(" ");
  $("phrase").textContent = $("visible").checked ? t.text : originalCue;
  const imported = await referenceRecord(t);
  $("provenance").textContent = imported
    ? `${imported.origin}: ${imported.source} · ${imported.language} · ${imported.filename}`
    : `${t.audio.origin}: ${t.audio.voice || "not prepared"} · ${t.audio.locale}`;
  controls();
}
async function refresh() {
  const takes = await db.all("takes");
  $("takes").replaceChildren();
  const pre = document.createElement("pre");
  pre.textContent = JSON.stringify(
    takes.map(({ blob, ...metadata }) => metadata),
    null,
    2,
  );
  $("takes").append(pre);
}
async function run(operation) {
  try {
    await operation();
  } catch (error) {
    status(error.message);
  } finally {
    controls();
  }
}
$("mode").onchange = () => {
  controls();
  status(
    $("mode").checked
      ? "Recording on. Start with the microphone."
      : "Recording off. No microphone permission is needed.",
  );
};
$("target").onchange = () => run(update);
$("visible").onchange = () => run(update);
$("record").onclick = () =>
  run(async () => {
    ++playbackGeneration;
    $("inspect").pause();
    await flow.start({
      targetId: target().id,
      targetRevision: target().revision,
      language: target().language,
      sessionId: "audio-probe",
      attemptId: crypto.randomUUID(),
      cueId: target().cues[0].id,
      support: $("visible").checked ? ["full-phrase"] : [],
      surface: "audio-probe",
      modality: "recorded-probe",
    });
  });
$("cancel").onclick = () =>
  run(async () => {
    await flow.cancel();
    status("Microphone request cancelled. A late stream will be stopped.");
  });
$("stop-recording").onclick = () =>
  run(async () => {
    const generation = ++playbackGeneration;
    const current = () => generation === playbackGeneration;
    const take = await flow.stop(true);
    if (!take || !current()) return;
    lastTake = take;
    pendingComparison = true;
    controls();
    try {
      if (!take.bytes || take.status !== "complete") {
        status("Capture empty/interrupted. Retry or turn recording off.");
        return;
      }
      $("phrase").textContent = $("visible").checked
        ? target().text
        : originalCue;
      await flow.play(take.blob, "own");
      if (!current()) return;
      $("phrase").textContent = target().text;
      const reference = await flow.reference(target());
      if (!current()) return;
      await flow.play(reference, "reference");
      if (!current()) return;
      status(
        take.saved
          ? "Comparison finished. Take retained locally."
          : "Comparison finished. Take is NOT saved; retry local storage.",
      );
    } catch (error) {
      if (current())
        status(`${error.message} Use the replay controls to recover.`);
    } finally {
      if (current()) pendingComparison = false;
      await refresh();
    }
  });
$("reveal").onclick = () => {
  $("phrase").textContent = target().text;
  status("Silent comparison. Reference audio is manual. Memory evidence only.");
};
$("own").onclick = () =>
  run(async () => {
    $("inspect").pause();
    await flow.play(lastTake.blob, "own");
  });
$("reference").onclick = () =>
  run(async () => {
    if (flow.recorder || flow.requesting) return;
    const generation = ++playbackGeneration;
    $("inspect").pause();
    const reference = await flow.reference(target());
    if (generation !== playbackGeneration || flow.recorder || flow.requesting)
      return;
    await flow.play(reference, "reference");
  });
$("stop").onclick = () => {
  ++playbackGeneration;
  pendingComparison = false;
  flow.stopPlayback();
  $("inspect").pause();
  status("Playback stopped. Use replay to continue.");
  controls();
};
$("retry").onclick = () =>
  run(async () => {
    await flow.retryStorage();
    $("retry").hidden = true;
    status("Take retained locally.");
    await refresh();
  });
$("refresh").onclick = () =>
  run(async () => {
    await refresh();
    const takes = await db.all("takes");
    const take = takes.at(-1);
    if (take) {
      lastTake = take;
      if (inspectURL) URL.revokeObjectURL(inspectURL);
      inspectURL = URL.createObjectURL(take.blob);
      $("inspect").src = inspectURL;
      $("inspect").hidden = false;
    }
  });
$("inspect").onplay = () => {
  if (flow.recorder) {
    $("inspect").pause();
    return;
  }
  flow.stopPlayback();
};
$("import").onclick = () =>
  run(async () => {
    const file = $("asset").files[0],
      source = $("source").value.trim();
    if (!file || !source || !$("matches").checked)
      throw new Error(
        "Choose a matching clip, check its wording/language and provide provenance.",
      );
    $("inspect").pause();
    flow.stopPlayback();
    await db.put("references", {
      id: `${target().id}:${$("origin").value}`,
      targetId: target().id,
      blob: file,
      mimeType: file.type,
      filename: file.name,
      source,
      origin: $("origin").value,
      language: target().language,
      at: Date.now(),
      targetRevision: target().revision,
      speechInput: target().text,
    });
    await update();
    status(
      "Original reference retained locally; generated fallback is not used.",
    );
  });
$("delete-takes").onclick = () =>
  run(async () => {
    if (
      !confirm(
        "Delete all disposable local test takes from both P1 prototypes?",
      )
    )
      return;
    await flow.cancel();
    $("inspect").pause();
    await db.clear("takes");
    lastTake = null;
    $("inspect").hidden = true;
    await refresh();
    status("Disposable takes deleted by explicit request.");
  });
$("download").onclick = () =>
  run(async () => {
    await navigator.serviceWorker.ready;
    const cache = await caches.open("phrase-gym-p1-v3");
    for (const t of pack.targets.filter((t) => t.audio.url)) {
      const response = await fetch(t.audio.url);
      if (!response.ok) throw new Error("Prepared clip missing.");
      await cache.put(t.audio.url, response);
    }
    status(
      "Prepared shell and English assets downloaded. Imported clips are already local. Disconnect and replay to verify.",
    );
  });
const scenarios = [
  "Generated reference: gesture, replay, stop and seek",
  "Imported actor/course clip with checked provenance",
  "Recording on: real microphone and detected MIME",
  "Automatic learner playback before reveal/reference",
  "Recording off: no microphone and manual reference",
  "Full phrase visible throughout supported capture",
  "Permission ignored then cancelled",
  "Permission denied and recording-off recovery",
  "Background / foreground interruption",
  "Lock screen / incoming interruption",
  "Speaker / headphones / disconnect",
  "Reload: saved and incomplete bytes preserved",
  "Downloaded-session playback disconnected",
  "Missing reference recovery",
  "Quota/write failure: no false Saved state",
  "Restart: every retry take remains distinct",
];
const report = (await db.get("settings", "probe-report")) || {
  id: "probe-report",
  device: "",
  notes: "",
  results: {},
};
$("device").value = report.device;
$("notes").value = report.notes;
for (const scenario of scenarios) {
  const label = document.createElement("label");
  label.textContent = scenario;
  const select = document.createElement("select");
  select.dataset.scenario = scenario;
  for (const result of ["untested", "pass", "fail", "limited"])
    select.add(new Option(result, result));
  select.value = report.results[scenario] || "untested";
  label.append(select);
  $("scenarios").append(label);
}
async function saveReport() {
  report.device = $("device").value;
  report.notes = $("notes").value;
  report.results = Object.fromEntries(
    [...$("scenarios").querySelectorAll("select")].map((s) => [
      s.dataset.scenario,
      s.value,
    ]),
  );
  await db.put("settings", report);
}
$("scenarios").onchange = () => run(saveReport);
$("device").onchange = () => run(saveReport);
$("notes").onchange = () => run(saveReport);
$("export").onclick = () =>
  run(async () => {
    await saveReport();
    const payload = {
      ...report,
      at: new Date().toISOString(),
      capabilities,
      fixture: pack.revision,
      takes: (await db.all("takes")).map(({ blob, ...metadata }) => metadata),
      events: await db.all("events"),
    };
    const url = URL.createObjectURL(
      new Blob([JSON.stringify(payload, null, 2)], {
        type: "application/json",
      }),
    );
    const a = document.createElement("a");
    a.href = url;
    a.download = "phrase-gym-device-report.json";
    a.click();
    setTimeout(() => URL.revokeObjectURL(url), 1000);
  });
function interrupted() {
  if (flow.recorder || flow.requesting)
    run(async () => {
      ++playbackGeneration;
      const take = await flow.cancel();
      await db.event("probe-interruption", { takeId: take?.id });
      status("Interrupted capture. Recoverable bytes retained as incomplete.");
      await refresh();
    });
}
window.addEventListener("pagehide", () => {
  ++playbackGeneration;
  flow.cancel();
});
document.addEventListener("visibilitychange", () => {
  if (document.hidden) interrupted();
});
if ("serviceWorker" in navigator)
  await navigator.serviceWorker.register("/sw.js");
await update();
await refresh();

$("inspect-reference").onclick = () =>
  run(async () => {
    if (flow.recorder || flow.requesting)
      throw new Error("Finish capture before playback.");
    flow.stopPlayback();
    $("inspect").pause();
    const generation = ++playbackGeneration;
    const blob = await flow.reference(target());
    if (generation !== playbackGeneration) return;
    if (inspectURL) URL.revokeObjectURL(inspectURL);
    inspectURL = URL.createObjectURL(blob);
    $("inspect").src = inspectURL;
    $("inspect").hidden = false;
    status(
      "Reference loaded. Use the audio controls for seek, replay and stop.",
    );
  });
