use crate::{Columns, Parameterized, PgParams, Sql, Term};

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
    params: PgParams,
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
        params: PgParams::new(),
    }
}
impl<'a> UpdateBuilder<'a> {
    /// Sets column-value pairs for the UPDATE statement
    /// This is more ergonomic than using separate columns() and values() methods
    /// as it keeps column-value pairs together, preventing mismatches.
    pub fn set(&'a mut self, pairs: Vec<(&'a str, &'a str)>) -> &'a mut UpdateBuilder<'a> {
        for (col, val) in pairs {
            self.columns.push(col);
            self.values.push(val);
        }
        self
    }

    /// Sets the columns to update (use with values())
    ///
    /// # Example
    /// ```
    /// use squeal::*;
    /// let mut ub = U("users");
    /// let update = ub.columns(vec!["name"]).values(vec!["'David'"]).build();
    /// assert_eq!(update.sql(), "UPDATE users SET name = 'David'");
    /// ```
    pub fn columns(&'a mut self, columns: Vec<&'a str>) -> &'a mut UpdateBuilder<'a> {
        for c in columns {
            self.columns.push(c);
        }
        self
    }
    /// Sets the values to update (use with columns())
    ///
    /// # Example
    /// ```
    /// use squeal::*;
    /// let mut ub = U("users");
    /// let update = ub.columns(vec!["email"]).values(vec!["'new@example.com'"]).build();
    /// assert_eq!(update.sql(), "UPDATE users SET email = 'new@example.com'");
    /// ```
    pub fn values(&'a mut self, values: Vec<&'a str>) -> &'a mut UpdateBuilder<'a> {
        for v in values {
            self.values.push(v);
        }
        self
    }
    /// Sets the FROM clause for PostgreSQL UPDATE...FROM syntax
    ///
    /// # Example
    /// ```
    /// use squeal::*;
    /// let mut ub = U("users");
    /// let update = ub.set(vec![("active", "false")]).from("banned").where_(eq("users.id", "banned.user_id")).build();
    /// assert_eq!(update.sql(), "UPDATE users SET active = false FROM banned WHERE users.id = banned.user_id");
    /// ```
    pub fn from(&'a mut self, from: &'a str) -> &'a mut UpdateBuilder<'a> {
        self.from = Some(from);
        self
    }
    /// Sets the WHERE clause
    ///
    /// # Example
    /// ```
    /// use squeal::*;
    /// let mut ub = U("users");
    /// let update = ub.set(vec![("active", "false")]).where_(eq("id", "5")).build();
    /// assert_eq!(update.sql(), "UPDATE users SET active = false WHERE id = 5");
    /// ```
    pub fn where_(&'a mut self, term: Term<'a>) -> &'a mut UpdateBuilder<'a> {
        self.where_clause = Some(term);
        self
    }
    /// Sets the RETURNING clause
    ///
    /// # Example
    /// ```
    /// use squeal::*;
    /// let mut ub = U("users");
    /// let update = ub.set(vec![("status", "'active'")]).returning(Columns::Selected(vec!["id", "status"])).build();
    /// assert_eq!(update.sql(), "UPDATE users SET status = 'active' RETURNING id, status");
    /// ```
    pub fn returning(&'a mut self, columns: Columns<'a>) -> &'a mut UpdateBuilder<'a> {
        self.returning = Some(columns);
        self
    }
    /// Builds the final Update statement
    ///
    /// # Example
    /// ```
    /// use squeal::*;
    /// let mut ub = U("users");
    /// let update = ub.set(vec![("name", "'Eve'")]).build();
    /// assert_eq!(update.sql(), "UPDATE users SET name = 'Eve'");
    /// ```
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

impl<'a> Parameterized for UpdateBuilder<'a> {
    fn param(&mut self) -> String {
        self.params.seq()
    }
}
