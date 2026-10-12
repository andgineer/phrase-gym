// Disposable P1 policy. No production scheduler or FSRS intervals are simulated.
export const RETRY_MS = 5 * 60 * 1000;
export const MAX_RETRIES = 2;

export function search(pack, language, query) {
  const words = query.trim().toLocaleLowerCase().split(/\s+/).filter(Boolean);
  if (!words.length) return [];
  return pack.targets.filter(
    (t) =>
      t.language === language &&
      words.every((w) =>
        [t.text, t.purpose, ...t.search, ...Object.values(t.meaning)]
          .join(" ")
          .toLocaleLowerCase()
          .includes(w),
      ),
  );
}

export function selectCue(target, coverage = {}) {
  const counts = coverage.hiddenCounts || target.chunks.map(() => 0);
  return [...target.cues].sort((a, b) => {
    const score = (c) =>
      c.hidden.reduce((n, i) => n + counts[i], 0) / c.hidden.length;
    return (
      score(a) - score(b) ||
      Number(a.id === coverage.lastCue) - Number(b.id === coverage.lastCue)
    );
  })[0];
}

export function noteCoverage(target, cue, coverage = {}) {
  const hiddenCounts = [
    ...(coverage.hiddenCounts || target.chunks.map(() => 0)),
  ];
  cue.hidden.forEach((i) => hiddenCounts[i]++);
  return { hiddenCounts, lastCue: cue.id };
}

export function makeSession(
  pack,
  language,
  lesson,
  policy,
  reviewed,
  now,
  dialogueId = null,
) {
  const dialogue = pack.dialogues.find(
    (d) => d.id === dialogueId && d.language === language,
  );
  let queue;
  if (dialogue) {
    queue = dialogue.turns.flatMap((turn, index) =>
      turn.target
        ? [
            {
              targetId: turn.target,
              turn: index,
              dialogueId,
              retry: false,
              early:
                !pack.targets.find((t) => t.id === turn.target).due ||
                reviewed[turn.target]?.result >= 2,
            },
          ]
        : [],
    );
  } else {
    queue = pack.targets
      .filter(
        (t) =>
          t.language === language &&
          (!lesson || t.lesson === lesson) &&
          (policy === "all" || (t.due && !(reviewed[t.id]?.result >= 2))),
      )
      .map((t) => ({
        targetId: t.id,
        retry: false,
        early: !t.due || reviewed[t.id]?.result >= 2,
      }));
  }
  return {
    id: crypto.randomUUID(),
    language,
    queue,
    cursor: 0,
    created: now,
    recording: false,
    phase: "cue",
    help: [],
    attempt: null,
    grades: [],
    repairs: [],
    completed: [],
    practiceRequested: false,
    dialogueId,
    finished: queue.length === 0,
  };
}

export function snapshotAttempt(
  session,
  item,
  target,
  now,
  recentExposure = [],
) {
  const support = [...new Set([...session.help, ...recentExposure])];
  return {
    id: crypto.randomUUID(),
    groupId: `${session.id}:${target.id}`,
    targetId: target.id,
    targetRevision: target.revision,
    cueId: session.cueId,
    created: now,
    modality: session.recording ? "recorded" : "silent-recall",
    support,
    eligible: support.length === 0 && target.kind !== "word-use",
    retry: item.retry,
    early: !!item.early,
    observed: true,
    result: null,
  };
}

export function finalize(session, attempt, result, now) {
  if (attempt.finalized)
    throw new Error("The original attempt already has a result.");
  const previous = session.grades.find(
    (g) => g.targetId === attempt.targetId && g.scheduleUpdated && !g.undone,
  );
  const scheduleUpdated =
    attempt.eligible &&
    !previous &&
    !attempt.retry &&
    !attempt.early &&
    result !== null;
  const event = {
    ...attempt,
    result,
    finalized: now,
    scheduleUpdated,
    undone: false,
  };
  session.grades.push(event);
  attempt.result = result;
  attempt.finalized = now;
  if (result === 1 || (result === null && attempt.support.length)) {
    const existing = session.repairs.find(
      (r) => r.targetId === attempt.targetId,
    );
    if (!existing)
      session.repairs.push({
        targetId: attempt.targetId,
        earliest: now + RETRY_MS,
        afterCount: session.completed.length,
        attempts: 0,
        done: false,
        deferred: false,
      });
    else {
      existing.earliest = now + RETRY_MS;
      existing.afterCount = session.completed.length;
    }
  }
  const repair = session.repairs.find((r) => r.targetId === attempt.targetId);
  if (attempt.retry && repair && result >= 2) repair.done = true;
  return event;
}

export function next(session, now) {
  const current = session.queue[session.cursor];
  if (current) session.completed.push(current.targetId);
  session.cursor++;
  const ready = session.repairs.find(
    (r) =>
      !r.done &&
      !r.deferred &&
      r.attempts < MAX_RETRIES &&
      now >= r.earliest &&
      new Set(
        session.completed.slice(r.afterCount).filter((id) => id !== r.targetId),
      ).size >= 2,
  );
  if (ready) {
    ready.attempts++;
    session.queue.splice(session.cursor, 0, {
      targetId: ready.targetId,
      retry: true,
    });
  }
  session.help = [];
  session.attempt = null;
  session.cueId = null;
  session.phase = "cue";
  session.captureSupported = false;
  session.hint = false;
  session.practiceRequested = false;
  session.finished = session.cursor >= session.queue.length;
  return session;
}

export function undo(session, eventId) {
  const grade = session.grades.find((g) => g.id === eventId);
  if (!grade || grade.undone) return false;
  grade.undone = true; // Keep the event and its takes; correction is explicit.
  session.repairs = session.repairs.filter(
    (r) => r.targetId !== grade.targetId,
  );
  session.cursor = Math.max(0, session.cursor - 1);
  session.completed.pop();
  session.finished = false;
  session.phase = "check";
  session.attempt = {
    ...grade,
    id: crypto.randomUUID(),
    result: null,
    finalized: undefined,
  };
  session.cueId = grade.cueId;
  session.help = [...grade.support];
  session.practiceRequested = false;
  return true;
}
