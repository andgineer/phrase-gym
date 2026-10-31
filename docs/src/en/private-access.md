# Private Tailscale access

With Tailscale already connected, from a source checkout run `uv run inv run --build --tailscale`. The task points the existing
HTTPS Serve route at `127.0.0.1:8000` and passes the exact machine origin to Phrase Gym.
The backend remains on loopback; only devices permitted by your Tailscale policy can use this
shared library. This does not enable public Funnel access. Run Coach with `PHRASE_GYM_PIPER_CONFIG` when preparing text-only material. The server starts
without Piper and continues to serve study and replay while Coach is stopped.

Use `tailscale serve status` to inspect the route. Stop an old Phrase Gym process before starting
the replacement; retain its data directory and recordings. On iPhone, use the HTTPS machine URL
with Tailscale connected. Microphone permission and actual playback still need attended checks.
