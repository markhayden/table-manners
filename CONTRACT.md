# table-manners, v0.4.0 (draft)

Status: **v0.4.0 draft, not binding until tagged**, 2026-11-01. v0.3.0 plus the operations
profile (operations as ordinary actions with typed ports, artifact types and references, the
`binding` refusal, artifacts in the result, the reserved `export` and `import` tools), the kind and
group of every action, the effective invocation on every result with one generic command line and
a portable invocation bundle, and allowed directories given at start in place of MCP roots
("Changes from v0.3" below). The text was approved on 2026-11-01; the schemas, types and examples
on this branch are v0.4.0's.

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
squint and rascal both implement. v0.4 adds **operations** (single steps a host runs alone on
typed artifacts), which rascal implements; its sections sit in the core's numbering where they
extend a core object (3.3, 3.4, 4.7, 5.1) and each is marked "(operations profile)". A tool that
works on text, audio or code follows the core and needs no profile until one is written for it,
against a real tool.

The words MUST, MUST NOT, SHOULD and MAY are used as in RFC 2119.

Sources: squint ADR 0018 and SPEC "External interface"; rascal's seed section in
`../playground-recommendation.md`; rascal ADR 0005 (surfaces) and ADR 0015 (MCP transport). For
v0.4: rascal PLAN M10 and ADR 0022 (portable masks); Wafflehouse's C1, C2 and R3.

# Part 1: the core

## 1. Scope, versioning and profiles

- The core defines the **result** (2) with the **invocation** (4.8), the **description
  document** (3), **long runs** (4) with the **session** (4.6), **invocation bundles** (4.9), the
  **MCP server conventions** (5) with the **path rule** (5.2), **exit codes** (6) and what a
  conforming tool provides (7). The images profile defines the **image reference** (8), the
  **location** (9), how images enter and leave a tool (10), and what it adds to the result and the
  description (11). The operations profile defines **artifact types** (3.3), the **artifact
  reference** (3.4), **operations** with their ports, binding and outputs (4.7), and the reserved
  **`export` and `import`** tools (5.1). It builds on the images profile: a tool that lists
  `operations` also lists `images`, because ports name images and locations and previews are
  image outs.
- A **profile** is a named set of additions to the core's objects. A tool lists the profiles it
  implements in its description (`profiles`); it MUST include every field a listed profile
  requires and MUST NOT emit a profile's fields without listing it. A new profile is a contract
  change, written when the first tool that needs it exists.
- The contract does not define what a tool does, its own files, its journal's lines, its config
  format or its internal frames. A tool MAY add fields in its own namespace (the result `body`,
  `x-<tool>` keys in the description); it MUST NOT add fields to the shared objects.
- The JSON Schemas under `schemas/` are the source of truth; this text explains them.
  `schemas/core/` holds the core's, `schemas/images/` the images profile's and
  `schemas/operations/` the operations profile's. Each schema's `$id`
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
  observe. v0.3 lets a host set a tool up and pay for a run safely: readiness without
  configuration (3.1), estimates bound to their inputs and checked before paid work (4.5), and
  report renders it can embed (3.2). v0.4 lets a host run one step alone on a saved artifact and
  move a call elsewhere: operations as ordinary runs on typed artifact references (4.7), the kind
  and group of every action (3), the effective invocation on every result (4.8), portable
  bundles a host builds from contract pieces (4.9), and allowed directories given at start, since
  MCP roots are deprecated after `2025-11-25` (5.2). The other proposals in `PROPOSALS.md` follow
  in later minors.

## 2. The result

One shape everywhere: the MCP tool result's `structuredContent`, the run resource, and the file
`run.json` written last into every run directory. `run.json` is written atomically (to a temporary
name in the same directory, then renamed), because the playground and a retrying agent read it
while the tool may still be running. A **header** the contract owns and a **body** the tool owns.

```jsonc
{
  "contract": "0.4.0",
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
  "invocation": { "action": "check", "arguments": { …resolved (4.8)… },
                  "effective": { "model": "claude-sonnet-5-5", "config_identity": "sha256:…" },
                  "requirements": [ "squint.toml", "ANTHROPIC_API_KEY" ] },
  "spend": { "usd": 0.0389, "unresolved_usd": 0, "basis": "measured" },
  "cache": "miss",
  "notes": [ "repair: the first submission omitted item C3; repaired in one turn" ],
  "body": { …the tool's own… }
  // the images profile adds "images" and "locations" (11.1)
  // the operations profile adds "artifacts" (4.7)
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
| `input` | invalid | an input the tool cannot take; for the images profile a digest mismatch, a path outside the allowed directories (5.2) or over a network, a size over the limit, a location on another image |
| `io` | error | reading or writing the tool's own files failed |
| `stale_estimate` | invalid | the call named an estimate (4.5) that no longer matches it: other inputs, configuration, tool version or session, or past its `expires` |
| `provider` | error | a remote model or service failed (a judge model, a filler endpoint) |
| `credential` | error | a provider rejected the credential (HTTP 401 or 403) during the run; never retryable (a credential missing before the run is the `not_ready` refusal, 3.1) |
| `cap` | error | the call's worst case is over its effective per-call cap (4.5) |
| `budget` | error | the session budget cannot reserve the call's worst case (4.5) |
| `timeout` | error | a bounded wait ran out |
| `tool_process` | error | a helper process (a tool pack, a filler command) died, hung or answered out of contract |
| `internal` | error | a bug in the tool |

`retryable` is the tool's judgement for this occurrence (a 529 is). `credential`, `cap`, `budget`
and `stale_estimate` are never retryable unchanged: a person fixes a key, a cap, the budget or
asks for a new estimate first.

### 2.3 The rest of the header

- `run` is the run's reference (4.3) plus `dir`: `run.id` is unique per tool, `run.resource`
  names the run resource, `run.session_id` the serving session (4.6) and `run.client_run_id` the
  caller's correlation id (4.4). `run.dir` is the run directory when the transport is local, else
  null. All five are required; the nullable ones are null, never absent.
- `input_identity`: a digest the tool computes over its resolved inputs and parameters, so two
  results with equal identities ran the same thing. Its recipe is the tool's.
- `invocation`: what the call resolved to, so it can be run again elsewhere (4.8). It is required
  in every result; it is null only on an `invalid` run whose arguments did not resolve.
  `input_identity` stays as the digest (its recipe is still the tool's); `invocation` is the
  readable record beside it.
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
  "contract": "0.4.0",
  "profiles": ["images"],
  "tool": { "name": "squint", "version": "0.2.0", "purpose": "Checks an image against what was asked of it." },
  "actions": [
    {
      "name": "source_fidelity",
      "title": "Source fidelity",
      "kind": "task",
      "purpose": "…",
      "params_schema": { …JSON Schema… },
      "cost": { "may_spend": true, "basis": "measured" },
      "readiness": { "state": "needs_setup", "requirements": [ …(3.1)… ] },
      "duration": { "typical_s": 40, "max_s": 600 },
      "renders": [ { "name": "report", "kind": "server", "ready_timeout_s": 10,
                     "command": ["squint", "report", "view", "{run.dir}", "--port", "0",
                                 "--frame-ancestor", "{frame_ancestor}"] } ]
      // the images profile adds "images", "locations" and "outputs" (11.2)
      // the operations profile adds "ports" to an operation (4.7)
    }
  ],
  "progress_interval_s": 10,
  "resources": [ "squint://runs{?cursor}", "squint://run/{id}/run.json", "squint://run/{id}/journal",
                 "squint://session" ],
  "vocabulary": { …(below)… },
  "x-squint": { }
  // the images profile adds "limits" (11.2)
  // the operations profile adds "artifact_types" (3.3)
}
```

- `profiles` lists the profiles the tool implements; `[]` for a core-only tool.
- `kind` is required on every action and is one of three words the contract owns:

  | `kind` | the action is | a host shows it |
  |---|---|---|
  | `task` | a whole job a person or agent asks for (squint's checks, rascal's `remove`) | with the tasks |
  | `operation` | one step on typed artifacts, run alone (4.7); only in a tool that lists `operations` | with the operations |
  | `utility` | neither | apart from both |

- `group` is optional: a word of the tool's own that sorts actions within a kind (rascal's `find`,
  `refine`, `prep`, `out`, `inspect`). Every group an action names MUST appear in
  `vocabulary.group`. A host groups by `kind`, then by `group`, and never by an action's name.
- `vocabulary` is a top-level object of word lists. The core owns `group`: required, and listing
  every `group` the actions use, whenever any action has one; absent or empty otherwise. The
  images profile adds its lists (11.2). `vocabulary` is required whenever any list in it is.
- `params_schema` is a JSON Schema for the action's arguments other than those a profile adds.
  Choices a caller may make (a model, a filler) are arguments in it, with the default the tool
  uses when they are absent; what the tool actually used is in the estimate (4.5) and the run.
- `cost.basis` is required when `cost.may_spend` is true.
- `readiness` says whether the action can run here, now, and what is missing (3.1).
- `duration` lets a client set its timeout up front, and `progress_interval_s` (at most 10) is the
  longest gap between progress notifications on a live run (4.1).
- Action names are lowercase with underscores (`check`, `seam_heal`); `describe`, `estimate`,
  `export` and `import` are reserved (5) and never appear in `actions`.
- `renders` show a run to a person (3.2).
- `describe` works with no configuration, no credentials and no network (3.1), and is cheap: a
  host calls it again to refresh readiness.

### 3.1 Readiness

An installed tool MUST describe itself without its configuration: a missing or invalid config
file, or missing credentials, never makes `describe` fail or exit non-zero. It says instead, per
action, what stands between the action and a run:

```jsonc
"readiness": {
  "state": "needs_setup",
  "requirements": [
    { "kind": "config", "name": "squint.toml", "state": "missing",
      "setup": "Run `squint init` or pass --config <path>." },
    { "kind": "credential", "name": "ANTHROPIC_API_KEY", "state": "present",
      "setup": "Set ANTHROPIC_API_KEY in the environment squint is started with." },
    { "kind": "model", "name": "claude-sonnet-5-5", "state": "valid" }
  ]
}
```

- `state` is `ready` (no requirement is `missing` or `invalid`), `needs_setup` (one is, and the
  person can fix it), or `unavailable` (this build or platform cannot run the action at all; a
  requirement says why).
- A requirement's `kind` is `config`, `credential`, `env` (a variable the tool reads that is not a
  secret, such as a model directory), `model` (a model or endpoint the action calls, as
  configured), `file` (a local file it needs, such as model weights) or `dependency` (a helper
  program or library).
- An action's readiness describes the action as the tool is configured now (a configured filler
  is part of its use), and its requirements name every variable that use reads, so a host that
  passes only the variables requirements name passes enough. A call is refused for a requirement only when
  that call needs it: a call that needs a `missing` or `invalid` requirement is refused before
  any run, as a JSON-RPC error `-32602` with `data` `{"reason": "not_ready", "run": null,
  "requirements": [...]}` (`schemas/core/refusal.json`) naming what it needs, so it writes no run,
  spends nothing and releases its `client_run_id` (4.4). A call that does not need it (an action
  whose filler is optional, called without one) runs. A provider that rejects a credential during
  a run is `credential` (2.2). An estimate's `readiness` (4.5) is the call's own.
- A configuration is `present` when it loads and `invalid` when it does not parse. The names in
  the examples here are illustrative; each tool names its own files, variables and commands.
- A requirement's `state`:

  | state | meaning |
  |---|---|
  | `missing` | not there: no such variable, file or config entry |
  | `present` | there, and not checked; finding a key never makes it `valid` |
  | `valid` | a provider accepted it in this session (a known answer, 4.5, that was not a rejection); for a `file`, the tool checked its digest in this session |
  | `invalid` | there and rejected: a provider answered 401 or 403 to it in this session, or the file or config entry does not parse |
  | `unknown` | the tool cannot tell (a credential held somewhere it does not inspect) |

- Describing is a **local inspection**: it MUST NOT make a network call, start a paid step or
  write a run. It need not hash large files: an unchecked file is `present`. `valid` and `invalid` for a credential come only from provider answers the session
  already had, so a fresh session shows `present` until a real call settles it.
- `name` is a variable, file or entry name; `setup` is a sentence a person can act on. Neither
  ever carries a credential value, and a tool MUST NOT print one.
- A credential a call needs and does not have is not a provider call: the call is refused
  `not_ready` before any run.

### 3.2 Renders

A render is a command that shows a run to a person. Its `command` is an argv; the host substitutes
`{run.dir}` (the run directory) and, for a server, `{frame_ancestor}`, each only where it is a whole
argument, and runs it without a shell. A render never modifies the run directory. Its
`command[0]` is the tool's own name (`tool.name`), and the host runs the same executable it
serves, never another program and never another install found on `PATH`.

- `kind: "document"` writes one self-contained file (images and styles inline, nothing it needs
  beside it) at a path the tool chooses outside the run directory, and exits. Its only stdout is
  one line, the absolute path of that file. Exit 0 means the file exists. The host reads that one
  file and nothing next to it.
- `kind: "server"` serves a page over HTTP on loopback. Once it is listening it prints exactly one
  line to stdout, the URL (`http://127.0.0.1:<port>/…`), and nothing else ever. It binds port 0
  unless the command says otherwise. It serves until its stdin closes or it receives SIGTERM, then
  exits; the host owns the process. It allows framing only by the one origin given as
  `{frame_ancestor}` (`Content-Security-Policy: frame-ancestors <origin>`), never loopback in
  general; the host passes its own origin, for example `http://127.0.0.1:4801`.
  `ready_timeout_s` (at most 60) bounds the wait for the URL line. It exits 0 when its stdin closes
  or on SIGTERM.
- A render that exits non-zero, or prints no line in time, failed to start. Its stderr is for
  people. A failed render never changes the run's status.
- Renders need a local run directory. A run whose `run.dir` is null has no render.

### 3.3 Artifact types (operations profile)

An **artifact** is a thing an operation takes or makes: an image, a location, or a record of the
tool's own (a mask with its zones and prescriptions, a plan). Its **type** says which. The
description lists every type the tool's ports name, once, at the top level, except `image`, which
the contract defines whole: it travels as a file (10.1, 4.9), needs no export and is its own
picture.

```jsonc
"artifact_types": [
  { "type": "location", "schema_versions": [1], "purpose": "where something is (9)", "portable": "json", "preview": true },
  { "type": "rascal:Mask", "schema_versions": [1], "purpose": "zones, profile, prescriptions",
    "portable": "bundle", "preview": true },
  { "type": "rascal:Plan", "schema_versions": [1], "purpose": "…", "portable": "none", "preview": false }
]
```

- The shared types are `image` (an image reference, 8) and `location` (9). The contract defines
  them, and they carry no prefix. Their schema version is 1.
- Every other type is the tool's: `<tool.name>:<Name>` (`rascal:Mask`, `rascal:BandProfile`,
  `rascal:Series`, `rascal:Plan`). It is opaque to the host and to other tools: a tool MUST NOT
  accept another tool's type, and a host never reads inside one.
- `schema_versions` lists the versions of the type's record the tool reads, as integers. A tool
  writes the newest it lists.
- `purpose` is a sentence for people.
- `portable` says how the artifact leaves the workspace it lives in: `bundle` (a file the tool
  writes and reads whole, 5.1), `json` (a JSON document, 5.1) or `none` (it cannot leave yet).
  A host labels an artifact of a `none` type **workspace-dependent** and never replaces it with
  something else, such as a rendering of it.
- `preview` says whether the tool draws an artifact of the type as an image for people (4.7).

### 3.4 The artifact reference (operations profile)

How an operation names an artifact (`schemas/operations/artifact-ref.json`). A host stores
references, never a tool's ids, and passes them back.

```jsonc
{ "type": "rascal:Mask", "schema_version": 1, "tool": "rascal",
  "identity": "sha256:9c1e…",                  // REQUIRED: the tool's recipe; never a workspace id
  "frame": "sha256:41ab…",                     // REQUIRED: opaque frame identity, null for frameless types
  "on": { …image reference (8)… },             // OPTIONAL: the root image it was made on
  "resource": "rascal://artifact/sha256:9c1e…" }
```

- `type` is one of the tool's `artifact_types`, and `schema_version` one of its versions.
- `tool` is the tool that made the artifact (`tool.name`).
- `identity` is a digest by the tool's recipe, written `sha256:<lowercase hex>` (rascal's
  recipe: ADR 0016, references replaced by identities). Equal identities are the same artifact in any workspace, on any
  machine. Workspace ids (`mask-3`, `crop:2`) never appear in a reference: ids are local, identity
  is not. For an `image`, the identity is `sha256:` followed by its `file_sha256` (8.1), so any
  tool computes the same one.
- `frame` names the pixel grid the artifact is drawn in (the tool's own frame identity: an image's
  size and projection, a crop chained to its parent). It is opaque: the tool alone compares frames.
  It is null for a type with no frame.
- `on` is the image reference of the root image the artifact was made on, when the tool knows it.
  A host uses it, with `type`, to offer only candidates made on the same file; it never checks
  frames itself.
- `resource` reads the artifact (4.7).

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
  "run": <the earlier request's run reference>}`. `run` is null only while the earlier request
  is still waiting for the permit, or when it was cancelled before allocation.
- A request that ends in a JSON-RPC error with no run (invalid params, any refusal in 5)
  **releases** its id: the client fixes the request and retries under the same id.
- A request cancelled before allocation keeps its id claimed for the session; a client that
  retries it uses a new id.
- The refusal never stands for the new request's result. The client reconciles the earlier run
  with its own saved attempt; it does not retry automatically.

This is not exactly-once execution across processes: a new session forgets the ids it never saw.

### 4.5 Spend limits and estimates

- A per-call cap in the tool's config. A call MAY lower it with the argument `max_spend_usd`; a
  value above the configured cap is `invalid` (`usage`), never silently lowered. Every action whose
  `cost.may_spend` is true declares `max_spend_usd` in its `params_schema` as an optional
  `{"type": "number", "exclusiveMinimum": 0}`, so a host can always pass its cap however
  closed the rest of the schema is. A cap of zero is not a cap; a caller that must not spend does
  not call.
- A per-session budget in the tool's config, across every call the server process serves.
- Both limits are enforced at the **limits check**, after allocation and before any paid step.
  The check fails the run with `cap` when the call's worst case is over its effective cap, and
  with `budget` when the session cannot hold it. Otherwise it **reserves** the worst case into the
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
- Every result reports spend (2.3).

**Estimates.** A tool with any action whose `cost.may_spend` is true serves the reserved MCP tool
`estimate` (5) and the command `<tool> estimate --json`; any other tool MAY. Its arguments are the action's name and
exactly the arguments the paid call would send:

```jsonc
{ "action": "source_fidelity", "arguments": { …the call's arguments, images included… } }
```

It answers with an estimate (`schemas/core/estimate.json`) as `structuredContent`:

```jsonc
{ "contract": "0.4.0", "tool": { "name": "squint", "version": "0.2.0" },
  "action": "source_fidelity",
  "binding": "sha256:…",
  "session_id": "0b7e3c5a-…",
  "effective": { "model": "claude-sonnet-5-5", "config_identity": "sha256:…" },
  "worst_case_usd": 0.12, "cap_usd": 0.5,
  "allowed": true, "refused": null,
  "readiness": { "state": "ready", "requirements": [ … ] },
  "session": { …the session observation (4.6)… },
  "expires": "2026-11-01T20:15:00.000Z" }
```

- An estimate MUST NOT spend, call a provider or write a run. Since it is never a run, its failures
  are JSON-RPC errors: arguments the input schema rejects are plain invalid params (`-32602`);
  any other wrong request (`max_spend_usd` above the configured cap, an `expect` that does not
  match, an unknown action) is `-32602` with refusal data `{"reason": "invalid_arguments",
  "run": null, "error": {…the error object, 2.2: `usage` or `input`, not retryable…}}`. A call
  that is not ready is not an error: its estimate is `refused: "readiness"`. A call whose
  artifacts do not bind gets the same `binding` refusal the call would (4.7). It never waits for the permit (see
  below). It is a control request, not history; a host never shows it as an attempt.
- `binding` is a digest the tool computes over the action, the arguments after resolving every
  image to its file digest (8.1) and every artifact reference to its identity (3.4), the tool version, the effective configuration
  (`effective.config_identity`) and the session id. Arguments that only choose how results are
  delivered (the images profile's `return_images` and `image_blocks`, 10.2) are left out. Its recipe is the tool's; two estimates with
  equal bindings priced the same call.
- `effective` names what the call would use where the arguments leave a choice to the tool:
  `model` (null when the action calls none) and `config_identity`, a digest of the configuration
  in force: the loaded file, or the tool's built-in defaults when it runs without one. A tool MAY add fields under `x-<tool>`.
- `worst_case_usd` is what the limits check would reserve. When the call cannot be priced (it
  needs a configuration or a model the tool lacks), the estimate is `refused: "readiness"` with
  `worst_case_usd` null and, when the missing piece is the configuration, `effective.config_identity`
  null; never a made-up zero. A call that needs nothing missing is priced as usual, at 0 when it
  spends nothing. `cap_usd` is the effective per-call cap
  (the configured cap lowered by `max_spend_usd`), null when uncapped; null is never zero.
- `allowed` is whether the limits check would pass right now. When it is false, `refused` is
  `cap`, `budget` or `readiness` (the action is not `ready`); otherwise null.
- `session` is one atomic observation of the session (4.6), the same document as
  `<tool>://session`. An estimate from the command line has no session: `session_id` and
  `session` are null, and no server accepts its binding.
- `expires` is when the tool stops honouring the binding (prices and catalogs change). It is the
  tool's choice, at most 24 hours after the estimate.

**Binding a call to its estimate.** A caller MAY send `_meta["table-manners/estimate"]: "<binding>"`
on the paid `tools/call`. The tool recomputes the binding for the call as it arrives; if it
differs, or the estimate has expired, the run fails with `stale_estimate` at the limits check,
before any paid step and with zero spend. A call that names no estimate is checked against the
limits alone. Either way the tool stays authoritative across concurrent clients: an estimate is
never a reservation, and `allowed: true` does not promise the limits check will pass.

- Calls are serial by default; a tool MAY allow more with a concurrency setting. Execution
  concurrency does not depend on whether an action costs money. While a run holds the execution
  permit, `describe`, `estimate`, `export`, `import`, every resource read (runs, a run, its
  journal, the session, an artifact) and cancellation MUST stay responsive: none of them waits
  for the permit.
- A per-period budget (a day, a month) is later work: it needs a durable ledger across processes.

### 4.6 The session

A **session** is one serving process's spend ledger. `<tool>://session`
(`schemas/core/session.json`) returns one atomic observation of it:

```jsonc
{ "contract": "0.4.0", "tool": { "name": "rascal", "version": "0.1.0" },
  "id": "0b7e3c5a-6f1d-4c2e-9a8b-3d4e5f6a7b8c", "started": "2026-10-31T18:02:11.530Z",
  "budget_usd": 2.0, "spent_usd": 0.16, "unresolved_usd": 0.04, "reserved_usd": 0.04,
  "remaining_usd": 1.8,
  "allowed_dirs": [ "/abs/host/library", "/abs/host/execution" ] }
```

- `id` is a random UUID (v4), lowercase, generated when `serve` starts.
- `allowed_dirs` lists the directories the session reads caller paths from (5.2): absolute,
  symlinks resolved, after the configuration narrowed them. It is required, and `[]` when every
  path is refused. Like the configuration, it is fixed for the session.
- **Configuration belongs to the session.** `serve` reads the tool's configuration once, at start;
  the host chooses it only by the tool's own `--config <path>` argument, or leaves the tool's
  default lookup. A host never reads or writes a tool's configuration, and passes the child only
  its own minimal environment plus the variables the description's requirements name (3.1); keys
  stay with the tool and the OS. To take up a changed configuration or credential, the host
  restarts the server deliberately and the person sees a new session id with a fresh ledger; it
  never restarts silently to refresh an allowance. A tool that reloads or resets its ledger in
  process starts a new session with a new id; neither squint nor rascal does.
- `started` is RFC 3339 UTC, like every timestamp.
- `spent_usd` is settled spend and includes `unresolved_usd`; `reserved_usd` is the worst cases
  held by runs still going.
- `budget_usd` and `remaining_usd` are null together, exactly when the session is uncapped; null
  is never a zero allowance. `remaining_usd = max(0, budget_usd - spent_usd - reserved_usd)`.
- Amounts are US dollars. Tools write them with at most six decimals (a property of the value:
  `1e-6` is six decimals) and clients compare them to
  1e-6.
- A session is not a daily budget or a provider account balance. When a server restarts, a client
  sees a new id; what an interrupted session left unresolved stays unresolved in its runs.

### 4.7 Operations (operations profile)

An **operation** is an action with `kind: "operation"`: one step a host runs alone on artifacts it
already holds (a Line on an image, that Line's location to Rasterize, a saved Mask to Feather with
a new sigma). It is an ordinary action: every call is a run with `run.json`, a run reference,
readiness, an estimate, progress and cancellation, as in 2 to 4.6. Nothing upstream reruns: an
operation works on the artifacts it is given or refuses.

**Ports.** An operation declares typed inputs and outputs:

```jsonc
"ports": {
  "in":  [ { "name": "location", "type": "location", "required": true, "many": false, "purpose": "…" },
           { "name": "image", "type": "image", "required": true, "purpose": "the frame" } ],
  "out": [ { "name": "mask", "type": "rascal:Mask", "required": true, "purpose": "…" } ]
}
// params_schema keeps the operation's own parameters, e.g. "sigma_px", "constants": {"enum": ["feather-v1"]}
```

- `ports` is required on every operation and absent on every other kind.
- A port has `name` (lowercase with underscores, unique within `in` and within `out`), `type`
  (`image`, `location` or one of `artifact_types`, 3.3), `required`, `many` (default false: the
  port takes or makes a list) and `purpose`.
- An operation's MCP input schema is its `params_schema` plus one property per in-port, named for
  the port: an artifact reference (3.4), or a non-empty array of them when `many`. An `image` port
  also takes an image in (10.1), and a `location` port a location (9). A port name never clashes
  with a parameter.
- For an operation, ports are the images profile's slots: its image and location inputs are
  ports only, and its `images` and `locations` (11.2) are `[]`.

**Binding.** Before it allocates a run, the tool binds each in-port: it resolves every reference
by `identity` in its own store and checks the type, the schema version and the frame. A tool
validates frames itself (crop and cut chains); a host only filters candidates by `type` and `on`.
A call that does not bind is refused before any run, as a JSON-RPC error `-32602` with refusal
data (`schemas/core/refusal.json`):

```jsonc
{ "reason": "binding", "run": null,
  "bindings": [ { "port": "mask", "problem": "unresolved", "expected": "rascal:Mask v1",
                  "got": "sha256:9c1e…", "remedy": "import its bundle with the import tool" } ] }
```

- `bindings` holds one entry per reference that failed, in port order and, within a `many` port,
  in the order given. A call with several problems names them all.
- `index` is the item's 0-based position within a `many` port, present only for items of a
  `many` port: `{"port": "masks", "index": 2, "problem": "wrong_frame", …}`.
- `problem` is one of:

  | `problem` | the reference |
  |---|---|
  | `wrong_type` | names a type the port does not take, or another tool's type |
  | `wrong_version` | names a schema version the tool does not read |
  | `wrong_frame` | is drawn in a frame the operation cannot use with its other inputs |
  | `unresolved` | names an identity the tool's store does not hold |

- `expected` says what the port takes and `got` what it was given, each a short phrase for
  people (a type and version, or an identity). `remedy` is a sentence a person can act on; it is
  required for `unresolved` and optional otherwise.
- Like `not_ready` (3.1), the refusal writes no run, spends nothing and releases the
  `client_run_id` (4.4). A host shows it on the form field the port names.
- An image or location given inline, by path or as an object, is checked as in 8.1 and 9.1; a
  mismatch there is an `invalid` run with code `input`, as for any action.

**Outputs.** The profile adds `artifacts` to the result:

```jsonc
"artifacts": {
  "in":  [ { "port": "mask", "artifact": { …reference… }, "preview": null } ],
  "out": [ { "port": "mask", "artifact": { …reference… },
             "preview": { "role": "preview", "image": { … }, "path": "/abs/run/preview-mask.png" } } ]
}
```

- `artifacts` is required in every result of an operations tool, with empty lists where there is
  nothing. It MAY be present on `error` and `cancelled` (2.3).
- `artifacts.in` lists each artifact the run took through a port, and `artifacts.out` each it
  made, as `{ port, artifact, preview }`. A `many` port appears once per artifact, in order.
- Every action of an operations tool fills `artifacts`, tasks included: a task lists each native
  artifact it produced in `artifacts.out` (with `port` naming the output, such as `mask`), so a
  mask a task made can be bound to an operation later.
- `preview` is an image out (10.2) with role `preview`, drawing the artifact for people. It is
  required on every out entry whose type declares `preview: true`, and null otherwise; on in
  entries it MAY be given. A preview never stands for the artifact: a host passes the reference on.
- An image an operation takes or makes also appears in `images.in` (with the port's name as
  `slot`) or `images.out`, and every preview in `images.out`. A location it takes or makes also
  appears in `locations.used` or `locations.found` (11.1), so consumers of the images profile keep
  working.
- The tool's own record of the step (rascal's native envelope) goes in `body`. A host never reads
  it; what it needs is in the header and `artifacts`.

**The artifact resource.** Every artifact a tool holds is readable by identity:

```jsonc
// <tool>://artifact/{identity}
{ "contract": "0.4.0", "tool": { "name": "rascal", "version": "0.1.0" },
  "artifact": { …reference… },
  "preview": { …image out with role "preview"… },
  "record": { …the tool's own… } }
```

- `<tool>://artifact/{identity}` returns the reference, its preview (null when the type has none)
  and the tool's own record, which a host never reads. `<tool>://artifact/{identity}/preview`
  returns the preview image itself, for a type with `preview: true`.
- A read of an identity the store does not hold fails with refusal data `{"reason": "binding",
  "run": null, "bindings": [{"port": null, "problem": "unresolved", …, "remedy": "…"}]}`.
- This is how a host reopens a saved binding after it restarts: it stores references, not ids,
  and reads them back. After the workspace moves, the host starts the tool on the new location
  (by the tool's own argv, 4.6) and the same identities resolve there; a host never searches
  folders for an artifact.

### 4.8 The invocation

Every result carries `invocation` (2.3): what the call resolved to, recorded by the tool, so the
call can be repeated from another directory, by another client or on another machine.

```jsonc
"invocation": {
  "action": "feather",
  "arguments": { "mask": { …artifact reference… }, "sigma_px": 6, "constants": "feather-v1" },
  "effective": { "model": null, "config_identity": "sha256:…" },
  "requirements": [ ]
}
```

- `action` is the action's name.
- `arguments` are the call's arguments as the tool resolved them: defaults applied; each image as
  `{ "expect": <image reference> }` with no locator (no `path`, no `inline`); each artifact as its
  reference (3.4); each location as given; `max_spend_usd` kept. Arguments that only choose how
  results are delivered (`return_images`, `image_blocks`, 10.2) are left out, as in the estimate's
  binding (4.5). Nothing in it depends on where files were, so it moves with the inputs.
- `effective` is the estimate's object (4.5): `model`, `config_identity` and any `x-<tool>`
  fields (squint's rule-set and catalog versions, rascal's constant sets and filler).
- `requirements` lists the `name` of each readiness requirement (3.1) the call used, never a
  value. For a `file` or `config` requirement it is the file name only (`rascal.toml`), never a
  path.

**The command line.** Every tool provides one generic command that runs any action from its MCP
arguments:

```
<tool> call <action> --arguments <file.json> [--config <path>] [--allow-dir <abs>]…
```

- The file holds exactly the arguments an MCP call to the action would send. The command prints
  the result (2) on stdout, writes a run like any command-line run, and exits as in 6.
- Paths in the arguments follow the path rule (5.2), with the directories given by `--allow-dir`.
- So the mapping is mechanical: an `invocation`'s `arguments`, with a locator added to each image,
  are the file for `<tool> call <invocation.action>`.
- `<tool> call --bundle <path>` replays an invocation bundle (4.9). It imports each artifact
  member (5.1), points each image at its input member, and calls each step with its
  `invocation.arguments` plus those locators. The bundle path follows the path rule.

### 4.9 Invocation bundles

A **portable bundle** carries a call, or a chain of calls, with everything it needs to run again
with the original workspace gone. The host builds it from contract pieces; the contract defines
its manifest (`schemas/core/invocation-bundle.json`); no tool builds one.

```
invocation.json   the manifest (below)
inputs/<file_sha256>.<ext>        each caller image, byte for byte
artifacts/<identity>.<ext>        each tool export file (5.1), byte for byte
```

In member names an identity's `:` is written `-` (`artifacts/sha256-9c1e….mask.tar`).

```jsonc
{ "bundle_version": 1,
  "steps": [
    { "contract": "0.4.0", "tool": { "name": "rascal", "version": "0.1.0" },
      "invocation": { …4.8… }, "result": { …the original run.json… },
      "inputs":    [ { "port": "image", "image": { …image reference… }, "member": "inputs/3f9a….png" } ],
      "artifacts": [ { "port": "mask", "artifact": { …reference… },
                       "member": "artifacts/sha256-9c1e….mask.tar", "sha256": "…" } ],
      "reference_only": [ { "slot": "references", "why": "the person left it out" } ],
      "unportable": [ { "port": "plan", "artifact": { …reference… }, "reason": "rascal:Plan has portable: none" } ] } ] }
```

- The container is an uncompressed ustar with fixed headers, as rascal's mask bundle (ADR 0022):
  every member has mode 0644, uid and gid 0, mtime 0 and empty user and group names, and members
  come in a fixed order (`invocation.json` first, then the rest sorted by name). Equal contents
  give equal bytes, so the archive's SHA-256 names it.
- `steps` lists the calls in order; a single call is one step. A later step names an earlier
  step's output by its reference, never by a path.
- `inputs` lists each image a step took, by `slot` (images profile) or `port` (operations), with
  its reference and its member. Members are the caller's files byte for byte, so their
  `file_sha256` still holds; a replay checks it.
- `artifacts` lists each native input with its export file (5.1) stored byte for byte and that
  file's `sha256`. A replay imports it, which checks it whole.
- Inputs are included by default. `reference_only` lists each input left out on purpose, by
  `slot` or `port`, with a sentence saying why; a replay needs it supplied.
- `unportable` lists each native input whose type is `portable: none` or whose export failed,
  with the reason. Such a bundle is labelled workspace-dependent. A native artifact is never
  replaced by a rendering (a PNG of a mask is not a mask).
- Replay, by a host or by `<tool> call --bundle` (4.8): import each artifact, put each input
  under an allowed directory (5.2), and call each step with its `invocation.arguments` plus
  locators. A live judgment (a model's verdict) is not promised to repeat; only the call is.
  An argument naming an earlier step's output binds to that step's output in this replay, by port,
  not to the recorded identity: a non-deterministic step (a filler) may produce a different
  artifact, and the chain uses it. The replay reports each such change; it is not an error.
- Reconstruction (rerunning an artifact's provenance instead of carrying it) is not a bundle
  feature: it would rerun upstream steps, which an operation never does.
- A host's evidence archive (results and pictures to read, not to run) is a separate thing and is
  never called an invocation bundle.

## 5. MCP server conventions

Lifted from `rascal serve` (rascal ADR 0015).

- `<tool> serve` speaks MCP over stdio at protocol revision **`2025-11-25`** only, and refuses a
  client that cannot meet it at `initialize`. Moving revision is a contract change.
- Capabilities: `tools` (with pagination), `resources` (run resources, the session, and the
  artifact resources of an operations tool), progress, cancellation. A tool does not use MCP
  roots: it never sends `roots/list` and never reads paths from the client's roots (5.2).
- One MCP tool per action in the description, plus `describe`, plus `estimate` when any action
  may spend (4.5), plus `export` and `import` when any artifact type is portable (5.1); each
  action tool's input schema is its action's `params_schema` with the arguments its profiles add
  (for an operation, its in-ports, 4.7). `estimate`'s input is `{action, arguments}` (4.5).
- `<tool> serve` accepts `--config <path>` (4.6) and, like `describe`, starts without a
  configuration: its actions then report `needs_setup`, and a call that needs what is missing is
  refused `not_ready` (3.1).
- `<tool> serve` accepts `--allow-dir <abs>`, repeatable, naming the directories it may read
  caller paths from (5.2). The host passes them in its argv like `--config`.
- A server MUST NOT require an injected context object: a client needs nothing but the
  description.
- Once a run exists, every outcome is an MCP tool result carrying the header and body as
  `structuredContent`: `isError: false` for `completed`, `isError: true` for `invalid` and
  `error`. The header is the point: spend to date, `retryable` and salvaged outputs (2.3) travel
  only in a result, and a JSON-RPC error reaches a model client as an opaque failure.
- JSON-RPC errors are reserved for failures before a run exists: an unknown tool, arguments the MCP
  layer rejects against the input schema, a reused `client_run_id` (4.4), a call that is not ready
  (3.1), an operation whose artifacts do not bind (4.7), the server itself failing. A refusal the contract names carries `data.reason` (`schemas/core/refusal.json`) so a
  host can tell it from other invalid params. Such an error releases the request's
  `client_run_id` (4.4).
- The contract's request and notification metadata lives in MCP's `_meta` under keys prefixed
  `table-manners/` (`table-manners/run`, `table-manners/client_run_id`,
  `table-manners/estimate`), so it needs no MCP revision of its own.
- Rust tools SHOULD use the official SDK, `rmcp`, pinned exactly; rascal and squint pin `=3.8.0`.
  A tool in another language uses its official SDK at the same revision.
- Every tool's test suite drives its server with an independently written client that shares no
  code with the server or the SDK. rascal's (`rascal_testkit::mcp_client`) is the reference.

### 5.1 Export and import (operations profile)

A tool that declares any artifact type `portable: bundle` or `portable: json` (3.3) serves the
reserved MCP tools `export` and `import`, and the commands `<tool> export --json <file>` and
`<tool> import --json <file>`, which read the same request from the file (or from stdin when it
is `-`), like `estimate`, and print the same output. Like `estimate` they are controls, not
runs: they write no run, never appear in `actions` and are never shown as history. Their
failures are JSON-RPC errors; on the command line a refusal exits 64 (6) with the refusal JSON
on stdout.

```jsonc
// export: in, then out
{ "artifact": { …reference… } }
{ "contract": "0.4.0", "tool": { "name": "rascal", "version": "0.1.0" },
  "artifact": { …reference… },
  "file": { "path": "/abs/rascal-data/exports/sha256-9c1e….mask.tar", "sha256": "…",
            "bytes": 1832448, "media_type": "application/x-tar" } }

// import: in, then out
{ "file": { "path": "/abs/allowed/….mask.tar" }, "sha256": "…" }
{ "inline": { "media_type": "application/json", "data": "<base64>" } }
{ "contract": "0.4.0", "tool": { "name": "rascal", "version": "0.1.0" },
  "artifact": { …reference… }, "already_present": false }
```

- **Export** writes the artifact's portable form into the tool's own data directory, outside any
  run directory, at a path the tool chooses, and returns its absolute path, SHA-256, size and
  media type. A `bundle` type exports the tool's bundle file (rascal's canonical ustar, ADR 0022);
  a `json` type exports a JSON document (`application/json`). Exporting the same artifact again
  gives the same bytes.
- **Import** reads a file by `path`, under the path rule (5.2), or bytes given `inline`, up to
  `limits.inline_bytes` (11.2). `sha256`, when given, is checked against the bytes. The tool
  checks the whole file before it records anything, records the artifact in its store and
  returns its reference. Import is idempotent by identity: importing an artifact already held
  records nothing new and answers `already_present: true`.
- A reference the store does not hold is refused for export with the `binding` refusal (4.7),
  `port: "artifact"`, problem `unresolved`. Any other wrong request (a type that is
  `portable: none`, a file that is not the type's portable form, such as a PNG given for a mask, a
  digest mismatch, a path outside the allowed directories) is `-32602` with refusal data
  `{"reason": "invalid_arguments", "run": null, "error": {…code `usage` or `input`, not
  retryable…}}`, as for `estimate` (4.5).
- A rendering of an artifact is never accepted as the artifact, and never exported in its place.

### 5.2 The path rule

A caller path is accepted only on a local transport (stdio), and only when its canonical form
(absolute, with `.`, `..` and every symlink resolved) lies inside a directory the session allows.

- The allowed directories are those given by `serve --allow-dir <abs>` (repeatable), resolved the
  same way at start, then narrowed by the tool's own configuration, which may only take away: the
  effective set is the overlap (a configured directory counts only where it lies inside a flag
  directory). With none, every path is refused. A relative, missing or non-directory
  `--allow-dir` is an error at start (exit 64), never silently dropped. `<tool>://session` lists them as `allowed_dirs` (4.6), and they do
  not change during the session.
- `<tool> call` (4.8) and `<tool> import` (5.1) take the same flag under the same rule. The rule
  governs `serve`, `call` and `import` only: a path a person passes on their own command line to
  any other command of the tool stays theirs, outside the contract.
- The rule covers every path a caller gives: images in (10.1), import files (5.1), bundles (4.8).
- A refused path is an `invalid` result with code `input` in a run, or `-32602` with the
  `invalid_arguments` refusal in a control. The refusal says nothing about the file, not even
  whether it exists.
- A tool writes only inside its own directories (its runs root, its workspace or store, its
  export area) and never at a path a caller gives. What it writes for a caller (images out,
  document renders, exports) is at a path it chooses and returns.
- Over any network transport every path is refused (10.1).
- MCP roots are not used. A host that offered roots passes `--allow-dir` instead.

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

1. `describe --json` and the `describe` MCP tool (3), validated against the schema in its tests,
   working with no configuration (3.1), with `kind` on every action; renders as in 3.2.
2. `serve` (5) with an action per description entry, progress with the run reference (4.1),
   cancellation (4.2), run resources and the paged runs list (4.3), correlation ids (4.4), spend
   limits with reservation and the three-way accounting where it spends (4.5), the session
   resource with `allowed_dirs` (4.6), and the path rule with `--allow-dir` and no MCP roots
   (5.2).
3. `run.json` (2), written atomically, in every run directory, on every surface, including
   cancelled and failed runs, with `invocation` (4.8); and `<tool> call` (4.8), including
   `--bundle` replay of an invocation bundle (4.9).
4. The exit codes (6).
5. A `playground` subcommand that finds, offers to install and starts the shared playground; it
   carries no UI.
6. Every profile it lists, conforming (12 for images). For operations: every operation with
   `ports` and a call that is a run (4.7); `artifact_types` (3.3) and artifact references with
   identities, never ids (3.4); the `binding` refusal in calls and in `estimate` (4.7);
   `artifacts` in every result, tasks' native outputs included, with a preview on each out entry
   whose type has one (4.7); the
   artifact resource and its preview (4.7); `export` and `import` when any type is portable (5.1).
7. The conformance fixtures in `fixtures/<tool>/` passing in its CI against its pinned release:
   `describe.json`, `run-*.json` results, `runs*.json` pages, `session.json` and `refusal-*.json`
   reuse refusals. A release is tagged only when every tool's fixtures validate against it. For
   v0.2 they include accepted, reused and mismatched client ids, cancels before and after
   allocation, unresolved spend, a reservation in flight, a restarted session and a read while
   the permit is held. For v0.3 they add: `describe-unconfigured.json` (no configuration),
   `describe-*.json` with a missing, a present, a valid and an invalid credential and a `ready`
   action; `estimate-*.json` (allowed, refused for `cap`, `budget` and `readiness`); results
   failed with `stale_estimate` (changed image, argument, configuration, session; expired), `cap`,
   `budget` and `credential`; a request refused with no run whose id is then accepted, and one
   cancelled before allocation whose id is still refused; and proof in the tool's own tests that
   `describe` and `estimate` made no provider call. For v0.4 they add:
   - `describe.json` with `kind` on every action and, for an operations tool, `group` with
     `vocabulary.group`, `ports` on every operation and `artifact_types` (portable `bundle`,
     `json` and `none`; with and without previews).
   - Operation results `run-op-*.json` for a chain run one step at a time (rascal: Line, then
     Rasterize on its location, then Feather on the saved Mask with a new sigma), each with
     `artifacts.in` and `artifacts.out`, a preview on every out entry whose type has one, and the
     same images and locations in `images` and `locations`; and a task result (rascal's
     `remove`) listing the Mask it made in `artifacts.out`, which a later operation binds.
   - `refusal-binding-*.json`, one per problem (`wrong_type`, `wrong_version`, `wrong_frame`,
     `unresolved` with its `remedy`), from a call and from `estimate`, one naming an item of a
     `many` port by `index`, and a request so refused whose `client_run_id` is then accepted.
   - `artifact-*.json`, an artifact resource read, and the refusal for an identity not held.
   - `export-*.json` and `import-*.json`: a Mask exported and imported into an empty store
     (`already_present: false`), imported again (`true`), a Location imported inline, and
     refusals for a PNG given as a mask, a `portable: none` type and an unresolved export.
   - `invocation` on every result, tasks included, and a `<tool> call --arguments` run of a
     result's invocation whose `invocation` matches the original.
   - An invocation bundle (`bundle-*/invocation.json`) replayed by `<tool> call --bundle` from
     another directory with the original workspace gone; squint's replays offline on its
     scripted provider.
   - `session.json` with `allowed_dirs`, and `invalid`/`input` results for a path outside them,
     a `..` escape, a symlink escape, a missing file (the same refusal as an existing one) and a
     session started with no `--allow-dir` (`allowed_dirs: []`, every path refused).

## Changes from v0.3

Breaking, as a minor before 1.0 may be (1):

- Every action carries `kind` (`task`, `operation` or `utility`) (3).
- Every result carries `invocation` (2.3, 4.8).
- MCP roots are gone: a tool never sends `roots/list`, and reads caller paths only inside the
  directories given by `serve --allow-dir`, narrowed by its configuration; with none, every path
  is refused. Paths are compared after resolving symlinks (5.2, 10.1).
- `<tool>://session` adds `allowed_dirs` (required) (4.6).
- A failed `roots/list` is no longer a case of a refusal releasing its id (4.4).
- `export` and `import` are reserved names, beside `describe` and `estimate` (3, 5).
- `vocabulary` is a core object; the images profile's four lists live in it as before, and the
  core adds `group` (3, 11.2).

Added: the operations profile: artifact types (3.3), the artifact reference (3.4), operations
with ports, the `binding` refusal, `artifacts` with previews in the result and the artifact
resource (4.7), and `export` and `import` (5.1); an action's optional `group` (3); the
`<tool> call` command and `--bundle` replay (4.8); the invocation bundle
(`schemas/core/invocation-bundle.json`, 4.9). Estimate bindings digest artifact references by
identity (4.5).

## Changes from v0.2

Breaking, as a minor before 1.0 may be (1):

- `estimate` is a reserved MCP tool with the input `{action, arguments}` and its own result
  shape (`schemas/core/estimate.json`); it is never a run and never appears in `actions` (4.5).
- `cost.needs_env` is gone; each action's `readiness` names its requirements instead (3.1).
- Every action carries `readiness` (3.1); `describe` and `serve` work with no configuration.
- Every render carries `kind`, and a server render prints only its URL and frames only for
  `{frame_ancestor}` (3.2).
- `max_spend_usd` above the configured cap is `invalid`, not lowered (4.5).
- `budget` covers the session only; the per-call cap is `cap` (2.2).
- A request refused with a JSON-RPC error releases its `client_run_id` (4.4).

Added: the `not_ready` refusal (3.1); error codes `credential`, `cap` and `stale_estimate` (2.2); estimate binding with
`_meta["table-manners/estimate"]` (4.5); configuration owned by the session and chosen by
`--config` (4.6); image slots that take a list (`many`, 10.1, 11.2).

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
  provenance) is an artifact of the tool's type, read at its artifact resource and moved by
  `export` and `import` (4.7, 5.1); receivers that do not understand it need only the rendering.
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

Any named image slot of any action accepts one of the following, or, when the slot is `many`
(11.2), a non-empty array of them:

```jsonc
{ "path": "/abs/path/to/pano.png", "expect": { …image reference… } }
{ "inline": { "media_type": "image/png", "data": "<base64>" }, "expect": { … } }
```

- `expect` is OPTIONAL. When present the tool verifies it (8.1).
- **Paths** are absolute, and are accepted only on local transports (stdio), only when their
  canonical form (symlinks resolved) lies inside the session's allowed directories: those given
  by `serve --allow-dir`, narrowed by the tool's config, and none when neither names any (5.2).
  Tools do not use MCP roots. A refused path is `invalid`, code `input`, and the refusal says
  nothing about the file. Over any network transport a path MUST be refused (`invalid`, code
  `input`); a path in a remote call is a request to read the host's filesystem.
- **Inline** bytes are bounded; each tool states its limit in its description
  (`limits.inline_bytes`, 11.2).
- A tool with state (rascal's workspace) treats a path or inline image as an implicit import: it
  imports, records the import as if the caller had asked for it, and runs on the result. It MUST
  NOT bypass its own record. Content addressing makes repeated imports one image.
- A tool MAY also accept its own ids (rascal's workspace ids) in the same slots. A client that
  knows only the contract never needs them. Artifact references (3.4) are the portable form, and
  an operations tool MUST accept them wherever it takes an artifact.

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
  through, its role and its reference, never its bytes. A `many` slot appears once per image, in
  the order given. `images.out` lists image outs (10.2), **most telling first**: the image a
  person should see first leads (an edit's result before the mask that made it), and a host
  shows the first by default while offering every one. The description's `outputs.images`
  (11.2) names the roles in the same order.
  Roles come from the tool's description (`candidate`, `source`, `result`, `mask`, …).
- `locations.used` are those the run took in; `locations.found` those it produced.
- Images and locations MAY be present on `error` and `cancelled` (2.3).
- An operation's image and location ports and its previews appear here too (4.7); an image taken
  through a port is listed with the port's name as `slot`.

### 11.2 The description

```jsonc
  // on each action
  "images": [ { "slot": "candidate", "role": "candidate", "required": true },
              { "slot": "references", "role": "reference", "required": false, "many": true } ],
  "locations": [ { "slot": "allowed", "kinds": ["mask", "bbox", "polygon"], "required": false } ],
  "outputs": { "images": [], "locations": ["found"] },
  // at the top level, in the core's vocabulary (3)
  "vocabulary": { "disposition": ["violation", "allowed", "unsure"],
                  "confidence": ["confident", "tentative"], "evidence": ["source", "prompt"],
                  "score_kind": [] },
  "limits": { "inline_bytes": 33554432 }
```

- `images`, `locations`, `outputs` and `limits` are REQUIRED in the description of an images
  tool, and absent otherwise; so are the four lists in `vocabulary`, which is the core's object
  (3).
- An action's `images` lists its image slots with the role each plays (`[]` for an operation,
  whose inputs are ports, 4.7); `many` (default false) says the slot takes a list; `locations` lists its
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
