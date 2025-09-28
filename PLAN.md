# bunner-qs refactor roadmap

## Phase 1 — Baseline cleanup and scaffolding ✅
- Remove legacy shim modules (`api/`, `core/`, `buffer_pool.rs`, `encoding.rs`, `ordered_map.rs`, `value.rs`, `options.rs`).
- Create domain directories: `config/`, `model/`, `parsing/`, `stringify/`, `serde_adapter/`, `memory/`, `util/`.
- Move existing implementations from `core/`, `nested.rs`, `serde_impl/`, and other files into the corresponding domain directories (no behaviour changes yet).
- Update `lib.rs` to compile with the new module declarations while still re-exporting the previous API.

## Phase 2 — Parsing module decomposition ✅
- Split `parse.rs` into focused modules (`runtime.rs`, `preflight.rs`, `decoder.rs`, `arena/`, `builder.rs`, `state.rs`, `key_path.rs`, `direct.rs`).
- Ensure `parsing::mod.rs` exposes only `parse`, `parse_with`, and `parse_query_map` (crate-private) while internal components remain private.
- Rewire existing code to use the new modules and delete residual helpers made obsolete by the split.

## Phase 3 — Stringify module decomposition ✅
- Break `stringify.rs` into `runtime.rs`, `walker.rs`, `writer.rs`, `validate.rs`, and `encode.rs` inside `stringify/`.
- Keep only `stringify` and `stringify_with` public; internal helpers become private to the module tree.
- Adjust serialization helpers to call into the new module structure.

## Phase 4 — Public API consolidation and polish
- Finalise `lib.rs` exports (only core functions, options, and error types).
- Refresh `prelude.rs` to match the new public surface.
- Update internal references (`serde_adapter`, `memory`, `util`) to guarantee one-way dependencies between domains.
- Run `cargo fmt` and `cargo check` to confirm the crate builds after the refactor.
