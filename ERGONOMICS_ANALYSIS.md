# Squeal Query Builder: Ergonomics Analysis & Improvement Proposal

**Date**: 2025-12-04
**Author**: Analysis based on research of popular query builders across ecosystems

## Executive Summary

This document analyzes popular query builders across multiple languages (Rust, Python, JavaScript, Go) and proposes ergonomic improvements to squeal while maintaining its core philosophy of simplicity, no magic, and escape hatches.

**Key Finding**: Squeal's foundation is solid, but several ergonomic improvements can make it significantly more developer-friendly without sacrificing its design principles.

---

## Research: Industry Patterns

### Rust Ecosystem

#### **Diesel**
- **Strengths**: Compile-time type safety, prevents runtime errors through type system
- **API**: DSL-based, highly type-safe but requires schema macros
- **Philosophy**: Type safety first, query should match Rust semantics
- **Ergonomics**: Excellent compile-time guarantees, steep learning curve

#### **sea-query**
- **Strengths**: Dynamic query building, parameter injection with auto-sequencing ($1, $2, etc.)
- **API**: Method chaining with Expr constructors, supports multiple databases
- **Philosophy**: Ergonomic AST construction, 100% safe Rust
- **Ergonomics**: Parameter binding alongside expressions prevents "off by one" errors

#### **SQLx**
- **Strengths**: Compile-time checked raw SQL without DSL
- **API**: Direct SQL with macro-based compile-time verification
- **Philosophy**: Raw SQL is fine, verify it at compile time
- **Ergonomics**: Familiar SQL syntax, async-first

### Python: SQLAlchemy

- **Strengths**: Mature (20+ years), comprehensive feature set
- **API**: Method chaining with explicit select() construct
- **Key Patterns**:
  - Eager loading to prevent N+1 queries
  - Automatic parameterization for security
  - Session management for transactions
  - Clear distinction between ORM and Core (query builder)
- **Ergonomics**: Highly readable, encourages best practices

### JavaScript/TypeScript Ecosystem

#### **Knex.js**
- **Strengths**: Pure query builder (not ORM), multi-database support
- **API**: Fluent chainable methods, raw SQL escape hatches
- **Philosophy**: Build SQL programmatically without abstractions
- **Ergonomics**: Simple, predictable, JavaScript-idiomatic

#### **Prisma**
- **Strengths**: Modern type-safe approach, auto-generated client
- **API**: Schema-first with generated type-safe API
- **Philosophy**: Developer experience through code generation
- **Ergonomics**: Excellent TypeScript integration, intuitive CRUD

### Go: Squirrel

- **Strengths**: Composable query parts, conditional building
- **API**: Builder pattern with Eq{} helpers, StatementBuilder for reuse
- **Key Patterns**:
  - Placeholder format handling (?, $1, etc.)
  - StatementBuilder for consistent configuration
  - Easy conditional query building
- **Ergonomics**: Conditional queries without string concatenation

---

## Current Squeal API Analysis

### Strengths

1. **Clean Foundation**: Simple trait system (Sql, Build)
2. **No Magic**: Direct structs, no macros or code generation
3. **Escape Hatches**: Op::O(&str) for custom operators, Term::Atom for raw SQL
4. **Dual Interface**: Both struct construction and fluent builders
5. **Type Safety**: Compile-time correctness without runtime overhead

### Pain Points

#### 1. **Verbose WHERE Clause Construction**

**Current**:
```rust
.where_(Term::Condition(
    Box::new(Term::Atom("status")),
    Op::Equals,
    Box::new(Term::Atom("'active'"))
))
```

**Issues**:
- Heavy nesting with Box::new()
- Repetitive pattern for simple conditions
- Cognitive overhead for common operations

#### 2. **UPDATE API Mismatch**

**Current**:
```rust
U("users")
    .columns(vec!["name", "status"])
    .values(vec!["'John'", "'active'"])
```

**Issues**:
- Columns and values are separate, easy to mismatch
- Not idiomatic for key-value updates
- Requires mental mapping between vectors

#### 3. **No Parameter Binding Support**

**Current**: Users must manually handle $1, $2 placeholders
**Issue**: Error-prone, "off by one" mistakes, no automatic sequencing

#### 4. **Limited Conditional Query Building**

**Current**: No helper for optional WHERE clauses
**Issue**: Users must manually manage Option wrapping for dynamic filters

#### 5. **No Helpers for Common Patterns**

Missing conveniences for:
- Building AND/OR chains
- IN clauses
- BETWEEN operators
- NULL checks (IS NULL, IS NOT NULL)
- Common comparisons (>, <, >=, <=, !=)

---

## Improvement Proposals

### Priority 1: WHERE Clause Ergonomics

#### Proposal 1A: Helper Functions for Conditions

Add non-magical helper functions that reduce boilerplate:

```rust
// New helper functions (can be simple functions, no macros)
pub fn eq<'a>(left: &'a str, right: &'a str) -> Term<'a> {
    Term::Condition(
        Box::new(Term::Atom(left)),
        Op::Equals,
        Box::new(Term::Atom(right))
    )
}

pub fn gt<'a>(left: &'a str, right: &'a str) -> Term<'a> {
    Term::Condition(
        Box::new(Term::Atom(left)),
        Op::O(">"),
        Box::new(Term::Atom(right))
    )
}

pub fn and<'a>(left: Term<'a>, right: Term<'a>) -> Term<'a> {
    Term::Condition(Box::new(left), Op::And, Box::new(right))
}

pub fn or<'a>(left: Term<'a>, right: Term<'a>) -> Term<'a> {
    Term::Condition(Box::new(left), Op::Or, Box::new(right))
}

// Usage becomes:
.where_(and(
    eq("status", "'active'"),
    gt("age", "18")
))
```

**Benefits**:
- Significantly reduced verbosity
- No macros, no magic - just simple functions
- Maintains escape hatches (can still use Term::Condition directly)
- More readable code

#### Proposal 1B: Extend Op Enum

Add common operators to Op enum:

```rust
pub enum Op<'a> {
    And,
    Or,
    Equals,
    NotEquals,        // !=
    GreaterThan,      // >
    LessThan,         // <
    GreaterOrEqual,   // >=
    LessOrEqual,      // <=
    Like,             // LIKE
    In,               // IN
    IsNull,           // IS NULL
    IsNotNull,        // IS NOT NULL
    O(&'a str),       // Escape hatch
}
```

**Benefits**:
- More semantic meaning in code
- Type-safe common operations
- Still have O(&str) for PostgreSQL-specific operators

### Priority 2: UPDATE API Improvement

#### Proposal 2A: Add `set()` Method

```rust
impl<'a> UpdateBuilder<'a> {
    // New method - more ergonomic
    pub fn set(&'a mut self, pairs: Vec<(&'a str, &'a str)>) -> &'a mut UpdateBuilder<'a> {
        for (col, val) in pairs {
            self.columns.push(col);
            self.values.push(val);
        }
        self
    }

    // Keep existing columns/values for backwards compatibility
}

// Usage:
U("users")
    .set(vec![
        ("name", "'John'"),
        ("status", "'active'")
    ])
    .where_(eq("id", "123"))
```

**Benefits**:
- Natural pairing of columns and values
- Harder to mismatch column/value counts
- More SQL-like semantics
- Backwards compatible (keep existing API)

### Priority 3: Parameter Binding Support

#### Proposal 3A: Placeholder Struct

```rust
pub struct Param {
    sequence: usize,
    format: PlaceholderFormat,
}

pub enum PlaceholderFormat {
    QuestionMark,  // MySQL: ?
    Dollar,        // PostgreSQL: $1, $2
    Colon,         // Oracle: :1, :2
}

// Add to builders
impl<'a> QueryBuilder<'a> {
    pub fn placeholder_format(&mut self, fmt: PlaceholderFormat) -> &mut Self {
        self.placeholder_format = Some(fmt);
        self
    }
}

// Usage:
let params = Params::new(PlaceholderFormat::Dollar);
Q()
    .select(vec!["*"])
    .from("users")
    .where_(eq("id", params.next()))  // Returns "$1"
    .and(eq("status", params.next()))  // Returns "$2"
```

**Benefits**:
- Automatic parameter sequencing
- No "off by one" errors
- Multi-database support
- Explicit, no magic

### Priority 4: Conditional Query Building

#### Proposal 4A: Add `where_opt()` Method

```rust
impl<'a> QueryBuilder<'a> {
    pub fn where_opt(&'a mut self, term: Option<Term<'a>>) -> &'a mut QueryBuilder<'a> {
        if let Some(t) = term {
            self.where_clause = Some(t);
        }
        self
    }

    // Also add and_where for chaining multiple conditions
    pub fn and_where(&'a mut self, term: Term<'a>) -> &'a mut QueryBuilder<'a> {
        match &self.where_clause {
            None => self.where_clause = Some(term),
            Some(existing) => {
                self.where_clause = Some(Term::Condition(
                    Box::new(existing.clone()),
                    Op::And,
                    Box::new(term)
                ));
            }
        }
        self
    }
}

// Usage:
let status_filter = if include_inactive {
    None
} else {
    Some(eq("status", "'active'"))
};

Q()
    .select(vec!["*"])
    .from("users")
    .where_opt(status_filter)
    .and_where(gt("age", "18"))  // Always applied
```

**Benefits**:
- Easy dynamic query building
- No manual Option management
- Common pattern in other builders

### Priority 5: Common Helper Functions

#### Proposal 5A: Convenience Functions

```rust
// IN clause helper
pub fn in_<'a>(column: &'a str, values: Vec<&'a str>) -> Term<'a> {
    Term::Atom(&format!("{} IN ({})", column, values.join(", ")))
}

// BETWEEN helper
pub fn between<'a>(column: &'a str, low: &'a str, high: &'a str) -> Term<'a> {
    Term::Atom(&format!("{} BETWEEN {} AND {}", column, low, high))
}

// IS NULL helper
pub fn is_null<'a>(column: &'a str) -> Term<'a> {
    Term::Condition(
        Box::new(Term::Atom(column)),
        Op::IsNull,
        Box::new(Term::Null)
    )
}

// Usage:
Q()
    .select(vec!["*"])
    .from("users")
    .where_(and(
        in_("status", vec!["'active'", "'pending'"]),
        between("age", "18", "65")
    ))
```

**Benefits**:
- Handles complex SQL patterns
- Still just functions, no magic
- Composable with existing API

---

## Implementation Strategy

### Phase 1: Core Ergonomics (Backwards Compatible)

1. Add comparison helper functions (eq, gt, lt, etc.)
2. Add logical operator helpers (and, or)
3. Extend Op enum with common operators
4. Add UpdateBuilder::set() method (keep existing columns/values)

**Impact**: Immediate ergonomics win, zero breaking changes

### Phase 2: Advanced Features

1. Add parameter binding support
2. Add where_opt() and and_where() methods
3. Add convenience functions (in_, between, is_null)

**Impact**: Enables dynamic query building, maintains simplicity

### Phase 3: Documentation & Examples

1. Update README with new patterns
2. Add cookbook for common use cases
3. Document migration from verbose to ergonomic API

---

## Comparison: Before & After

### SELECT with WHERE

**Before**:
```rust
Q()
    .select(vec!["id", "name"])
    .from("users")
    .where_(Term::Condition(
        Box::new(Term::Atom("status")),
        Op::Equals,
        Box::new(Term::Condition(
            Box::new(Term::Atom("'active'")),
            Op::And,
            Box::new(Term::Condition(
                Box::new(Term::Atom("age")),
                Op::O(">"),
                Box::new(Term::Atom("18"))
            ))
        ))
    ))
    .build()
```

**After** (with helper functions):
```rust
Q()
    .select(vec!["id", "name"])
    .from("users")
    .where_(and(
        eq("status", "'active'"),
        gt("age", "18")
    ))
    .build()
```

### UPDATE

**Before**:
```rust
U("users")
    .columns(vec!["name", "email", "status"])
    .values(vec!["'John'", "'john@example.com'", "'active'"])
    .where_(Term::Condition(
        Box::new(Term::Atom("id")),
        Op::Equals,
        Box::new(Term::Atom("123"))
    ))
    .build()
```

**After**:
```rust
U("users")
    .set(vec![
        ("name", "'John'"),
        ("email", "'john@example.com'"),
        ("status", "'active'")
    ])
    .where_(eq("id", "123"))
    .build()
```

### Dynamic Filters

**Before**:
```rust
let mut qb = Q();
qb.select(vec!["*"]).from("users");

let mut conditions = Vec::new();
if let Some(status) = status_filter {
    conditions.push(Term::Condition(
        Box::new(Term::Atom("status")),
        Op::Equals,
        Box::new(Term::Atom(status))
    ));
}
if let Some(min_age) = min_age_filter {
    conditions.push(Term::Condition(
        Box::new(Term::Atom("age")),
        Op::O(">="),
        Box::new(Term::Atom(min_age))
    ));
}

// Manually combine conditions...
```

**After**:
```rust
Q()
    .select(vec!["*"])
    .from("users")
    .where_opt(status_filter.map(|s| eq("status", s)))
    .where_opt(min_age_filter.map(|age| gte("age", age)))
    .build()
```

---

## Maintaining Squeal's Philosophy

All proposals maintain squeal's core principles:

### ✅ Keep it Simple & Stupid
- All helpers are simple functions, no complex abstractions
- Can still use verbose API for clarity when needed

### ✅ No Magic
- No macros required (except for advanced type safety, which is opt-in)
- No code generation
- No attribute macros
- Clear, explicit behavior

### ✅ Escape Hatches Built In
- Op::O(&str) still available for any operator
- Term::Atom(&str) for raw SQL fragments
- Can always drop to direct struct construction

### ✅ Valid Construction = Valid SQL
- Helpers build the same Term structures
- No new validation layer
- Same guarantees as before

---

## Recommendations Summary

### Must Have (High Impact, Low Risk)
1. **Comparison helper functions** (eq, gt, lt, gte, lte, ne)
2. **Logical helper functions** (and, or)
3. **UpdateBuilder::set()** method for paired updates
4. **Extended Op enum** with common operators

### Should Have (High Impact, Medium Risk)
5. **where_opt()** and **and_where()** for dynamic queries
6. **Common SQL helpers** (in_, between, is_null)

### Nice to Have (Medium Impact, Complexity)
7. **Parameter binding support** (requires design consideration)
8. **Builder reuse patterns** (like Go's StatementBuilder)

### Documentation Improvements
9. **Cookbook** with common patterns
10. **Migration guide** showing verbose vs ergonomic approaches
11. **Examples** of dynamic query building

---

## References

### Rust
- [Diesel](https://diesel.rs/) - Type-safe ORM and query builder
- [sea-query](https://github.com/SeaQL/sea-query) - Dynamic SQL query builder
- [SQLx](https://github.com/launchbadge/sqlx) - Compile-time checked SQL

### Python
- [SQLAlchemy](https://www.sqlalchemy.org/) - Comprehensive SQL toolkit

### JavaScript/TypeScript
- [Knex.js](http://knexjs.org/) - SQL query builder
- [Prisma](https://www.prisma.io/) - Next-generation ORM

### Go
- [Squirrel](https://github.com/Masterminds/squirrel) - Fluent SQL generator

### Best Practices
- [Dynamic WHERE Clauses](https://hexdocs.pm/ecto/dynamic-queries.html) - Ecto patterns
- [Query Builder Ergonomics](https://use-the-index-luke.com/) - SQL performance patterns

---

## Next Steps

1. **Community Feedback**: Share this analysis with users/stakeholders
2. **Prioritize**: Determine which improvements to implement first
3. **Prototype**: Implement Phase 1 helpers in a branch
4. **Test**: Ensure backwards compatibility
5. **Document**: Update examples and guides
6. **Release**: Iterative rollout with clear migration path

---

**Conclusion**: Squeal has an excellent foundation. With targeted ergonomic improvements, it can compete with established query builders while maintaining its unique philosophy of simplicity and explicitness.
