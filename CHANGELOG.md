* Unreleased
Add `.github/workflows/release.yml`: pushing a tag `vX.Y.Z` publishes the crate to crates.io (#35). Requires the repository secret `CARGO_REGISTRY_TOKEN`.
Add `BuildError`, `Insert::validate`, `Update::validate`, `InsertBuilder::build_checked` and `UpdateBuilder::build_checked`.
`Insert` and `Update` now reject empty columns, empty rows, and value counts that differ from the column count (#22, #23, #24).
Behaviour change: `build()` and `sql()` on such input panic. Before, they rendered invalid SQL or silently truncated.

* 0.1.2
Add `Term::Any`, `Term::All`, `eq_any` and `ne_all` for PostgreSQL array comparison (`id = ANY($1)`, `id != ALL($1)`).

* 0.0.1
Pushed to crates.io to hold the namespace.
