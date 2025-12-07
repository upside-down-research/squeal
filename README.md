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
- `JOIN` operations (INNER, LEFT, RIGHT, FULL, CROSS) with table or subquery sources
- `WITH` / Common Table Expressions (CTEs) for complex queries
- `INSERT` statements with single or multiple value sets, or from SELECT queries
- `INSERT ... ON CONFLICT` (UPSERT) with DO NOTHING or DO UPDATE
- `UPDATE` statements with SET and WHERE clauses
- `DELETE` statements with WHERE conditions
- `RETURNING` clauses for INSERT, UPDATE, and DELETE
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
    .where_(eq("active", "true"))
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
    .where_(lt("last_login", "'2024-01-01'"))
    .build();

assert_eq!(
    update.sql(),
    "UPDATE users SET status = 'inactive' WHERE last_login < '2024-01-01'"
);

// DELETE with fluent builder
let delete = D("logs")
    .where_(lt("created_at", "NOW() - INTERVAL '30 days'"))
    .build();
```

### Direct Struct Construction

For maximum control, you can construct query structs directly:

```rust
use squeal::*;

let query = Query {
    with_clause: None,
    select: Some(Select::new(Columns::Star, None)),
    from: Some(FromSource::Table("products")),
    joins: vec![],
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

// Use Op::O for custom PostgreSQL operators
let custom_op = Term::Condition(
    Box::new(Term::Atom("data")),
    Op::O("@>"),  // PostgreSQL JSONB contains operator
    Box::new(Term::Atom("'{\"type\": \"click\"}'"))
);

let query = Q()
    .select(vec!["*"])
    .from("events")
    .where_(custom_op)
    .build();
```

## Advanced Features

### JOINs

Combine data from multiple tables with type-safe JOIN operations:

```rust
use squeal::*;

// Find all users with their order count
let query = Q()
    .select(vec!["users.name", "users.email", "COUNT(orders.id) as order_count"])
    .from("users")
    .inner_join("orders", eq("users.id", "orders.user_id"))
    .group_by(vec!["users.id", "users.name", "users.email"])
    .order_by(vec![OrderedColumn::Desc("order_count")])
    .build();

// LEFT JOIN to include users with no orders
let query_with_nulls = Q()
    .select(vec!["users.name", "COALESCE(orders.total, 0) as total"])
    .from("users")
    .left_join("orders", eq("users.id", "orders.user_id"))
    .build();
```

### Common Table Expressions (WITH clause)

Build complex queries with CTEs for better readability:

```rust
use squeal::*;

// Calculate monthly revenue and compare to average
let monthly_revenue = Q()
    .select(vec![
        "DATE_TRUNC('month', created_at) as month",
        "SUM(total) as revenue"
    ])
    .from("orders")
    .group_by(vec!["month"])
    .build();

let query = Q()
    .with("monthly_revenue", monthly_revenue)
    .select(vec![
        "month",
        "revenue",
        "(revenue - AVG(revenue) OVER ()) as diff_from_avg"
    ])
    .from("monthly_revenue")
    .order_by(vec![OrderedColumn::Desc("month")])
    .build();

// Result: WITH monthly_revenue AS (SELECT ...) SELECT month, revenue, ...
```

### UPSERT (INSERT ... ON CONFLICT)

Handle unique constraint violations gracefully:

```rust
use squeal::*;

// Insert user, do nothing if email already exists
let insert = I("users")
    .columns(vec!["email", "name", "created_at"])
    .values(vec!["'alice@example.com'", "'Alice'", "NOW()"])
    .on_conflict_do_nothing(vec!["email"])
    .build();

// Result: INSERT INTO users (email, name, created_at) VALUES (...)
//         ON CONFLICT (email) DO NOTHING

// Insert or update: update the name if email exists
let upsert = I("users")
    .columns(vec!["email", "name", "login_count"])
    .values(vec!["'bob@example.com'", "'Bob Smith'", "'1'"])
    .on_conflict_do_update(
        vec!["email"],
        vec![
            ("name", "'Bob Smith'"),
            ("login_count", "users.login_count + 1"),
            ("updated_at", "NOW()")
        ]
    )
    .returning(Columns::Selected(vec!["id", "email", "updated_at"]))
    .build();

// Result: INSERT INTO users (...) VALUES (...)
//         ON CONFLICT (email) DO UPDATE SET name = '...', login_count = ...
//         RETURNING id, email, updated_at
```

### Multiple Row INSERT

Efficiently insert multiple rows in a single statement:

```rust
use squeal::*;

let insert = I("products")
    .columns(vec!["name", "price", "category"])
    .rows(vec![
        vec!["'Laptop'", "'999.99'", "'electronics'"],
        vec!["'Mouse'", "'24.99'", "'electronics'"],
        vec!["'Desk'", "'299.99'", "'furniture'"],
    ])
    .returning(Columns::Selected(vec!["id", "name"]))
    .build();

// Result: INSERT INTO products (name, price, category) VALUES
//         ('Laptop', 999.99, 'electronics'),
//         ('Mouse', 24.99, 'electronics'),
//         ('Desk', 299.99, 'furniture')
//         RETURNING id, name
```

### INSERT ... SELECT

Copy data from one table to another:

```rust
use squeal::*;

// Archive old orders
let select_old_orders = Q()
    .select(vec!["id", "user_id", "total", "created_at"])
    .from("orders")
    .where_(lt("created_at", "'2023-01-01'"))
    .build();

let archive = I("orders_archive")
    .columns(vec!["order_id", "user_id", "total", "order_date"])
    .select(select_old_orders)
    .build();

// Result: INSERT INTO orders_archive (order_id, user_id, total, order_date)
//         SELECT id, user_id, total, created_at FROM orders
//         WHERE created_at < '2023-01-01'
```

### RETURNING Clause

Get values from INSERT, UPDATE, or DELETE operations:

```rust
use squeal::*;

// Get the auto-generated ID after insert
let insert = I("posts")
    .columns(vec!["title", "content", "author_id"])
    .values(vec!["'Hello World'", "'First post!'", "'1'"])
    .returning(Columns::Selected(vec!["id", "created_at"]))
    .build();

// Update and return the modified rows
let update = U("users")
    .columns(vec!["status", "updated_at"])
    .values(vec!["'inactive'", "NOW()"])
    .where_(lt("last_login", "NOW() - INTERVAL '1 year'"))
    .returning(Columns::Selected(vec!["id", "email"]))
    .build();

// Delete and track what was removed
let delete = D("sessions")
    .where_(lt("expires_at", "NOW()"))
    .returning(Columns::Selected(vec!["user_id", "session_id"]))
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
