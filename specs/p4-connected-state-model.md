# P4 connected state and resource ownership

Current contract: [online-only](decision-online-only.md), accepted 2026-10-13. The [earlier offline audit](p4-state-model.md) is historical; its pack/projection/lifecycle model is withdrawn.

| State/resource | Owner and invariant |
|---|---|
| Session/card/support/queue/results | Actual server reads and receipts. Views map acknowledged attempts; no predicted state or cached outage restoration. |
| Initiated event/start/transition | IndexedDB journal, original ID/payload/context. Receipt checkpoint precedes deletion. Conflict evidence is retained. |
| Lookup exposure | Separate localStorage key per original event ID/payload; server receipt precedes deletion. |
| Pending take | IndexedDB plus application-lifetime volatile retention on quota failure. Each take has its own ID and initiating context. |
| Take deletion | Conditional transaction requires exact acknowledged ID/hash/size/completeness, protecting newer snapshots. |
| Capture | Exclusive lease and resource-affine cleanup; late permission releases its stream. Stop/cancel remains usable while normal interaction is hidden. |
| Connection error | Small API error flag and retry composable. Signals request recovery; actual writes/reads establish usable state. No per-action health probe. |
| Retry | Resend original writes, read actual session/settings/library, recover pending transfers. Failed recovery keeps normal UI blocked. |
| Earlier shell/media/pack | Owned cleanup only. Retirement never wipes evidence, reloads active audio or enables an offline trainer. |

Previously acknowledged state can remain in memory for capture cleanup and pinned references, but cannot authorize learning while server recovery is blocked. Terminal capture/failure evidence can finish an already initiated operation; it is not a new grade. Quota failure must remain visible and available audio stays retained in the open application. No eviction-survival, arbitrary simultaneous-device or seamless outage guarantee is asserted.

Focused composables separate session/events/lifecycle from media/capture and root connection handling. The server retains its existing atomic review transaction and normal repair policy. Narrow historical payload decoding preserves receipt identity only for earlier planners. New API contracts have no offline deferral option.
