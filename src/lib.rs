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
//!      from: "table",
//!      where_clause: Some(Term::Condition(
//!        Box::new(Term::Atom("a")),
//!      Op::O("<>"),
//!      Box::new(Term::Atom("b")))),
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
/// let result = Select::new(Columns::Selected(vec!["a", "b"])).sql();
/// assert_eq!(result, "SELECT a, b");
/// ```
pub enum Columns<'a> {
    Star,
    Selected(Vec<&'a str>),
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
pub struct Select<'a> {
    pub cols: Columns<'a>,
}

impl Select<'_> {
    pub fn new(c: Columns) -> Select {
        Select { cols: c }
    }
}

impl Sql for Select<'_> {
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
pub enum Op<'a>  {
    And,
    Or,
    Equals,
    O(&'a str),
}

impl<'a> Sql for Op<'a> {
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
pub enum Term<'a> {
    /// An atom is a single identifier.
    Atom(&'a str),
    /// A condition is a combination of two terms and an operator.
    Condition(Box<Term<'a>>, Op<'a>, Box<Term<'a>>),
    /// A parenthesized term.
    Parens(Box<Term<'a>>),
    /// A null term.
    Null,
}

impl<'a> Sql for Term<'a> {
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
/// It is constructed with a Term, similar to a Where clause.
pub struct Having<'a> {
    pub term: Term<'a>,
}

impl<'a> Having<'a> {
    pub fn new(t: Term<'a>) -> Having {
        Having { term: t }
    }
}

impl<'a> Sql for Having<'a> {
    fn sql(&self) -> String {
        format!("{}", self.term.sql())
    }
}


/// The OrderedColumn enum is used to specify the order by clause in a query.
/// It is used in the OrderBy struct.
/// It is used to specify the columns, and optionally, whether they are ascending or descending.
pub enum OrderedColumn<'a> {
    Asc(&'a str),
    Desc(&'a str),
}

/// The OrderBy struct is used to specify the order by clause in a query.
/// It is used in the Query struct.
/// It is used to specify the columns, and optionally, whether they are ascending or descending.
/// Each column can be ascending or descending
pub struct OrderBy<'a> {
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


/// The Query struct is the top-level object that represents a query.
/// The user is expected to construct the Query object and then call the sql() method to get the
/// SQL string.
///
pub struct Query<'a> {
    /// The select clause.
    pub select: Select<'a>,
    /// The table name for the select clause.
    pub from: &'a str,
    /// The conditions for the where clause, if it exists.
    pub where_clause: Option<Term<'a>>,
    pub group_by: Option<Vec<&'a str>>,
    pub having: Option<Having<'a>>,
    pub order_by: Option<OrderBy<'a>>,
    pub limit: Option<u64>,
    pub offset: Option<u64>,
}

impl<'a> Sql for Query<'a> {
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
        let result = Select::new(Columns::Selected(vec!["a","b"])).sql();
        assert_eq!(result, "SELECT a, b");
    }

    #[test]
    fn select_cols2() {
        let result = Select::new(Columns::Selected(vec![
            "a", "b","c",
        ]))
            .sql();
        assert_eq!(result, "SELECT a, b, c");
    }

    #[test]
    fn op_o() {
        let result = Op::O("<>").sql();
        assert_eq!(result, "<>");
    }

    #[test]
    fn term_atom() {
        let result = Term::Atom("a").sql();
        assert_eq!(result, "a");
    }

    #[test]
    fn term_condition() {
        let result = Term::Condition(
            Box::new(Term::Atom("a")),
            Op::O("<>"),
            Box::new(Term::Atom("b")),
        )
            .sql();
        assert_eq!(result, "a <> b");
    }

    #[test]
    fn term_condition2() {
        let result = Term::Condition(
            Box::new(Term::Atom("a")),
            Op::O("<>"),
            Box::new(Term::Condition(
                Box::new(Term::Atom("b")),
                Op::O("<>"),
                Box::new(Term::Atom("c")),
            )),
        )
            .sql();
        assert_eq!(result, "a <> b <> c");
    }

    #[test]
    fn query() {
        let result = Query {
            select: Select::new(Columns::Star),
            from: "table",
            where_clause: Some(Term::Condition(
                Box::new(Term::Atom("a")),
                Op::O("<>"),
                Box::new(Term::Atom("b")),
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
            select: Select::new(Columns::Selected(vec!["a", "b"])),
            from: "table",
            where_clause: Some(Term::Condition(
                Box::new(Term::Atom("a")),
                Op::O("<>"),
                Box::new(Term::Atom("b")),
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
            select: Select::new(Columns::Selected(vec!["a", "b"])),
            from: "table",
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
            select: Select::new(Columns::Selected(vec!["a", "b"])),
            from: "table",
            where_clause: Some(Term::Condition(
                Box::new(Term::Atom("a")),
                Op::Equals,
                Box::new(Term::Condition(
                    Box::new(Term::Atom("b")),
                    Op::And,
                    Box::new(Term::Parens(Box::new(Term::Condition(
                        Box::new(Term::Atom("c")),
                        Op::Equals,
                        Box::new(Term::Condition(
                            Box::new(Term::Atom("d")),
                            Op::Or,
                            Box::new(Term::Atom("e")),
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
            select: Select::new(Columns::Selected(vec!["a", "b"])),
            from: "table",
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
            select: Select::new(Columns::Selected(vec!["a", "b"])),
            from: "table",
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
            columns: vec![OrderedColumn::Asc("a")],
        }
            .sql();
        assert_eq!(result, "ORDER BY a ASC");
    }

    #[test]
    fn order_by2() {
        let result = OrderBy {
            columns: vec![
                OrderedColumn::Asc("a"),
                OrderedColumn::Desc("b"),
            ],
        }
            .sql();
        assert_eq!(result, "ORDER BY a ASC, b DESC");
    }

    #[test]
    fn test_having_simple() {
        let result = Having::new(
            Term::Condition(
                Box::new(Term::Atom("a")),
                Op::O("<>"),
                Box::new(Term::Atom("b")),
            )
        ).sql();
        assert_eq!(result, "a <> b");
    }

    // Here we test both GROUP BY and HAVING; grouping by County and HAVING on a column called paid,
    // where sum of (paid) has to be over 10000 in the HAVING clause.
    #[test]
    fn test_group_by_having() {
        let result = Query {
            select: Select::new(Columns::Selected(vec!["County", "sum(paid)"])),
            from: "table",
            where_clause: None,
            group_by: Some(vec!["County"]),
            having: Some(Having::new(
                Term::Condition(
                    Box::new(Term::Atom("sum(paid)")),
                    Op::O(">"),
                    Box::new(Term::Atom("10000")),
                )
            )),
            order_by: None,
            limit: None,
            offset: None,
        }.sql();
        assert_eq!(result, "SELECT County, sum(paid) FROM table GROUP BY County HAVING sum(paid) > 10000");
    }
}
