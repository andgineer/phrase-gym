import * as db from "./storage.mjs";

export async function referenceRecord(target) {
  const legacy = await db.get("references", target.id);
  return (
    (await db.get("references", `${target.id}:actor`)) ||
    (legacy?.origin === "actor" ? legacy : null) ||
    (await db.get("references", `${target.id}:generated`)) ||
    legacy
  );
}

export class AudioFlow {
  constructor(change = () => {}, { keepReady = false } = {}) {
    this.change = change;
    this.keepReady = keepReady;
    this.previousSessionType = null;
    this.player = null;
    this.recorder = null;
    this.stream = null;
    this.generation = 0;
    this.pending = null;
  }
  static capabilities() {
    return {
      secureContext: isSecureContext,
      mediaRecorder: typeof MediaRecorder !== "undefined",
      microphone: !!navigator.mediaDevices?.getUserMedia,
      userAgent: navigator.userAgent,
      standalone:
        matchMedia("(display-mode: standalone)").matches ||
        !!navigator.standalone,
      mimeTypes: [
        "audio/webm;codecs=opus",
        "audio/mp4",
        "audio/ogg;codecs=opus",
      ].filter(
        (t) =>
          typeof MediaRecorder !== "undefined" &&
          MediaRecorder.isTypeSupported(t),
      ),
    };
  }
  stopPlayback() {
    if (this.player) {
      this.player.pause();
      this.player.dispatchEvent(new Event("cancelled"));
      this.player.src = "";
      this.player = null;
    }
  }
  async play(source, label) {
    if (this.recorder || this.requesting)
      throw new Error("Finish or cancel capture before playback.");
    this.stopPlayback();
    const url = source instanceof Blob ? URL.createObjectURL(source) : source;
    const player = new Audio(url);
    this.player = player;
    this.change(label);
    try {
      await new Promise((resolve, reject) => {
        player.onended = resolve;
        player.onerror = () =>
          reject(
            new Error("Audio could not be decoded or the asset is missing."),
          );
        player.addEventListener(
          "cancelled",
          () => reject(new DOMException("Playback stopped.", "AbortError")),
          { once: true },
        );
        player.play().catch(reject);
      });
    } finally {
      player.pause();
      if (source instanceof Blob) URL.revokeObjectURL(url);
      if (this.player === player) {
        this.player = null;
        this.change("idle");
      }
    }
  }
  async reference(target) {
    const imported = await referenceRecord(target);
    if (imported) return imported.blob;
    if (!target.audio.url)
      throw new Error(
        "Reference audio is not prepared. Import a matching phrase clip in the audio probe.",
      );
    const response = await fetch(target.audio.url);
    if (!response.ok)
      throw new Error(
        "Reference audio is missing. Download assets while connected or import the clip.",
      );
    return response.blob();
  }
  releaseMicrophone() {
    this.stream?.getTracks().forEach((track) => track.stop());
    this.stream = null;
    if (this.previousSessionType !== null) {
      try {
        navigator.audioSession.type = this.previousSessionType;
      } catch {}
      this.previousSessionType = null;
    }
  }
  async start(metadata) {
    if (this.recorder || this.requesting)
      throw new Error("Capture is already active.");
    if (this.pending)
      throw new Error(
        "A take is not saved. Keep this tab open and retry local storage before recording again.",
      );
    this.stopPlayback();
    const token = ++this.generation;
    this.requesting = true;
    this.change("permission");
    let stream;
    try {
      if (
        !navigator.mediaDevices?.getUserMedia ||
        typeof MediaRecorder === "undefined"
      ) {
        throw new Error(
          "Recording needs a secure origin and MediaRecorder. You can still reveal and rate without recording.",
        );
      }
      if (navigator.audioSession && this.previousSessionType === null) {
        this.previousSessionType = navigator.audioSession.type;
        try {
          navigator.audioSession.type = "play-and-record";
        } catch {}
      }
      const reusable =
        this.keepReady &&
        this.stream
          ?.getAudioTracks()
          .some((track) => track.readyState === "live");
      stream = reusable
        ? this.stream
        : await navigator.mediaDevices.getUserMedia({
            audio: {
              echoCancellation: false,
              noiseSuppression: false,
              autoGainControl: false,
            },
            video: false,
          });
      if (token !== this.generation) {
        stream.getTracks().forEach((t) => t.stop());
        return null;
      }
      this.stream = stream;
      const mimeType = AudioFlow.capabilities().mimeTypes[0];
      const recorder = new MediaRecorder(stream, mimeType ? { mimeType } : {});
      const take = {
        id: crypto.randomUUID(),
        created: Date.now(),
        mimeType: recorder.mimeType,
        status: "incomplete",
        bytes: 0,
        ...metadata,
        captureSettings: Object.fromEntries(
          Object.entries(stream.getAudioTracks()[0].getSettings()).filter(
            ([key]) =>
              [
                "echoCancellation",
                "noiseSuppression",
                "autoGainControl",
                "sampleRate",
                "channelCount",
                "latency",
              ].includes(key),
          ),
        ),
      };
      const chunks = [];
      let writes = Promise.resolve();
      this.recorder = recorder;
      this.completion = new Promise((resolve) => {
        recorder.ondataavailable = (e) => {
          if (!e.data.size) return;
          chunks.push(e.data);
          const snapshot = {
            ...take,
            blob: new Blob(chunks, { type: recorder.mimeType }),
          };
          snapshot.bytes = snapshot.blob.size;
          this.pending = snapshot;
          writes = writes
            .then(() => db.put("takes", snapshot))
            .catch((error) => {
              this.storageError = error;
              this.change("storage-error", error);
            });
        };
        recorder.onerror = () => {
          take.status = "incomplete";
          this.change("capture-error");
        };
        recorder.onstop = async () => {
          if (!this.keepReady) this.releaseMicrophone();
          const final = {
            ...take,
            blob: new Blob(chunks, { type: recorder.mimeType }),
            ended: Date.now(),
          };
          final.bytes = final.blob.size;
          if (!final.bytes) final.status = "empty";
          this.pending = final.bytes ? final : null;
          await writes;
          try {
            if (final.bytes) await db.put("takes", final);
            else
              await db.event("empty-capture", {
                ...metadata,
                captureId: take.id,
              });
            this.storageError = null;
            this.pending = null;
            final.saved = true;
          } catch (error) {
            this.storageError = error;
            final.saved = false;
            this.change("storage-error", error);
          }
          this.recorder = null;
          this.change(final.bytes ? "captured" : "empty");
          resolve(final);
        };
      });
      this.take = take;
      recorder.start(500);
      this.change("recording");
      return take;
    } catch (error) {
      stream?.getTracks().forEach((t) => t.stop());
      this.recorder = null;
      this.releaseMicrophone();
      this.change("capture-error", error);
      throw error;
    } finally {
      if (token === this.generation) this.requesting = false;
    }
  }
  async stop(complete = true) {
    if (!this.recorder) return null;
    this.take.status = complete ? "complete" : "incomplete";
    if (this.recorder.state !== "inactive") this.recorder.stop();
    // Wait for the final dataavailable/onstop flush before releasing tracks.
    return this.completion;
  }
  async cancel() {
    ++this.generation;
    this.requesting = false;
    this.stopPlayback();
    const take = await this.stop(false);
    this.releaseMicrophone();
    return take;
  }
  async retryStorage() {
    if (!this.pending) return;
    await db.put("takes", this.pending);
    this.pending = null;
    this.storageError = null;
  }
}
