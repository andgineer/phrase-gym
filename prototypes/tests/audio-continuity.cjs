const { chromium, webkit } = require("playwright");
const assert = require("node:assert/strict");
const fs = require("node:fs");
const os = require("node:os");
const path = require("node:path");
const origin = process.env.PHRASE_GYM_TEST_ORIGIN || "http://127.0.0.1:8871";
(async () => {
  const reports = [];
  for (const [name, engine] of Object.entries({ chromium, webkit })) {
    const profile = fs.mkdtempSync(path.join(os.tmpdir(), "phrase-gym-audio-"));
    const browser = await engine.launchPersistentContext(profile, {
      headless: true,
    });
    try {
      const page = await browser.newPage();
      await page.goto(origin + "/audio-probe/");
      const report = await page.evaluate(async () => {
        const { AudioFlow } = await import("/shared/audio.mjs");
        const context = new AudioContext();
        await context.resume();
        const source = context.createOscillator();
        source.frequency.value = 440;
        const gain = context.createGain();
        gain.gain.value = 0.2;
        const destination = context.createMediaStreamDestination();
        source.connect(gain).connect(destination);
        source.start();
        let acquisitions = 0;
        navigator.mediaDevices.getUserMedia = async () => {
          acquisitions++;
          return destination.stream;
        };
        const flow = new AudioFlow(() => {}, { keepReady: true });
        const takes = [];
        for (let i = 0; i < 3; i++) {
          await flow.start({
            targetId: "synthetic-continuity",
            modality: "synthetic-test",
          });
          await new Promise((resolve) => setTimeout(resolve, 2100));
          const take = await flow.stop();
          if (!take.saved || flow.pending)
            throw new Error(
              `Take ${i}: ${flow.storageError?.name}: ${flow.storageError?.message}; saved=${take.saved}; pending=${!!flow.pending}`,
            );
          const decoded = await context.decodeAudioData(
            await take.blob.arrayBuffer(),
          );
          const samples = decoded.getChannelData(0),
            windowSize = Math.round(decoded.sampleRate / 10);
          const rms = [];
          // Exclude encoder startup/trailing padding; inspect every 100ms in the middle.
          for (
            let offset = windowSize * 2;
            offset + windowSize * 2 < samples.length;
            offset += windowSize
          ) {
            let sum = 0;
            for (let j = 0; j < windowSize; j++)
              sum += samples[offset + j] ** 2;
            rms.push(Math.sqrt(sum / windowSize));
          }
          takes.push({
            duration: decoded.duration,
            bytes: take.bytes,
            saved: take.saved,
            windows: rms.length,
            minRms: Math.min(...rms),
            maxRms: Math.max(...rms),
          });
        }
        const readyBetweenTakes =
          destination.stream.getAudioTracks()[0].readyState;
        await flow.cancel();
        const released = destination.stream.getAudioTracks()[0].readyState;
        source.stop();
        await context.close();
        return { acquisitions, readyBetweenTakes, released, takes };
      });
      assert.equal(report.acquisitions, 1);
      assert.equal(report.readyBetweenTakes, "live");
      assert.equal(report.released, "ended");
      for (const take of report.takes) {
        assert.ok(take.saved && take.bytes > 1000);
        assert.ok(take.duration > 1.8 && take.duration < 2.6);
        assert.ok(take.windows >= 14);
        assert.ok(
          take.minRms > 0.05,
          `${name}: middle-of-take dropout ${take.minRms}`,
        );
        assert.ok(take.maxRms < 0.25, `${name}: unexpected amplification`);
      }
      reports.push({ engine: name, ...report });
      console.log(
        "PASS",
        name,
        "three continuous takes, one stream acquisition, release on cancel",
      );
    } finally {
      await browser.close();
      fs.rmSync(profile, { recursive: true, force: true });
    }
  }
  fs.mkdirSync("/tmp/phrase-gym-p1-browser", { recursive: true });
  fs.writeFileSync(
    "/tmp/phrase-gym-p1-browser/audio-continuity.json",
    JSON.stringify(reports, null, 2),
  );
})().catch((error) => {
  console.error(error);
  process.exitCode = 1;
});
