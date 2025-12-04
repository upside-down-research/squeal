# squeal

A type-safe SQL query builder for Rust targeting PostgreSQL.

[![License: LGPL v3](https://img.shields.io/badge/License-LGPL%20v3-blue.svg)](https://www.gnu.org/licenses/lgpl-3.0)

**squeal** provides a simple, type-safe way to construct SQL queries using Rust structures. It offers both direct struct construction and fluent builder APIs with escape hatches built in for complex use cases.

## Philosophy

- Keep it simple & stupid
- No attributes, macros, or other "magic"
- Escape hatches built in for custom SQL
- Any valid construction produces syntactically valid SQL

## Features

- 🔒 **Type-safe query construction** - Catch errors at compile time
- 🔨 **Fluent builder API** - Chain methods for readable query construction
- 📦 **Direct struct construction** - Full control when needed
- 🎯 **PostgreSQL targeting** - Optimized for PostgreSQL dialect
- 🚀 **Zero runtime overhead** - Queries are built, not interpreted

### Supported Operations

- `SELECT` queries with WHERE, GROUP BY, HAVING, ORDER BY, LIMIT, OFFSET
- `INSERT` statements with single or multiple value sets
- `UPDATE` statements with SET and WHERE clauses
- `DELETE` statements with WHERE conditions
- `CREATE TABLE` DDL statements
- `DROP TABLE` DDL statements

## Installation

Add to your `Cargo.toml`:

```toml
[dependencies]
squeal = "0.0.6"
```

**MSRV**: Rust 1.75.0

## Quick Start

```rust
use squeal::*;

// SELECT query with fluent builder
let query = Q()
    .select(vec!["id", "name", "email"])
    .from("users")
    .where_(Term::Condition(
        Box::new(Term::Atom("active")),
        Op::Equals,
        Box::new(Term::Atom("true"))
    ))
    .order_by(vec![OrderedColumn::Desc("created_at")])
    .limit(10)
    .build();

assert_eq!(
    query.sql(),
    "SELECT id, name, email FROM users WHERE active = true ORDER BY created_at DESC LIMIT 10"
);

// INSERT with fluent builder
let insert = I("users")
    .columns(vec!["name", "email"])
    .values(vec!["'Alice'", "'alice@example.com'"])
    .build();

assert_eq!(
    insert.sql(),
    "INSERT INTO users (name, email) VALUES ('Alice', 'alice@example.com')"
);

// UPDATE with fluent builder
let update = U("users")
    .columns(vec!["status"])
    .values(vec!["'inactive'"])
    .where_(Term::Condition(
        Box::new(Term::Atom("last_login")),
        Op::O("<"),
        Box::new(Term::Atom("'2024-01-01'"))
    ))
    .build();

assert_eq!(
    update.sql(),
    "UPDATE users SET status = 'inactive' WHERE last_login < '2024-01-01'"
);

// DELETE with fluent builder
let delete = D("logs")
    .where_(Term::Condition(
        Box::new(Term::Atom("created_at")),
        Op::O("<"),
        Box::new(Term::Atom("NOW() - INTERVAL '30 days'"))
    ))
    .build();
```

### Direct Struct Construction

For maximum control, you can construct query structs directly:

```rust
use squeal::*;

let query = Query {
    select: Some(Select::new(Columns::Star)),
    from: Some("products"),
    where_clause: None,
    group_by: None,
    having: None,
    order_by: Some(vec![OrderedColumn::Asc("price")]),
    limit: Some(100),
    offset: Some(0),
    for_update: false,
};

assert_eq!(query.sql(), "SELECT * FROM products ORDER BY price ASC LIMIT 100 OFFSET 0");
```

### Escape Hatches

Use custom operators and raw SQL fragments when needed:

```rust
use squeal::*;

let query = Q()
    .select(vec!["*"])
    .from("events")
    .where_(Term::Condition(
        Box::new(Term::Atom("data")),
        Op::O("@>"),  // PostgreSQL JSONB contains operator
        Box::new(Term::Atom("'{\"type\": \"click\"}'"))
    ))
    .build();
```

## Development

### Build and Test

```bash
# Build the project
cargo build

# Run tests (excludes Docker tests)
cargo test

# Run all tests including PostgreSQL integration tests (requires Docker)
cargo test --features postgres-docker

# Run benchmarks
cargo bench

# Generate documentation
cargo doc --open
```

### Code Quality

```bash
# Run clippy (required before committing)
cargo clippy

# Auto-fix clippy warnings
cargo clippy --fix --lib -p squeal
```

## Project Status

**Version**: 0.0.6 (pre-release)

This library is in active development. The API is stabilizing but may still change. It is suitable for experimentation and early adoption, but not recommended for production use until version 1.0.

## Related Projects

- PostgreSQL Rust driver: [rust-postgres](https://docs.rs/postgres/latest/postgres/)
- Similar library for Go: [sqlf](https://github.com/leporo/sqlf)

## Repository

- **GitLab**: https://gitlab.com/upside-down-research/oss/squeal

## License

LGPL 3.0 or later. See [LICENSE.md](LICENSE.md) for details.

## Contributing

Contributions are welcome! Please ensure:
- All tests pass (`cargo test`)
- No clippy warnings (`cargo clippy`)
- New features include tests
- Code follows the existing style

For more detailed development guidelines, see [CLAUDE.md](CLAUDE.md).
