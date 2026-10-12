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
async function onboard(page, native = "en") {
  await page.goto(origin + "/learn/en");
  await page.locator("#first-native").selectOption(native);
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
      "Language roles, concept lookup, lesson navigation, exposure and reload",
      async (page, context) => {
        await onboard(page, "ru");
        await tap(page, "[data-action=settings]");
        await page.locator("#ui-mode").check();
        await tap(page, "[data-action=settings]");
        await page.locator("#lesson-filter").fill("rap");
        assert.equal(await page.locator("#lessons option").count(), 2);
        await page.locator("#phrase-search").fill("greetings");
        assert.equal(await page.locator("#search-results button").count(), 3);
        await tap(page, "[data-action=phrase][data-id=en-see]");
        await page.locator(".cue").waitFor();
        assert.ok(
          (await page.locator("main").innerText()).includes(
            "Рад снова тебя видеть",
          ),
        );
        assert.equal(
          (await read(page, "events")).filter((e) => e.type === "result")
            .length,
          0,
        );
        await tap(page, "[data-action=back]");
        assert.equal(
          await page.locator("#phrase-search").inputValue(),
          "greetings",
        );
        await tap(page, "[data-action=phrase][data-id=en-see]");
        await tap(page, "[data-action=lesson]");
        assert.equal(
          await page.locator(".results [data-action=phrase]").count(),
          4,
        );
        await tap(page, "[data-action=lesson-start]");
        await page.locator("[data-action=explain]").waitFor();
        await tap(page, "[data-action=explain]");
        await tap(page, "[data-action=back]");
        await tap(page, "[data-action=reveal]");
        assert.equal(await page.locator("[data-action=grade]").count(), 0);
        await page.reload();
        assert.equal(
          await page.locator("[data-action=supported-continue]").count(),
          1,
        );
        await page.locator("#language").selectOption("sr");
        await page.waitForURL("**/learn/sr");
        assert.ok(page.url().includes("/learn/sr"));
        const second = await context.newPage();
        await second.goto(origin + "/learn/en");
        assert.equal(await second.locator("#language").inputValue(), "en");
        assert.equal(await page.locator("#language").inputValue(), "sr");
        await page.locator("#language").selectOption("en");
        await page.waitForURL("**/learn/en");
        await tap(page, "[data-action=resume]");
        assert.equal(
          await page.locator("[data-action=supported-continue]").count(),
          1,
        );
        assert.equal(
          await page.evaluate(
            () => document.documentElement.scrollWidth <= innerWidth,
          ),
          true,
        );
        await page.screenshot({
          path: path.join(artifacts, "supported-320.png"),
          fullPage: true,
        });
      },
    );
    await check(
      "Original grades, extra practice, Undo, no-grade problems and summary",
      async (page) => {
        await onboard(page);
        await tap(page, "[data-action=start]");
        await tap(page, "[data-action=reveal]");
        await tap(page, '[data-action=grade][data-rating="2"]');
        await tap(page, "[data-action=undo]");
        await tap(page, "[data-action=practice]");
        await tap(page, '[data-action=grade][data-rating="3"]');
        assert.equal(
          await page.locator("[data-action=continue-practice]").count(),
          1,
        );
        let s = (await read(page, "sessions"))[0];
        assert.equal(s.grades.filter((g) => !g.undone).length, 1);
        assert.equal(s.grades.filter((g) => !g.undone)[0].result, 3);
        await page.reload();
        assert.equal(
          await page.locator("[data-action=continue-practice]").count(),
          1,
        );
        await tap(page, "[data-action=continue-practice]");
        await tap(page, "[data-action=reveal]");
        await tap(page, "[data-action=problem]");
        await tap(page, "[data-action=no-grade][data-reason=unclear-cue]");
        s = (await read(page, "sessions"))[0];
        assert.equal(s.grades.at(-1).scheduleUpdated, false);
        await tap(page, "[data-action=pause]");
        await tap(page, "[data-action=resume]");
        await page.screenshot({
          path: path.join(artifacts, "drill-320.png"),
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
        await page.screenshot({
          path: path.join(artifacts, "drill-text-200.png"),
          fullPage: true,
        });
      },
    );
    await check(
      "Separated retry after two intervening targets and simulated elapsed time",
      async (page) => {
        await onboard(page);
        await tap(page, "[data-action=start]");
        await tap(page, "[data-action=reveal]");
        await tap(page, '[data-action=grade][data-rating="1"]');
        await page.getByText("Prototype diagnostics", { exact: true }).click();
        await tap(page, "[data-action=clock]");
        for (let i = 0; i < 2; i++) {
          await tap(page, "[data-action=reveal]");
          await tap(page, '[data-action=grade][data-rating="3"]');
        }
        let s = (await read(page, "sessions"))[0];
        assert.equal(s.queue[s.cursor].retry, true);
        assert.equal(s.queue[s.cursor].targetId, "en-day");
        await tap(page, "[data-action=reveal]");
        await tap(page, '[data-action=grade][data-rating="3"]');
        s = (await read(page, "sessions"))[0];
        assert.equal(
          s.grades.filter((g) => g.targetId === "en-day" && g.scheduleUpdated)
            .length,
          1,
        );
        assert.equal(s.grades.find((g) => g.targetId === "en-day").result, 1);
      },
    );
    await check(
      "Prepared dialogue, isolated repair and return to exchange",
      async (page) => {
        await onboard(page);
        await page.getByText("Prepared dialogue", { exact: true }).click();
        await tap(page, "[data-action=dialogue][data-id=break]");
        assert.ok(
          (await page.locator("main").innerText()).includes(
            "Hi! Good to see you.",
          ),
        );
        await tap(page, "[data-action=isolate]");
        await tap(page, "[data-action=reveal]");
        await tap(page, '[data-action=grade][data-rating="1"]');
        assert.equal(
          await page.locator("[data-action=continue-practice]").count(),
          1,
        );
        await tap(page, "[data-action=continue-practice]");
        assert.ok(
          (await page.locator("main").innerText()).includes(
            "I’ve just finished a meeting.",
          ),
        );
      },
    );
    await check(
      "Real MediaRecorder with synthetic input, ordered comparison and all takes retained",
      async (page) => {
        await onboard(page);
        await page.locator("#recording-mode").check();
        await tap(page, "[data-action=start]");
        await tap(page, "[data-action=record]");
        await page.locator("[data-action=stop-recording]").waitFor();
        await page.waitForTimeout(1200);
        await page.locator("[data-action=stop-recording]").click();
        await waitPhase(page, "own");
        assert.ok((await page.locator(".cue").innerText()).includes("[…]"));
        await page
          .locator('[data-action=grade][data-rating="3"]')
          .waitFor({ timeout: 20000 });
        let takes = await read(page, "takes");
        assert.equal(takes.length, 1);
        assert.ok(takes[0].blobBytes > 1000);
        assert.equal(takes[0].status, "complete");
        await tap(page, "[data-action=practice]");
        await tap(page, '[data-action=grade][data-rating="3"]');
        await tap(page, "[data-action=record]");
        await page.locator("[data-action=stop-recording]").waitFor();
        assert.ok(!(await page.locator(".cue").innerText()).includes("[…]"));
        await page.waitForTimeout(1200);
        await page.locator("[data-action=stop-recording]").click();
        await page
          .locator("[data-action=continue-practice]")
          .waitFor({ timeout: 20000 });
        takes = await read(page, "takes");
        assert.equal(takes.length, 2);
        assert.ok(takes.some((t) => t.modality === "supported-practice"));
        await page.reload();
        takes = await read(page, "takes");
        assert.equal(takes.length, 2);
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
    await check(
      "Downloaded session can reload and decode prepared assets disconnected",
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
        await tap(page, "[data-action=reveal]");
        await tap(page, "[data-action=play-reference]");
        await page.waitForTimeout(2200);
        assert.equal(await page.locator(".error").count(), 0);
      },
    );
    await check(
      "Explanation-card recording keeps the full phrase visible and reload retains drill origin",
      async (page) => {
        await onboard(page);
        await page.locator("#recording-mode").check();
        await tap(page, "[data-action=start]");
        await tap(page, "[data-action=explain]");
        await tap(page, "[data-action=record]");
        await page.locator("[data-action=stop-recording]").waitFor();
        assert.ok(!(await page.locator(".cue").innerText()).includes("[…]"));
        await page.waitForTimeout(1100);
        await tap(page, "[data-action=stop-recording]");
        const takes = await read(page, "takes");
        assert.equal(takes.length, 1);
        assert.equal(takes[0].modality, "supported-practice");
        assert.ok(takes[0].support.includes("explanation"));
        assert.equal(
          (await read(page, "events")).filter((e) => e.type === "result")
            .length,
          0,
        );
        await page.reload();
        await tap(page, "[data-action=back]");
        assert.ok(page.url().endsWith("/session"));
        await page.locator("#recording-mode").uncheck();
        await tap(page, "[data-action=reveal]");
        assert.equal(await page.locator("[data-action=grade]").count(), 0);
      },
    );
    await check(
      "Reload during learner playback retains original evidence and enables manual replay",
      async (page) => {
        await onboard(page);
        await page.locator("#recording-mode").check();
        await tap(page, "[data-action=start]");
        await tap(page, "[data-action=record]");
        await page.locator("[data-action=stop-recording]").waitFor();
        await page.waitForTimeout(1100);
        await page.locator("[data-action=stop-recording]").click();
        await page.waitForFunction(
          () => document.documentElement.dataset.working === "true",
        );
        await page.waitForTimeout(100);
        await page.reload();
        await page.locator("[data-action=play-own]").waitFor();
        assert.equal(await page.locator("[data-action=grade]").count(), 3);
        const s = (await read(page, "sessions"))[0];
        assert.equal(s.attempt.eligible, true);
        assert.equal(s.grades.length, 0);
      },
    );
  } catch (error) {
    console.error(error);
    results.push({ name: "harness", status: "fail", error: error.stack });
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
