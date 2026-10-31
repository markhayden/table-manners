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

## From Wafflehouse's client review (2026-10-31): run association and budget sessions (C4)

Proposed for v0.2 review, not binding v0.1 fields. Keep MCP revision 2025-11-25. Both current
servers identify a run only in human progress-message text, with different formats; the host
will not parse those messages. Both budgets are in-memory per-process ledgers without a shared
session identity. This proposal adds observation/correlation, not a durable ledger or job service.

### Structured association

- Use MCP's existing notification `_meta` container. Candidate key `table-manners/run` contains
  `{id, resource, session_id, client_run_id}` on the first and later progress notifications once
  a run exists; keep the ordinary strictly increasing progress and human message.
- Accept optional request `_meta["table-manners/client_run_id"]` alongside the progress token.
  Echo it as proposed `run.client_run_id` plus `run.session_id` in terminal results, and corresponding
  fields in run-list entries. Final names and nullable/CLI cases need shared schema approval.
- Persist run/correlation before paid work; reject reused non-null client IDs within a server
  session before another execution. This is not cross-process exactly-once execution.
- Resource lookup after a lost notification may use the explicit session/client-ID pair only;
  never action, timestamp or path similarity. Pre-allocation cancellation may still have no run,
  and absence from a recent-run list does not prove zero work/spend. No automatic retry.
- rmcp 3.8.0 ProgressNotificationParam already has metadata support. No MCP version bump or
  private human-message parser is required. Cancelled calls still return no ordinary result.

### Observable session budget

- Proposed declared resource `<tool>://session`: `{id, started, budget_usd, spent_usd,
  reserved_usd, remaining_usd}`. Define numeric/nullable rules and rounding tolerance in a shared
  schema. Null budget/remainder denotes documented uncapped/free sessions, not zero allowance.
- Session IDs change on every server start and any ledger reset. Return an atomic observation;
  status/session/run-resource reads stay responsive while execution occupies the serial permit.
- Give estimates shared session/budget context outside private result bodies, with its placement
  agreed alongside C1's estimate mapping. The tool's cap/reservation enforcement is authoritative.
- Squint currently serializes estimates and writes zero-spend estimate run resources. That is
  allowed; clients can treat their own estimate calls as controls rather than user history.
- Host config recheck must not silently restart a server. Deliberate restart invalidates estimates
  and shows a new allowance session. Unknown spend from an old interrupted session stays unknown.
  A process allowance is not a daily budget or provider account balance.

### Acceptance fixtures

Paired Rascal/Squint examples and independent-client checks for structured first/later progress,
correlation match/mismatch/reuse, cancel before/after allocation, no response with cancelled run,
missing resource, live resource reads while busy, estimate session identity, reservations,
server restart and config-ledger reset. Existing Squint cancelled fixtures plus the v0.1 text
are the terminal-shape reference; do not invent a Rascal fixture and claim tool provenance.
No live paid calls are needed. This proposal does not settle native artifacts (C2), reports (C3),
or the future replacement for roots after the pinned MCP revision.

### Separate host-owned coordination (H1)

Wafflehouse has also drafted the launcher ABI now, not at its later integration milestone:
canonical `wafflehouse`, `WAFFLEHOUSE_BIN` override, PATH then `~/.local/bin/wafflehouse`, and
`--tool <id>`. Tools locate/install/invoke only; Wafflehouse owns locks, attach/readiness,
browser bootstrap and opening. Review that owner packet independently of the core schemas;
production installer publication remains a separate release subgate.

## rascal's review of C4 (2026-10-31, rascal `f0e45fb`; revised after Wafflehouse's reply)

rascal agrees with C4's direction: `_meta`, no MCP revision bump, per-process ledgers, no retry or
job service. rascal already allocates the run and sends its resource in the first progress message
before the limits check and any filler call (`actions.rs` `run_action`); a cancel while waiting for
the call permit leaves no run. The amendments below are what rascal would implement; squint's
review decides the names with it.

1. **One run reference.** A shared definition of the run's identity, `{id, resource, session_id,
   client_run_id}`, used by `table-manners/run` in progress `_meta`, run-list entries and the
   result's `run`. The result's `run` adds `dir`, optional, as squint proposes: a remote server has
   no local directory. `session_id` and `client_run_id` are required and nullable.
2. **A bounded correlation id.** `table-manners/client_run_id`: 1 to 128 characters of
   `[A-Za-z0-9._:-]`; it lands in files, listings and logs.
3. **A reused id never selects another request.** The rejection is squint's structured
   `client_run_id_reused` error, a JSON-RPC error before any run exists, and its `data` carries the
   earlier run's reference. The client reconciles that reference with its own saved attempt; it
   is never the new request's result. The id is claimed atomically before a run is allocated, so
   concurrent submissions of one id execute at most once.
4. **The run is listed from allocation.** A run is in the runs resource with state `running` and
   its reference from the moment the first notification names it, before paid work.
5. **Reserve, then settle.** The limits check reserves the call's worst case into `reserved_usd`
   in the same atomic step; the run's end settles it (6). rascal has the race this prevents
   (two calls under `--concurrency 2` both pass the check against the same `spent`).
6. **Known charges are kept; unknown charges stay unknown.** A provider call is *not sent*
   (refused before its request left the tool: no credential, connection refused, authentication
   rejected), *known* (the provider answered), or *unresolved* (the request was sent and no
   answer settles it: cancelled or killed mid-call, timed out, a failure status). A not-sent call
   costs nothing and releases its reservation; known and unresolved calls count their worst case
   whatever the run's status. Proposal: the result's `spend` adds `unresolved_usd` (the part of
   `usd` not settled by an answer) and the session adds `unresolved_usd` (the part of `spent_usd`),
   so a host shows "at most" rather than zero or a false exact figure. rascal fixes its own
   accounting now and reports the unresolved part in its body until v0.2 names the field.
7. **Reads never wait on the execution permit.** `describe`, `estimate` and every resource read,
   `<tool>://session` included, MUST answer while a run holds the permit. Recommended over
   "estimates may be serialized": the host then needs no rule about when it may ask.
8. **Estimates need not be runs.** rascal's `estimate` writes no run; squint's writes a zero-spend
   run. Allow both; an estimate MUST NOT spend and carries the same `session` object as the
   resource, at a place C1 fixes in the estimate's result.
9. **Session schema details.** `budget_usd` and `remaining_usd` are null together when uncapped;
   `remaining_usd = max(0, budget_usd - spent_usd - reserved_usd)`; amounts are compared to
   1e-6 USD. rascal reads its configuration once at `serve` start, so its session id is per
   process; a reload, if one is added, changes it.

Fixtures: rascal produces its own cancelled and unresolved-spend runs from its independent client,
alongside squint's; no paid calls.
