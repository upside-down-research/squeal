# CLAUDE.md - Development Guide for squeal

## Project Overview

**squeal** is a SQL query builder library for Rust targeting PostgreSQL. It provides a type-safe way to construct SQL queries using Rust structures, with both direct struct construction and fluent builder APIs.

- **Version**: 0.0.6 (pre-release)
- **MSRV**: Rust 1.90
- **License**: LGPL 3.0 or later
- **Repository**: https://gitlab.com/upside-down-research/oss/squeal

## Project Structure

```
squeal/
├── src/
│   └── lib.rs           # Core library (all types, traits, and builders)
├── tests/
│   └── integration_test.rs  # Integration tests (includes Docker PostgreSQL tests)
├── benches/
│   └── benchmark.rs     # Criterion benchmarks
├── Cargo.toml           # Project manifest
├── README.md            # Project readme
├── CHANGELOG.md         # Version history
└── LICENSE.md           # LGPL 3.0 license
```

## Development Commands

### Build
```bash
cargo build
```

### Run Tests
```bash
# Run all unit and integration tests (excludes Docker tests)
cargo test

# Run with Docker-based PostgreSQL integration tests (requires Docker)
cargo test --features postgres-docker
```

### Linting (REQUIRED)
```bash
# Run clippy - ALL warnings must be addressed before committing
cargo clippy

# Auto-fix clippy warnings where possible
cargo clippy --fix --lib -p squeal
```

### Benchmarks
```bash
cargo bench
```

### Documentation
```bash
cargo doc --open
```

## Code Quality Requirements

### MANDATORY: High Test Coverage
- Every new feature MUST include corresponding unit tests
- Complex functionality should have integration tests
- Tests are located in:
  - `src/lib.rs` (inline `#[cfg(test)]` module for unit tests)
  - `tests/integration_test.rs` (integration and Docker tests)

### MANDATORY: Clippy Clean
- Run `cargo clippy` before every commit
- ALL clippy warnings must be fixed
- Use `cargo clippy --fix` for auto-fixable issues
- Common issues to watch for:
  - `useless_format` - Use `.to_string()` instead of `format!("{}", x)`
  - `clone_on_copy` - Don't clone Copy types
  - `needless_borrow` - Remove unnecessary `&` references
  - `single_char_add_str` - Use `push(')')` instead of `push_str(")")`
  - `mismatched_lifetime_syntaxes` - Ensure consistent lifetime annotations

## Architecture

### Core Traits

- **`Sql`** - Implemented by all query-building types; provides `sql() -> String`
- **`Build`** - Trait for builder pattern structs

### Query Types

| Type | Builder | Function | Description |
|------|---------|----------|-------------|
| `Query` | `QueryBuilder` | `Q()` | SELECT queries |
| `Insert` | `InsertBuilder` | `I(table)` | INSERT statements |
| `Update` | `UpdateBuilder` | `U(table)` | UPDATE statements |
| `Delete` | `DeleteBuilder` | `D(table)` | DELETE statements |
| `CreateTable` | `TableBuilder` | `T(table)` | CREATE TABLE DDL |
| `DropTable` | `TableBuilder` | `T(table)` | DROP TABLE DDL |

### Key Enums

- **`Columns`** - `Star` for `*`, `Selected(Vec<&str>)` for specific columns
- **`Op`** - Operators: `And`, `Or`, `Equals`, `O(&str)` for custom operators
- **`Term`** - WHERE clause terms: `Atom`, `Condition`, `Parens`, `Null`
- **`OrderedColumn`** - `Asc(&str)` or `Desc(&str)` for ORDER BY

### Usage Examples

```rust
use squeal::*;

// Fluent SELECT query
let mut qb = Q();
let query = qb.select(vec!["a", "b"])
    .from("users")
    .where_(Term::Condition(
        Box::new(Term::Atom("active")),
        Op::Equals,
        Box::new(Term::Atom("true"))))
    .limit(10)
    .build();
assert_eq!(query.sql(), "SELECT a, b FROM users WHERE active = true LIMIT 10");

// Fluent INSERT
let insert = I("users")
    .columns(vec!["name", "email"])
    .values(vec!["'John'", "'john@example.com'"])
    .build();
assert_eq!(insert.sql(), "INSERT INTO users (name, email) VALUES ('John', 'john@example.com')");

// Direct struct construction
let query = Query {
    select: Some(Select::new(Columns::Star)),
    from: Some("table"),
    where_clause: None,
    group_by: None,
    having: None,
    order_by: None,
    limit: Some(10),
    offset: None,
    for_update: false,
};
```

## Testing Guidelines

### Unit Test Pattern
Tests are in the `#[cfg(test)]` module at the bottom of `src/lib.rs`:

```rust
#[test]
fn test_feature_name() {
    let result = /* construct query */;
    assert_eq!(result.sql(), "EXPECTED SQL STRING");
}
```

### Integration Test Pattern
Docker-based tests require the `postgres-docker` feature:

```rust
#[test]
#[cfg_attr(not(feature = "postgres-docker"), ignore)]
fn test_with_postgres() -> Result<(), String> {
    let mut harness = DockerTests::new();
    let (node, mut conn) = harness.get_new_node_and_connection();
    // Execute actual SQL against PostgreSQL
    Ok(())
}
```

## Dependencies

### Runtime
- `postgres` (0.19.7) - PostgreSQL client library

### Dev/Test
- `criterion` (0.4) - Benchmarking framework
- `testcontainers` (0.15) - Docker-based integration testing
- `testcontainers-modules` (0.2.1) - PostgreSQL testcontainer module

## Known Issues / TODO

1. The benchmark file (`benches/benchmark.rs`) uses outdated API (Query without Option wrappers, Q() with table argument) - needs updating
2. Multiple clippy warnings about lifetime syntax inconsistencies need fixing
3. The `Build` trait is defined but not consistently used across builders

## Design Philosophy

From the library docs:
- "Keep it simple & stupid"
- No attributes, macros, or other "magic"
- Escape hatches built in (`Op::O(&str)` for custom operators, raw string atoms)
- Any valid construction should produce syntactically valid SQL
