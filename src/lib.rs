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
//!      select: Some(Select::new(Columns::Star)),
//!      from: Some("table"),
//!      where_clause: Some(Term::Condition(
//!        Box::new(Term::Atom("a")),
//!      Op::O("<>"),
//!      Box::new(Term::Atom("b")))),
//!      group_by: None,
//!     having: None,
//!     order_by: None,
//!     limit: None,
//!     offset: None,
//!     for_update: false,
//! }.sql();
//!
//! assert_eq!(result, "SELECT * FROM table WHERE a <> b");
//! ```
//! Note the verbosity of the Enum scoping. This is not intentional and an artifact of
//! this still being in early development.
//!
//! Example using Q() fluent interface:
//! ```
//! use squeal::*;
//! let mut qb = Q();
//! let result = qb.select(vec!["a", "sum(b)"])
//!   .from("the_table")
//!   .where_(Term::Condition(
//!      Box::new(Term::Atom("a")),
//!      Op::O(">"),
//!      Box::new(Term::Atom("10"))))
//!   .group_by(vec!["b"])
//!   .having(Term::Condition(
//!      Box::new(Term::Atom("a")),
//!      Op::O(">"),
//!      Box::new(Term::Atom("1000"))))
//!   .limit(19)
//!   .offset(10);
//! let q = result.build();
//! assert_eq!(q.sql(), "SELECT a, sum(b) FROM the_table WHERE a > 10 GROUP BY b HAVING a > 1000 LIMIT 19 OFFSET 10");
//!
//!


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
    fn build(&self) -> Self;
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
/// assert_eq!(result, "*");
/// ```
///
/// Specific columns:
/// ```
/// use squeal::*;
/// let result = Select::new(Columns::Selected(vec!["a", "b"])).sql();
/// assert_eq!(result, "a, b");
/// ```
#[derive(Clone)]
pub enum Columns<'a> {
    Star,
    Selected(Vec<&'a str>),
}

impl<'a> Sql for Columns<'a> {
    fn sql(&self) -> String {
        match &self {
            Columns::Star => "*".to_string(),
            Columns::Selected(v) => v.join(", ").to_string(),
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
    pub cols: Columns<'a>,
}

impl<'a> Select<'a> {
    pub fn new(c: Columns<'a>) -> Select<'a> {
        Select { cols: c }
    }
}

impl<'a> Sql for Select<'a> {
    fn sql(&self) -> String {
        self.cols.sql()
    }
}

/// The Op enum is used to specify the operator in a condition.
/// It is used in the Term struct.
///
/// The Op::O variant is an escape hatch to allow you to use any operator you want.
#[derive(Clone)]
pub enum Op<'a> {
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
#[derive(Clone)]
pub struct Having<'a> {
    pub term: Term<'a>,
}

impl<'a> Having<'a> {
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
    Asc(&'a str),
    Desc(&'a str),
}

/// The OrderBy struct is used to specify the order by clause in a query.
/// It is used in the Query struct.
/// It is used to specify the columns, and optionally, whether they are ascending or descending.
/// Each column can be ascending or descending
#[derive(Clone)]
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
#[derive(Clone)]
pub struct Query<'a> {
    /// The select clause.
    pub select: Option<Select<'a>>,
    /// The table name for the select clause.
    pub from: Option<&'a str>,
    /// The conditions for the where clause, if it exists.
    pub where_clause: Option<Term<'a>>,
    pub group_by: Option<Vec<&'a str>>,
    pub having: Option<Having<'a>>,
    pub order_by: Option<OrderBy<'a>>,
    pub limit: Option<u64>,
    pub offset: Option<u64>,
    pub for_update: bool,
}

/// The QueryBuilder struct is a fluent interface for building a Query.
/// It is not intended to be used directly, but rather through the Q() function.
/// See the integration_test.rs for an example of usage.
pub struct QueryBuilder<'a> {
    pub select: Option<Select<'a>>,
    pub from: Option<&'a str>,
    pub where_clause: Option<Term<'a>>,
    pub group_by: Option<Vec<&'a str>>,
    pub having: Option<Having<'a>>,
    pub order_by: Option<OrderBy<'a>>,
    pub limit: Option<u64>,
    pub offset: Option<u64>,
    pub for_update: bool,
}

/// The Q function is a fluent interface for building a Query.
/// The user is expected to construct the Query object and then call the sql() method to get the SQL string.
/// The goal is any valid construction of a QueryBuilder is a valid Query and will, at least, syntactically, be valid SQL.
#[allow(non_snake_case)]
pub fn Q<'a>() -> QueryBuilder<'a> {
    QueryBuilder {
        select: None,
        from: None,
        where_clause: None,
        group_by: None,
        having: None,
        order_by: None,
        limit: None,
        offset: None,
        for_update: false,
    }
}

impl<'a> QueryBuilder<'a> {
    pub fn build(&self) -> Query<'a> {
        Query {
            select: self.select.clone(),
            from: self.from,
            where_clause: self.where_clause.clone(),
            group_by: self.group_by.clone(),
            having: self.having.clone(),
            order_by: self.order_by.clone(),
            limit: self.limit,
            offset: self.offset,
            for_update: self.for_update,
        }
    }
    pub fn select(&'a mut self, cols: Vec<&'a str>) -> &'a mut QueryBuilder<'a> {
        self.select = Some(Select::new(Columns::Selected(cols)));
        self
    }
    pub fn from(&'a mut self, table: &'a str) -> &'a mut QueryBuilder<'a> {
        self.from = Some(table);
        self
    }
    pub fn where_(&'a mut self, term: Term<'a>) -> &'a mut QueryBuilder<'a> {
        self.where_clause = Some(term);
        self
    }
    pub fn group_by(&'a mut self, cols: Vec<&'a str>) -> &'a mut QueryBuilder<'a> {
        self.group_by = Some(cols);
        self
    }
    pub fn having(&'a mut self, term: Term<'a>) -> &'a mut QueryBuilder<'a> {
        self.having = Some(Having::new(term));
        self
    }
    pub fn order_by(&'a mut self, cols: Vec<OrderedColumn<'a>>) -> &'a mut QueryBuilder<'a> {
        self.order_by = Some(OrderBy { columns: cols });
        self
    }
    pub fn limit(&'a mut self, limit: u64) -> &'a mut QueryBuilder<'a> {
        self.limit = Some(limit);
        self
    }
    pub fn offset(&'a mut self, offset: u64) -> &'a mut QueryBuilder<'a> {
        self.offset = Some(offset);
        self
    }
    pub fn for_update(&'a mut self) -> &'a mut QueryBuilder<'a> {
        self.for_update = true;
        self
    }
}

impl<'a> Sql for Query<'a> {
    fn sql(&self) -> String {
        let mut result = String::new();

        if let Some(select) = &self.select {
            result.push_str(&format!("SELECT {}", select.sql()));
        }
        if let Some(from) = &self.from {
            result.push_str(&format!(" FROM {}", from));
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

/// CreateTable is used to specify a create table query.
pub struct CreateTable<'a> {
    pub table: &'a str,
    /// The columns to insert. Note that they must be syntactically correct.
    pub columns: Vec<String>,
}

impl<'a> Sql for CreateTable<'a> {
    fn sql(&self) -> String {
        let mut result = format!("CREATE TABLE {} (", self.table);
        let mut first = true;
        for c in &self.columns {
            if !first {
                result.push_str(", ");
            }
            first = false;
            result.push_str(&c.to_string());
        }
        result.push(')');
        result
    }
}

/// DropTable is used to specify a drop table query.
pub struct DropTable<'a> {
    pub table: &'a str,
}

impl<'a> Sql for DropTable<'a> {
    fn sql(&self) -> String {
        let result = format!("DROP TABLE {}", self.table);
        result
    }
}

/// The TableBuilder struct is a fluent interface for building a Table.
/// Tables can be built into DROP or CREATE forms.
pub struct TableBuilder<'a> {
    pub table: &'a str,
    pub columns: Vec<Vec<String>>,
}

/// Defines a fluent interface for building a Table.
#[allow(non_snake_case)]
pub fn T<'a>(s: &'a str) -> TableBuilder<'a> {
    TableBuilder {
        table: s,
        columns: Vec::new(),
    }
}

impl<'a> TableBuilder<'a> {
    pub fn build_create_table(&self) -> CreateTable<'a> {
        let mut table_cols = Vec::new();
        for c in &self.columns {
            table_cols.push(c.join(" "));
        }
        CreateTable {
            table: self.table,
            columns: table_cols,
        }
    }
    pub fn build_drop_table(&self) -> DropTable<'a> {
        DropTable {
            table: self.table,
        }
    }
    pub fn table(&mut self, table: &'a str) -> &mut TableBuilder<'a> {
        self.table = table;
        self
    }
    pub fn column(&mut self, column: &str, datatype: &str, other: Vec<&str>) -> &mut TableBuilder<'a> {
        let mut col = vec![column, datatype];
        col.extend(other);
        let str_cols = col.iter().map(|s| s.to_string()).collect();
        self.columns.push(str_cols);
        self
    }
}


/// The Insert struct is used to specify an insert query.
/// The user is expect to construct the Insert object and then call the sql() method to
/// get the SQL string.
///
///  # Examples
/// ```
/// use squeal::*;
/// let result = Insert {
///    table: "table",
///    columns: vec!["a", "b"],
///    values: vec!["1", "2"],
///    returning: None,
/// }.sql();
/// assert_eq!(result, "INSERT INTO table (a, b) VALUES (1, 2)");
/// ```
/// Note that the values are not escaped, so you must do that yourself.
/// If using a prepared statement, you will have to specify the Placeholder and pass in the values to
/// the execution call at the callsite rather than the preparation site.
#[derive(Clone)]
pub struct Insert<'a> {
    /// The table name for the insert clause.
    pub table: &'a str,
    /// The columns to insert.
    pub columns: Vec<&'a str>,
    /// The values to insert.
    pub values: Vec<&'a str>,
    pub returning: Option<Columns<'a>>,
}

impl<'a> Sql for Insert<'a> {
    fn sql(&self) -> String {
        let mut result = format!("INSERT INTO {} (", self.table);
        let mut first = true;
        for c in &self.columns {
            if !first {
                result.push_str(", ");
            }
            first = false;
            result.push_str(c.as_ref());
        }
        result.push_str(") VALUES (");
        let mut first = true;
        for v in &self.values {
            if !first {
                result.push_str(", ");
            }
            first = false;
            result.push_str(v.as_ref());
        }
        result.push(')');

        if self.returning.is_some() {
            result.push_str(&format!(" RETURNING {}", self.returning.as_ref().unwrap().sql()));
        }

        result
    }
}

pub struct InsertBuilder<'a> {
    table: &'a str,
    columns: Vec<&'a str>,
    values: Vec<&'a str>,
    returning: Option<Columns<'a>>,
}

/// Defines a fluent interface for building an Insert.
/// The user is expect to construct the Insert object and then call the sql() method to
/// get the SQL string.
///
/// # Example
/// ```
/// use squeal::*;
/// let result = I("table")
///    .columns(vec!["a", "b"])
///    .values(vec!["1", "2"])
///    .build()
///    .sql();
/// assert_eq!(result, "INSERT INTO table (a, b) VALUES (1, 2)");
/// ```
///
#[allow(non_snake_case)]
pub fn I<'a>(table: &'a str) -> InsertBuilder<'a> {
    InsertBuilder {
        table,
        columns: Vec::new(),
        values: Vec::new(),
        returning: None,
    }
}

impl<'a> InsertBuilder<'a> {
    pub fn build(&self) -> Insert<'a> {
        Insert {
            table: self.table,
            columns: self.columns.clone(),
            values: self.values.clone(),
            returning: self.returning.clone(),
        }
    }
    pub fn columns(&'a mut self, columns: Vec<&'a str>) -> &'a mut InsertBuilder<'a> {
        for c in columns {
            self.columns.push(c);
        }
        self
    }
    pub fn values(&'a mut self, values: Vec<&'a str>) -> &'a mut InsertBuilder<'a> {
        for v in values {
            self.values.push(v);
        }
        self
    }
    pub fn returning(&'a mut self, columns: Columns<'a>) -> &'a mut InsertBuilder<'a> {
        self.returning = Some(columns);
        self
    }
}


/// The Update struct is used to specify an update query.
/// The user is expect to construct the Update object and then call the sql() method to
/// get the SQL string.
///
#[derive(Clone)]
pub struct Update<'a> {
    /// The table name for the update clause.
    pub table: &'a str,
    /// The columns to update.
    pub columns: Vec<&'a str>,
    /// The values to update.
    pub values: Vec<&'a str>,
    /// A table expression allowing columns from other tables to appear in the WHERE condition and
    /// update expressions. -- pg 16 docs.
    pub from: Option<&'a str>,
    /// The conditions for the where clause, if it exists.
    pub where_clause: Option<Term<'a>>,
    /// The columns to return, if any
    pub returning: Option<Columns<'a>>,
}

impl<'a> Sql for Update<'a> {
    fn sql(&self) -> String {
        let mut result = format!("UPDATE {} SET ", self.table);
        let mut first = true;
        for (c, v) in self.columns.iter().zip(self.values.iter()) {
            if !first {
                result.push_str(", ");
            }
            first = false;
            result.push_str(&format!("{} = {}", c, v));
        }
        if let Some(from) = &self.from {
            result.push_str(&format!(" FROM {}", from));
        }
        if let Some(conditions) = &self.where_clause {
            result.push_str(&format!(" WHERE {}", conditions.sql()));
        }
        if let Some(returning) = &self.returning {
            result.push_str(&format!(" RETURNING {}", returning.sql()));
        }
        result
    }
}

/// The UpdateBuilder struct is a fluent interface for building an Update.
/// It is not intended to be used directly, but rather through the U() function.
/// See the integration_test.rs for an example of usage.
pub struct UpdateBuilder<'a> {
    table: &'a str,
    columns: Vec<&'a str >,
    values: Vec<&'a str>,
    from: Option<&'a str>,
    where_clause: Option<Term<'a>>,
    returning: Option<Columns<'a>>,
}

/// Defines a fluent interface for building an Update.
/// The user is expect to construct the Update object and then call the sql() method to
/// get the SQL string.
///
/// # Example
/// ```
/// use squeal::*;
/// let mut u = U("table");
/// let result = u
///   .columns(vec!["a", "b"])
///   .values(vec!["1", "2"])
///   .where_(Term::Condition(
///     Box::new(Term::Atom("a")),
///     Op::Equals,
///     Box::new(Term::Atom("b"))))
///   .build();
/// assert_eq!(result.sql(), "UPDATE table SET a = 1, b = 2 WHERE a = b");
/// ```
///
#[allow(non_snake_case)]
pub fn U<'a>(table: &'a str) -> UpdateBuilder<'a> {
    UpdateBuilder {
        table,
        columns: Vec::new(),
        values: Vec::new(),
        from: None,
        where_clause: None,
        returning: None,
    }
}
impl<'a> UpdateBuilder<'a> {
    pub fn columns(&'a mut self, columns: Vec<&'a str>) -> &'a mut UpdateBuilder<'a> {
        for c in columns {
            self.columns.push(c);
        }
        self
    }
    pub fn values(&'a mut self, values: Vec<&'a str>) -> &'a mut UpdateBuilder<'a> {
        for v in values {
            self.values.push(v);
        }
        self
    }
    pub fn from(&'a mut self, from: &'a str) -> &'a mut UpdateBuilder<'a> {
        self.from = Some(from);
        self
    }
    pub fn where_(&'a mut self, term: Term<'a>) -> &'a mut UpdateBuilder<'a> {
        self.where_clause = Some(term);
        self
    }
    pub fn returning(&'a mut self, columns: Columns<'a>) -> &'a mut UpdateBuilder<'a> {
        self.returning = Some(columns);
        self
    }
    pub fn build(&self) -> Update<'a> {
        Update {
            table: self.table,
            columns: self.columns.clone(),
            values: self.values.clone(),
            from: self.from,
            where_clause: self.where_clause.clone(),
            returning: self.returning.clone(),
        }
    }

}


#[derive(Clone)]
pub struct Delete<'a> {
    /// The table name for the delete clause.
    pub table: &'a str,
    /// The conditions for the where clause, if it exists.
    pub where_clause: Option<Term<'a>>,
}

impl<'a> Sql for Delete<'a> {
    fn sql(&self) -> String {
        let mut result = format!("DELETE FROM {}", self.table);
        if let Some(conditions) = &self.where_clause {
            result.push_str(&format!(" WHERE {}", conditions.sql()));
        }
        result
    }
}

/// The DeleteBuilder struct is a fluent interface for building a Delete.
/// It is not intended to be used directly, but rather through the D() function.
/// See the integration_test.rs for an example of usage.
///
pub struct DeleteBuilder<'a> {
    table: &'a str,
    where_clause: Option<Term<'a>>,
}
impl <'a> DeleteBuilder<'a> {
    pub fn build(&self) -> Delete<'a> {
        Delete {
            table: self.table,
            where_clause: self.where_clause.clone(),
        }
    }
    pub fn where_(&'a mut self, term: Term<'a>) -> &'a mut DeleteBuilder<'a> {
        self.where_clause = Some(term);
        self
    }
}

/// Defines a fluent interface for building a Delete.
/// The user is expect to construct the Delete object and then call the sql() method to
/// get the SQL string.
#[allow(non_snake_case)]
pub fn D<'a>(table: &'a str) -> DeleteBuilder<'a> {
    DeleteBuilder {
        table,
        where_clause: None,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn select_star() {
        let result = Select::new(Columns::Star).sql();
        assert_eq!(result, "*");
    }

    #[test]
    fn select_cols() {
        let result = Select::new(Columns::Selected(vec!["a", "b"])).sql();
        assert_eq!(result, "a, b");
    }

    #[test]
    fn select_cols2() {
        let result = Select::new(Columns::Selected(vec![
            "a", "b", "c",
        ]))
            .sql();
        assert_eq!(result, "a, b, c");
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
            select: Some(Select::new(Columns::Star)),
            from: Some("table"),
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
            for_update: false,
        }
            .sql();
        assert_eq!(result, "SELECT * FROM table WHERE a <> b");
    }

    #[test]
    fn query2() {
        let result = Query {
            select: Some(Select::new(Columns::Selected(vec!["a", "b"]))),
            from: Some("table"),
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
            for_update: false,
        }
            .sql();
        assert_eq!(result, "SELECT a, b FROM table WHERE a <> b");
    }

    #[test]
    fn query3() {
        let result = Query {
            select: Some(Select::new(Columns::Selected(vec!["a", "b"]))),
            from: Some("table"),
            where_clause: None,
            group_by: None,
            having: None,
            order_by: None,
            limit: None,
            offset: None,
            for_update: false,
        }
            .sql();
        assert_eq!(result, "SELECT a, b FROM table");
    }

    /// Extra-complicated query test with AND, OR, parents, and a variety of operators.
    #[test]
    fn query4() {
        let result = Query {
            select: Some(Select::new(Columns::Selected(vec!["a", "b"]))),
            from: Some("table"),
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
            for_update: false,
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
            select: Some(Select::new(Columns::Selected(vec!["a", "b"]))),
            from: Some("table"),
            where_clause: None,
            group_by: None,
            having: None,
            order_by: None,
            limit: Some(19),
            offset: None,
            for_update: false,
        }
            .sql();
        assert_eq!(result, "SELECT a, b FROM table LIMIT 19");
    }

    #[test]
    fn offset_check() {
        let result = Query {
            select: Some(Select::new(Columns::Selected(vec!["a", "b"]))),
            from: Some("table"),
            where_clause: None,
            group_by: None,
            having: None,
            order_by: None,
            limit: None,
            offset: Some(10),
            for_update: false,
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
            select: Some(Select::new(Columns::Selected(vec!["County", "sum(paid)"]))),
            from: Some("table"),
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
            for_update: false,
        }.sql();
        assert_eq!(result, "SELECT County, sum(paid) FROM table GROUP BY County HAVING sum(paid) > 10000");
    }

    #[test]
    fn test_create_table_simple() {
        let result = CreateTable {
            table: "table",
            columns: vec!["a int".to_string(), "b int".to_string()],
        }.sql();
        assert_eq!(result, "CREATE TABLE table (a int, b int)");
    }

    #[test]
    fn test_create_table_primary_keys_and_foreign_keys() {
        let result = CreateTable {
            table: "table",
            columns: vec!["a int".to_string(), "b int".to_string(), "PRIMARY KEY (a)".to_string(), "FOREIGN KEY (b) REFERENCES table2 (b)".to_string()],
        }.sql();
        assert_eq!(result, "CREATE TABLE table (a int, b int, PRIMARY KEY (a), FOREIGN KEY (b) REFERENCES table2 (b))");
    }

    #[test]
    fn test_drop_table_simple() {
        let result = DropTable {
            table: "table",
        }.sql();
        assert_eq!(result, "DROP TABLE table");
    }

    #[test]
    fn test_create_table_fluent_interface() {
        let result = T("table")
            .column("a", "int", vec![])
            .column("b", "int", vec![])
            .build_create_table()
            .sql();
        assert_eq!(result, "CREATE TABLE table (a int, b int)");
    }

    #[test]
    fn test_create_table_complicated_fluent() {
        // this will test foreign keys, primary keys, and other constraints
        let result = T("table")
            .column("a", "int", vec![])
            .column("b", "int", vec![])
            .column("c", "int", vec!["PRIMARY KEY"])
            .column("d", "int", vec!["FOREIGN KEY REFERENCES table2 (d)"])
            .build_create_table()
            .sql();
        assert_eq!(result, "CREATE TABLE table (a int, b int, c int PRIMARY KEY, d int FOREIGN KEY REFERENCES table2 (d))");
    }

    #[test]
    fn test_insert_simple() {
        let result = Insert {
            table: "table",
            columns: vec!["a", "b"],
            values: vec!["1", "2"],
            returning: None,
        }.sql();
        assert_eq!(result, "INSERT INTO table (a, b) VALUES (1, 2)");
    }

    #[test]
    fn test_insert_i_fluent() {
        let result = I("table")
            .columns(vec!["a", "b"])
            .values(vec!["1", "2"])
            .build()
            .sql();
        assert_eq!(result, "INSERT INTO table (a, b) VALUES (1, 2)");
    }

    #[test]
    fn test_insert_with_returning_and_complicated() {
        let result = I("table")
            .columns(vec!["a", "b"])
            .values(vec!["1", "2"])
            .returning(Columns::Selected(vec!["a", "b"]))
            .build()
            .sql();
        assert_eq!(result, "INSERT INTO table (a, b) VALUES (1, 2) RETURNING a, b");
    }
    #[test]
    fn test_delete_simple() {
        let result = D("table")
            .where_(Term::Condition(
                Box::new(Term::Atom("a")),
                Op::Equals,
                Box::new(Term::Atom("b")),
            ))
            .build()
            .sql();
        assert_eq!(result, "DELETE FROM table WHERE a = b");
    }
    #[test]
    fn test_delete_complex() {
        let result = D("table")
            .where_(Term::Condition(
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
                )))
            .build()
            .sql();
        assert_eq!(result, "DELETE FROM table WHERE a = b AND (c = d OR e)");
    }

    #[test]
    fn test_update_simple() {
        let result = U("table")
            .columns(vec!["a", "b"])
            .values(vec!["1", "2"])
            .where_(Term::Condition(
                Box::new(Term::Atom("a")),
                Op::Equals,
                Box::new(Term::Atom("b")),
            ))
            .build()
            .sql();
        assert_eq!(result, "UPDATE table SET a = 1, b = 2 WHERE a = b");
    }
    #[test]
    fn test_update_complex() {
        let result = U("table")
            .columns(vec!["a", "b"])
            .values(vec!["1", "2"])
            .from("table2")
            .where_(Term::Condition(
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
                )))
            .returning(Columns::Selected(vec!["a", "b"]))
            .build()
            .sql();
        assert_eq!(result, "UPDATE table SET a = 1, b = 2 FROM table2 WHERE a = b AND (c = d OR e) RETURNING a, b");
    }
}
