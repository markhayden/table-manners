# Proposed for v0.2

Changes the tools' conformance work found the contract needs. Each names who found it and the
evidence; none is binding until it lands in CONTRACT.md and the schemas at a tag. v0.2 is cut before
rascal's conformance merges (CONTRACT.md 1).

## From squint's conformance (M7, 2026-10-31)

1. **An image slot that takes a list.** squint's use cases take any number of reference images;
   v0.1 slots take one image in each, so squint declares `references` and says under `x-squint`
   that it takes a list. Proposal: an image slot's `many: true`, and the MCP input schema an array
   of images in. rascal's ops already have list ports (`port.many`).
2. **MCP roots after `2025-11-25`.** Section 10.1 confines paths to the client's MCP roots;
   SEP-2577 deprecates roots in a later revision (rmcp 3.8.0 warns on `list_roots`). Both tools
   serve `2025-11-25` only, so nothing breaks now, but the path rule needs its replacement written
   before any tool moves revision.
3. **Readiness.** `describe` today needs the tool's configuration to name its credentials; a host
   cannot tell an installed but unconfigured tool from an unavailable one. Proposal (with
   wafflehouse C1): `describe` works without configuration and states what is missing.
4. **Authentication failures are not retryable.** A provider's 401 or 403 and a transient 5xx are
   both `provider`; squint marks every provider failure `retryable`. Proposal: a code or a rule
   that keeps `retryable: false` for credential failures (wafflehouse S1.4).
5. **How a render reports its address.** `renders` gives a command, not how the host learns the
   URL it serves or how long it lives (wafflehouse C3).

## Settled by squint's conformance, no change needed

- Progress as elapsed seconds, nudged so it never repeats, with a heartbeat inside the stated
  interval, works with rmcp and the independent clients of both tools.
- `invalid` and `error` as results with `isError`, JSON-RPC errors only before a run exists.
- A cancelled request gets no response: rmcp drops it; the run's `run.json` says `cancelled`.
