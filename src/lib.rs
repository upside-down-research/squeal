//! Simple Query Builder for Rust
//!
//! Provides a straightforward way to build SQL queries in Rust.
//! Conceptually, you build a list of properly termed objects in a "Query" object, and then
//! call the sql() method on the Query object to get the SQL string.
//!
//! Escape hatches are built in to allow you to use any SQL you want and have it integrated properly.
//!
//! Part of the design goal is not to use attributes, macros, or other "magic" to make this work.
//!
//! "Keep it simple & stupid."
//!
//! # Examples
//!
//! ```
//! use squeal::*;
//!
//! let result = Query {
//!      select: Select::new(Columns::Star),
//!      from: "table".to_string(),
//!      where_clause: Some(Term::Condition(
//!        Box::new(Term::Atom("a".to_string())),
//!      Op::O("<>".to_string()),
//!      Box::new(Term::Atom("b".to_string())))),
//!      group_by: None,
//!     having: None,
//!     order_by: None,
//!     limit: None,
//!     offset: None,
//! }.sql();
//!
//! assert_eq!(result, "SELECT * FROM table WHERE a <> b");
//! ```
//! Note the verbosity of the to_string and Enum scoping. This is not intentional and an artifact of
//! this still being in early development.

/// The Sql trait is implemented by all objects that can be used in a query.
/// It provides a single method, sql(), that returns a String.
///
/// This is not intended to be implemented by the user.
pub trait Sql {
    fn sql(&self) -> String;
}

/// The Columns enum is used to specify which columns to select.
///
/// It is used in the Select struct.
///
/// # Examples
///
/// Wildcard:
/// ```
/// use squeal::*;
/// let result = Select::new(Columns::Star).sql();
/// assert_eq!(result, "SELECT *");
/// ```
///
/// Specific columns:
/// ```
/// use squeal::*;
/// let result = Select::new(Columns::Selected(vec!["a".to_string(), "b".to_string()])).sql();
/// assert_eq!(result, "SELECT a, b");
/// ```
pub enum Columns {
    Star,
    Selected(Vec<String>),
}

/// The Select struct is used to specify which columns to select.
/// It is used in the Query struct.
///
/// It is constructed with the Columns enum.
///
/// For examples, see the Columns enum.
///
/// It does not currently support DISTINCT, functions, or other SELECT features besides simple
/// projection.
pub struct Select {
    pub cols: Columns,
}

impl Select {
    pub fn new(c: Columns) -> Select {
        Select { cols: c }
    }
}

impl Sql for Select {
    fn sql(&self) -> String {
        match &self.cols {
            Columns::Star => "SELECT *".to_string(),
            Columns::Selected(v) => format!("SELECT {}", v.join(", ")),
        }
    }
}

/// The Op enum is used to specify the operator in a condition.
/// It is used in the Term struct.
///
/// The Op::O variant is an escape hatch to allow you to use any operator you want.
pub enum Op {
    And,
    Or,
    Equals,
    O(String),
}

impl Sql for Op {
    fn sql(&self) -> String {
        match &self {
            Op::And => "AND",
            Op::Or => "OR",
            Op::Equals => "=",
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
/// let result = Term::Atom("a".to_string()).sql();
/// assert_eq!(result, "a");
/// ```
///
/// A number of different conditions and complex combinations:
/// ```
/// use squeal::*;
/// let result = Term::Condition(
///    Box::new(Term::Atom("a".to_string())),
///   Op::O("<>".to_string()),
/// Box::new(Term::Atom("b".to_string())),
/// ).sql();
/// assert_eq!(result, "a <> b");
/// ```
/// An example setting up `a = b AND (c = d OR e <> f)`:
///
/// ```
/// use squeal::*;
/// let result = Term::Condition(
///   Box::new(Term::Atom("a".to_string())),
/// Op::Equals,
/// Box::new(Term::Condition(
///   Box::new(Term::Atom("b".to_string())),
/// Op::And,
/// Box::new(Term::Parens(Box::new(Term::Condition(
///  Box::new(Term::Atom("c".to_string())),
/// Op::Equals,
/// Box::new(Term::Condition(
/// Box::new(Term::Atom("d".to_string())),
/// Op::Or,
/// Box::new(Term::Atom("e".to_string())),
/// ))))))))).sql();
/// assert_eq!(result, "a = b AND (c = d OR e)");
/// ```
///
///
///
pub enum Term {
    /// An atom is a single identifier.
    Atom(String),
    /// A condition is a combination of two terms and an operator.
    Condition(Box<Term>, Op, Box<Term>),
    /// A parenthesized term.
    Parens(Box<Term>),
    /// A null term.
    Null,
}

impl Sql for Term {
    fn sql(&self) -> String {
        match &self {
            Term::Atom(s) => s.to_string(),
            Term::Condition(t1, op, t2) => format!("{} {} {}", t1.sql(), op.sql(), t2.sql()),
            Term::Null => "".to_string(),
            Term::Parens(t) => format!("({})", t.sql()),
        }
    }
}

/// The Having struct is used to specify the having clause in a query.
/// It is used in the Query struct.
///
pub struct Having {
    pub term: Term,
}

impl Having {
    pub fn new(t: Term) -> Having {
        Having { term: t }
    }
}

impl Sql for Having {
    fn sql(&self) -> String {
        format!("{}", self.term.sql())
    }
}


pub enum OrderedColumn {
    Asc(String),
    Desc(String),
}

/// The OrderBy struct is used to specify the order by clause in a query.
/// It is used in the Query struct.
/// It is used to specify the columns, and optionally, whether they are ascending or descending.
/// Each column can be ascending or descending
pub struct OrderBy {
    pub columns: Vec<OrderedColumn>,
}

impl Sql for OrderBy {
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


/// The Query struct is the top-level object that represents a query.
/// The user is expected to construct the Query object and then call the sql() method to get the
/// SQL string.
pub struct Query {
    /// The select clause.
    pub select: Select,
    /// The table name for the select clause.
    pub from: String,
    /// The conditions for the where clause, if it exists.
    pub where_clause: Option<Term>,
    pub group_by: Option<Vec<String>>,
    pub having: Option<Having>,
    pub order_by: Option<OrderBy>,
    pub limit: Option<u64>,
    pub offset: Option<u64>,
}

impl Sql for Query {
    fn sql(&self) -> String {
        let mut result = self.select.sql();
        result.push_str(&format!(" FROM {}", self.from));
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
        result
    }
}


#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn select_star() {
        let result = Select::new(Columns::Star).sql();
        assert_eq!(result, "SELECT *");
    }

    #[test]
    fn select_cols() {
        let result = Select::new(Columns::Selected(vec!["a".to_string(), "b".to_string()])).sql();
        assert_eq!(result, "SELECT a, b");
    }

    #[test]
    fn select_cols2() {
        let result = Select::new(Columns::Selected(vec![
            "a".to_string(),
            "b".to_string(),
            "c".to_string(),
        ]))
            .sql();
        assert_eq!(result, "SELECT a, b, c");
    }

    #[test]
    fn op_o() {
        let result = Op::O("<>".to_string()).sql();
        assert_eq!(result, "<>");
    }

    #[test]
    fn term_atom() {
        let result = Term::Atom("a".to_string()).sql();
        assert_eq!(result, "a");
    }

    #[test]
    fn term_condition() {
        let result = Term::Condition(
            Box::new(Term::Atom("a".to_string())),
            Op::O("<>".to_string()),
            Box::new(Term::Atom("b".to_string())),
        )
            .sql();
        assert_eq!(result, "a <> b");
    }

    #[test]
    fn term_condition2() {
        let result = Term::Condition(
            Box::new(Term::Atom("a".to_string())),
            Op::O("<>".to_string()),
            Box::new(Term::Condition(
                Box::new(Term::Atom("b".to_string())),
                Op::O("<>".to_string()),
                Box::new(Term::Atom("c".to_string())),
            )),
        )
            .sql();
        assert_eq!(result, "a <> b <> c");
    }

    #[test]
    fn query() {
        let result = Query {
            select: Select::new(Columns::Star),
            from: "table".to_string(),
            where_clause: Some(Term::Condition(
                Box::new(Term::Atom("a".to_string())),
                Op::O("<>".to_string()),
                Box::new(Term::Atom("b".to_string())),
            )),
            group_by: None,
            having: None,
            order_by: None,
            limit: None,
            offset: None,
        }
            .sql();
        assert_eq!(result, "SELECT * FROM table WHERE a <> b");
    }

    #[test]
    fn query2() {
        let result = Query {
            select: Select::new(Columns::Selected(vec!["a".to_string(), "b".to_string()])),
            from: "table".to_string(),
            where_clause: Some(Term::Condition(
                Box::new(Term::Atom("a".to_string())),
                Op::O("<>".to_string()),
                Box::new(Term::Atom("b".to_string())),
            )),
            group_by: None,
            having: None,
            order_by: None,
            limit: None,
            offset: None,
        }
            .sql();
        assert_eq!(result, "SELECT a, b FROM table WHERE a <> b");
    }

    #[test]
    fn query3() {
        let result = Query {
            select: Select::new(Columns::Selected(vec!["a".to_string(), "b".to_string()])),
            from: "table".to_string(),
            where_clause: None,
            group_by: None,
            having: None,
            order_by: None,
            limit: None,
            offset: None,
        }
            .sql();
        assert_eq!(result, "SELECT a, b FROM table");
    }

    /// Extra-complicated query test with AND, OR, parents, and a variety of operators.
    #[test]
    fn query4() {
        let result = Query {
            select: Select::new(Columns::Selected(vec!["a".to_string(), "b".to_string()])),
            from: "table".to_string(),
            where_clause: Some(Term::Condition(
                Box::new(Term::Atom("a".to_string())),
                Op::Equals,
                Box::new(Term::Condition(
                    Box::new(Term::Atom("b".to_string())),
                    Op::And,
                    Box::new(Term::Parens(Box::new(Term::Condition(
                        Box::new(Term::Atom("c".to_string())),
                        Op::Equals,
                        Box::new(Term::Condition(
                            Box::new(Term::Atom("d".to_string())),
                            Op::Or,
                            Box::new(Term::Atom("e".to_string())),
                        )),
                    ))))),
                )),
            ),
            group_by: None,
            having: None,
            order_by: None,
            limit: None,
            offset: None,
        }
            .sql();
        assert_eq!(
            result,
            "SELECT a, b FROM table WHERE a = b AND (c = d OR e)"
        );
    }

    #[test]
    fn limit_check() {
        let result = Query {
            select: Select::new(Columns::Selected(vec!["a".to_string(), "b".to_string()])),
            from: "table".to_string(),
            where_clause: None,
            group_by: None,
            having: None,
            order_by: None,
            limit: Some(19),
            offset: None,
        }
            .sql();
        assert_eq!(result, "SELECT a, b FROM table LIMIT 19");
    }

    #[test]
    fn offset_check() {
        let result = Query {
            select: Select::new(Columns::Selected(vec!["a".to_string(), "b".to_string()])),
            from: "table".to_string(),
            where_clause: None,
            group_by: None,
            having: None,
            order_by: None,
            limit: None,
            offset: Some(10),
        }
            .sql();
        assert_eq!(result, "SELECT a, b FROM table OFFSET 10");
    }

    #[test]
    fn order_by() {
        let result = OrderBy {
            columns: vec![OrderedColumn::Asc("a".to_string())],
        }
            .sql();
        assert_eq!(result, "ORDER BY a ASC");
    }

    #[test]
    fn order_by2() {
        let result = OrderBy {
            columns: vec![
                OrderedColumn::Asc("a".to_string()),
                OrderedColumn::Desc("b".to_string()),
            ],
        }
            .sql();
        assert_eq!(result, "ORDER BY a ASC, b DESC");
    }

    #[test]
    fn test_having_simple() {
        let result = Having::new(
            Term::Condition(
                Box::new(Term::Atom("a".to_string())),
                Op::O("<>".to_string()),
                Box::new(Term::Atom("b".to_string())),
            )
        ).sql();
        assert_eq!(result, "a <> b");
    }

    // Here we test both GROUP BY and HAVING; grouping by County and HAVING on a column called paid,
    // where sum of (paid) has to be over 10000 in the HAVING clause.
    #[test]
    fn test_group_by_having() {
        let result = Query {
            select: Select::new(Columns::Selected(vec!["County".to_string(), "sum(paid)".to_string()])),
            from: "table".to_string(),
            where_clause: None,
            group_by: Some(vec!["County".to_string()]),
            having: Some(Having::new(
                Term::Condition(
                    Box::new(Term::Atom("sum(paid)".to_string())),
                    Op::O(">".to_string()),
                    Box::new(Term::Atom("10000".to_string())),
                )
            )),
            order_by: None,
            limit: None,
            offset: None,
        }.sql();
        assert_eq!(result, "SELECT County, sum(paid) FROM table GROUP BY County HAVING sum(paid) > 10000");
    }
}
