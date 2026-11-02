# What each tool changes to conform

The contract was drawn from both tools as they stand on 2026-10-30: rascal at `917c5c3` (its
`rascal serve` is the MCP reference) and squint at `457bd2b` (no MCP server yet). This page lists,
field by field, where each differs today. It is a work list for each tool's conformance milestone,
not part of the contract; it is deleted once both conform.

**Status 2026-11-01.** v0.3.0 is published; rascal (M12), squint (M9) and Wafflery (formerly
Wafflehouse, as a client) pin `=0.3.0`. v0.4 is on branch `v0.4`; its work list is first below. The
v0.1 columns below are kept for review.

## v0.4 (operations, portable invocations, allowed directories)

Work for each tool's next conformance milestone (rascal's M10 for the operations profile);
Wafflery adopts it as the client (kinds and groups, artifact references and the binding refusal,
`export`/`import`, invocation bundles, `--allow-dir` in its argv). Both tools build against this
branch by path until v0.4.0 is published, then pin `=0.4.0`.

| contract | squint | rascal |
|---|---|---|
| every action carries `kind` (`task`, `operation`, `utility`); optional `group`, listed in the core's `vocabulary.group` (3) | done (M10) | to do |
| `vocabulary` is a core object; the images profile's four lists live in it as before (3, 11.2) | done (M10) | to do |
| `export` and `import` are reserved action names, beside `describe` and `estimate` (3, 5) | done (M10) | to do |
| `profiles` lists `operations` (with `images`); top-level `artifact_types` with `schema_versions`, `portable` and `preview` (1, 3.3) | n/a (no operations profile in v0.4) | to do |
| artifact references: type, schema version, tool, identity (never a workspace id), opaque frame, optional `on`, resource; accepted wherever an artifact is taken (3.4, 10.1) | n/a (no operations profile in v0.4) | to do |
| operations: `kind: "operation"` with `ports` (`in`, `out`), in-ports in the MCP input schema, `images`/`locations` `[]`; every call a run (4.7) | n/a (no operations profile in v0.4) | to do |
| the `binding` refusal before any run, from calls and `estimate`: per-reference `port`, `index` for `many` items, `problem`, `expected`, `got`, `remedy` for `unresolved`; releases the `client_run_id` (4.5, 4.7) | n/a (no operations profile in v0.4) | to do |
| estimate bindings digest artifact references by identity (4.5) | n/a (no operations profile in v0.4) | to do |
| `artifacts {in, out}` in every result, tasks' native outputs included, a `preview` on each out entry whose type has one; images and locations also in `images`/`locations` (4.7, 11.1) | n/a (no operations profile in v0.4) | to do |
| `<tool>://artifact/{identity}` and `/preview`; a missed read is the `binding` refusal with `port: null` (4.7) | n/a (no operations profile in v0.4) | to do |
| `export` and `import` tools and `<tool> export --json`/`import --json`: controls, never runs; refusals `binding` or `invalid_arguments`; import idempotent by identity (5.1) | n/a (no operations profile in v0.4) | to do |
| `invocation` on every result (null only on an unresolved `invalid` run): resolved arguments without locators, `effective`, requirement names (file names only, never paths) (2.3, 4.8) | done (M10) | to do |
| `<tool> call <action> --arguments <file> [--config] [--allow-dir]…` and `--bundle` replay (4.8, 4.9) | done (M10) | to do |
| invocation bundle manifest and byte-stable ustar layout (host-built; tools replay it) (4.9) | done (M10) | to do |
| MCP roots gone: `serve`/`call`/`import --allow-dir <abs>`, narrowed by configuration, none means every path refused, symlinks resolved; never write at a caller path (5, 5.2, 10.1) | done (M10) | to do |
| `<tool>://session` adds `allowed_dirs` (4.6) | done (M10) | to do |
| v0.4 fixtures (7, item 7) | done (M10) | to do |

## v0.3 (readiness, bound estimates, renders)

Work for rascal milestone M12 (fixtures delivered from rascal `867735d`) and squint milestone M9 (fixtures from squint `9c4be13`); Wafflery adopts it as the client. Both tools
build against this branch by path until v0.3.0 is published, then pin `=0.3.0`.

| contract | squint | rascal |
|---|---|---|
| `describe` and `serve` work with no configuration; `--config <path>` (3.1, 4.6) | done (M9) | done (M12) |
| per-action `readiness`, requirement states from local inspection and this session's answers; `cost.needs_env` gone (3.1) | done (M9) | done (M12) |
| `estimate` reserved: `{action, arguments}` in, `estimate.json` out, never a run, out of `actions` (4.5) | done (M9) | done (M12) |
| `binding` over action, args with images by digest, tool version, config identity, session; `expires` (4.5) | done (M9) | done (M12) |
| `_meta["table-manners/estimate"]` checked at the limits check: `stale_estimate` (4.5) | done (M9) | done (M12) |
| `cap` and `budget` split; `max_spend_usd` above the configured cap is `invalid` (2.2, 4.5) | done (M9) | done (M12) |
| every paid action's `params_schema` declares `max_spend_usd`, `exclusiveMinimum: 0` (4.5) | done (M9) | done (M12) |
| `credential` for 401/403 and a missing key, never retryable (2.2) | done (M9) | done (M12) |
| JSON-RPC refusal releases the `client_run_id` (4.4) | done (M9) | done (M12) |
| renders: `kind`, URL-only stdout, stdin/SIGTERM lifetime, `--frame-ancestor` (3.2) | done (M9) | done (M12) |
| image slots `many` (10.1, 11.2) | done (M9) | n/a (no action takes a list; tested) |
| v0.3 fixtures (7, item 7) | done (M9) | done (M12) |

## v0.2 (run association and the session)

rascal pins `=0.2.0` from milestone M11 (tag `m11`); squint's fixtures come from its commit
`4a07267` (milestone M8's follow-ups). Their fixtures are in `fixtures/rascal/` and
`fixtures/squint/`.

| contract | squint | rascal |
|---|---|---|
| `run.session_id`, `run.client_run_id` on results and runs entries; persisted at allocation | done (8.3) | done (M11) |
| progress `_meta` `table-manners/run` on every notification from allocation; no run in `message` | done (8.3) | done (M11) |
| `_meta["table-manners/client_run_id"]`, claimed at acceptance; `-32602` `client_run_id_reused` with the earlier run | done (8.3) | done (M11) |
| `<tool>://runs{?cursor}` with `next_cursor` | done (8.5) | done (M11) |
| reserve at the limits check, settle at the end | done (8.4) | done (M9) |
| not sent / known / unresolved; `spend.unresolved_usd` | done (8.4) | done (M11) |
| `<tool>://session` with a UUID v4 id | done (8.2) | done (M11) |
| describe, estimates, reads and cancel never wait on the execution permit | done (`m7-review`) | done (M9) |
| permit wait cancellable; a cancel before allocation leaves no run | done (`m7-review`) | done (M11) |
| no private session fields in the estimate body | done (8.2) | done (M11) |

## Locations

| today | contract | squint | rascal |
|---|---|---|---|
| `frame: "raw:candidate"` / `frame: "raw:img-3"` | `image`: an image reference (8) | change | change |
| one shape per location, flat on the object (`kind`, `bbox`) | `parts: [{ "bbox": … }]` | change | change |
| `group` shared by the pieces of a seam-split location | one location, several `parts`, `straddles_wrap: true` | n/a | change |
| `polygon: [[x, y], …]`, at most 64 vertices | same name, at most 256 | raise | raise |
| `line`: `column` or `path`, `width_px`; `stroke`: `path`, `radius_px` | same | n/a | none |
| `view`: `yaw`, `pitch`, `fov` | same, input only, `fov` 10 to 120; squint owns the sign convention (9.2) | none | accepts; never emits |
| `whole_image`, `absent` | same; `absent` needs `reason` | add reasons | add reasons |
| no mask kind | `mask`: a rendering and a threshold | read it (mask handoff) | export it |
| squint's `confidence` words | `confidence`, listed in `vocabulary` | list them | n/a |
| rascal's detector score, from the segmenter pack | `score` with `score_kind: "owlv2-detection"`, listed in `vocabulary.score_kind` | n/a | change |

## Images

| today | contract | squint | rascal |
|---|---|---|---|
| squint: `--candidate`/`--source` paths | image in: `{ path }` or `{ inline }`, optional `expect` | add inline and `expect` | |
| rascal: `workspace_import { path \| base64, projection }`, then ops by artifact id | image in on every op's slots, an implicit journaled import; ids still accepted | | change |
| rascal's envelope `images` (frame, parent, identity) | image outs: role, reference, one locator | add | change |
| rascal sends an image block for every image | image blocks only with `image_blocks: true` | n/a | change |
| rascal's pixel digest (`pixels`, a zero byte, dimensions, samples) | the same recipe (8.2) | adopt | none |

## Results

| today | contract | squint | rascal |
|---|---|---|---|
| squint: `envelope-v1` from the copied dejaview schema, plus its run directory | `run.json`: header plus body; the envelope moves into `body` | change | |
| rascal: `envelope-v1` (status `ok`, `measured_nothing`, `not_applicable`, `invalid_args`) | status `completed`, `cancelled`, `error`, `invalid`; rascal's own outcome stays in `body` | | change |
| rascal: operational failures as JSON-RPC errors with `data.class` (`usage`, `io`, `image`, `format`, `mask_invariant`, `op`, `operational`) | results with `isError: true` and `error.code`; JSON-RPC errors only before a run exists | | change |
| rascal: cancellation as a JSON-RPC error with `class: cancelled` | no response to the cancelled request; `run.json` written with `cancelled` | | change |
| neither tool writes `run.json` | every run, written atomically, last | add | add |
| spend: squint's ledger per run; rascal spends only through fillers | `spend { usd, basis }` on every result | change | add |

## Server and command line

| today | contract | squint | rascal |
|---|---|---|---|
| `rascal serve`: tools only, rmcp `=3.8.0`, `2025-11-25` | plus resources (runs, run.json, journal) and progress | add `serve` | add |
| self-description: rascal's op manifests in `tools/list`; squint's `rules`, `tools list`, `catalog` | `describe --json` and the `describe` tool (3), with `"profiles": ["images"]` | add | add |
| exit codes: rascal 0, 1 (`--strict`), 3, 64; squint 0, 1, 2, 3, 64 | plus 130 and 143 on cancellation | add | add |
| no `playground` subcommand | `playground` finds and starts the shared one | add | add |
