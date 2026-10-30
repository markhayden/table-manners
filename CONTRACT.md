# The waffle tools contract, v0.1 (draft)

Status: draft, 2026-10-30, with rascal's first review folded in. Not tagged. Field names in this text become binding when
v0.1 is tagged; until then they are the agreed shape, not the wire.

This is the external interface shared by the waffle image tools: squint (checks an image against
what was asked), rascal (finds, masks and composites regions), and the tools after them. Each tool
keeps its own intent, internals and vocabulary. What an agent, a script or the playground sees from
outside is the same in every tool, so tools chain in one pipeline without glue code:

```
generate ──► squint check ──► rascal remove ──► filler paints ──► squint check
   (any)        finds "the bird is here"   masks it     inside the mask    with the mask as the
                                                                           region change was allowed
```

The words MUST, MUST NOT, SHOULD and MAY are used as in RFC 2119.

Sources: squint ADR 0018 and SPEC "External interface"; rascal's seed section in
`../playground-recommendation.md`; rascal ADR 0005 (surfaces) and ADR 0015 (MCP transport).

## 1. Scope and versioning

- The contract defines: the **image reference** (1 below), the **location** (3), how images enter
  and leave a tool (4), the **result** (5), the **description document** (6), **long runs** (7),
  the **MCP server conventions** (8), **exit codes** (9), and what a conforming tool provides (10).
- It does not define what a tool does, its own files, its journal's lines, its config format or its
  internal frames. A tool MAY add fields in its own namespace (the result body, `x-<tool>` keys in
  the description); it MUST NOT add fields to the shared objects.
- The JSON Schemas under `schemas/` are the source of truth; this text explains them. The Rust
  crate `waffle-tools-contract` holds serde types generated to match, with no logic.
- Releases are tagged `vMAJOR.MINOR.PATCH`. Before 1.0 a minor release may break; after 1.0 only a
  major one may. Every result and description carries `contract` (the version the tool conforms
  to). A tool pins one release, keeps a hash-checked copy of its schemas, and SHOULD warn in CI
  when a newer tag exists; an update is a deliberate commit in the tool.
- v0.1 is the minimum the chain needs. squint's conformance is expected to find what it gets wrong
  about long paid runs; v0.2 follows before rascal's conformance merges.

## 2. The image reference

How a tool names an image to another tool. Tools never share ids, workspaces or frame names.

```jsonc
{
  "file_sha256": "3f9a…",            // REQUIRED: SHA-256 of the file's bytes, lowercase hex
  "width": 2048,                     // REQUIRED: pixels as stored (EXIF orientation not applied)
  "height": 1024,                    // REQUIRED
  "projection": "equirectangular",   // REQUIRED: "flat" | "equirectangular"
  "topology": "wrap",                // REQUIRED: "bounded" | "wrap"
  "pixel_sha256": "a1c4…"            // OPTIONAL: canonical pixel digest (2.2)
}
```

### 2.1 The file digest is the check

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

### 2.2 The pixel digest is a convenience

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

## 3. The location

Where something is, on the original image. A location never refers to a crop, a view, an aligned
copy or any other frame internal to a tool.

```jsonc
{
  "image": { …image reference… },     // REQUIRED
  "kind": "bbox",                    // REQUIRED, 3.2
  "parts": [ { "bbox": [0.41, 0.22, 0.47, 0.31] } ],
  "straddles_wrap": false,
  "label": "a seagull on the left lamp post",
  "disposition": "violation",        // the tool's own word, OPTIONAL
  "confidence": "confident",         // the tool's own word, OPTIONAL
  "score": null,                     // a number in [0, 1], OPTIONAL, with score_kind
  "score_kind": null,                // what the score is ("owlv2-detection"), REQUIRED with score
  "evidence": "source",              // the evidence kind where there is one, OPTIONAL
  "reason": null,                    // REQUIRED for `absent`, else OPTIONAL
  "source": { "tool": "squint", "version": "0.2.0", "run": "decision-…", "item": "F1" }
}
```

### 3.1 Coordinates

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
- A receiver MUST refuse a location whose `image` does not match the image it was given (2.1).

### 3.2 Kinds

| `kind` | each part holds | notes |
|---|---|---|
| `bbox` | `bbox: [x0, y0, x1, y1]`, `x0 < x1`, `y0 < y1` | |
| `polygon` | `points: [[x, y], …]`, 3 to 256 vertices, simple | room for a view's footprint cut at the seam |
| `line` | `column: x` or `path: [[x, y], …]`; and `width_px` | rascal's seam line |
| `stroke` | `path: [[x, y], …]`, `radius_px` | a brush stroke |
| `mask` | `mask: { "image": {reference}, "threshold": 128 }` | a greyscale 8-bit rendering at the image's width and height; pixels at or above `threshold` are inside |
| `whole_image` | nothing (`parts: []`) | a global defect: a broken render, a wrong projection |
| `absent` | nothing (`parts: []`); `reason` required | the thing is missing; no geometry, never an invented box |
| `view` | `yaw`, `pitch`, `fov` | **input only**, on `equirectangular` images |

- A part never crosses the wrap edge (x = 0 on a `wrap` image). A shape that does is emitted as two
  parts with `straddles_wrap: true`. On a `bounded` image `straddles_wrap` MUST be false.
- `view` is a convenience a tool MAY accept as input (yaw 0 is the centre column increasing to the
  right; pitch 0 the horizon increasing upward; `fov` the horizontal field of a square view in
  degrees, 10 to 120). A tool MUST NOT emit one: a view's footprint leaves as `polygon` parts.
- A `mask` crosses as its rendering. A tool's own mask record (rascal's zones, prescriptions and
  provenance) MAY be offered as an MCP resource for tools that understand it; receivers that do
  not need only the rendering.
- `disposition`, `confidence` and `evidence` are the producing tool's vocabulary, carried so an
  agent can show or filter them. The tool MUST list each word it can emit in its description
  (`vocabulary`, 6), so the playground can render them. A receiver MUST NOT assume another tool's
  values.
- `score` is for a tool with a real measured number (rascal's detector score), named by
  `score_kind`. It is never comparable across tools or kinds, and a tool whose confidence is a
  model's category (squint) does not invent one.
- A tool's job roles (squint's `candidate` and `source`) are not part of a location; the image
  reference is.

## 4. Images in and out

### 4.1 In

Any named image slot of any action accepts one of:

```jsonc
{ "path": "/abs/path/to/pano.png", "expect": { …image reference… } }
{ "inline": { "media_type": "image/png", "data": "<base64>" }, "expect": { … } }
```

- `expect` is OPTIONAL. When present the tool verifies it (2.1).
- **Paths** are accepted only on local transports (stdio), only inside the client's MCP roots and
  any narrower roots in the tool's config. Over any network transport a path MUST be refused
  (`invalid`, code `input`); a path in a remote call is a request to read the host's filesystem.
- **Inline** bytes are bounded; each tool states its limit in its description (`limits.inline_bytes`).
- A tool with state (rascal's workspace) treats a path or inline image as an implicit import: it
  imports, records the import as if the caller had asked for it, and runs on the result. It MUST
  NOT bypass its own record. Content addressing makes repeated imports one image.
- A tool MAY also accept its own ids (rascal's workspace ids) in the same slots. A client that
  knows only the contract never needs them.

### 4.2 Out

Every image a tool returns is an **image out**: its reference, its role, and one locator.

```jsonc
{ "role": "mask", "image": { …reference… }, "path": "/abs/run/mask-grey.png" }
{ "role": "mask", "image": { … }, "inline": { "media_type": "image/png", "data": "…" } }
{ "role": "mask", "image": { … }, "resource": "rascal://run/r-8c1e/images/mask-grey.png" }
```

- The caller chooses the locator per call (`return_images: "path" | "inline" | "resource"`).
  Default: `path` on stdio, `resource` on a network transport. MCP image content blocks are sent
  only when the caller asks (`image_blocks: true`); a chain returning a 20 MB panorama on every
  call is bandwidth for nothing.
- v0.1 requires `path` and `inline`. `resource` is required with a network transport and
  RECOMMENDED otherwise; run resources (7.3) are required in v0.1 regardless.

## 5. The result

One shape everywhere: the MCP tool result's `structuredContent`, the run resource, and the file
`run.json` written last into every run directory. `run.json` is written atomically (to a temporary
name in the same directory, then renamed), because the playground and a retrying agent read it
while the tool may still be running. A **header** the contract owns and a **body**
the tool owns.

```jsonc
{
  "contract": "0.1.0",
  "tool": { "name": "squint", "version": "0.2.0" },
  "action": "check",
  "run": { "id": "decision-1793386884140-53022", "resource": "squint://run/decision-…/run.json",
           "dir": "/abs/squint-out/decision-…" },
  "status": "completed",
  "error": null,
  "started": "2026-10-30T19:09:42.118Z",
  "finished": "2026-10-30T19:10:21.904Z",
  "duration_ms": 39786,
  "input_identity": "sha256:…",
  "spend": { "usd": 0.0389, "basis": "measured" },
  "cache": "miss",
  "images": { "in": [ …image outs with roles… ], "out": [ … ] },
  "locations": { "used": [ … ], "found": [ … ] },
  "notes": [ "repair: the first submission omitted item C3; repaired in one turn" ],
  "body": { …the tool's own… }
}
```

### 5.1 Status

| `status` | meaning | the agent should |
|---|---|---|
| `completed` | the action ran to its end | read the body; a squint `fail` verdict is `completed` |
| `cancelled` | stopped by the caller or a signal | read what is there; spend to date is reported |
| `error` | an operational failure | read `error.code` and `error.retryable` |
| `invalid` | the request was wrong | fix the request; never retry it unchanged |

### 5.2 Error

Present exactly when `status` is `error` or `invalid`:

```jsonc
{ "code": "provider", "retryable": true, "reason": "the provider answered 529 three times" }
```

| `code` | typical status | meaning |
|---|---|---|
| `usage` | invalid | an unknown action, a missing or malformed argument |
| `input` | invalid | an image or location the tool cannot take: a digest mismatch, a path outside roots or over a network, a size over the limit, a location on another image |
| `io` | error | reading or writing the tool's own files failed |
| `provider` | error | a remote model or service failed (a judge model, a filler endpoint) |
| `budget` | error | the per-call cap or the session budget would be exceeded |
| `timeout` | error | a bounded wait ran out |
| `tool_process` | error | a helper process (a tool pack, a filler command) died, hung or answered out of contract |
| `internal` | error | a bug in the tool |

`retryable` is the tool's judgement for this occurrence (a 529 is, a 401 is not).

### 5.3 The rest of the header

- `run.id` is unique per tool; `run.resource` names the run resource (7.3); `run.dir` is the run
  directory when the transport is local, else null.
- `input_identity`: a digest the tool computes over its resolved inputs and parameters, so two
  results with equal identities ran the same thing. Its recipe is the tool's.
- `spend.basis` is `measured` (metered by the provider) or `estimated` (a configured price).
  `spend.usd` is what this run cost, including a cancelled or failed run's spend to date; 0 for a
  cache hit.
- `cache`: `hit`, `miss` or `off`.
- `images.in` and `images.out` carry roles from the tool's description (`candidate`, `source`,
  `result`, `mask`, …). `locations.used` are those the run took in; `locations.found` those it
  produced. Images and locations MAY be present on `error` and `cancelled`, and a tool MUST NOT
  drop an output it completed because a later step failed: an agent can salvage a mask and call a
  different filler.
- `notes`: human sentences for caveats on a run that still completed (a repair, a tool's refusal,
  a location clipped at the edge, an override applied).

## 6. The description document

`<tool> describe --json` prints it, and the MCP tool `describe` returns it.

```jsonc
{
  "contract": "0.1.0",
  "tool": { "name": "squint", "version": "0.2.0", "purpose": "Checks an image against what was asked of it." },
  "actions": [
    {
      "name": "check",
      "title": "Source fidelity",
      "purpose": "…",
      "images": [ { "slot": "candidate", "role": "candidate", "required": true },
                  { "slot": "source", "role": "source", "required": true } ],
      "locations": [],
      "params_schema": { …JSON Schema… },
      "outputs": { "images": [], "locations": ["found"] },
      "cost": { "may_spend": true, "basis": "measured", "needs_env": ["GEMINI_API_KEY"] },
      "duration": { "typical_s": 40, "max_s": 600 },
      "renders": [ { "name": "report", "command": ["squint", "report", "view", "{run.dir}", "--port", "0"] } ]
    }
  ],
  "vocabulary": { "disposition": ["violation", "allowed", "unsure"],
                  "confidence": ["confident", "tentative"], "evidence": ["source", "prompt"] },
  "limits": { "inline_bytes": 33554432 },
  "progress_interval_s": 10,
  "resources": [ "squint://runs", "squint://run/{id}/run.json", "squint://run/{id}/journal" ],
  "x-squint": { }
}
```

- `needs_env` names variables; a tool MUST NOT print their values. A client shows whether each is
  set.
- `duration` lets a client set its timeout up front, and `progress_interval_s` (at most 10) is the
  longest gap between progress notifications on a live run (7.1).
- `vocabulary` lists every word the tool emits in a location's `disposition`, `confidence` and
  `evidence` (3.2).

## 7. Long runs

### 7.1 Progress is a heartbeat

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

The first progress notification names the run resource.

MCP tasks (experimental in `2025-11-25`, moving to an extension) are not used in v0.1.

### 7.2 Cancellation

On `notifications/cancelled` (or SIGINT or SIGTERM on the command line) a tool stops its paid work
at once, writes the run resource and `run.json` with status `cancelled` and spend to date, and
returns no MCP result (the protocol says a cancelled request gets none).

### 7.3 Runs are resources

Every run a server performs is readable by id: `<tool>://run/<id>/run.json` at least, and
`<tool>://run/<id>/journal` for the journal. `<tool>://runs` lists recent runs newest first (id,
action, status, started), so a run is always discoverable, including after a cancel or a dropped
connection on a call that sent no progress token. An agent whose call timed out or whose
connection dropped reads the resource instead of calling again; the playground reads the same
resources for its history.

### 7.4 Retries do not pay twice

A tool with a paid step is idempotent on identical inputs within its cache policy: an identical
retry is answered from the cache with `cache: "hit"` and `spend.usd: 0`.

### 7.5 Spend limits

- A per-call cap in the tool's config. A call MAY lower it (`max_spend_usd`); it MUST NOT raise it.
- A per-session budget in the tool's config, across every call the server process serves.
- Every result reports spend (5.3). A tool that may spend provides an `estimate` action returning
  the worst case for a given call and the session budget left.
- Calls are serial by default; a tool MAY allow more with a concurrency setting.
- A per-period budget (a day, a month) is later work: it needs a durable ledger across processes.

## 8. MCP server conventions

Lifted from `rascal serve` (rascal ADR 0015).

- `<tool> serve` speaks MCP over stdio at protocol revision **`2025-11-25`** only, and refuses a
  client that cannot meet it at `initialize`. Moving revision is a contract change.
- Capabilities: `tools` (with pagination), `resources` (run resources), progress, cancellation.
- One MCP tool per action in the description, plus `describe`; each tool's input schema is its
  action's image slots, location slots and `params_schema`. A server MUST NOT require an injected
  context object: a client needs nothing but the description.
- Once a run exists, every outcome is an MCP tool result carrying the header and body as
  `structuredContent`: `isError: false` for `completed`, `isError: true` for `invalid` and
  `error`. The header is the point: spend to date, `retryable` and salvaged outputs (5.3) travel
  only in a result, and a JSON-RPC error reaches a model client as an opaque failure.
- JSON-RPC errors are reserved for failures before a run exists: an unknown tool, arguments the MCP
  layer rejects against the input schema, the server itself failing.
- Image content blocks only on request (4.2).
- Rust tools SHOULD use the official SDK, `rmcp`, pinned exactly; rascal and squint pin `=3.8.0`.
  A tool in another language uses its official SDK at the same revision.
- Every tool's test suite drives its server with an independently written client that shares no
  code with the server or the SDK. rascal's (`rascal_testkit::mcp_client`) is the reference.

## 9. Exit codes

| code | meaning |
|---|---|
| 0 | success (`completed`, and the tool's own success when it distinguishes one) |
| 3 | `error` |
| 64 | any `invalid` (`usage` or `input`; the result's `error.code` tells them apart) |
| 130 | cancelled by SIGINT |
| 143 | cancelled by SIGTERM |
| 1, 2 | the tool's own (squint: fail, unsure; rascal: nothing found under `--strict`) |

## 10. Conformance

A conforming tool provides, at the contract version it names:

1. `describe --json` and the `describe` MCP tool (6), validated against the schema in its tests.
2. `serve` (8) with an action per description entry, progress (7.1), cancellation (7.2), run
   resources and the runs list (7.3), and spend limits where it spends (7.5).
3. `run.json` (5), written atomically, in every run directory, on every surface, including
   cancelled and failed runs.
4. Image references on every image in and out (2), verification of `expect` (4.1), and the
   contract's locations on every location that leaves it (3).
5. The exit codes (9).
6. A `playground` subcommand that finds, offers to install and starts the shared playground; it
   carries no UI.
7. The conformance fixtures in `fixtures/<tool>/` passing in its CI against its pinned release.

## 11. The cross-tool acceptance test

In this repository, `acceptance/`: fixtures and a script that runs a squint check on a candidate
with a planted defect, passes the finding's location to a rascal remove, passes rascal's mask back
to a squint source-fidelity check as the region where change was intended, and asserts the header
at each step. squint's steps replay recorded model replies, so the test runs offline and costs
nothing. Both tools run it in CI against the contract's tagged release. It lands with the mask
handoff (squint's region-scoped `allowed`, rascal's `mask` export), which is its definition of done;
v0.1 ships the first half (squint finding to rascal mask).
