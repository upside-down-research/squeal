use crate::{Columns, Parameterized, PgParams, Sql, Term};

/// The Delete struct represents a DELETE statement
///
/// # Example
/// ```
/// use squeal::*;
/// let delete = Delete {
///     table: "users",
///     where_clause: Some(eq("id", "123")),
///     returning: None,
/// };
/// assert_eq!(delete.sql(), "DELETE FROM users WHERE id = 123");
/// ```
#[derive(Clone)]
pub struct Delete<'a> {
    /// The table name for the delete clause.
    pub table: &'a str,
    /// The conditions for the where clause, if it exists.
    pub where_clause: Option<Term<'a>>,
    /// The columns to return, if any
    pub returning: Option<Columns<'a>>,
}

impl<'a> Sql for Delete<'a> {
    fn sql(&self) -> String {
        let mut result = format!("DELETE FROM {}", self.table);
        if let Some(conditions) = &self.where_clause {
            result.push_str(&format!(" WHERE {}", conditions.sql()));
        }
        if let Some(returning) = &self.returning {
            result.push_str(&format!(" RETURNING {}", returning.sql()));
        }
        result
    }
}

/// The DeleteBuilder struct is a fluent interface for building a Delete.
/// It is not intended to be used directly, but rather through the D() function.
/// See the integration_test.rs for an example of usage.
pub struct DeleteBuilder<'a> {
    table: &'a str,
    where_clause: Option<Term<'a>>,
    returning: Option<Columns<'a>>,
    params: PgParams,
}
impl <'a> DeleteBuilder<'a> {
    /// Builds the final Delete statement
    ///
    /// # Example
    /// ```
    /// use squeal::*;
    /// let mut db = D("users");
    /// let delete = db.where_(eq("id", "10")).build();
    /// assert_eq!(delete.sql(), "DELETE FROM users WHERE id = 10");
    /// ```
    pub fn build(&self) -> Delete<'a> {
        Delete {
            table: self.table,
            where_clause: self.where_clause.clone(),
            returning: self.returning.clone(),
        }
    }
    /// Sets the WHERE clause
    ///
    /// # Example
    /// ```
    /// use squeal::*;
    /// let mut db = D("users");
    /// let delete = db.where_(eq("active", "false")).build();
    /// assert_eq!(delete.sql(), "DELETE FROM users WHERE active = false");
    /// ```
    pub fn where_(&'a mut self, term: Term<'a>) -> &'a mut DeleteBuilder<'a> {
        self.where_clause = Some(term);
        self
    }

    /// Sets the RETURNING clause
    ///
    /// # Example
    /// ```
    /// use squeal::*;
    /// let mut db = D("users");
    /// let delete = db.where_(eq("id", "10")).returning(Columns::Star).build();
    /// assert_eq!(delete.sql(), "DELETE FROM users WHERE id = 10 RETURNING *");
    /// ```
    pub fn returning(&'a mut self, columns: Columns<'a>) -> &'a mut DeleteBuilder<'a> {
        self.returning = Some(columns);
        self
    }
}

impl<'a> Parameterized for DeleteBuilder<'a> {
    fn param(&mut self) -> String {
        self.params.seq()
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
        returning: None,
        params: PgParams::new(),
    }
}
