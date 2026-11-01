# What each tool changes to conform

The contract was drawn from both tools as they stand on 2026-10-30: rascal at `917c5c3` (its
`rascal serve` is the MCP reference) and squint at `457bd2b` (no MCP server yet). This page lists,
field by field, where each differs today. It is a work list for each tool's conformance milestone,
not part of the contract; it is deleted once both conform.

**Status 2026-11-01.** Both tools conform to v0.2.0 and pin `=0.2.0` from crates.io: rascal since
milestone M11 (tag `m11`), squint since milestone M8 (tag `m8`). Wafflehouse pins `=0.2.0` as a
client. v0.3 is drafted in CONTRACT.md on branch `v0.3`; its work list is below. The v0.1 columns below are kept for review.

## v0.3 (readiness, bound estimates, renders)

Work for rascal milestone M12 and squint milestone M9; Wafflery adopts it as the client. Both tools
build against this branch by path until v0.3.0 is published, then pin `=0.3.0`.

| contract | squint | rascal |
|---|---|---|
| `describe` and `serve` work with no configuration; `--config <path>` (3.1, 4.6) | to do | to do |
| per-action `readiness`, requirement states from local inspection and this session's answers; `cost.needs_env` gone (3.1) | to do | to do |
| `estimate` reserved: `{action, arguments}` in, `estimate.json` out, never a run, out of `actions` (4.5) | to do (drops estimate runs) | to do (args become the call's arguments) |
| `binding` over action, args with images by digest, tool version, config identity, session; `expires` (4.5) | to do | to do |
| `_meta["table-manners/estimate"]` checked at the limits check: `stale_estimate` (4.5) | to do | to do |
| `cap` and `budget` split; `max_spend_usd` above the configured cap is `invalid` (2.2, 4.5) | to do | to do |
| `credential` for 401/403 and a missing key, never retryable (2.2) | to do | to do |
| JSON-RPC refusal releases the `client_run_id` (4.4) | to do | to do |
| renders: `kind`, URL-only stdout, stdin/SIGTERM lifetime, `--frame-ancestor` (3.2) | to do (`report view`, `report export`) | to do (`dash`) |
| image slots `many` (10.1, 11.2) | to do (`references`) | n/a unless an action takes a list |
| v0.3 fixtures (7, item 7) | to do | to do |

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
