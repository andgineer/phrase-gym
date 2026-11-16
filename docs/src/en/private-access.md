# Private Tailscale access

With Tailscale already connected, from a source checkout run `uv run inv run --build --tailscale`. The task points the existing
HTTPS Serve route at `127.0.0.1:8000` and passes the exact machine origin to Phrase Gym.
After that setup, plain `uv run inv run` (or `inv run` in the activated environment)
automatically reuses the existing private HTTPS route, including after a restart.
Add `--build` when the browser client needs rebuilding. Without a matching Serve route,
`inv run` remains local-only; it does not create or change Tailscale routes automatically.
The backend remains on loopback; only devices permitted by your Tailscale policy can use this
shared library. This does not enable public Funnel access.

Use `tailscale serve status` to inspect the route. Stop an old Phrase Gym process before starting
the replacement; retain its data directory and recordings. On iPhone, use the HTTPS machine URL
with Tailscale connected. Microphone permission and actual playback still need attended checks.
