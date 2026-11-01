# What each tool changes to conform

The contract was drawn from both tools as they stand on 2026-10-30: rascal at `917c5c3` (its
`rascal serve` is the MCP reference) and squint at `457bd2b` (no MCP server yet). This page lists,
field by field, where each differs today. It is a work list for each tool's conformance milestone,
not part of the contract; it is deleted once both conform.

**Status 2026-10-31.** squint conforms to v0.1.0 on its `m7-contract` branch (milestone M7)
except the `playground` subcommand, which waits on the launcher agreement with wafflehouse (H1);
its fixtures are in `fixtures/squint/` and what it found is in `PROPOSALS.md`. rascal's conformance
is under way in its milestone 8. The squint column below is the starting point, kept for review.

## v0.2 (run association and the session)

Both tools build against this draft by path: rascal on its branch `m11-table-manners-v02`
(milestone M11), squint on `m8-v0.2` (milestone M8, `6fef419`). Their fixtures are in
`fixtures/rascal/` and `fixtures/squint/`.

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
