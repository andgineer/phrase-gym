import test from "node:test";
import assert from "node:assert/strict";
import { readFileSync } from "node:fs";
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
} from "../drill-flow/core.mjs";
const pack = JSON.parse(
  readFileSync(new URL("../drill-flow/fixtures.json", import.meta.url)),
);
const target = (id) => pack.targets.find((t) => t.id === id);
const fresh = () => makeSession(pack, "en", null, "due", {}, 1000);
function attempt(s, now = 1000) {
  s.cueId = target(s.queue[s.cursor].targetId).cues[0].id;
  return snapshotAttempt(
    s,
    s.queue[s.cursor],
    target(s.queue[s.cursor].targetId),
    now,
  );
}

test("fixture contracts, rejected candidates and word objectives", () => {
  assert.equal(pack.targets.length, 12);
  assert.equal(pack.dialogues.length, 2);
  assert.equal(new Set(pack.targets.map((t) => t.id)).size, 12);
  for (const t of pack.targets) {
    assert.equal(t.text, t.chunks.join(" "));
    assert.ok(t.purpose && t.allowed.length);
    assert.ok(
      pack.lessons.some((l) => l.id === t.lesson && l.language === t.language),
    );
    for (const c of t.cues) {
      assert.ok(c.hidden.length && c.hidden.length < t.chunks.length);
      assert.ok(c.rationale && c.competitors.length);
    }
  }
  assert.ok(pack.rejectedCues.some((c) => c.id === "leak"));
  assert.ok(target("en-look").cues.every((c) => c.hidden.includes(1)));
});
test("lookup searches concept-only synonyms and scopes languages", () => {
  assert.ok(search(pack, "en", "greetings").length >= 3);
  assert.ok(
    search(pack, "en", "greetings").every((t) => !t.text.includes("greetings")),
  );
  assert.ok(search(pack, "sr", "greetings").every((t) => t.language === "sr"));
  assert.deepEqual(search(pack, "en", "no-such-phrase"), []);
});
test("cue coverage favours an important previously supplied chunk and rotates stored cues", () => {
  const t = target("en-day");
  const first = selectCue(t, pack.coverageSeed[t.id]);
  assert.deepEqual(first.hidden, [0]);
  let coverage = noteCoverage(t, first, pack.coverageSeed[t.id]);
  coverage = noteCoverage(t, first, coverage);
  coverage = noteCoverage(t, first, coverage);
  assert.equal(selectCue(t, coverage).id, t.cues[1].id);
  assert.equal(selectCue(target("en-touch"), {}).id, "en-touch-cue-1");
});
test("due-only / all are explicit; new items drill directly without fabricated grades", () => {
  const s = fresh();
  assert.equal(s.grades.length, 0);
  assert.equal(s.phase, "cue");
  assert.ok(!s.queue.some((i) => i.targetId === "en-use"));
  assert.ok(
    makeSession(pack, "en", "requests", "all", {}, 1000).queue.some(
      (i) => i.targetId === "en-use",
    ),
  );
  assert.ok(
    !makeSession(
      pack,
      "en",
      null,
      "due",
      { "en-day": { result: 3 } },
      1000,
    ).queue.some((i) => i.targetId === "en-day"),
  );
});
test("help before attempt prevents success; help after attempt cannot rewrite evidence", () => {
  const s = fresh();
  s.help = ["explanation"];
  const supported = attempt(s);
  assert.equal(supported.eligible, false);
  assert.equal(finalize(s, supported, null, 1001).scheduleUpdated, false);
  assert.throws(() => finalize(s, supported, 3, 1002));
  const other = fresh();
  const original = attempt(other);
  other.help.push("explanation");
  assert.equal(original.eligible, true);
  assert.deepEqual(original.support, []);
  const used = target("en-use"),
    use = snapshotAttempt(other, { retry: false }, used, 1000);
  assert.equal(use.eligible, false);
});
test("one eligible result per target/group; every retry has its own preserved result", () => {
  const s = fresh(),
    first = attempt(s);
  const original = finalize(s, first, 1, 1000);
  const second = {
    ...first,
    id: crypto.randomUUID(),
    result: null,
    finalized: undefined,
    retry: true,
  };
  assert.equal(finalize(s, second, 3, 1000 + RETRY_MS).scheduleUpdated, false);
  assert.equal(original.result, 1);
  assert.equal(s.grades.length, 2);
  assert.equal(s.repairs[0].done, true);
});
test("repair needs elapsed time AND two distinct intervening targets; caps retries", () => {
  const s = fresh();
  finalize(s, attempt(s), 1, 1000);
  next(s, 1000 + RETRY_MS);
  assert.equal(s.queue[s.cursor].retry, false);
  next(s, 1000 + RETRY_MS);
  assert.equal(s.queue[s.cursor].retry, false);
  next(s, 1000 + RETRY_MS);
  assert.equal(s.queue[s.cursor].retry, true);
  assert.equal(s.repairs[0].attempts, 1);
  const tooSoon = fresh();
  finalize(tooSoon, attempt(tooSoon), 1, 1000);
  next(tooSoon, 2000);
  next(tooSoon, 2000);
  next(tooSoon, 2000);
  assert.equal(tooSoon.queue[tooSoon.cursor].retry, false);
  s.repairs[0].attempts = 2;
  s.repairs[0].done = false;
  next(s, 1000 + RETRY_MS * 3);
  assert.equal(s.queue[s.cursor]?.retry, false);
});
test("undo retains the old result as a corrected event; dialogue turns share target IDs", () => {
  const s = fresh(),
    a = attempt(s),
    e = finalize(s, a, 2, 1000);
  next(s, 1001);
  assert.equal(undo(s, e.id), true);
  assert.equal(e.undone, true);
  assert.equal(s.phase, "check");
  assert.equal(s.attempt.result, null);
  const d = makeSession(pack, "sr", null, "due", {}, 1000, "cafe");
  assert.deepEqual(
    d.queue.map((i) => i.targetId),
    ["sr-coffee", "sr-bill"],
  );
  assert.ok(d.queue.every((i) => i.dialogueId === "cafe"));
});

test("failed targets remain due later; early voluntary practice never promotes", () => {
  assert.ok(
    makeSession(
      pack,
      "en",
      null,
      "due",
      { "en-day": { result: 1 } },
      1000,
    ).queue.some((i) => i.targetId === "en-day"),
  );
  const s = makeSession(
    pack,
    "en",
    "rapport",
    "all",
    { "en-day": { result: 3 } },
    1000,
  );
  const a = attempt(s);
  assert.equal(a.early, true);
  assert.equal(finalize(s, a, 3, 1001).scheduleUpdated, false);
});

test("Dialogue voluntary practice preserves an earlier due result", () => {
  const s = makeSession(
    pack,
    "en",
    null,
    "due",
    { "en-day": { result: 3 } },
    1000,
    "break",
  );
  const index = s.queue.findIndex((item) => item.targetId === "en-day");
  s.cursor = index;
  const a = attempt(s);
  assert.equal(a.early, true);
  assert.equal(finalize(s, a, 1, 1001).scheduleUpdated, false);
});

test("Cue situations do not supply the hidden chunk verbatim", () => {
  const normalize = (text) =>
    text
      .toLocaleLowerCase()
      .replace(/[^\p{L}\p{N}\s]/gu, "")
      .replace(/\s+/g, " ")
      .trim();
  for (const t of pack.targets.filter((t) => t.kind !== "word-use")) {
    for (const c of t.cues)
      for (const i of c.hidden) {
        assert.ok(
          !normalize(c.context).includes(normalize(t.chunks[i])),
          `${c.id} supplies ${t.chunks[i]}`,
        );
      }
  }
});
