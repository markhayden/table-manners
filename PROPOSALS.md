# Proposals

Changes the tools' conformance work found the contract needs. Each names who found it and the
evidence; none is binding until it lands in CONTRACT.md and the schemas at a tag.

**v0.2 carried C4** (run association and budget sessions, below).

**v0.3 (draft in CONTRACT.md, 2026-11-01)** adopts, decided by the coordinating session that owns
this repository: squint's items 1, 3, 4 and 5 below; Wafflehouse's C1 (readiness and its paid-path
follow-up: configuration lifetime, estimate binding, the per-call cap, and estimates as controls);
C3 (render readiness, framing and lifetime); and option (a) of "a claim released when the request
gets no run". Two choices go further than the proposals, to take tech debt out rather than carry
two shapes: estimates are never runs (squint's zero-spend estimate runs go), and `cost.needs_env`
is replaced by per-action readiness requirements.

**Left for v0.4:** item 2 (the replacement for MCP roots), C2 (invocation snapshots and portable
artifacts), the operations profile (Wafflehouse R3, rascal M10.1), and C1's grouping of tasks,
operations and utilities, which needs that profile's definition of an operation.

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
  (Superseded by point 9 of the tools' review below.)
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

## rascal's and squint's review of C4 (2026-10-31; rascal `f0e45fb`, squint `eef1f10`)

Both tools agree with C4's direction: `_meta`, no MCP revision bump, per-process ledgers, no retry
or job service. rascal coordinates this text; squint's review (its ten points and the refinements
agreed after the owner's read) is folded in below. Nothing here is binding until it lands in
CONTRACT.md and the schemas at a tag.

1. **One run reference.** A shared v0.2 definition, `run_ref`, of the run's identity: `{id,
   resource, session_id, client_run_id}`. Progress `_meta` (`table-manners/run`), runs-list
   entries and the result's `run` all `$ref` it; the result's `run` adds `dir`, required and null when
   the server has no local directory (as in v0.1). `session_id` and `client_run_id` are required
   and nullable, so absence never means "unknown". A run made outside a server session (a tool's
   CLI) has `session_id: null`; a request that sent no correlation id has `client_run_id: null`. The result's `run` has
   `additionalProperties: false` in v0.1, so this is a breaking change, which v0.2 accepts.
2. **A bounded correlation id.** `table-manners/client_run_id` matches `^[A-Za-z0-9._:-]{1,128}$`;
   it lands in files, listings and logs.
3. **A reused id never selects another request.** No run exists for the new request, so the
   refusal is a JSON-RPC error: `-32602` with `data: {"reason": "client_run_id_reused", "run":
   <run_ref>}`, the earlier run's reference. The reason string is part of the contract so hosts
   can tell it from other invalid params. The client reconciles that reference with its own saved
   attempt; it is never the new request's result. The id is claimed atomically before a run is
   allocated, so concurrent submissions of one id execute at most once.
4. **The run is named from allocation, on every notification.** Once the run is allocated, every
   progress notification carries `table-manners/run` in `_meta`, heartbeats included. The run's
   URI leaves the `message` text: there is no dual format.
5. **The run is listed from allocation.** The tool persists the association (session id, client
   run id, action) when it creates the run, before reservation and paid work, so a `running`
   entry in the runs resource carries its reference.
6. **Runs-list pagination.** Paging is in the resource, not MCP's `resources/list`: a templated
   `<tool>://runs{?cursor}` returns a page and its `next_cursor`. The cursor is opaque to
   clients; an absent cursor reads page one, the last page's `next_cursor` is null, and page size
   is the tool's choice. The cursor encodes a position in
   the newest-first order (started, id), not an offset. Runs started after page one was read may
   be missed; re-reading page one is how a client sees them.
7. **Reserve, then settle.** The limits check reserves the call's worst case into
   `reserved_usd` in the same atomic step; the run's end settles it (8). This prevents two
   concurrent calls passing the check against the same `spent_usd`.
8. **Known charges are kept; unknown charges stay unknown.** A provider call is *not sent* (refused
   before its request left the tool: no credential, connection refused), *known* (the provider
   answered: success, authentication rejected, or a failure status that reports usage), or
   *unresolved* (the request was sent and no answer settles it: cancelled or killed mid-call, timed
   out, a failure status whose answer reports no usage). A not-sent call costs nothing and releases
   its reservation. A known call counts the charge its answer reports, or the usage it reports
   (tokens, images) at the tool's per-unit prices; either is `spend.basis: measured`; an answer
   that refuses the request before any work (authentication rejected) counts zero; a successful
   answer that reports neither a charge nor usage counts the tool's configured price for the call,
   and the run's `spend.basis` is then `estimated`. An unresolved call counts its worst case. All
   of this holds whatever the run's status. The result's `spend` adds `unresolved_usd` (the part of
   `usd` not settled by an answer) and the session adds `unresolved_usd` (the part of `spent_usd`),
   so a host shows "at most" rather than zero or a false exact figure.
9. **Responsiveness while a run executes.** `describe`, local estimates, status, the session and
   every resource read, and cancellation MUST stay responsive while a run holds the execution
   permit. Execution concurrency itself does not depend on whether an action costs money.
10. **Queued cancellation.** Waiting for the permit is itself cancellable, and cancellation is
    checked again before allocation. A cancel that lands before allocation leaves no run; once a
    run is allocated, a cancelled run is kept with its spend.
11. **Estimates need not be runs.** rascal's `estimate` writes no run; squint's writes a
    zero-spend run. Both are allowed; an estimate MUST NOT spend. It reports the effective
    per-call cap (the configured cap lowered by `max_spend_usd`) and the same session observation
    as the resource, at a place C1 fixes; tool-private session fields in estimate bodies go away.
    The per-call cap and the session budget are separate limits and both are enforced.
12. **The `<tool>://session` resource.** It carries `contract` and `tool` like every resource.
    `budget_usd` and `remaining_usd` are null only together (uncapped);
    `remaining_usd = max(0, budget_usd - spent_usd - reserved_usd)`; amounts use the result
    spend's rounding and compare to 1e-6 USD. `started` is RFC 3339 UTC like every timestamp;
    `spent_usd` includes `unresolved_usd`. The session id is a random UUID (v4) generated at
    `serve` start; neither tool reloads configuration, so it is fixed for the process.

**Tagging.** table-manners tags v0.2 only after the agreed schemas and representative fixtures
from both tools validate: accepted, reused and mismatched client ids; cancel before and after
allocation; unresolved spend; reserved moving; a session restart; reads while the permit is held.
C4 does not wait on C1: if C1's shapes are not ready, C4 ships as v0.2 and C1 follows in v0.3.

**Agreed for later proposals, not C4.** C1: a credential's readiness is `present`, `valid` or
`unknown` (not checked); finding a key never sets `valid`. C3: the readiness line carries only the
URL, no pid (the host owns the child), and framing is allowed only for one explicit host origin
the host passes (for example `--frame-ancestor http://127.0.0.1:<port>`), never loopback in
general.

## For v0.3: a claim released when the request gets no run (squint's M8 review, 2026-11-01)

**The case.** A client sends `client_run_id` X and the request is refused with a JSON-RPC error
before any run exists: invalid params (a bad job or arguments) or a failed `roots/list`. The
client fixes its arguments and retries with X, and gets `-32602 client_run_id_reused` with
`run: null`. That reads as "the earlier request is still waiting, or was cancelled before a run",
but there never was a run and there will not be one. Both tools claim at acceptance (4.2), so this
is the contract's question, not either tool's bug.

**Options.** (a) Release the claim when the request ends in a JSON-RPC error with no run; a
cancellation keeps it claimed, as 4.2 says a claimed id stays claimed. (b) Keep it claimed and add
a refusal reason or note so `run: null` cannot be read as "still waiting". (c) Document only: an id
is single-use from acceptance, and a client uses a fresh one after any error.

**Proposed: (a)**, squint and rascal agreeing. It is the smallest change, and a client retrying a
corrected request under the same id is the case the id exists for. `run: null` on a reuse refusal
then means only a request still waiting for the permit or one cancelled before allocation.
Fixtures: a refused request whose id is accepted on the retry, and a cancelled one whose id is
still refused.
