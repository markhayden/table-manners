# table-manners, v0.2.0

Status: v0.2.0, ready to tag, 2026-11-01. v0.1.0 (2026-10-30) plus run association and the
session ledger, as squint and rascal agreed them (PROPOSALS.md, C4; "Changes from v0.1" below).
Both tools' fixtures validate (7); field names and the schemas are binding at the `v0.2.0` tag.

table-manners is the contract the waffle tools keep at a shared table: one external interface for
squint (checks an image against what was asked), rascal (finds, masks and composites regions),
and the tools after them. Each tool keeps its own intent, internals and vocabulary. What an agent,
a script or the playground sees from outside is the same in every tool, so tools chain in one
pipeline without glue code:

```
generate ──► squint check ──► rascal remove ──► filler paints ──► squint check
   (any)        finds "the bird is here"   masks it     inside the mask    with the mask as the
                                                                           region change was allowed
```

The contract has a **core** every tool follows (sections 1 to 7) and **profiles** a tool adds
for the kind of thing it works on. v0.1 has one profile, **images** (sections 8 to 12), which
squint and rascal both implement. A tool that works on text, audio or code follows the core and
needs no profile until one is written for it, against a real tool.

The words MUST, MUST NOT, SHOULD and MAY are used as in RFC 2119.

Sources: squint ADR 0018 and SPEC "External interface"; rascal's seed section in
`../playground-recommendation.md`; rascal ADR 0005 (surfaces) and ADR 0015 (MCP transport).

# Part 1: the core

## 1. Scope, versioning and profiles

- The core defines the **result** (2), the **description document** (3), **long runs** (4)
  with the **session** (4.6), the **MCP server conventions** (5), **exit codes** (6) and what a conforming tool provides (7). The
  images profile defines the **image reference** (8), the **location** (9), how images enter and
  leave a tool (10), and what it adds to the result and the description (11).
- A **profile** is a named set of additions to the core's objects. A tool lists the profiles it
  implements in its description (`profiles`); it MUST include every field a listed profile
  requires and MUST NOT emit a profile's fields without listing it. A new profile is a contract
  change, written when the first tool that needs it exists.
- The contract does not define what a tool does, its own files, its journal's lines, its config
  format or its internal frames. A tool MAY add fields in its own namespace (the result `body`,
  `x-<tool>` keys in the description); it MUST NOT add fields to the shared objects.
- The JSON Schemas under `schemas/` are the source of truth; this text explains them.
  `schemas/core/` holds the core's and `schemas/images/` the images profile's. Each schema's `$id`
  is `urn:table-manners:<dir>:<name>` and references between them use those ids. The Rust crate
  `table-manners` holds serde types matching them and embeds them (`SCHEMAS`), with no logic.
  `examples/` holds valid and invalid documents for each schema; the crate's tests check both the
  schemas and the types against them.
- `MAPPING.md` lists what squint and rascal change to conform, field by field.
- Releases are tagged `vMAJOR.MINOR.PATCH`. Before 1.0 a minor release may break; after 1.0 only a
  major one may. Every result and description carries `contract` (the version the tool conforms
  to). A tool pins one release, keeps a hash-checked copy of its schemas, and SHOULD warn in CI
  when a newer tag exists; an update is a deliberate commit in the tool.
- v0.1 was the minimum the chain needs. v0.2 adds what squint's conformance and the playground's
  review found about long paid runs: a run a client can correlate, and a spend ledger it can
  observe. The other proposals in `PROPOSALS.md` follow in later minors.

## 2. The result

One shape everywhere: the MCP tool result's `structuredContent`, the run resource, and the file
`run.json` written last into every run directory. `run.json` is written atomically (to a temporary
name in the same directory, then renamed), because the playground and a retrying agent read it
while the tool may still be running. A **header** the contract owns and a **body** the tool owns.

```jsonc
{
  "contract": "0.2.0",
  "tool": { "name": "squint", "version": "0.2.0" },
  "action": "check",
  "run": { "id": "decision-1793386884140-53022", "resource": "squint://run/decision-…/run.json",
           "session_id": "0b7e3c5a-6f1d-4c2e-9a8b-3d4e5f6a7b8c", "client_run_id": "host-7f3a.attempt:1",
           "dir": "/abs/squint-out/decision-…" },
  "status": "completed",
  "error": null,
  "started": "2026-10-30T19:09:42.118Z",
  "finished": "2026-10-30T19:10:21.904Z",
  "duration_ms": 39786,
  "input_identity": "sha256:…",
  "spend": { "usd": 0.0389, "unresolved_usd": 0, "basis": "measured" },
  "cache": "miss",
  "notes": [ "repair: the first submission omitted item C3; repaired in one turn" ],
  "body": { …the tool's own… }
  // the images profile adds "images" and "locations" (11.1)
}
```

### 2.1 Status

| `status` | meaning | the agent should |
|---|---|---|
| `completed` | the action ran to its end | read the body; a squint `fail` verdict is `completed` |
| `cancelled` | stopped by the caller or a signal | read what is there; spend to date is reported |
| `error` | an operational failure | read `error.code` and `error.retryable` |
| `invalid` | the request was wrong | fix the request; never retry it unchanged |

### 2.2 Error

Present exactly when `status` is `error` or `invalid`, and null otherwise:

```jsonc
{ "code": "provider", "retryable": true, "reason": "the provider answered 529 three times" }
```

| `code` | typical status | meaning |
|---|---|---|
| `usage` | invalid | an unknown action, a missing or malformed argument |
| `input` | invalid | an input the tool cannot take; for the images profile a digest mismatch, a path outside roots or over a network, a size over the limit, a location on another image |
| `io` | error | reading or writing the tool's own files failed |
| `provider` | error | a remote model or service failed (a judge model, a filler endpoint) |
| `budget` | error | the per-call cap or the session budget would be exceeded |
| `timeout` | error | a bounded wait ran out |
| `tool_process` | error | a helper process (a tool pack, a filler command) died, hung or answered out of contract |
| `internal` | error | a bug in the tool |

`retryable` is the tool's judgement for this occurrence (a 529 is, a 401 is not).

### 2.3 The rest of the header

- `run` is the run's reference (4.3) plus `dir`: `run.id` is unique per tool, `run.resource`
  names the run resource, `run.session_id` the serving session (4.6) and `run.client_run_id` the
  caller's correlation id (4.4). `run.dir` is the run directory when the transport is local, else
  null. All five are required; the nullable ones are null, never absent.
- `input_identity`: a digest the tool computes over its resolved inputs and parameters, so two
  results with equal identities ran the same thing. Its recipe is the tool's.
- `spend.usd` is what this run cost by the accounting in 4.5, including a cancelled or failed
  run's spend to date; 0 for a cache hit. `spend.unresolved_usd` is the part of it no provider
  answer settled: an upper bound, so a client shows "at most", never zero or an exact figure.
  `unresolved_usd` never exceeds `usd`.
  `spend.basis` describes the settled part: `measured` when every known call's figure came from
  its answer (a charge, or usage priced per unit), `estimated` when any known call counts a
  configured price. The unsettled part is marked by `unresolved_usd`, not by `basis`.
- `cache`: `hit`, `miss` or `off`.
- `notes`: human sentences for caveats on a run that still completed (a repair, a tool's refusal,
  a location clipped at the edge, an override applied).
- Outputs a profile adds MAY be present on `error` and `cancelled`, and a tool MUST NOT drop an
  output it completed because a later step failed: an agent can salvage a mask and call a
  different filler.

## 3. The description document

`<tool> describe --json` prints it, and the MCP tool `describe` returns it.

```jsonc
{
  "contract": "0.1.0",
  "profiles": ["images"],
  "tool": { "name": "squint", "version": "0.2.0", "purpose": "Checks an image against what was asked of it." },
  "actions": [
    {
      "name": "check",
      "title": "Source fidelity",
      "purpose": "…",
      "params_schema": { …JSON Schema… },
      "cost": { "may_spend": true, "basis": "measured", "needs_env": ["GEMINI_API_KEY"] },
      "duration": { "typical_s": 40, "max_s": 600 },
      "renders": [ { "name": "report", "command": ["squint", "report", "view", "{run.dir}", "--port", "0"] } ]
      // the images profile adds "images", "locations" and "outputs" (11.2)
    }
  ],
  "progress_interval_s": 10,
  "resources": [ "squint://runs{?cursor}", "squint://run/{id}/run.json", "squint://run/{id}/journal",
                 "squint://session" ],
  "x-squint": { }
  // the images profile adds "vocabulary" and "limits" (11.2)
}
```

- `profiles` lists the profiles the tool implements; `[]` for a core-only tool.
- `params_schema` is a JSON Schema for the action's arguments other than those a profile adds.
- `needs_env` names variables; a tool MUST NOT print their values. A client shows whether each is
  set. `cost.basis` is required when `cost.may_spend` is true.
- `duration` lets a client set its timeout up front, and `progress_interval_s` (at most 10) is the
  longest gap between progress notifications on a live run (4.1).
- Action names are lowercase with underscores (`check`, `seam_heal`); `describe` is reserved.
- `renders` are commands that show a run to a person; `{run.dir}` is substituted.

## 4. Long runs

### 4.1 Progress is a heartbeat

An action is a plain MCP `tools/call`. When the call carries a progress token, the tool sends
`notifications/progress` on every step it journals and at least every `progress_interval_s`
(at most ten seconds) while it waits, so a client that resets its timeout on progress never cuts
off a live run. The payload is plain: `progress`, `total` where known, and a one-line `message`
("turn 2: waiting on the provider, 42 s, $0.11 so far"). The rich timeline is the journal
resource, not the progress stream.

MCP requires `progress` to increase on every notification, and a client may not reset its timeout
on a repeated value. A run with a step count reports steps done; a heartbeat between steps, or a
run with no step count, reports elapsed seconds with `total` omitted, and the words go in
`message`. A tool never resends the same `progress` value.

From the moment the run is allocated, every progress notification, heartbeats included, carries
the run's reference (4.3) in its `_meta` under `table-manners/run`:

```jsonc
{ "progressToken": 7, "progress": 3, "message": "turn 1: waiting on the provider, 12 s",
  "_meta": { "table-manners/run": { "id": "decision-…", "resource": "squint://run/decision-…/run.json",
                                    "session_id": "0b7e3c5a-…", "client_run_id": "host-7f3a.attempt:1" } } }
```

A notification before allocation (a call waiting for the execution permit, 4.5) carries none. The
run is never named in `message`, which is for people: a client reads `_meta`, never the words.

MCP tasks (experimental in `2025-11-25`, moving to an extension) are not used in v0.2.

### 4.2 Cancellation

On `notifications/cancelled` (or SIGINT or SIGTERM on the command line) a tool stops its paid work
at once, writes the run resource and `run.json` with status `cancelled` and spend to date, and
returns no MCP result (the protocol says a cancelled request gets none).

Waiting for the execution permit is itself cancellable, and a tool checks for cancellation again
just before it allocates the run. A cancel that lands before allocation leaves no run and no
spend. Once a run is allocated, a cancelled run is kept, with its spend to date (4.5).

### 4.3 Runs are resources

Every run a server performs is readable by id: `<tool>://run/<id>/run.json` at least, and
`<tool>://run/<id>/journal` for the journal. An agent whose call timed out or whose connection
dropped reads the resource instead of calling again; the playground reads the same resources for
its history.

A run's **reference** (`schemas/core/run-ref.json`) is its identity wherever it appears: progress
`_meta` (4.1), the result's `run` (which adds `dir`, 2.3), runs-list entries and a reuse refusal
(4.4). It holds `id`, `resource`, `session_id` and `client_run_id`, all required. A run made
outside a server session (a tool's CLI) has `session_id: null`; a request that sent no
correlation id has `client_run_id: null`.

A tool persists the reference, with the action, when it allocates the run, before it reserves
spend or does paid work. From then the run is listed as `running`, its journal is readable, and
its `run.json` exists once it has ended.

`<tool>://runs{?cursor}` lists runs newest first, ordered by (`started`, `id`), one page at a
time: each entry is the run's reference with `action`, `status` and `started`, and the page
carries `next_cursor`. An absent cursor reads page one; the next page is
`<tool>://runs?cursor=<next_cursor>`; the last page's `next_cursor` is null. The cursor is opaque
to clients and encodes a position in that order, not an offset; page size is the tool's choice.
The cursor uses only URI-unreserved characters (`[A-Za-z0-9._~-]`, at most 256), so it goes into
the query unencoded.
Runs started after page one was read may be missed by later pages; re-reading page one is how a
client sees them. MCP's `resources/list` paging does not page entries inside this resource.

A client that lost a call's notifications finds its run by the pair (`session_id`,
`client_run_id`) only, never by action, time or path. Absence from the list does not prove that
no work or spend happened: a cancel before allocation leaves no run, and a session that ended
leaves its runs but not its ledger.

### 4.4 Retries do not pay twice

A tool with a paid step is idempotent on identical inputs within its cache policy: an identical
retry is answered from the cache with `cache: "hit"` and `spend.usd: 0`.

A caller MAY correlate a call by sending `_meta["table-manners/client_run_id"]` in the
`tools/call` request beside its progress token: 1 to 128 characters of `[A-Za-z0-9._:-]`, since it
lands in files, listings and logs. The tool echoes it in the run's reference. Within a session a
non-null id is used at most once:

- The tool claims the id atomically when it accepts the request, before the request waits for the
  permit, so concurrent requests with one id execute at most once.
- A request whose id is already claimed is refused before any run exists, as a JSON-RPC error:
  `-32602` with `data` per `schemas/core/refusal.json`, `{"reason": "client_run_id_reused",
  "run": <the earlier request's run reference>}`. `run` is null while the earlier request has no
  run (it is still waiting, or it was cancelled before allocation).
- A claimed id stays claimed for the session even when its request is cancelled before
  allocation; a client that retries such a request uses a new id.
- The refusal never stands for the new request's result. The client reconciles the earlier run
  with its own saved attempt; it does not retry automatically.

This is not exactly-once execution across processes: a new session forgets the ids it never saw.

### 4.5 Spend limits

- A per-call cap in the tool's config. A call MAY lower it (`max_spend_usd`); it MUST NOT raise it.
- A per-session budget in the tool's config, across every call the server process serves.
- Both limits are enforced. The limits check **reserves** the call's worst case into the
  session's `reserved_usd` in the same atomic step, so two concurrent calls never pass the check
  against the same spend; the run's end **settles** the reservation into `spent_usd`.
- Every provider call is one of three:
  - *not sent*: refused before its request left the tool (no credential, connection refused). It
    costs nothing and releases its reservation.
  - *known*: the provider answered (success, authentication rejected, or a failure status that
    reports usage). It counts the charge its answer reports, or the usage it reports (tokens,
    images) at the tool's per-unit prices; either is `measured`. An answer that refuses the
    request before any work (authentication rejected) counts zero. A successful answer that
    reports neither a charge nor usage counts the tool's configured price for the call, and the
    run's `spend.basis` is then `estimated`.
  - *unresolved*: the request was sent and no answer settles it (cancelled or killed mid-call,
    timed out, a failure status whose answer reports no usage). It counts its worst case, and the
    amount is also counted in `unresolved_usd`.

  All of this holds whatever the run's status. Unknown spend is never reported as zero and its
  reservation is never released.
- Every result reports spend (2.3). A tool that may spend provides an `estimate` action. An
  estimate MUST NOT spend; it reports the worst case for the given call and the effective per-call
  cap (the configured cap lowered by `max_spend_usd`). An estimate MAY be a run (squint's is, with
  zero spend) or not (rascal's). Where the session observation sits in an estimate's result is
  fixed with readiness (PROPOSALS.md, C1); until then a client reads `<tool>://session`, and a
  tool carries no private copy of the session in its estimate body.
- Calls are serial by default; a tool MAY allow more with a concurrency setting. Execution
  concurrency does not depend on whether an action costs money. While a run holds the execution
  permit, `describe`, local estimates, every resource read (runs, a run, its journal, the
  session) and cancellation MUST stay responsive: none of them waits for the permit.
- A per-period budget (a day, a month) is later work: it needs a durable ledger across processes.

### 4.6 The session

A **session** is one serving process's spend ledger. `<tool>://session`
(`schemas/core/session.json`) returns one atomic observation of it:

```jsonc
{ "contract": "0.2.0", "tool": { "name": "rascal", "version": "0.1.0" },
  "id": "0b7e3c5a-6f1d-4c2e-9a8b-3d4e5f6a7b8c", "started": "2026-10-31T18:02:11.530Z",
  "budget_usd": 2.0, "spent_usd": 0.16, "unresolved_usd": 0.04, "reserved_usd": 0.04,
  "remaining_usd": 1.8 }
```

- `id` is a random UUID (v4), lowercase, generated when `serve` starts. A tool that reloads its
  configuration or resets its ledger starts a new session with a new id; neither squint nor rascal
  does today, so their sessions last the process.
- `started` is RFC 3339 UTC, like every timestamp.
- `spent_usd` is settled spend and includes `unresolved_usd`; `reserved_usd` is the worst cases
  held by runs still going.
- `budget_usd` and `remaining_usd` are null together, exactly when the session is uncapped; null
  is never a zero allowance. `remaining_usd = max(0, budget_usd - spent_usd - reserved_usd)`.
- Amounts are US dollars. Tools write them with at most six decimals and clients compare them to
  1e-6.
- A session is not a daily budget or a provider account balance. When a server restarts, a client
  sees a new id; what an interrupted session left unresolved stays unresolved in its runs.

## 5. MCP server conventions

Lifted from `rascal serve` (rascal ADR 0015).

- `<tool> serve` speaks MCP over stdio at protocol revision **`2025-11-25`** only, and refuses a
  client that cannot meet it at `initialize`. Moving revision is a contract change.
- Capabilities: `tools` (with pagination), `resources` (run resources), progress, cancellation.
- One MCP tool per action in the description, plus `describe`; each tool's input schema is its
  action's `params_schema` with the arguments its profiles add. A server MUST NOT require an
  injected context object: a client needs nothing but the description.
- Once a run exists, every outcome is an MCP tool result carrying the header and body as
  `structuredContent`: `isError: false` for `completed`, `isError: true` for `invalid` and
  `error`. The header is the point: spend to date, `retryable` and salvaged outputs (2.3) travel
  only in a result, and a JSON-RPC error reaches a model client as an opaque failure.
- JSON-RPC errors are reserved for failures before a run exists: an unknown tool, arguments the MCP
  layer rejects against the input schema, a reused `client_run_id` (4.4), the server itself
  failing. A refusal the contract names carries `data.reason` (`schemas/core/refusal.json`) so a
  host can tell it from other invalid params.
- The contract's request and notification metadata lives in MCP's `_meta` under keys prefixed
  `table-manners/` (`table-manners/run`, `table-manners/client_run_id`), so it needs no MCP
  revision of its own.
- Rust tools SHOULD use the official SDK, `rmcp`, pinned exactly; rascal and squint pin `=3.8.0`.
  A tool in another language uses its official SDK at the same revision.
- Every tool's test suite drives its server with an independently written client that shares no
  code with the server or the SDK. rascal's (`rascal_testkit::mcp_client`) is the reference.

## 6. Exit codes

| code | meaning |
|---|---|
| 0 | success (`completed`, and the tool's own success when it distinguishes one) |
| 3 | `error` |
| 64 | any `invalid` (`usage` or `input`; the result's `error.code` tells them apart) |
| 130 | cancelled by SIGINT |
| 143 | cancelled by SIGTERM |
| 1, 2 | the tool's own (squint: fail, unsure; rascal: nothing found under `--strict`) |

## 7. Conformance

A conforming tool provides, at the contract version it names:

1. `describe --json` and the `describe` MCP tool (3), validated against the schema in its tests.
2. `serve` (5) with an action per description entry, progress with the run reference (4.1),
   cancellation (4.2), run resources and the paged runs list (4.3), correlation ids (4.4), spend
   limits with reservation and the three-way accounting where it spends (4.5), and the session
   resource (4.6).
3. `run.json` (2), written atomically, in every run directory, on every surface, including
   cancelled and failed runs.
4. The exit codes (6).
5. A `playground` subcommand that finds, offers to install and starts the shared playground; it
   carries no UI.
6. Every profile it lists, conforming (12 for images).
7. The conformance fixtures in `fixtures/<tool>/` passing in its CI against its pinned release:
   `describe.json`, `run-*.json` results, `runs*.json` pages, `session.json` and `refusal-*.json`
   reuse refusals. A release is tagged only when every tool's fixtures validate against it. For
   v0.2 they include accepted, reused and mismatched client ids, cancels before and after
   allocation, unresolved spend, a reservation in flight, a restarted session and a read while
   the permit is held.

## Changes from v0.1

Breaking, as a minor before 1.0 may be (1):

- The result's `run` adds `session_id` and `client_run_id` (required, nullable); runs-list entries
  are run references with `action`, `status` and `started`; the runs list is paged and adds
  `next_cursor` (4.3).
- `spend` adds `unresolved_usd` (required), and `measured` covers usage priced per unit (2.3, 4.5).
- Progress names the run in `_meta`, not in `message` (4.1).

Added: the run reference (`run-ref.json`), correlation ids and the reuse refusal (`refusal.json`,
4.4), reservation and the three-way accounting (4.5), responsiveness while a run executes (4.5),
cancellable permit waits (4.2), and the session (`session.json`, 4.6).

# Part 2: the images profile

For tools that take and return images. squint and rascal implement it.

## 8. The image reference

How a tool names an image to another tool. Tools never share ids, workspaces or frame names.

```jsonc
{
  "file_sha256": "3f9a…",            // REQUIRED: SHA-256 of the file's bytes, lowercase hex
  "width": 2048,                     // REQUIRED: pixels as stored (EXIF orientation not applied)
  "height": 1024,                    // REQUIRED
  "projection": "equirectangular",   // REQUIRED: "flat" | "equirectangular"
  "topology": "wrap",                // REQUIRED: "bounded" | "wrap"
  "pixel_sha256": "a1c4…"            // OPTIONAL: canonical pixel digest (8.2)
}
```

### 8.1 The file digest is the check

- `file_sha256`, `width` and `height` describe the file the caller holds. A receiver given a
  reference with an image MUST verify all three and MUST refuse a mismatch (`invalid`, code
  `input`).
- The first tool in a chain computes the reference. Every image a tool returns carries one, so an
  agent always has a reference to pass on, and later calls carry it as the expected fingerprint.
- A tool that converts an image internally (rascal stores a canonical PNG) MUST still report the
  digest of the file it was given for that input, and the digest of the file it wrote for each
  output.
- `topology` is separate from `projection` because a flat image can wrap (a tiling texture, a
  strip cut from a panorama). An equirectangular image is always `wrap`.

### 8.2 The pixel digest is a convenience

- Canonical form: decode the file; keep 8-bit grey (1 channel), 8-bit RGB (3) and 8-bit RGBA (4)
  as decoded, and convert every other layout to 8-bit RGB. No colour-profile conversion, no EXIF
  orientation; rows top to bottom, pixels left to right, channels interleaved.
- Digest: SHA-256 over the ASCII bytes `pixels`, one zero byte, then width, height and channel
  count as little-endian u32, then the samples. `pixel_sha256` is its lowercase hex. This is
  rascal's existing recipe (rascal ADR 0016), byte for byte, so its identities and fixtures stand.
- Decoders MAY disagree on lossy formats (JPEG chroma upsampling and IDCT differ by a level between
  libraries). A receiver whose `pixel_sha256` differs while `file_sha256` matches MUST NOT treat it
  as an error; it MAY add a note.
- `vectors/pixels/` holds PNG test vectors with their digests. A tool that emits `pixel_sha256`
  MUST reproduce them.

## 9. The location

Where something is, on the original image. A location never refers to a crop, a view, an aligned
copy or any other frame internal to a tool.

```jsonc
{
  "image": { …image reference… },     // REQUIRED
  "kind": "bbox",                    // REQUIRED, 9.2
  "parts": [ { "bbox": [0.41, 0.22, 0.47, 0.31] } ],
  "straddles_wrap": false,           // REQUIRED
  "label": "a seagull on the left lamp post",
  "disposition": "violation",        // the tool's own word, OPTIONAL
  "confidence": "confident",         // the tool's own word, OPTIONAL
  "evidence": "source",              // the tool's own word, OPTIONAL
  "source": { "tool": "squint", "version": "0.2.0", "run": "decision-…", "item": "F1" }
}
// OPTIONAL as well: "score" (a number in [0, 1]) with "score_kind" (what it is, such as
// "owlv2-detection"), each requiring the other; "reason", REQUIRED for `absent`. An absent
// optional field is left out, never written as null. `source` (tool and version, with run and
// item where there are some) is left out of a location an agent wrote itself.
```

### 9.1 Coordinates

- Fractions of `width` and `height` from the top-left corner: x to the right, y down, `0` the
  left or top edge of the image and `1` the right or bottom edge. Coordinates measure **edges**, not
  pixel centres: a box `[0, 0, 1, 1]` is the whole image, and pixel column `i` spans
  `[i / width, (i + 1) / width]`.
- Written with enough digits for a quarter pixel at the longest side (`ceil(log10(4 * max(width,
  height)))` decimals).
- A receiver that rasterises snaps outward: the near edge floors, the far edge ceils, so a
  location never loses a pixel it touches.
- Lengths are in pixels of the referenced image, and the unit is in the name: `width_px` of a
  `line`, `radius_px` of a `stroke`. A fraction would need an axis, and a radius on a non-square
  image has no honest one; the reference pins the dimensions, so the conversion is exact. A length
  MUST be positive and is rounded up when rasterised.
- A receiver MUST refuse a location whose `image` does not match the image it was given (8.1).

### 9.2 Kinds

| `kind` | each part holds | notes |
|---|---|---|
| `bbox` | `bbox: [x0, y0, x1, y1]`, `x0 < x1`, `y0 < y1` | |
| `polygon` | `polygon: [[x, y], …]`, 3 to 256 vertices, simple | room for a view's footprint cut at the seam |
| `line` | `column: x` or `path: [[x, y], …]` (2 to 256 points); and `width_px` | rascal's seam line |
| `stroke` | `path: [[x, y], …]` (2 to 256 points), `radius_px` | a brush stroke |
| `mask` | `mask: { "image": {reference}, "threshold": 128 }`, one part | a greyscale 8-bit rendering at the image's width and height; pixels at or above `threshold` (1 to 255) are inside |
| `whole_image` | nothing (`parts: []`) | a global defect: a broken render, a wrong projection |
| `absent` | nothing (`parts: []`); `reason` required | the thing is missing; no geometry, never an invented box |
| `view` | `yaw`, `pitch`, `fov`, one part | **input only**, on `equirectangular` images |

- A part never crosses the wrap edge (x = 0 on a `wrap` image). A shape that does is emitted as two
  or more parts with `straddles_wrap: true`. On a `bounded` image `straddles_wrap` MUST be false.
  (rascal's `group`, shared by the pieces of one split location, becomes one location's `parts`.)
- The field holding each part's shape is named for its kind where the kind has one field (`bbox`,
  `polygon`, `mask`), as squint's and rascal's locations already name them.
- `view` is a convenience a tool MAY accept as input (yaw 0 is the centre column increasing to the
  right; pitch 0 the horizon increasing upward; `fov` the horizontal field of a square view in
  degrees, 10 to 120). A tool MUST NOT emit one: a view's footprint leaves as `polygon` parts.
- The producer sets `threshold` to match what its mask means, and a receiver uses the value it is
  given, never an assumed 128. rascal's rendering is 255 in the core and a gradient across the
  blend zone; for "change was allowed here" it sets `threshold: 1`, anything the composite may
  touch.
- A `mask` crosses as its rendering. A tool's own mask record (rascal's zones, prescriptions and
  provenance) MAY be offered as an MCP resource for tools that understand it; receivers that do
  not need only the rendering.
- `disposition`, `confidence` and `evidence` are the producing tool's vocabulary, carried so an
  agent can show or filter them. The tool MUST list each word it can emit in its description
  (`vocabulary`, 11.2), so the playground can render them. A receiver MUST NOT assume another tool's
  values.
- `score` is for a tool with a real measured number (rascal's detector score), named by
  `score_kind`. It is never comparable across tools or kinds, and a tool whose confidence is a
  model's category (squint) does not invent one.
- A tool's job roles (squint's `candidate` and `source`) are not part of a location; the image
  reference is.

## 10. Images in and out

### 10.1 In

Any named image slot of any action accepts one of:

```jsonc
{ "path": "/abs/path/to/pano.png", "expect": { …image reference… } }
{ "inline": { "media_type": "image/png", "data": "<base64>" }, "expect": { … } }
```

- `expect` is OPTIONAL. When present the tool verifies it (8.1).
- **Paths** are absolute, and are accepted only on local transports (stdio), only inside the
  client's MCP roots and any narrower roots in the tool's config. Over any network transport a
  path MUST be refused (`invalid`, code `input`); a path in a remote call is a request to read the
  host's filesystem.
- **Inline** bytes are bounded; each tool states its limit in its description
  (`limits.inline_bytes`, 11.2).
- A tool with state (rascal's workspace) treats a path or inline image as an implicit import: it
  imports, records the import as if the caller had asked for it, and runs on the result. It MUST
  NOT bypass its own record. Content addressing makes repeated imports one image.
- A tool MAY also accept its own ids (rascal's workspace ids) in the same slots. A client that
  knows only the contract never needs them.

### 10.2 Out

Every image a tool returns is an **image out**: its reference, its role, and one locator.

```jsonc
{ "role": "mask", "image": { …reference… }, "path": "/abs/run/mask-grey.png" }
{ "role": "mask", "image": { … }, "inline": { "media_type": "image/png", "data": "…" } }
{ "role": "mask", "image": { … }, "resource": "rascal://run/r-8c1e/images/mask-grey.png" }
```

- The caller chooses the locator per call (`return_images: "path" | "inline" | "resource"`).
  Default: `path` on stdio, `resource` on a network transport. MCP image content blocks are sent
  only when the caller asks (`image_blocks: true`); a chain returning a 20 MB panorama on every
  call is bandwidth for nothing. `return_images` and `image_blocks` are arguments every action of
  an images tool accepts beside its `params_schema`.
- v0.1 requires `path` and `inline`. `resource` is required with a network transport and
  RECOMMENDED otherwise; run resources (4.3) are required in v0.1 regardless.

## 11. What the profile adds to the result and the description

### 11.1 The result

```jsonc
  "images": { "in": [ { "slot": "candidate", "role": "candidate", "image": { …reference… } } ],
              "out": [ …image outs (10.2)… ] },
  "locations": { "used": [ … ], "found": [ … ] }
```

- Both are REQUIRED in every result of an images tool, empty lists where there is nothing.
- `images.in` lists each image the run took in as `{ slot, role, image }`: the slot it came
  through, its role and its reference, never its bytes. `images.out` lists image outs (10.2).
  Roles come from the tool's description (`candidate`, `source`, `result`, `mask`, …).
- `locations.used` are those the run took in; `locations.found` those it produced.
- Images and locations MAY be present on `error` and `cancelled` (2.3).

### 11.2 The description

```jsonc
  // on each action
  "images": [ { "slot": "candidate", "role": "candidate", "required": true },
              { "slot": "source", "role": "source", "required": true } ],
  "locations": [ { "slot": "allowed", "kinds": ["mask", "bbox", "polygon"], "required": false } ],
  "outputs": { "images": [], "locations": ["found"] },
  // at the top level
  "vocabulary": { "disposition": ["violation", "allowed", "unsure"],
                  "confidence": ["confident", "tentative"], "evidence": ["source", "prompt"],
                  "score_kind": [] },
  "limits": { "inline_bytes": 33554432 }
```

- All five are REQUIRED in the description of an images tool, and absent otherwise.
- An action's `images` lists its image slots with the role each plays; `locations` lists its
  location slots with the kinds each accepts; `outputs` names the roles of the images it returns
  and whether it fills `locations.found`.
- `vocabulary` lists every word the tool emits in a location's `disposition`, `confidence`,
  `evidence` and `score_kind` (9.2).
- `limits.inline_bytes` bounds an inline image (10.1).

## 12. Conformance

An images tool provides, besides the core (7):

1. Image references on every image in and out (8), and verification of `expect` (10.1).
2. The contract's locations on every location that leaves it (9).
3. Images in by `path` and `inline`, and images out by `path` and `inline` (10).
4. The vectors in `vectors/pixels/` reproduced, if it emits `pixel_sha256` (8.2).

# The cross-tool acceptance test

In this repository, `acceptance/`: fixtures and a script that runs a squint check on a candidate
with a planted defect, passes the finding's location to a rascal remove, passes rascal's mask back
to a squint source-fidelity check as the region where change was intended, and asserts the header
at each step. squint's steps replay recorded model replies, so the test runs offline and costs
nothing. Both tools run it in CI against the contract's tagged release. It lands with the mask
handoff (squint's region-scoped `allowed`, rascal's `mask` export), which is its definition of done;
v0.1 ships the first half (squint finding to rascal mask).
