const { chromium } = require("playwright");
const assert = require("node:assert/strict");
const path = require("node:path");
const fs = require("node:fs");
const origin = process.env.PHRASE_GYM_TEST_ORIGIN || "http://127.0.0.1:8871";
const results = [];
async function idle(page) {
  await page.waitForFunction(
    () => document.documentElement.dataset.working !== "true",
  );
}
async function tap(page, selector) {
  await page.locator(selector).click();
  await idle(page);
}
async function read(page, store) {
  return page.evaluate(
    async (store) =>
      (await (await import("/shared/storage.mjs")).all(store)).map(
        ({ blob, ...meta }) => ({ ...meta, blobBytes: blob?.size }),
      ),
    store,
  );
}
async function session(page) {
  return (await read(page, "sessions")).find((s) => s.language === "en");
}
async function history(page) {
  return (await read(page, "settings")).find((s) => s.id === "history");
}
async function onboard(page) {
  await page.goto(origin + "/learn/en");
  await page.locator("#first-native").selectOption("en");
  await tap(page, "[data-action=onboard]");
}
async function capture(page, ms = 700) {
  await tap(page, "[data-action=record]");
  await page.waitForTimeout(ms);
  await tap(page, "[data-action=stop-recording]");
}
(async () => {
  const browser = await chromium.launch({
    headless: true,
    args: [
      "--use-fake-device-for-media-stream",
      "--use-fake-ui-for-media-stream",
      `--use-file-for-fake-audio-capture=${path.resolve(__dirname, "../media/en-day.wav")}`,
      "--autoplay-policy=no-user-gesture-required",
    ],
  });
  async function check(name, operation) {
    const context = await browser.newContext({
      permissions: ["microphone"],
      serviceWorkers: "block",
    });
    const page = await context.newPage();
    const errors = [];
    page.on("pageerror", (e) => errors.push(e.message));
    try {
      await onboard(page);
      await operation(page, context);
      assert.deepEqual(errors, []);
      results.push({ name, status: "pass" });
      console.log("PASS", name);
    } catch (error) {
      results.push({ name, status: "fail", error: error.stack });
      console.error("FAIL", name, error.stack, errors);
    } finally {
      await context.close();
    }
  }
  try {
    await check(
      "Visible search and lesson answers create support, never review receipts",
      async (page) => {
        await page.locator("#phrase-search").fill("day");
        await idle(page);
        await tap(page, "[data-action=start]");
        await tap(page, "[data-action=reveal]");
        assert.equal((await session(page)).attempt.eligible, false);
        assert.ok(
          (await session(page)).attempt.support.includes("search-result"),
        );
        assert.deepEqual((await history(page)).reviewed, {});
        await tap(page, "[data-action=pause]");
        await page.goto(origin + "/learn/en/lesson/rapport");
        await page.locator("[data-action=lesson-start]").waitFor();
        await tap(page, "[data-action=lesson-start]");
        await tap(page, "[data-action=reveal]");
        assert.equal((await session(page)).attempt.eligible, false);
        assert.ok(
          (await session(page)).attempt.support.includes("lesson-list"),
        );
      },
    );
    await check(
      "A stale browsing tab cannot overwrite review receipts or cue coverage",
      async (page, context) => {
        const second = await context.newPage();
        await second.goto(origin + "/learn/sr");
        await second.locator("#phrase-search").waitFor();
        await tap(page, "[data-action=start]");
        await tap(page, "[data-action=reveal]");
        await tap(page, '[data-action=grade][data-rating="3"]');
        const before = await history(page);
        await tap(second, "[data-action=settings]");
        await second.locator("#ui-mode").check();
        await second.waitForFunction(
          async () =>
            (
              await (
                await import("/shared/storage.mjs")
              ).get("settings", "preferences")
            ).learningUI,
        );
        await second.locator("#phrase-search").fill("kafu");
        await idle(second);
        const after = await history(page);
        assert.deepEqual(after.reviewed, before.reviewed);
        assert.deepEqual(after.coverage, before.coverage);
        assert.ok(after.exposures.some((e) => e.targetId === "sr-coffee"));
      },
    );
    await check(
      "Undo and regrade create exactly one separated retry",
      async (page) => {
        await tap(page, "[data-action=start]");
        await tap(page, "[data-action=reveal]");
        await tap(page, '[data-action=grade][data-rating="1"]');
        await page.getByText("Prototype diagnostics", { exact: true }).click();
        await tap(page, "[data-action=clock]");
        for (let i = 0; i < 2; i++) {
          await tap(page, "[data-action=reveal]");
          await tap(page, '[data-action=grade][data-rating="3"]');
        }
        await tap(page, "[data-action=undo]");
        await tap(page, '[data-action=grade][data-rating="3"]');
        let s = await session(page);
        assert.equal(s.queue.filter((i) => i.retry).length, 1);
        assert.equal(s.repairs[0].attempts, 1);
        await tap(page, "[data-action=reveal]");
        await tap(page, '[data-action=grade][data-rating="3"]');
        s = await session(page);
        assert.equal(s.queue[s.cursor].retry, false);
        assert.equal(s.repairs[0].done, true);
      },
    );
    await check(
      "Original recording remains replayable after explanation practice and Undo",
      async (page) => {
        await page.locator("#recording-mode").check();
        await tap(page, "[data-action=start]");
        await capture(page, 650);
        const original = (await read(page, "takes"))[0];
        await tap(page, "[data-action=explain]");
        await capture(page, 1400);
        await tap(page, "[data-action=back]");
        assert.equal((await session(page)).attempt.takeId, original.id);
        await page.evaluate(() => {
          window.objectURL = URL.createObjectURL;
          URL.createObjectURL = function (blob) {
            window.replayBytes = blob.size;
            return window.objectURL.call(URL, blob);
          };
        });
        await tap(page, "[data-action=play-own]");
        assert.equal(
          await page.evaluate(() => window.replayBytes),
          original.blobBytes,
        );
        await tap(page, '[data-action=grade][data-rating="3"]');
        await tap(page, "[data-action=undo]");
        assert.equal(await page.locator("[data-action=play-own]").count(), 1);
        await tap(page, "[data-action=play-own]");
        assert.equal(
          await page.evaluate(() => window.replayBytes),
          original.blobBytes,
        );
      },
    );
    await check(
      "Unsaved audio survives advancement and blocks replacement until recovery",
      async (page) => {
        await page.evaluate(() => {
          window.originalPut = IDBObjectStore.prototype.put;
          IDBObjectStore.prototype.put = function (value) {
            if (this.name === "takes")
              throw new DOMException("Injected quota", "QuotaExceededError");
            return window.originalPut.call(this, value);
          };
        });
        await page.locator("#recording-mode").check();
        await tap(page, "[data-action=start]");
        await capture(page);
        assert.equal((await read(page, "takes")).length, 0);
        await tap(page, '[data-action=grade][data-rating="3"]');
        assert.equal(
          await page.locator("[data-action=retry-storage]").count(),
          1,
        );
        await tap(page, "[data-action=record]");
        assert.equal(
          await page.locator("[data-action=stop-recording]").count(),
          0,
        );
        assert.match(await page.locator("main").innerText(), /not saved/);
        await page.evaluate(() => {
          IDBObjectStore.prototype.put = window.originalPut;
        });
        await tap(page, "[data-action=retry-storage]");
        const recovered = (await read(page, "takes"))[0];
        assert.equal(recovered.targetId, "en-day");
        assert.ok(recovered.blobBytes > 0);
        await capture(page);
        const takes = await read(page, "takes");
        assert.equal(takes.length, 2);
        assert.ok(takes.some((t) => t.id === recovered.id));
      },
    );
    await check(
      "Pausing a delayed reference cancels playback and resumes the original self-check",
      async (page) => {
        await page.locator("#recording-mode").check();
        await tap(page, "[data-action=start]");
        await page.evaluate(() => {
          window.playCalls = 0;
          const play = HTMLMediaElement.prototype.play;
          HTMLMediaElement.prototype.play = function () {
            window.playCalls++;
            return play.call(this);
          };
        });
        let release;
        const delayed = new Promise((resolve) => (release = resolve));
        let request;
        await page.route("**/media/en-day.wav", (route) => {
          request = route;
          release();
        });
        await tap(page, "[data-action=record]");
        await page.waitForTimeout(650);
        await page.locator("[data-action=stop-recording]").click();
        await delayed;
        await page.locator("[data-action=pause]").click();
        await page.locator("#phrase-search").waitFor();
        await request.fulfill({
          status: 200,
          contentType: "audio/wav",
          body: fs.readFileSync(path.resolve(__dirname, "../media/en-day.wav")),
        });
        await idle(page);
        assert.equal(await page.evaluate(() => window.playCalls), 1);
        await tap(page, "[data-action=resume]");
        assert.equal(await page.locator("[data-action=grade]").count(), 3);
        assert.equal((await session(page)).phase, "check");
      },
    );
    await check(
      "Actor and generated imports coexist and actor audio remains selected",
      async (page) => {
        await page.goto(origin + "/audio-probe/");
        await page
          .getByText("Import actor/course or prepared audio", { exact: true })
          .click();
        await page
          .locator("#source")
          .fill("Synthetic actor stand-in for regression only");
        await page.locator("#origin").selectOption("actor");
        await page.locator("#matches").check();
        await page
          .locator("#asset")
          .setInputFiles(path.resolve(__dirname, "../media/en-day.wav"));
        await tap(page, "#import");
        await page.locator("#source").fill("Synthetic generated stand-in");
        await page.locator("#origin").selectOption("generated");
        await page
          .locator("#asset")
          .setInputFiles(path.resolve(__dirname, "../media/en-see.wav"));
        await tap(page, "#import");
        const references = await read(page, "references");
        assert.equal(references.length, 2);
        assert.match(await page.locator("#provenance").innerText(), /^actor:/);
        assert.equal(
          await page.evaluate(async () => {
            const { AudioFlow } = await import("/shared/audio.mjs");
            return (await new AudioFlow().reference({ id: "en-day" })).size;
          }),
          references.find((r) => r.origin === "actor").blobBytes,
        );
      },
    );
    await check(
      "Undo ends active capture and retains the interrupted take",
      async (page) => {
        await page.evaluate(() => {
          window.streams = [];
          const get = navigator.mediaDevices.getUserMedia.bind(
            navigator.mediaDevices,
          );
          navigator.mediaDevices.getUserMedia = async (constraints) => {
            const stream = await get(constraints);
            window.streams.push(stream);
            return stream;
          };
        });
        await page.locator("#recording-mode").check();
        await tap(page, "[data-action=start]");
        await capture(page, 650);
        await tap(page, '[data-action=grade][data-rating="3"]');
        await tap(page, "[data-action=record]");
        await page.waitForTimeout(700);
        await tap(page, "[data-action=undo]");
        assert.equal(
          await page.evaluate(
            () =>
              window.streams
                .flatMap((s) => s.getTracks())
                .filter((t) => t.readyState === "live").length,
          ),
          0,
        );
        const takes = await read(page, "takes");
        assert.equal(takes.length, 2);
        assert.equal(
          takes.find((t) => t.targetId === "en-see").status,
          "incomplete",
        );
        assert.equal((await session(page)).attempt.targetId, "en-day");
        assert.equal(await page.locator("[data-action=grade]").count(), 3);
      },
    );
    await check(
      "Failed result transactions roll back every store and allow one retry",
      async (page) => {
        await tap(page, "[data-action=start]");
        for (const failingStore of ["events", "settings", "sessions"]) {
          await tap(page, "[data-action=reveal]");
          const before = await session(page);
          const oldHistory = await history(page);
          const events = await read(page, "events");
          await page.evaluate((store) => {
            window.originalPut = IDBObjectStore.prototype.put;
            IDBObjectStore.prototype.put = function (v) {
              if (
                this.name === store &&
                ((store === "events" && v.type === "result") ||
                  (store === "settings" && v.id === "history") ||
                  store === "sessions")
              )
                throw new DOMException(
                  "Injected result quota",
                  "QuotaExceededError",
                );
              return window.originalPut.call(this, v);
            };
          }, failingStore);
          await tap(page, '[data-action=grade][data-rating="3"]');
          assert.deepEqual(await history(page), oldHistory);
          assert.deepEqual(await read(page, "events"), events);
          assert.deepEqual(await session(page), before);
          await page.evaluate(() => {
            IDBObjectStore.prototype.put = window.originalPut;
          });
          await tap(page, '[data-action=grade][data-rating="3"]');
          const after = await session(page);
          assert.equal(after.cursor, before.cursor + 1);
          assert.equal(
            after.grades.filter(
              (g) => g.targetId === before.attempt.targetId && !g.undone,
            ).length,
            1,
          );
        }
      },
    );
    await check(
      "Explanation capture disables replay and reference never plays into a live microphone",
      async (page) => {
        await page.evaluate(() => {
          window.streams = [];
          window.plays = [];
          const get = navigator.mediaDevices.getUserMedia.bind(
            navigator.mediaDevices,
          );
          navigator.mediaDevices.getUserMedia = async (constraints) => {
            const stream = await get(constraints);
            window.streams.push(stream);
            return stream;
          };
          const play = HTMLMediaElement.prototype.play;
          HTMLMediaElement.prototype.play = function () {
            window.plays.push(
              window.streams.some((s) =>
                s.getTracks().some((t) => t.readyState === "live"),
              ),
            );
            return play.call(this);
          };
        });
        await page.locator("#recording-mode").check();
        await tap(page, "[data-action=start]");
        await tap(page, "[data-action=explain]");
        await tap(page, "[data-action=record]");
        assert.equal(
          await page.locator("[data-action=play-reference]").isDisabled(),
          true,
        );
        await page
          .locator("[data-action=play-reference]")
          .evaluate((el) =>
            el.dispatchEvent(new MouseEvent("click", { bubbles: true })),
          );
        assert.deepEqual(await page.evaluate(() => window.plays), []);
        await page.waitForTimeout(650);
        await tap(page, "[data-action=stop-recording]");
        assert.deepEqual(await page.evaluate(() => window.plays), [
          false,
          false,
        ]);
      },
    );
    await check(
      "Home can enable recording for a new session after Finish",
      async (page) => {
        await tap(page, "[data-action=start]");
        await tap(page, "[data-action=pause]");
        await tap(page, "[data-action=finish]");
        await page.locator("#recording-mode").check();
        await page.waitForFunction(async () =>
          (await (await import("/shared/storage.mjs")).all("events")).some(
            (e) => e.type === "recording-mode" && e.enabled,
          ),
        );
        await tap(page, "[data-action=start]");
        assert.equal((await session(page)).recording, true);
        assert.equal(await page.locator("[data-action=record]").count(), 1);
      },
    );
    await check(
      "Reveal exposure survives a finished session without rewriting the original grade",
      async (page) => {
        await tap(page, "[data-action=start]");
        await tap(page, "[data-action=reveal]");
        await tap(page, '[data-action=grade][data-rating="1"]');
        assert.equal((await history(page)).reviewed["en-day"].result, 1);
        await tap(page, "[data-action=pause]");
        await tap(page, "[data-action=finish]");
        await tap(page, "[data-action=start]");
        await tap(page, "[data-action=reveal]");
        assert.equal((await session(page)).attempt.eligible, false);
        assert.ok(
          (await session(page)).attempt.support.includes("answer-reveal"),
        );
        await tap(page, "[data-action=supported-continue]");
        assert.equal((await history(page)).reviewed["en-day"].result, 1);
        await tap(page, "[data-action=pause]");
        await tap(page, "[data-action=finish]");
        await tap(page, "[data-action=start]");
        await page.getByText("Prototype diagnostics", { exact: true }).click();
        await tap(page, "[data-action=clock]");
        await tap(page, "[data-action=reveal]");
        assert.equal((await session(page)).attempt.eligible, true);
      },
    );
    await check(
      "Automatic comparison journals answer exposure for the following session",
      async (page) => {
        await page.locator("#recording-mode").check();
        await tap(page, "[data-action=start]");
        await capture(page, 650);
        await tap(page, '[data-action=grade][data-rating="1"]');
        await tap(page, "[data-action=pause]");
        await tap(page, "[data-action=finish]");
        await page.locator("#recording-mode").uncheck();
        await tap(page, "[data-action=start]");
        await tap(page, "[data-action=reveal]");
        assert.equal((await session(page)).attempt.eligible, false);
        assert.ok(
          (await session(page)).attempt.support.includes("answer-reveal"),
        );
      },
    );
    await check(
      "Probe Stop cancels both manual and automatic delayed reference playback",
      async (page) => {
        await page.goto(origin + "/audio-probe/");
        await page.locator("#reference").waitFor();
        await page.evaluate(() => {
          window.playCalls = 0;
          const play = HTMLMediaElement.prototype.play;
          HTMLMediaElement.prototype.play = function () {
            window.playCalls++;
            return play.call(this);
          };
        });
        for (const automatic of [false, true]) {
          let release;
          const delayed = new Promise((resolve) => (release = resolve));
          let request;
          await page.route("**/media/en-day.wav", (route) => {
            request = route;
            release();
          });
          if (automatic) {
            await page.locator("#mode").check();
            await page.locator("#record").click();
            await page.locator("#stop-recording").waitFor({ state: "visible" });
            await page.waitForTimeout(650);
            await page.locator("#stop-recording").click();
          } else await page.locator("#reference").click();
          await delayed;
          await page.locator("#stop").click();
          await request.fulfill({
            status: 200,
            contentType: "audio/wav",
            body: fs.readFileSync(
              path.resolve(__dirname, "../media/en-day.wav"),
            ),
          });
          await page.waitForTimeout(300);
          assert.equal(
            await page.evaluate(() => window.playCalls),
            automatic ? 1 : 0,
          );
          assert.match(
            await page.locator("#status").innerText(),
            /Playback stopped/,
          );
          await page.unroute("**/media/en-day.wav");
        }
      },
    );
    await check(
      "Starting capture cancels delayed manual reference playback on both surfaces",
      async (page) => {
        await page.locator("#recording-mode").check();
        await tap(page, "[data-action=start]");
        await tap(page, "[data-action=explain]");
        for (const probe of [false, true]) {
          if (probe) {
            await tap(page, "[data-action=back]");
            await tap(page, "[data-action=pause]");
            await page.goto(origin + "/audio-probe/");
            await page.locator("#mode").check();
          }
          await page.evaluate(() => {
            window.playCalls = 0;
            const play = HTMLMediaElement.prototype.play;
            HTMLMediaElement.prototype.play = function () {
              window.playCalls++;
              return play.call(this);
            };
          });
          let release;
          const waiting = new Promise((resolve) => (release = resolve));
          let request;
          await page.route("**/media/en-day.wav", (route) => {
            request = route;
            release();
          });
          await page
            .locator(probe ? "#reference" : "[data-action=play-reference]")
            .click();
          await waiting;
          await page
            .locator(probe ? "#record" : "[data-action=record]")
            .click();
          await page
            .locator(probe ? "#stop-recording" : "[data-action=stop-recording]")
            .waitFor({ state: "visible" });
          await request.fulfill({
            status: 200,
            contentType: "audio/wav",
            body: fs.readFileSync(
              path.resolve(__dirname, "../media/en-day.wav"),
            ),
          });
          await page.waitForTimeout(300);
          assert.equal(await page.evaluate(() => window.playCalls), 0);
          await page.unroute("**/media/en-day.wav");
        }
      },
    );
    await check(
      "Stopping learner replay preserves original eligibility and logs the revealed answer",
      async (page) => {
        await page.locator("#recording-mode").check();
        await tap(page, "[data-action=start]");
        await tap(page, "[data-action=record]");
        await page.waitForTimeout(1400);
        await page.locator("[data-action=stop-recording]").click();
        await page.waitForFunction(
          async () =>
            (await (await import("/shared/storage.mjs")).get("sessions", "en"))
              ?.phase === "own",
        );
        await tap(page, "[data-action=stop-audio]");
        assert.equal((await session(page)).attempt.eligible, true);
        assert.ok(
          (await history(page)).exposures.some(
            (e) => e.targetId === "en-day" && e.kind === "answer-reveal",
          ),
        );
        await tap(page, '[data-action=grade][data-rating="1"]');
        await tap(page, "[data-action=pause]");
        await tap(page, "[data-action=finish]");
        await page.locator("#recording-mode").uncheck();
        await tap(page, "[data-action=start]");
        await tap(page, "[data-action=reveal]");
        assert.equal((await session(page)).attempt.eligible, false);
        assert.ok(
          (await session(page)).attempt.support.includes("answer-reveal"),
        );
      },
    );
  } finally {
    await browser.close();
    const directory =
      process.env.PHRASE_GYM_TEST_ARTIFACTS || "/tmp/phrase-gym-p1-browser";
    fs.mkdirSync(directory, { recursive: true });
    fs.writeFileSync(
      path.join(directory, "review-results.json"),
      JSON.stringify({ syntheticMicrophone: true, results }, null, 2),
    );
  }
  if (results.some((r) => r.status === "fail")) process.exitCode = 1;
})();
