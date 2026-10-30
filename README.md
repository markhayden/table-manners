# waffle-tools-contract

The external interface shared by the waffle image tools: [squint](https://github.com/markhayden/squint)
(checks an image against what was asked of it), [rascal](https://github.com/markhayden/rascal)
(finds, masks and composites regions) and the tools after them. Each tool keeps its own intent and
internals; what an agent or a pipeline sees from outside is the same in all of them, so they chain
without glue code.

- [`CONTRACT.md`](CONTRACT.md): the specification (v0.1, draft).
- `schemas/`, `vectors/`, `fixtures/`, `acceptance/` and the `waffle-tools-contract` Rust crate
  follow once the text is agreed.

Apache-2.0.
