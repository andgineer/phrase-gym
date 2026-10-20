# Rust preview access

Server and Coach currently use no application authentication, tokens, credential files or login cookies. Social sign-in is deferred. The local machine and devices permitted by the existing Tailscale network policy are trusted operators.

Build the retained client, then launch with a disposable data directory:

```sh
npm --prefix frontend run build
cargo xtask run --data-dir /private/tmp/phrase-gym-preview
```

Open `http://127.0.0.1:8010/learn/en`. The equivalent binary command is `phrase-gym-server serve --data-dir /private/tmp/phrase-gym-preview --client-dir frontend/dist --port 8010`. Coach defaults to port 8000; use `cargo xtask run --port 8000` for its ordinary automatic local connection, or save the desired origin in Coach's **Server connection** settings.

## Tailscale connection

For an **already configured** tailnet-only Tailscale Serve HTTPS proxy forwarding to this Server's loopback port, pass its exact origin:

```sh
cargo xtask run --port 8000 --tailscale-origin https://machine.tailnet.ts.net
```

Save that same URL in Coach. There is no secret to generate, copy or enter. The option only allows the exact origin; it does not configure Tailscale, change its policy or deploy anything. Tailscale must already be connected and Serve must forward to `http://127.0.0.1:8000`. Setting up or changing that forwarding requires separate operator authorization.

Server always listens on `127.0.0.1`, validates the actual socket peer and never accepts forwarded address headers as network identity. Remote access relies on [Tailscale Serve](https://tailscale.com/docs/features/tailscale-serve) and the tailnet access policy. Use no public proxy, public bind, port forward or Funnel. Serve [removes client-supplied identity/Funnel headers and marks Funnel traffic](https://github.com/tailscale/tailscale/blob/main/ipn/ipnlocal/serve.go); Server rejects that marker, including with a forged user header. Tagged tailnet devices do not need a user-login header. The local machine and processes are inside this trust boundary, and its operator attribution does not identify an individual user.

Before accepting remote operation, inspect the existing Serve configuration, verify its exact HTTPS origin and loopback target, confirm Funnel is disabled, exercise browser and Coach access from an allowed tailnet device, and verify denial from a device outside the tailnet. Automated local socket/router tests establish fail-closed application behavior; they do not certify a machine's real network configuration. No actual remote deployment or network changes are implied by these commands.

The initial profile records the [specified learner preferences](../specs/decision-personal-preferences.md). Profile mutation/restore remains an explicit operator workflow using stable mutation IDs and expected revisions. Preparation model tools receive only bounded attempt files and cannot call profile or recording APIs. All trusted operator API clients share access in this network-only mode; fine-grained application identity is deferred with social sign-in.

The generated Rust contract includes `PreparedEnglishLesson` for one lesson of one to five short fixed English phrases. It pins a personal-profile revision plus guide and assessment versions, groups stable source identities, supplies one or two situations, optional partner turns, one partial hint and original reference media. The core validator checks structure, identity/revision preconditions, supplied-audio integrity and exact role/text/language bindings. It rejects gapped fields, complete/accepted answers in prompts or lesson titles, unresolved English preferences and unsupported kinds. These checks do not certify semantic prompt quality or language usage. The production English inbox and fenced atomic publication path is implemented. These commands establish Server access. Remaining Coach integration and generated-lesson acceptance are tracked in the [implementation plan](../specs/plan/implementation-plan.md#m2-generated-lesson-demo-on-local-macos).
