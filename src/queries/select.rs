use crate::{Distinct, Query, Sql};

/// A single expression in a SELECT clause
#[derive(Clone)]
pub enum SelectExpression<'a> {
    /// A simple column name or expression
    Column(&'a str),
    /// A subquery with an optional alias
    Subquery(Box<Query<'a>>, Option<&'a str>),
}

impl<'a> Sql for SelectExpression<'a> {
    fn sql(&self) -> String {
        match self {
            SelectExpression::Column(col) => col.to_string(),
            SelectExpression::Subquery(query, alias) => {
                if let Some(a) = alias {
                    format!("({}) AS {}", query.sql(), a)
                } else {
                    format!("({})", query.sql())
                }
            }
        }
    }
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
/// let result = Select::new(Columns::Star, None).sql();
/// assert_eq!(result, "*");
/// ```
///
/// Specific columns:
/// ```
/// use squeal::*;
/// let result = Select::new(Columns::Selected(vec!["a", "b"]), None).sql();
/// assert_eq!(result, "a, b");
/// ```
#[derive(Clone)]
pub enum Columns<'a> {
    /// Wildcard selector (*)
    Star,
    /// Specific column names
    Selected(Vec<&'a str>),
    /// Mix of columns and subqueries
    Expressions(Vec<SelectExpression<'a>>),
}

impl<'a> Sql for Columns<'a> {
    fn sql(&self) -> String {
        match &self {
            Columns::Star => "*".to_string(),
            Columns::Selected(v) => v.join(", ").to_string(),
            Columns::Expressions(exprs) => exprs
                .iter()
                .map(|e| e.sql())
                .collect::<Vec<String>>()
                .join(", "),
        }
    }
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
#[derive(Clone)]
pub struct Select<'a> {
    /// The columns to select
    pub cols: Columns<'a>,
    pub distinct: Option<Distinct<'a>>,
}

impl<'a> Select<'a> {
    /// Creates a new Select with the given columns
    ///
    /// # Example
    /// ```
    /// use squeal::*;
    /// let select = Select::new(Columns::Star, None);
    /// assert_eq!(select.sql(), "*");
    /// ```
    pub fn new(c: Columns<'a>, d: Option<Distinct<'a>>) -> Select<'a> {
        Select {
            cols: c,
            distinct: d,
        }
    }
}

impl<'a> Sql for Select<'a> {
    fn sql(&self) -> String {
        let mut s = String::new();
        if let Some(d) = &self.distinct {
            match d {
                Distinct::All => s.push_str("DISTINCT "),
                Distinct::On(cols) => s.push_str(&format!("DISTINCT ON ({}) ", cols.join(", "))),
            }
        }
        s.push_str(&self.cols.sql());
        s
    }
}
