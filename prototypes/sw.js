const CACHE = "phrase-gym-p1-v7";
const SHELL = [
  "/drill-flow/index.html",
  "/drill-flow/app.mjs",
  "/drill-flow/core.mjs",
  "/drill-flow/i18n.mjs",
  "/drill-flow/fixtures.json",
  "/shared/audio.mjs",
  "/shared/storage.mjs",
  "/shared/style.css",
  "/shared/icon.svg",
  "/audio-probe/index.html",
  "/audio-probe/probe.mjs",
  "/manifest.webmanifest",
];
self.addEventListener("install", (event) =>
  event.waitUntil(
    caches
      .open(CACHE)
      .then((cache) => cache.addAll(SHELL))
      .then(() => self.skipWaiting()),
  ),
);
self.addEventListener("activate", (event) =>
  event.waitUntil(self.clients.claim()),
);
self.addEventListener("fetch", (event) => {
  const url = new URL(event.request.url);
  if (url.origin !== location.origin || event.request.method !== "GET") return;
  event.respondWith(
    (async () => {
      const cache = await caches.open(CACHE);
      try {
        return await fetch(event.request);
      } catch {
        const cached = await cache.match(event.request, { ignoreSearch: true });
        if (cached) return cached;
        if (event.request.mode === "navigate")
          return cache.match(
            url.pathname.startsWith("/audio-probe")
              ? "/audio-probe/index.html"
              : "/drill-flow/index.html",
          );
        return new Response("Prepared asset unavailable offline", {
          status: 503,
        });
      }
    })(),
  );
});
