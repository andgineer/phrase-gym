const { chromium } = require("playwright");
const assert = require("node:assert/strict");
const fs = require("node:fs");
const path = require("node:path");
const origin = process.env.PHRASE_GYM_TEST_ORIGIN || "http://127.0.0.1:8871";
const artifacts =
  process.env.PHRASE_GYM_TEST_ARTIFACTS || "/tmp/phrase-gym-p1-browser";
fs.mkdirSync(artifacts, { recursive: true });
const results = [];
async function tap(page, selector) {
  await page.locator(selector).click();
  await page.waitForFunction(
    () => document.documentElement.dataset.working !== "true",
  );
}
async function read(page, store) {
  return page.evaluate(async (store) => {
    const db = await import("/shared/storage.mjs");
    return (await db.all(store)).map(({ blob, ...meta }) => ({
      ...meta,
      blobBytes: blob?.size,
    }));
  }, store);
}
async function onboard(page, native = "en", options = {}) {
  await page.goto(origin + "/learn/en");
  await page.locator("#first-native").selectOption(native);
  if (options.learningUI) await page.locator("#first-ui-mode").check();
  await tap(page, "[data-action=onboard]");
  await page.locator("#phrase-search").waitFor();
}
async function waitPhase(page, phase) {
  await page.waitForFunction(async (phase) => {
    const db = await import("/shared/storage.mjs");
    return (await db.get("sessions", "en"))?.phase === phase;
  }, phase);
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
  async function check(name, fn) {
    const context = await browser.newContext({
      viewport: { width: 320, height: 800 },
      permissions: ["microphone"],
    });
    const page = await context.newPage();
    page.setDefaultTimeout(10000);
    const errors = [];
    page.on("pageerror", (e) => errors.push(e.message));
    try {
      await fn(page, context);
      assert.deepEqual(errors, []);
      results.push({ name, status: "pass" });
      console.log("PASS", name);
    } catch (error) {
      results.push({ name, status: "fail", error: error.stack });
      await page.screenshot({
        path: path.join(artifacts, `failure-${results.length}.png`),
        fullPage: true,
      });
      console.error("FAIL", name, error.message);
    } finally {
      await context.close();
    }
  }
  try {
    await check(
      "Unified card, UI languages and silent self-check without microphone permission",
      async (page) => {
        await page.addInitScript(() => {
          window.requests = 0;
          navigator.mediaDevices.getUserMedia = () => {
            window.requests++;
            throw new Error("Unexpected microphone access");
          };
        });
        await onboard(page, "ru", { learningUI: true });
        assert.equal(await page.locator("html").getAttribute("lang"), "en");
        await tap(page, "[data-action=start]");
        assert.equal(await page.locator("[data-action=grade]").count(), 3);
        assert.equal(
          await page
            .locator(
              "#recording-mode, [data-action=practice], [data-action=supported-continue], [data-action=missed], [data-action=stop-audio]",
            )
            .count(),
          0,
        );
        await page.screenshot({
          path: path.join(artifacts, "unified-cue-320.png"),
          fullPage: true,
        });
        await tap(page, "[data-action=hint]");
        assert.equal(await page.locator(".gap").count(), 0);
        assert.equal(await page.locator("[data-action=record]").count(), 1);
        assert.equal(await page.locator("[data-action=grade]").count(), 3);
        await tap(page, '[data-action=grade][data-rating="3"]');
        const grade = (await read(page, "sessions"))[0].grades[0];
        assert.equal(grade.modality, "silent-recall");
        assert.equal(grade.result, 3);
        assert.equal(await page.evaluate(() => window.requests), 0);
        await tap(page, "[data-action=settings]");
        await page.locator("#ui-mode").uncheck();
        await page.waitForFunction(
          () => document.documentElement.lang === "ru",
        );
        await tap(page, "[data-action=settings]");
        await tap(page, "[data-action=explain]");
        assert.ok(page.url().endsWith("/session"));
        assert.ok(await page.locator(".phrase-details [lang=ru]").count());
        assert.equal(
          await page.locator(".answer-actions [data-action=explain]").count(),
          1,
        );
        await page.screenshot({
          path: path.join(artifacts, "unified-explanation-320.png"),
          fullPage: true,
        });
        await page.evaluate(
          () => (document.documentElement.style.fontSize = "34px"),
        );
        assert.equal(
          await page.evaluate(
            () => document.documentElement.scrollWidth <= innerWidth,
          ),
          true,
        );
        await tap(page, '[data-action=grade][data-rating="2"]');
        const supported = (await read(page, "sessions"))[0].grades.at(-1);
        assert.equal(supported.result, 2);
        assert.equal(supported.eligible, false);
        assert.equal(supported.scheduleUpdated, false);
      },
    );
    await check(
      "Lookup, lessons, inline explanation, reload and language-scoped sessions",
      async (page, context) => {
        await onboard(page, "ru");
        await page.locator("#lesson-filter").fill("rap");
        assert.equal(await page.locator("#lessons option").count(), 2);
        await page.locator("#phrase-search").fill("greetings");
        await tap(page, "[data-action=phrase][data-id=en-see]");
        assert.ok(
          (await page.locator("main").innerText()).includes(
            "Рад снова тебя видеть",
          ),
        );
        await tap(page, "[data-action=lesson]");
        assert.equal(
          await page.locator(".results [data-action=phrase]").count(),
          4,
        );
        await tap(page, "[data-action=lesson-start]");
        await tap(page, "[data-action=explain]");
        await page.reload();
        await page.locator(".phrase-details").waitFor();
        assert.equal(await page.locator("[data-action=grade]").count(), 3);
        await tap(page, '[data-action=grade][data-rating="3"]');
        assert.equal(
          (await read(page, "sessions"))[0].grades[0].scheduleUpdated,
          false,
        );
        await page.locator("#language").selectOption("sr");
        await page.waitForURL("**/learn/sr");
        const second = await context.newPage();
        await second.goto(origin + "/learn/en");
        assert.equal(await second.locator("#language").inputValue(), "en");
        assert.equal(await page.locator("#language").inputValue(), "sr");
        await page.locator("#language").selectOption("en");
        await tap(page, "[data-action=resume]");
        assert.equal((await read(page, "sessions"))[0].cursor, 1);
      },
    );
    await check(
      "Self-check before recording, quiet advancement and separated retries",
      async (page) => {
        await onboard(page);
        await tap(page, "[data-action=start]");
        await tap(page, '[data-action=grade][data-rating="1"]');
        assert.equal(await page.locator("[data-action=undo]").count(), 0);
        assert.equal(await page.getByText("Previous rating saved").count(), 0);
        await page.getByText("Prototype diagnostics", { exact: true }).click();
        await tap(page, "[data-action=clock]");
        for (let i = 0; i < 2; i++)
          await tap(page, '[data-action=grade][data-rating="3"]');
        let s = (await read(page, "sessions"))[0];
        assert.equal(s.queue[s.cursor].retry, true);
        await tap(page, "[data-action=hint]");
        await tap(page, '[data-action=grade][data-rating="3"]');
        s = (await read(page, "sessions"))[0];
        assert.equal(
          s.grades.filter(
            (g) => g.targetId === "en-day" && g.scheduleUpdated && !g.undone,
          ).length,
          1,
        );
      },
    );
    await check(
      "Prepared dialogue retains context and advances from one card",
      async (page) => {
        await onboard(page);
        await page
          .locator("details")
          .filter({ has: page.locator("[data-action=dialogue]") })
          .locator("summary")
          .click();
        await tap(page, "[data-action=dialogue]");
        const before = (await read(page, "sessions"))[0];
        await tap(page, "[data-action=isolate]");
        await tap(page, "[data-action=hint]");
        await tap(page, '[data-action=grade][data-rating="3"]');
        const after = (await read(page, "sessions"))[0];
        assert.equal(after.cursor, before.cursor + 1);
        assert.equal(after.isolated, false);
      },
    );
    await check(
      "Repeated takes reuse microphone; stop icon restores record and preserves first response",
      async (page) => {
        await onboard(page);
        await page.evaluate(() => {
          window.streams = [];
          const original = navigator.mediaDevices.getUserMedia.bind(
            navigator.mediaDevices,
          );
          navigator.mediaDevices.getUserMedia = async (constraints) => {
            const stream = await original(constraints);
            window.streams.push(stream);
            return stream;
          };
        });
        await tap(page, "[data-action=start]");
        for (let i = 0; i < 3; i++) {
          await tap(page, "[data-action=record]");
          await page.waitForTimeout(1200);
          await page.locator("[data-action=stop-recording]").click();
          await page.locator("[data-action=stop-audio]").waitFor();
          assert.equal(await page.locator(".gap").count(), 0);
          await tap(page, "[data-action=stop-audio]");
          assert.equal(await page.locator("[data-action=record]").count(), 1);
          assert.equal(
            await page.locator("[data-action=stop-audio]").count(),
            0,
          );
          assert.equal(await page.locator("[data-action=grade]").count(), 3);
        }
        const takes = (await read(page, "takes")).sort(
          (a, b) => a.created - b.created,
        );
        assert.equal(takes.length, 3);
        assert.ok(
          takes.every((t) => t.blobBytes > 1000 && t.status === "complete"),
        );
        assert.equal(
          (await read(page, "sessions"))[0].attempt.takeId,
          takes[0].id,
        );
        assert.equal(takes[1].modality, "supported-practice");
        assert.equal(await page.evaluate(() => window.streams.length), 1);
        assert.equal(takes[0].captureSettings.echoCancellation, false);
        await tap(page, "[data-action=play-own]");
        assert.equal(await page.locator("[data-action=stop-audio]").count(), 0);
        await tap(page, '[data-action=grade][data-rating="3"]');
        await tap(page, "[data-action=pause]");
        assert.equal(
          await page.evaluate(
            () =>
              window.streams
                .flatMap((s) => s.getTracks())
                .filter((t) => t.readyState === "live").length,
          ),
          0,
        );
      },
    );
    await check(
      "Inline explanation stays visible during recording and remains rateable",
      async (page) => {
        await onboard(page);
        await tap(page, "[data-action=start]");
        await tap(page, "[data-action=explain]");
        await tap(page, "[data-action=record]");
        assert.equal(await page.locator(".gap").count(), 0);
        assert.equal(await page.locator(".phrase-details").count(), 1);
        assert.equal(
          await page.locator("[data-action=grade]:disabled").count(),
          3,
        );
        await page.waitForTimeout(700);
        await tap(page, "[data-action=stop-recording]");
        assert.equal(
          await page.locator("[data-action=grade]:enabled").count(),
          3,
        );
        await tap(page, '[data-action=grade][data-rating="2"]');
        const g = (await read(page, "sessions"))[0].grades[0];
        assert.equal(g.result, 2);
        assert.equal(g.scheduleUpdated, false);
      },
    );
    await check(
      "Downloaded card reloads and plays reference offline without recording",
      async (page, context) => {
        await onboard(page);
        await page.getByText("Prototype diagnostics", { exact: true }).click();
        await tap(page, "[data-action=download]");
        await page
          .getByText("Prepared session downloaded", { exact: true })
          .waitFor();
        await context.setOffline(true);
        await page.reload();
        await tap(page, "[data-action=start]");
        await tap(page, "[data-action=play-reference]");
        assert.equal(await page.locator(".error").count(), 0);
        assert.equal(await page.locator("[data-action=record]").count(), 1);
        assert.equal(await page.locator(".gap").count(), 0);
      },
    );
    await check(
      "Probe permission-free silent mode, interruption and local actor import",
      async (page) => {
        await page.goto(origin + "/audio-probe/");
        await tap(page, "#reveal");
        assert.equal((await read(page, "takes")).length, 0);
        await page
          .getByText("Import actor/course or prepared audio", { exact: true })
          .click();
        await page.locator("#origin").selectOption("actor");
        await page
          .locator("#source")
          .fill(
            "Synthetic test file; tests import transport only, not actor authenticity",
          );
        await page.locator("#matches").check();
        await page
          .locator("#asset")
          .setInputFiles(path.resolve(__dirname, "../media/en-day.wav"));
        await tap(page, "#import");
        await page.waitForFunction(() =>
          document
            .querySelector("#status")
            .textContent.includes("Original reference retained"),
        );
        assert.equal((await read(page, "references"))[0].origin, "actor");
        await page.locator("#mode").check();
        await tap(page, "#record");
        await page.locator("#stop-recording").waitFor();
        await page.waitForTimeout(1200);
        await page.reload();
        const takes = await read(page, "takes");
        assert.equal(takes.length, 1);
        assert.equal(takes[0].status, "incomplete");
        assert.ok(takes[0].blobBytes > 1000);
        await page
          .getByText("Local storage inspection", { exact: true })
          .click();
        await tap(page, "#refresh");
        await page.locator("#inspect").waitFor({ state: "visible" });
      },
    );
    await check(
      "Probe quota failure is visible, recovered bytes retry without duplicates",
      async (page) => {
        await page.goto(origin + "/audio-probe/");
        await page.evaluate(() => {
          window.originalPut = IDBObjectStore.prototype.put;
          IDBObjectStore.prototype.put = function (...args) {
            if (this.name === "takes")
              throw new DOMException(
                "Injected quota failure",
                "QuotaExceededError",
              );
            return window.originalPut.apply(this, args);
          };
        });
        await page.locator("#mode").check();
        await tap(page, "#record");
        await page.locator("#stop-recording").waitFor();
        await page.waitForTimeout(1100);
        await page.locator("#stop-recording").click();
        await page.locator("#retry").waitFor({ state: "visible" });
        assert.equal((await read(page, "takes")).length, 0);
        await page.evaluate(() => {
          IDBObjectStore.prototype.put = window.originalPut;
        });
        await tap(page, "#retry");
        await page.waitForFunction(
          () => document.querySelector("#retry").hidden,
        );
        assert.equal((await read(page, "takes")).length, 1);
      },
    );
  } finally {
    await browser.close();
    fs.writeFileSync(
      path.join(artifacts, "results.json"),
      JSON.stringify(
        { browser: "Chromium", syntheticMicrophone: true, origin, results },
        null,
        2,
      ),
    );
  }
  if (results.some((r) => r.status === "fail")) process.exitCode = 1;
})();
