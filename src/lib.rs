#![forbid(unsafe_code)]
//! # squeal - Type-Safe SQL Query Builder for PostgreSQL
//!
//! **squeal** is a lightweight, type-safe SQL query builder for Rust that targets PostgreSQL.
//! It provides both fluent builder APIs and direct struct construction for maximum flexibility.
//!
//! ## Philosophy
//!
//! - **Keep it simple** - No macros, no magic, just Rust types
//! - **Type-safe** - Catch errors at compile time, not runtime
//! - **Escape hatches** - Custom operators and raw SQL when you need them
//! - **Zero overhead** - Queries are built at compile time
//!
//! ## Quick Start
//!
//! ```
//! use squeal::*;
//!
//! // Build a SELECT query with the fluent API
//! let mut qb = Q();
//! let query = qb.select(vec!["id", "name", "email"])
//!     .from("users")
//!     .where_(eq("active", "true"))
//!     .order_by(vec![OrderedColumn::Desc("created_at")])
//!     .limit(10)
//!     .build();
//!
//! assert_eq!(
//!     query.sql(),
//!     "SELECT id, name, email FROM users WHERE active = true ORDER BY created_at DESC LIMIT 10"
//! );
//! ```
//!
//! ## Core Concepts
//!
//! ### Fluent Builders
//!
//! All query types have builder functions with short, memorable names:
//! - `Q()` - Build SELECT queries
//! - `I(table)` - Build INSERT statements
//! - `U(table)` - Build UPDATE statements
//! - `D(table)` - Build DELETE statements
//! - `T(table)` - Build CREATE/DROP TABLE DDL
//!
//! ### Terms and Conditions
//!
//! The `Term` enum represents WHERE clause conditions. Use helper functions
//! for common patterns:
//!
//! ```
//! use squeal::*;
//!
//! // Simple equality: active = true
//! let simple = eq("active", "true");
//!
//! // Complex conditions with AND/OR
//! let complex = Term::Condition(
//!     Box::new(eq("status", "'published'")),
//!     Op::And,
//!     Box::new(gt("views", "1000"))
//! );
//!
//! let mut qb = Q();
//! let query = qb.select(vec!["title", "views"])
//!     .from("posts")
//!     .where_(complex)
//!     .build();
//!
//! assert_eq!(
//!     query.sql(),
//!     "SELECT title, views FROM posts WHERE status = 'published' AND views > 1000"
//! );
//! ```
//!
//! ## Advanced Features
//!
//! ### JOINs
//!
//! All standard JOIN types are supported:
//!
//! ```
//! use squeal::*;
//!
//! let mut qb = Q();
//! let query = qb.select(vec!["users.name", "COUNT(orders.id) as order_count"])
//!     .from("users")
//!     .left_join("orders", eq("users.id", "orders.user_id"))
//!     .group_by(vec!["users.id", "users.name"])
//!     .order_by(vec![OrderedColumn::Desc("order_count")])
//!     .build();
//!
//! assert_eq!(
//!     query.sql(),
//!     "SELECT users.name, COUNT(orders.id) as order_count FROM users LEFT JOIN orders ON users.id = orders.user_id GROUP BY users.id, users.name ORDER BY order_count DESC"
//! );
//! ```
//!
//! ### Common Table Expressions (WITH)
//!
//! Build complex queries with CTEs for better readability:
//!
//! ```
//! use squeal::*;
//!
//! // Define a CTE to find high-value customers
//! let mut qb1 = Q();
//! let high_value = qb1.select(vec!["user_id", "SUM(total) as lifetime_value"])
//!     .from("orders")
//!     .group_by(vec!["user_id"])
//!     .having(gt("SUM(total)", "1000"))
//!     .build();
//!
//! // Use the CTE in the main query
//! let mut qb2 = Q();
//! let query = qb2.with("high_value_customers", high_value)
//!     .select(vec!["users.name", "hvc.lifetime_value"])
//!     .from("users")
//!     .inner_join("high_value_customers hvc", eq("users.id", "hvc.user_id"))
//!     .order_by(vec![OrderedColumn::Desc("hvc.lifetime_value")])
//!     .build();
//!
//! assert_eq!(
//!     query.sql(),
//!     "WITH high_value_customers AS (SELECT user_id, SUM(total) as lifetime_value FROM orders GROUP BY user_id HAVING SUM(total) > 1000) SELECT users.name, hvc.lifetime_value FROM users INNER JOIN high_value_customers hvc ON users.id = hvc.user_id ORDER BY hvc.lifetime_value DESC"
//! );
//! ```
//!
//! ### UPSERT (INSERT ... ON CONFLICT)
//!
//! Handle unique constraint violations gracefully:
//!
//! ```
//! use squeal::*;
//!
//! // Insert or update user login count
//! let mut ib = I("users");
//! let upsert = ib.columns(vec!["email", "name", "login_count"])
//!     .values(vec!["'alice@example.com'", "'Alice'", "'1'"])
//!     .on_conflict_do_update(
//!         vec!["email"],
//!         vec![
//!             ("login_count", "users.login_count + 1"),
//!             ("last_login", "NOW()")
//!         ]
//!     )
//!     .returning(Columns::Selected(vec!["id", "login_count"]))
//!     .build();
//!
//! assert_eq!(
//!     upsert.sql(),
//!     "INSERT INTO users (email, name, login_count) VALUES ('alice@example.com', 'Alice', '1') ON CONFLICT (email) DO UPDATE SET login_count = users.login_count + 1, last_login = NOW() RETURNING id, login_count"
//! );
//! ```
//!
//! ### RETURNING Clauses
//!
//! Get auto-generated values and track modifications:
//!
//! ```
//! use squeal::*;
//!
//! // Get the ID of newly inserted row
//! let mut ib = I("posts");
//! let insert = ib.columns(vec!["title", "content"])
//!     .values(vec!["'Hello World'", "'My first post'"])
//!     .returning(Columns::Selected(vec!["id", "created_at"]))
//!     .build();
//!
//! assert_eq!(
//!     insert.sql(),
//!     "INSERT INTO posts (title, content) VALUES ('Hello World', 'My first post') RETURNING id, created_at"
//! );
//!
//! // Track what was deleted
//! let mut db = D("old_logs");
//! let delete = db.where_(lt("created_at", "NOW() - INTERVAL '90 days'"))
//!     .returning(Columns::Selected(vec!["id", "created_at"]))
//!     .build();
//!
//! assert_eq!(
//!     delete.sql(),
//!     "DELETE FROM old_logs WHERE created_at < NOW() - INTERVAL '90 days' RETURNING id, created_at"
//! );
//! ```
//!
//! ## Parameterized Queries
//!
//! Use the `Parameterized` trait for prepared statements:
//!
//! ```
//! use squeal::*;
//!
//! let mut qb = Q();
//! let param = qb.param();
//! let query = qb.select(vec!["*"])
//!     .from("users")
//!     .where_(eq("email", &param))
//!     .build();
//!
//! assert_eq!(query.sql(), "SELECT * FROM users WHERE email = $1");
//! ```
//!
//! ## More Information
//!
//! See the [README](https://github.com/upside-down-research/squeal) for complete documentation
//! and examples.

pub mod queries;

pub use queries::create_table::{CreateTable, T, TableBuilder};
pub use queries::delete::{D, Delete, DeleteBuilder};
pub use queries::drop_table::DropTable;
pub use queries::insert::{I, Insert, InsertBuilder, InsertSource, OnConflict};
pub use queries::select::{Columns, Select, SelectExpression};
pub use queries::update::{U, Update, UpdateBuilder};

/// The Sql trait is implemented by all objects that can be used in a query.
/// It provides a single method, sql(), that returns a String.
///
/// This is not intended to be implemented by the user.
pub trait Sql {
    /// Returns the fragment which will be assembled in the given query.
    fn sql(&self) -> String;
}

/// The Build trait is used by the XBuilder structs to build the X struct.
/// This is a means of providing a nice factory/fluent interface.
pub trait Build {
    /// Builds the final struct from the builder
    fn build(&self) -> Self;
}

/// The Parameterized trait provides PostgreSQL parameter placeholder generation.
/// Implemented by all query builder structs to provide consistent param() API.
pub trait Parameterized {
    /// Returns the next PostgreSQL parameter placeholder ($1, $2, $3, etc.)
    /// Each builder maintains its own isolated counter.
    fn param(&mut self) -> String;
}
#[derive(Clone)]
pub enum Distinct<'a> {
    All,
    On(Vec<&'a str>),
}

/// The Op enum is used to specify the operator in a condition.
/// It is used in the Term struct.
///
/// The Op::O variant is an escape hatch to allow you to use any operator you want.
#[derive(Clone)]
pub enum Op<'a> {
    /// Logical AND operator
    And,
    /// Logical OR operator
    Or,
    /// Equality operator (=)
    Equals,
    /// Not equals operator (!=)
    NotEquals,
    /// Greater than operator (>)
    GreaterThan,
    /// Less than operator (<)
    LessThan,
    /// Greater than or equal operator (>=)
    GreaterOrEqual,
    /// Less than or equal operator (<=)
    LessOrEqual,
    /// LIKE operator for pattern matching
    Like,
    /// IN operator for set membership
    In,
    /// EXISTS operator for subquery existence testing
    Exists,
    /// NOT EXISTS operator for subquery non-existence testing
    NotExists,
    /// ANY operator for comparing against any value in a subquery
    Any,
    /// ALL operator for comparing against all values in a subquery
    All,
    /// Custom operator escape hatch
    O(&'a str),
}

impl<'a> Sql for Op<'a> {
    fn sql(&self) -> String {
        match &self {
            Op::And => "AND",
            Op::Or => "OR",
            Op::Equals => "=",
            Op::NotEquals => "!=",
            Op::GreaterThan => ">",
            Op::LessThan => "<",
            Op::GreaterOrEqual => ">=",
            Op::LessOrEqual => "<=",
            Op::Like => "LIKE",
            Op::In => "IN",
            Op::Exists => "EXISTS",
            Op::NotExists => "NOT EXISTS",
            Op::Any => "ANY",
            Op::All => "ALL",
            Op::O(s) => s,
        }
        .to_string()
    }
}

/// The Term enum is used to specify a condition in a query (WHERE clause).
/// It is used in the Query struct.
///
/// A Term can be an atom, a condition, parentheses or null. Observant minds might notice that
/// this is a fragment of a grammar and simply a reified syntax tree.
///
/// # Examples
///
/// Atom:
/// ```
/// use squeal::*;
/// let result = Term::Atom("a").sql();
/// assert_eq!(result, "a");
/// ```
///
/// A number of different conditions and complex combinations:
/// ```
/// use squeal::*;
/// let result = Term::Condition(
///    Box::new(Term::Atom("a")),
///   Op::O("<>"),
/// Box::new(Term::Atom("b")),
/// ).sql();
/// assert_eq!(result, "a <> b");
/// ```
/// An example setting up `a = b AND (c = d OR e <> f)`:
///
/// ```
/// use squeal::*;
/// let result = Term::Condition(
///   Box::new(Term::Atom("a")),
/// Op::Equals,
/// Box::new(Term::Condition(
///   Box::new(Term::Atom("b")),
/// Op::And,
/// Box::new(Term::Parens(Box::new(Term::Condition(
///  Box::new(Term::Atom("c")),
/// Op::Equals,
/// Box::new(Term::Condition(
/// Box::new(Term::Atom("d")),
/// Op::Or,
/// Box::new(Term::Atom("e")),
/// ))))))))).sql();
/// assert_eq!(result, "a = b AND (c = d OR e)");
/// ```
///
///
///
#[derive(Clone)]
pub enum Term<'a> {
    /// An atom is a single identifier.
    Atom(&'a str),
    /// A condition is a combination of two terms and an operator.
    Condition(Box<Term<'a>>, Op<'a>, Box<Term<'a>>),
    /// A parenthesized term.
    Parens(Box<Term<'a>>),
    /// A null term.
    Null,
    /// A subquery that can be used in WHERE clauses
    Subquery(Box<Query<'a>>),
    Not(Box<Term<'a>>),
    Cast(Box<Term<'a>>, &'a str),
    PgCast(Box<Term<'a>>, &'a str),
    Case(CaseExpression<'a>),
    Coalesce(Vec<Term<'a>>),
    NullIf(Box<Term<'a>>, Box<Term<'a>>),
    Concat(Vec<Term<'a>>),
    Substring(Box<Term<'a>>, Option<Box<Term<'a>>>, Option<Box<Term<'a>>>),
    Upper(Box<Term<'a>>),
    Lower(Box<Term<'a>>),
    Now,
    CurrentDate,
    Interval(&'a str),
    DateAdd(Box<Term<'a>>, Box<Term<'a>>),
    DateSub(Box<Term<'a>>, Box<Term<'a>>),
}

impl<'a> Sql for CaseExpression<'a> {
    fn sql(&self) -> String {
        let mut s = "CASE".to_string();
        for wt in &self.when_thens {
            s.push_str(&format!(" WHEN {} THEN {}", wt.when.sql(), wt.then.sql()));
        }
        if let Some(et) = &self.else_term {
            s.push_str(&format!(" ELSE {}", et.sql()));
        }
        s.push_str(" END");
        s
    }
}

#[derive(Clone)]
pub struct WhenThen<'a> {
    pub when: Term<'a>,
    pub then: Term<'a>,
}

#[derive(Clone)]
pub struct CaseExpression<'a> {
    pub when_thens: Vec<WhenThen<'a>>,
    pub else_term: Option<Box<Term<'a>>>,
}

impl<'a> Sql for Term<'a> {
    fn sql(&self) -> String {
        match &self {
            Term::Atom(s) => s.to_string(),
            Term::Condition(t1, op, t2) => format!("{} {} {}", t1.sql(), op.sql(), t2.sql()),
            Term::Null => "".to_string(),
            Term::Parens(t) => format!("({})", t.sql()),
            Term::Subquery(q) => format!("({})", q.sql()),
            Term::Not(t) => format!("NOT {}", t.sql()),
            Term::Cast(t, ty) => format!("CAST({} AS {})", t.sql(), ty),
            Term::PgCast(t, ty) => format!("{}::{}", t.sql(), ty),
            Term::Case(c) => c.sql(),
            Term::Coalesce(terms) => {
                let terms_sql: Vec<String> = terms.iter().map(|t| t.sql()).collect();
                format!("COALESCE({})", terms_sql.join(", "))
            }
            Term::NullIf(t1, t2) => format!("NULLIF({}, {})", t1.sql(), t2.sql()),
            Term::Concat(terms) => {
                let terms_sql: Vec<String> = terms.iter().map(|t| t.sql()).collect();
                format!("CONCAT({})", terms_sql.join(", "))
            }
            Term::Substring(t, from, for_) => {
                let mut s = format!("SUBSTRING({}", t.sql());
                if let Some(f) = from {
                    s.push_str(&format!(" FROM {}", f.sql()));
                }
                if let Some(f) = for_ {
                    s.push_str(&format!(" FOR {}", f.sql()));
                }
                s.push(')');
                s
            }
            Term::Upper(t) => format!("UPPER({})", t.sql()),
            Term::Lower(t) => format!("LOWER({})", t.sql()),
            Term::Now => "NOW()".to_string(),
            Term::CurrentDate => "CURRENT_DATE".to_string(),
            Term::Interval(s) => format!("INTERVAL '{}'", s),
            Term::DateAdd(t1, t2) => format!("{} + {}", t1.sql(), t2.sql()),
            Term::DateSub(t1, t2) => format!("{} - {}", t1.sql(), t2.sql()),
        }
    }
}

// Helper functions for building WHERE clauses ergonomically

/// Creates an equality condition (=)
pub fn eq<'a>(left: &'a str, right: &'a str) -> Term<'a> {
    Term::Condition(
        Box::new(Term::Atom(left)),
        Op::Equals,
        Box::new(Term::Atom(right)),
    )
}

/// Creates a not-equals condition (!=)
pub fn ne<'a>(left: &'a str, right: &'a str) -> Term<'a> {
    Term::Condition(
        Box::new(Term::Atom(left)),
        Op::NotEquals,
        Box::new(Term::Atom(right)),
    )
}

/// Creates a greater-than condition (>)
pub fn gt<'a>(left: &'a str, right: &'a str) -> Term<'a> {
    Term::Condition(
        Box::new(Term::Atom(left)),
        Op::GreaterThan,
        Box::new(Term::Atom(right)),
    )
}

/// Creates a less-than condition (<)
pub fn lt<'a>(left: &'a str, right: &'a str) -> Term<'a> {
    Term::Condition(
        Box::new(Term::Atom(left)),
        Op::LessThan,
        Box::new(Term::Atom(right)),
    )
}

/// Creates a greater-than-or-equal condition (>=)
pub fn gte<'a>(left: &'a str, right: &'a str) -> Term<'a> {
    Term::Condition(
        Box::new(Term::Atom(left)),
        Op::GreaterOrEqual,
        Box::new(Term::Atom(right)),
    )
}

/// Creates a less-than-or-equal condition (<=)
pub fn lte<'a>(left: &'a str, right: &'a str) -> Term<'a> {
    Term::Condition(
        Box::new(Term::Atom(left)),
        Op::LessOrEqual,
        Box::new(Term::Atom(right)),
    )
}

/// Creates a LIKE condition
pub fn like<'a>(left: &'a str, right: &'a str) -> Term<'a> {
    Term::Condition(
        Box::new(Term::Atom(left)),
        Op::Like,
        Box::new(Term::Atom(right)),
    )
}

/// Combines two terms with AND
pub fn and<'a>(left: Term<'a>, right: Term<'a>) -> Term<'a> {
    Term::Condition(Box::new(left), Op::And, Box::new(right))
}

/// Combines two terms with OR
pub fn or<'a>(left: Term<'a>, right: Term<'a>) -> Term<'a> {
    Term::Condition(Box::new(left), Op::Or, Box::new(right))
}

/// Negates a term with NOT
pub fn not<'a>(term: Term<'a>) -> Term<'a> {
    Term::Not(Box::new(term))
}

/// Creates a CAST expression
pub fn cast<'a>(term: Term<'a>, type_name: &'a str) -> Term<'a> {
    Term::Cast(Box::new(term), type_name)
}

/// Creates a PostgreSQL-style CAST expression (::)
pub fn pg_cast<'a>(term: Term<'a>, type_name: &'a str) -> Term<'a> {
    Term::PgCast(Box::new(term), type_name)
}

/// Creates a CASE expression
pub fn case<'a>(when_thens: Vec<WhenThen<'a>>, else_term: Option<Term<'a>>) -> Term<'a> {
    Term::Case(CaseExpression {
        when_thens,
        else_term: else_term.map(Box::new),
    })
}

/// Creates a COALESCE expression
pub fn coalesce<'a>(terms: Vec<Term<'a>>) -> Term<'a> {
    Term::Coalesce(terms)
}

/// Creates a NULLIF expression
pub fn nullif<'a>(left: Term<'a>, right: Term<'a>) -> Term<'a> {
    Term::NullIf(Box::new(left), Box::new(right))
}

/// Creates a CONCAT expression
pub fn concat<'a>(terms: Vec<Term<'a>>) -> Term<'a> {
    Term::Concat(terms)
}

/// Creates a SUBSTRING expression
pub fn substring<'a>(term: Term<'a>, from: Option<Term<'a>>, for_: Option<Term<'a>>) -> Term<'a> {
    Term::Substring(Box::new(term), from.map(Box::new), for_.map(Box::new))
}

/// Creates a UPPER expression
pub fn upper<'a>(term: Term<'a>) -> Term<'a> {
    Term::Upper(Box::new(term))
}

/// Creates a LOWER expression
pub fn lower<'a>(term: Term<'a>) -> Term<'a> {
    Term::Lower(Box::new(term))
}

/// Creates a NOW() expression
pub fn now<'a>() -> Term<'a> {
    Term::Now
}

/// Creates a CURRENT_DATE expression
pub fn current_date<'a>() -> Term<'a> {
    Term::CurrentDate
}

/// Creates an INTERVAL expression
pub fn interval<'a>(s: &'a str) -> Term<'a> {
    Term::Interval(s)
}

/// Creates a date addition expression
pub fn date_add<'a>(left: Term<'a>, right: Term<'a>) -> Term<'a> {
    Term::DateAdd(Box::new(left), Box::new(right))
}

/// Creates a date subtraction expression
pub fn date_sub<'a>(left: Term<'a>, right: Term<'a>) -> Term<'a> {
    Term::DateSub(Box::new(left), Box::new(right))
}

/// Wraps a term in parentheses
pub fn parens<'a>(term: Term<'a>) -> Term<'a> {
    Term::Parens(Box::new(term))
}

// Convenience functions for common SQL patterns

/// Creates an IN clause
/// Example: in_("status", vec!["'active'", "'pending'"]) => "status IN ('active', 'pending')"
pub fn in_<'a>(column: &'a str, values: Vec<&'a str>) -> Term<'a> {
    let values_str = values.join(", ");
    Term::Atom(Box::leak(
        format!("{} IN ({})", column, values_str).into_boxed_str(),
    ))
}

/// Creates a BETWEEN clause
/// Example: between("age", "18", "65") => "age BETWEEN 18 AND 65"
pub fn between<'a>(column: &'a str, low: &'a str, high: &'a str) -> Term<'a> {
    Term::Atom(Box::leak(
        format!("{} BETWEEN {} AND {}", column, low, high).into_boxed_str(),
    ))
}

/// Creates an IS NULL condition
/// Example: is_null("deleted_at") => "deleted_at IS NULL"
pub fn is_null<'a>(column: &'a str) -> Term<'a> {
    Term::Atom(Box::leak(format!("{} IS NULL", column).into_boxed_str()))
}

/// Creates an IS NOT NULL condition
/// Example: is_not_null("created_at") => "created_at IS NOT NULL"
pub fn is_not_null<'a>(column: &'a str) -> Term<'a> {
    Term::Atom(Box::leak(
        format!("{} IS NOT NULL", column).into_boxed_str(),
    ))
}

// Nested query helpers

/// Creates an EXISTS condition with a subquery
/// Example: exists(subquery) => "EXISTS (SELECT ...)"
pub fn exists<'a>(subquery: Query<'a>) -> Term<'a> {
    let sql = format!("EXISTS ({})", subquery.sql());
    Term::Atom(Box::leak(sql.into_boxed_str()))
}

/// Creates a NOT EXISTS condition with a subquery
/// Example: not_exists(subquery) => "NOT EXISTS (SELECT ...)"
pub fn not_exists<'a>(subquery: Query<'a>) -> Term<'a> {
    let sql = format!("NOT EXISTS ({})", subquery.sql());
    Term::Atom(Box::leak(sql.into_boxed_str()))
}

/// Creates an IN condition with a subquery
/// Example: in_subquery("user_id", subquery) => "user_id IN (SELECT ...)"
pub fn in_subquery<'a>(column: &'a str, subquery: Query<'a>) -> Term<'a> {
    Term::Condition(
        Box::new(Term::Atom(column)),
        Op::In,
        Box::new(Term::Subquery(Box::new(subquery))),
    )
}

/// Creates a comparison with ANY (subquery)
/// Example: any("price", Op::GreaterThan, subquery) => "price > ANY (SELECT ...)"
pub fn any<'a>(column: &'a str, op: Op<'a>, subquery: Query<'a>) -> Term<'a> {
    let sql = format!("{} {} ANY ({})", column, op.sql(), subquery.sql());
    Term::Atom(Box::leak(sql.into_boxed_str()))
}

/// Creates a comparison with ALL (subquery)
/// Example: all("price", Op::LessThan, subquery) => "price < ALL (SELECT ...)"
pub fn all<'a>(column: &'a str, op: Op<'a>, subquery: Query<'a>) -> Term<'a> {
    let sql = format!("{} {} ALL ({})", column, op.sql(), subquery.sql());
    Term::Atom(Box::leak(sql.into_boxed_str()))
}

// PostgreSQL parameter helpers

/// Returns a PostgreSQL parameter placeholder
/// Example: p(1) => "$1", p(2) => "$2"
pub fn p(n: usize) -> String {
    format!("${}", n)
}

/// PostgreSQL parameter counter for auto-sequencing
/// Automatically generates $1, $2, $3, etc. as you call seq()
///
/// # Example
/// ```
/// use squeal::*;
/// let mut pg = PgParams::new();
/// let p1 = pg.seq();  // "$1"
/// let p2 = pg.seq();  // "$2"
/// let mut qb = Q();
/// let query = qb
///     .select(vec!["*"])
///     .from("users")
///     .where_(and(
///         eq("id", &p1),
///         eq("status", &p2)
///     ))
///     .build();
/// assert_eq!(query.sql(), "SELECT * FROM users WHERE id = $1 AND status = $2");
/// ```
pub struct PgParams {
    count: usize,
}

impl PgParams {
    /// Creates a new parameter counter starting at 0
    pub fn new() -> Self {
        PgParams { count: 0 }
    }

    /// Returns the next parameter placeholder ($1, $2, $3, etc.)
    pub fn seq(&mut self) -> String {
        self.count += 1;
        format!("${}", self.count)
    }
}

impl Default for PgParams {
    fn default() -> Self {
        Self::new()
    }
}

/// The Having struct is used to specify the having clause in a query.
/// It is used in the Query struct.
///
/// It is constructed with a Term, similar to a Where clause.
#[derive(Clone)]
pub struct Having<'a> {
    /// The condition for the HAVING clause
    pub term: Term<'a>,
}

impl<'a> Having<'a> {
    /// Creates a new Having clause with the given term
    ///
    /// # Example
    /// ```
    /// use squeal::*;
    /// let having = Having::new(gt("count", "5"));
    /// assert_eq!(having.sql(), "count > 5");
    /// ```
    pub fn new(t: Term<'a>) -> Having<'a> {
        Having { term: t }
    }
}

impl<'a> Sql for Having<'a> {
    fn sql(&self) -> String {
        self.term.sql().to_string()
    }
}

/// The OrderedColumn enum is used to specify the order by clause in a query.
/// It is used in the OrderBy struct.
/// It is used to specify the columns, and optionally, whether they are ascending or descending.
#[derive(Clone)]
pub enum OrderedColumn<'a> {
    /// Ascending order
    Asc(&'a str),
    /// Descending order
    Desc(&'a str),
}

/// The OrderBy struct is used to specify the order by clause in a query.
/// It is used in the Query struct.
/// It is used to specify the columns, and optionally, whether they are ascending or descending.
/// Each column can be ascending or descending
#[derive(Clone)]
pub struct OrderBy<'a> {
    /// List of columns with their sort order
    pub columns: Vec<OrderedColumn<'a>>,
}

impl<'a> Sql for OrderBy<'a> {
    fn sql(&self) -> String {
        let mut result = "ORDER BY ".to_string();
        let mut first = true;
        for c in &self.columns {
            if !first {
                result.push_str(", ");
            }
            first = false;
            match c {
                OrderedColumn::Asc(s) => result.push_str(&format!("{} ASC", s)),
                OrderedColumn::Desc(s) => result.push_str(&format!("{} DESC", s)),
            }
        }
        result
    }
}

/// The FromSource enum represents the source of data in a FROM clause.
/// It can be either a simple table name or a subquery with an alias.
///
/// # Examples
///
/// Simple table:
/// ```
/// use squeal::*;
/// let from = FromSource::Table("users");
/// assert_eq!(from.sql(), "users");
/// ```
///
/// Subquery with alias:
/// ```
/// use squeal::*;
/// let subquery = Query {
///     with_clause: None,
///     select: Some(Select::new(Columns::Star, None)),
///     from: Some(FromSource::Table("users")),
///     joins: vec![],
///     where_clause: None,
///     group_by: None,
///     having: None,
///     order_by: None,
///     limit: None,
///     offset: None,
///     for_update: false,
/// };
/// let from = FromSource::Subquery(Box::new(subquery), "u");
/// assert_eq!(from.sql(), "(SELECT * FROM users) AS u");
/// ```
#[derive(Clone)]
pub enum FromSource<'a> {
    /// A simple table name
    Table(&'a str),
    /// A subquery with an alias
    Subquery(Box<Query<'a>>, &'a str),
}

impl<'a> Sql for FromSource<'a> {
    fn sql(&self) -> String {
        match self {
            FromSource::Table(table) => table.to_string(),
            FromSource::Subquery(query, alias) => format!("({}) AS {}", query.sql(), alias),
        }
    }
}

/// Join type for SQL JOIN clauses
#[derive(Clone)]
pub enum JoinType {
    /// INNER JOIN
    Inner,
    /// LEFT JOIN (LEFT OUTER JOIN)
    Left,
    /// RIGHT JOIN (RIGHT OUTER JOIN)
    Right,
    /// FULL JOIN (FULL OUTER JOIN)
    Full,
    /// CROSS JOIN
    Cross,
}

impl Sql for JoinType {
    fn sql(&self) -> String {
        match self {
            JoinType::Inner => "INNER JOIN",
            JoinType::Left => "LEFT JOIN",
            JoinType::Right => "RIGHT JOIN",
            JoinType::Full => "FULL JOIN",
            JoinType::Cross => "CROSS JOIN",
        }
        .to_string()
    }
}

/// Represents a JOIN clause in a SQL query
#[derive(Clone)]
pub struct Join<'a> {
    /// The type of join (INNER, LEFT, RIGHT, FULL, CROSS)
    pub join_type: JoinType,
    /// The table or subquery to join
    pub source: FromSource<'a>,
    /// The join condition (ON clause), None for CROSS JOIN
    pub on: Option<Term<'a>>,
}

impl<'a> Sql for Join<'a> {
    fn sql(&self) -> String {
        let mut result = format!("{} {}", self.join_type.sql(), self.source.sql());
        if let Some(condition) = &self.on {
            result.push_str(&format!(" ON {}", condition.sql()));
        }
        result
    }
}

/// Represents a Common Table Expression (CTE) in a WITH clause
#[derive(Clone)]
pub struct Cte<'a> {
    /// The name of the CTE
    pub name: &'a str,
    /// The query that defines the CTE
    pub query: Box<Query<'a>>,
}

impl<'a> Sql for Cte<'a> {
    fn sql(&self) -> String {
        format!("{} AS ({})", self.name, self.query.sql())
    }
}

/// The Query struct is the top-level object that represents a query.
/// The user is expected to construct the Query object and then call the sql() method to get the
/// SQL string.
///
#[derive(Clone)]
pub struct Query<'a> {
    /// WITH clause (Common Table Expressions)
    pub with_clause: Option<Vec<Cte<'a>>>,
    /// The select clause.
    pub select: Option<Select<'a>>,
    /// The table name for the select clause.
    pub from: Option<FromSource<'a>>,
    /// JOIN clauses
    pub joins: Vec<Join<'a>>,
    /// The conditions for the where clause, if it exists.
    pub where_clause: Option<Term<'a>>,
    /// The columns to group by, if any.
    pub group_by: Option<Vec<&'a str>>,
    /// The having clause conditions, if any.
    pub having: Option<Having<'a>>,
    /// The order by clause, if any.
    pub order_by: Option<OrderBy<'a>>,
    /// The maximum number of rows to return.
    pub limit: Option<u64>,
    /// The number of rows to skip.
    pub offset: Option<u64>,
    /// Whether to lock rows with FOR UPDATE.
    pub for_update: bool,
}

/// The QueryBuilder struct is a fluent interface for building a Query.
/// It is not intended to be used directly, but rather through the Q() function.
/// See the integration_test.rs for an example of usage.
pub struct QueryBuilder<'a> {
    /// WITH clause (Common Table Expressions)
    pub with_clause: Option<Vec<Cte<'a>>>,
    /// The select clause
    pub select: Option<Select<'a>>,
    /// The table to select from
    pub from: Option<FromSource<'a>>,
    /// JOIN clauses
    pub joins: Vec<Join<'a>>,
    /// The WHERE clause conditions
    pub where_clause: Option<Term<'a>>,
    /// The columns to GROUP BY
    pub group_by: Option<Vec<&'a str>>,
    /// The HAVING clause conditions
    pub having: Option<Having<'a>>,
    /// The ORDER BY clause
    pub order_by: Option<OrderBy<'a>>,
    /// The LIMIT value
    pub limit: Option<u64>,
    /// The OFFSET value
    pub offset: Option<u64>,
    /// Whether to use FOR UPDATE
    pub for_update: bool,
    /// PostgreSQL parameter counter
    pub params: PgParams,
}

/// The Q function is a fluent interface for building a Query.
/// The user is expected to construct the Query object and then call the sql() method to get the SQL string.
/// The goal is any valid construction of a QueryBuilder is a valid Query and will, at least, syntactically, be valid SQL.
#[allow(non_snake_case)]
pub fn Q<'a>() -> QueryBuilder<'a> {
    QueryBuilder {
        with_clause: None,
        select: None,
        from: None,
        joins: Vec::new(),
        where_clause: None,
        group_by: None,
        having: None,
        order_by: None,
        limit: None,
        offset: None,
        for_update: false,
        params: PgParams::new(),
    }
}

impl<'a> QueryBuilder<'a> {
    /// Builds the final Query from this builder
    ///
    /// # Example
    /// ```
    /// use squeal::*;
    /// let mut qb = Q();
    /// let query = qb.select(vec!["*"]).from("users").build();
    /// assert_eq!(query.sql(), "SELECT * FROM users");
    /// ```
    pub fn build(&self) -> Query<'a> {
        Query {
            with_clause: self.with_clause.clone(),
            select: self.select.clone(),
            from: self.from.clone(),
            joins: self.joins.clone(),
            where_clause: self.where_clause.clone(),
            group_by: self.group_by.clone(),
            having: self.having.clone(),
            order_by: self.order_by.clone(),
            limit: self.limit,
            offset: self.offset,
            for_update: self.for_update,
        }
    }

    /// Adds a WITH clause (Common Table Expression)
    ///
    /// # Example
    /// ```
    /// use squeal::*;
    /// let cte_query = Query {
    ///     with_clause: None,
    ///     select: Some(Select::new(Columns::Selected(vec!["id", "name"]), None)),
    ///     from: Some(FromSource::Table("users")),
    ///     joins: vec![],
    ///     where_clause: Some(eq("active", "true")),
    ///     group_by: None,
    ///     having: None,
    ///     order_by: None,
    ///     limit: None,
    ///     offset: None,
    ///     for_update: false,
    /// };
    /// let mut qb = Q();
    /// let query = qb.with("active_users", cte_query)
    ///     .select(vec!["*"])
    ///     .from("active_users")
    ///     .build();
    /// assert_eq!(query.sql(), "WITH active_users AS (SELECT id, name FROM users WHERE active = true) SELECT * FROM active_users");
    /// ```
    pub fn with(&'a mut self, name: &'a str, query: Query<'a>) -> &'a mut QueryBuilder<'a> {
        let cte = Cte {
            name,
            query: Box::new(query),
        };
        match &mut self.with_clause {
            None => self.with_clause = Some(vec![cte]),
            Some(ctes) => ctes.push(cte),
        }
        self
    }

    /// Sets the columns to SELECT
    ///
    /// # Example
    /// ```
    /// use squeal::*;
    /// let mut qb = Q();
    /// let query = qb.select(vec!["id", "name"]).from("users").build();
    /// assert_eq!(query.sql(), "SELECT id, name FROM users");
    /// ```
    pub fn select(&'a mut self, cols: Vec<&'a str>) -> &'a mut QueryBuilder<'a> {
        self.select = Some(Select::new(Columns::Selected(cols), None));
        self
    }

    /// Sets the SELECT clause with expressions (columns and/or subqueries)
    ///
    /// # Example
    /// ```
    /// use squeal::*;
    /// let subquery = Query {
    ///     with_clause: None,
    ///     select: Some(Select::new(Columns::Selected(vec!["COUNT(*)"]), None)),
    ///     from: Some(FromSource::Table("orders")),
    ///     joins: vec![],
    ///     where_clause: None,
    ///     group_by: None,
    ///     having: None,
    ///     order_by: None,
    ///     limit: None,
    ///     offset: None,
    ///     for_update: false,
    /// };
    /// let mut qb = Q();
    /// let query = qb.select_expressions(vec![
    ///     SelectExpression::Column("id"),
    ///     SelectExpression::Subquery(Box::new(subquery), Some("order_count"))
    /// ]).from("users").build();
    /// assert_eq!(query.sql(), "SELECT id, (SELECT COUNT(*) FROM orders) AS order_count FROM users");
    /// ```
    pub fn select_expressions(
        &'a mut self,
        exprs: Vec<SelectExpression<'a>>,
    ) -> &'a mut QueryBuilder<'a> {
        self.select = Some(Select::new(Columns::Expressions(exprs), None));
        self
    }
    /// Sets the SELECT clause to be DISTINCT
    pub fn distinct(&'a mut self) -> &'a mut QueryBuilder<'a> {
        if let Some(s) = &mut self.select {
            s.distinct = Some(Distinct::All);
        }
        self
    }

    /// Sets the SELECT clause to be DISTINCT ON the given columns
    pub fn distinct_on(&'a mut self, cols: Vec<&'a str>) -> &'a mut QueryBuilder<'a> {
        if let Some(s) = &mut self.select {
            s.distinct = Some(Distinct::On(cols));
        }
        self
    }
    /// Sets the table to SELECT FROM
    ///
    /// # Example
    /// ```
    /// use squeal::*;
    /// let mut qb = Q();
    /// let query = qb.select(vec!["*"]).from("products").build();
    /// assert_eq!(query.sql(), "SELECT * FROM products");
    /// ```
    pub fn from(&'a mut self, table: &'a str) -> &'a mut QueryBuilder<'a> {
        self.from = Some(FromSource::Table(table));
        self
    }

    /// Sets a subquery as the FROM source
    ///
    /// # Example
    /// ```
    /// use squeal::*;
    /// let subquery = Query {
    ///     with_clause: None,
    ///     select: Some(Select::new(Columns::Star, None)),
    ///     from: Some(FromSource::Table("users")),
    ///     joins: vec![],
    ///     where_clause: None,
    ///     group_by: None,
    ///     having: None,
    ///     order_by: None,
    ///     limit: None,
    ///     offset: None,
    ///     for_update: false,
    /// };
    /// let mut qb = Q();
    /// let query = qb.select(vec!["*"]).from_subquery(subquery, "u").build();
    /// assert_eq!(query.sql(), "SELECT * FROM (SELECT * FROM users) AS u");
    /// ```
    pub fn from_subquery(
        &'a mut self,
        subquery: Query<'a>,
        alias: &'a str,
    ) -> &'a mut QueryBuilder<'a> {
        self.from = Some(FromSource::Subquery(Box::new(subquery), alias));
        self
    }

    /// Adds an INNER JOIN clause
    ///
    /// # Example
    /// ```
    /// use squeal::*;
    /// let mut qb = Q();
    /// let query = qb.select(vec!["users.name", "orders.total"])
    ///     .from("users")
    ///     .inner_join("orders", eq("users.id", "orders.user_id"))
    ///     .build();
    /// assert_eq!(query.sql(), "SELECT users.name, orders.total FROM users INNER JOIN orders ON users.id = orders.user_id");
    /// ```
    pub fn inner_join(&'a mut self, table: &'a str, on: Term<'a>) -> &'a mut QueryBuilder<'a> {
        self.joins.push(Join {
            join_type: JoinType::Inner,
            source: FromSource::Table(table),
            on: Some(on),
        });
        self
    }

    /// Adds a LEFT JOIN clause
    ///
    /// # Example
    /// ```
    /// use squeal::*;
    /// let mut qb = Q();
    /// let query = qb.select(vec!["users.name", "orders.total"])
    ///     .from("users")
    ///     .left_join("orders", eq("users.id", "orders.user_id"))
    ///     .build();
    /// assert_eq!(query.sql(), "SELECT users.name, orders.total FROM users LEFT JOIN orders ON users.id = orders.user_id");
    /// ```
    pub fn left_join(&'a mut self, table: &'a str, on: Term<'a>) -> &'a mut QueryBuilder<'a> {
        self.joins.push(Join {
            join_type: JoinType::Left,
            source: FromSource::Table(table),
            on: Some(on),
        });
        self
    }

    /// Adds a RIGHT JOIN clause
    ///
    /// # Example
    /// ```
    /// use squeal::*;
    /// let mut qb = Q();
    /// let query = qb.select(vec!["users.name", "orders.total"])
    ///     .from("users")
    ///     .right_join("orders", eq("users.id", "orders.user_id"))
    ///     .build();
    /// assert_eq!(query.sql(), "SELECT users.name, orders.total FROM users RIGHT JOIN orders ON users.id = orders.user_id");
    /// ```
    pub fn right_join(&'a mut self, table: &'a str, on: Term<'a>) -> &'a mut QueryBuilder<'a> {
        self.joins.push(Join {
            join_type: JoinType::Right,
            source: FromSource::Table(table),
            on: Some(on),
        });
        self
    }

    /// Adds a FULL JOIN clause
    ///
    /// # Example
    /// ```
    /// use squeal::*;
    /// let mut qb = Q();
    /// let query = qb.select(vec!["users.name", "orders.total"])
    ///     .from("users")
    ///     .full_join("orders", eq("users.id", "orders.user_id"))
    ///     .build();
    /// assert_eq!(query.sql(), "SELECT users.name, orders.total FROM users FULL JOIN orders ON users.id = orders.user_id");
    /// ```
    pub fn full_join(&'a mut self, table: &'a str, on: Term<'a>) -> &'a mut QueryBuilder<'a> {
        self.joins.push(Join {
            join_type: JoinType::Full,
            source: FromSource::Table(table),
            on: Some(on),
        });
        self
    }

    /// Adds a CROSS JOIN clause (no ON condition required)
    ///
    /// # Example
    /// ```
    /// use squeal::*;
    /// let mut qb = Q();
    /// let query = qb.select(vec!["users.name", "colors.name"])
    ///     .from("users")
    ///     .cross_join("colors")
    ///     .build();
    /// assert_eq!(query.sql(), "SELECT users.name, colors.name FROM users CROSS JOIN colors");
    /// ```
    pub fn cross_join(&'a mut self, table: &'a str) -> &'a mut QueryBuilder<'a> {
        self.joins.push(Join {
            join_type: JoinType::Cross,
            source: FromSource::Table(table),
            on: None,
        });
        self
    }

    /// Adds a JOIN clause with a subquery as the source
    ///
    /// # Example
    /// ```
    /// use squeal::*;
    /// let subquery = Query {
    ///     with_clause: None,
    ///     select: Some(Select::new(Columns::Selected(vec!["user_id", "COUNT(*) as order_count"]), None)),
    ///     from: Some(FromSource::Table("orders")),
    ///     joins: vec![],
    ///     where_clause: None,
    ///     group_by: Some(vec!["user_id"]),
    ///     having: None,
    ///     order_by: None,
    ///     limit: None,
    ///     offset: None,
    ///     for_update: false,
    /// };
    /// let mut qb = Q();
    /// let query = qb.select(vec!["users.name", "oc.order_count"])
    ///     .from("users")
    ///     .join_subquery(JoinType::Left, subquery, "oc", eq("users.id", "oc.user_id"))
    ///     .build();
    /// assert_eq!(query.sql(), "SELECT users.name, oc.order_count FROM users LEFT JOIN (SELECT user_id, COUNT(*) as order_count FROM orders GROUP BY user_id) AS oc ON users.id = oc.user_id");
    /// ```
    pub fn join_subquery(
        &'a mut self,
        join_type: JoinType,
        subquery: Query<'a>,
        alias: &'a str,
        on: Term<'a>,
    ) -> &'a mut QueryBuilder<'a> {
        self.joins.push(Join {
            join_type,
            source: FromSource::Subquery(Box::new(subquery), alias),
            on: Some(on),
        });
        self
    }

    /// Sets the WHERE clause
    ///
    /// # Example
    /// ```
    /// use squeal::*;
    /// let mut qb = Q();
    /// let query = qb.select(vec!["*"]).from("users").where_(eq("id", "1")).build();
    /// assert_eq!(query.sql(), "SELECT * FROM users WHERE id = 1");
    /// ```
    pub fn where_(&'a mut self, term: Term<'a>) -> &'a mut QueryBuilder<'a> {
        self.where_clause = Some(term);
        self
    }

    /// Sets WHERE clause only if the Option contains Some value
    /// Useful for conditional/dynamic query building
    pub fn where_opt(&'a mut self, term: Option<Term<'a>>) -> &'a mut QueryBuilder<'a> {
        if let Some(t) = term {
            self.where_clause = Some(t);
        }
        self
    }

    /// Adds a condition to the WHERE clause with AND
    /// If no WHERE clause exists yet, this becomes the first condition
    /// Otherwise, it ANDs the new condition with the existing one
    pub fn and_where(&'a mut self, term: Term<'a>) -> &'a mut QueryBuilder<'a> {
        match &self.where_clause {
            None => self.where_clause = Some(term),
            Some(existing) => {
                self.where_clause = Some(Term::Condition(
                    Box::new(existing.clone()),
                    Op::And,
                    Box::new(term),
                ));
            }
        }
        self
    }

    /// Sets the GROUP BY clause
    ///
    /// # Example
    /// ```
    /// use squeal::*;
    /// let mut qb = Q();
    /// let query = qb.select(vec!["category", "count(*)"]).from("products").group_by(vec!["category"]).build();
    /// assert_eq!(query.sql(), "SELECT category, count(*) FROM products GROUP BY category");
    /// ```
    pub fn group_by(&'a mut self, cols: Vec<&'a str>) -> &'a mut QueryBuilder<'a> {
        self.group_by = Some(cols);
        self
    }
    /// Sets the HAVING clause
    ///
    /// # Example
    /// ```
    /// use squeal::*;
    /// let mut qb = Q();
    /// let query = qb.select(vec!["category", "count(*)"]).from("products").group_by(vec!["category"]).having(gt("count(*)", "5")).build();
    /// assert_eq!(query.sql(), "SELECT category, count(*) FROM products GROUP BY category HAVING count(*) > 5");
    /// ```
    pub fn having(&'a mut self, term: Term<'a>) -> &'a mut QueryBuilder<'a> {
        self.having = Some(Having::new(term));
        self
    }
    /// Sets the ORDER BY clause
    ///
    /// # Example
    /// ```
    /// use squeal::*;
    /// let mut qb = Q();
    /// let query = qb.select(vec!["*"]).from("users").order_by(vec![OrderedColumn::Desc("created_at")]).build();
    /// assert_eq!(query.sql(), "SELECT * FROM users ORDER BY created_at DESC");
    /// ```
    pub fn order_by(&'a mut self, cols: Vec<OrderedColumn<'a>>) -> &'a mut QueryBuilder<'a> {
        self.order_by = Some(OrderBy { columns: cols });
        self
    }
    /// Sets the LIMIT clause
    ///
    /// # Example
    /// ```
    /// use squeal::*;
    /// let mut qb = Q();
    /// let query = qb.select(vec!["*"]).from("users").limit(10).build();
    /// assert_eq!(query.sql(), "SELECT * FROM users LIMIT 10");
    /// ```
    pub fn limit(&'a mut self, limit: u64) -> &'a mut QueryBuilder<'a> {
        self.limit = Some(limit);
        self
    }
    /// Sets the OFFSET clause
    ///
    /// # Example
    /// ```
    /// use squeal::*;
    /// let mut qb = Q();
    /// let query = qb.select(vec!["*"]).from("users").offset(20).build();
    /// assert_eq!(query.sql(), "SELECT * FROM users OFFSET 20");
    /// ```
    pub fn offset(&'a mut self, offset: u64) -> &'a mut QueryBuilder<'a> {
        self.offset = Some(offset);
        self
    }
    /// Adds FOR UPDATE to lock selected rows
    ///
    /// # Example
    /// ```
    /// use squeal::*;
    /// let mut qb = Q();
    /// let query = qb.select(vec!["*"]).from("users").for_update().build();
    /// assert_eq!(query.sql(), "SELECT * FROM users FOR UPDATE");
    /// ```
    pub fn for_update(&'a mut self) -> &'a mut QueryBuilder<'a> {
        self.for_update = true;
        self
    }
}

impl<'a> Parameterized for QueryBuilder<'a> {
    fn param(&mut self) -> String {
        self.params.seq()
    }
}

impl<'a> Sql for Query<'a> {
    fn sql(&self) -> String {
        let mut result = String::new();

        if let Some(ctes) = &self.with_clause {
            result.push_str("WITH ");
            let mut first = true;
            for cte in ctes {
                if !first {
                    result.push_str(", ");
                }
                first = false;
                result.push_str(&cte.sql());
            }
            result.push(' ');
        }

        if let Some(select) = &self.select {
            result.push_str(&format!("SELECT {}", select.sql()));
        }
        if let Some(from) = &self.from {
            result.push_str(&format!(" FROM {}", from.sql()));
        }
        for join in &self.joins {
            result.push_str(&format!(" {}", join.sql()));
        }
        if let Some(conditions) = &self.where_clause {
            result.push_str(&format!(" WHERE {}", conditions.sql()));
        }
        if let Some(group_by) = &self.group_by {
            result.push_str(&format!(" GROUP BY {}", group_by.join(", ")));
        }
        if let Some(having) = &self.having {
            result.push_str(&format!(" HAVING {}", having.sql()));
        }
        if let Some(order_by) = &self.order_by {
            result.push_str(&format!(" {}", order_by.sql()));
        }
        if let Some(limit) = &self.limit {
            result.push_str(&format!(" LIMIT {}", limit));
        }
        if let Some(offset) = &self.offset {
            result.push_str(&format!(" OFFSET {}", offset));
        }
        if self.for_update {
            result.push_str(" FOR UPDATE");
        }
        result
    }
}
