use crate::{Columns, Parameterized, PgParams, Sql};

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
    /// Optional RETURNING clause columns
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

/// Builder for constructing INSERT statements with a fluent interface
pub struct InsertBuilder<'a> {
    table: &'a str,
    columns: Vec<&'a str>,
    values: Vec<&'a str>,
    returning: Option<Columns<'a>>,
    params: PgParams,
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
        params: PgParams::new(),
    }
}

impl<'a> InsertBuilder<'a> {
    /// Builds the final Insert statement
    ///
    /// # Example
    /// ```
    /// use squeal::*;
    /// let mut ib = I("users");
    /// let insert = ib.columns(vec!["name"]).values(vec!["'Alice'"]).build();
    /// assert_eq!(insert.sql(), "INSERT INTO users (name) VALUES ('Alice')");
    /// ```
    pub fn build(&self) -> Insert<'a> {
        Insert {
            table: self.table,
            columns: self.columns.clone(),
            values: self.values.clone(),
            returning: self.returning.clone(),
        }
    }
    /// Sets the columns to insert into
    ///
    /// # Example
    /// ```
    /// use squeal::*;
    /// let mut ib = I("users");
    /// let insert = ib.columns(vec!["name", "email"]).values(vec!["'Alice'", "'alice@example.com'"]).build();
    /// assert_eq!(insert.sql(), "INSERT INTO users (name, email) VALUES ('Alice', 'alice@example.com')");
    /// ```
    pub fn columns(&'a mut self, columns: Vec<&'a str>) -> &'a mut InsertBuilder<'a> {
        for c in columns {
            self.columns.push(c);
        }
        self
    }
    /// Sets the values to insert
    ///
    /// # Example
    /// ```
    /// use squeal::*;
    /// let mut ib = I("users");
    /// let insert = ib.columns(vec!["name"]).values(vec!["'Bob'"]).build();
    /// assert_eq!(insert.sql(), "INSERT INTO users (name) VALUES ('Bob')");
    /// ```
    pub fn values(&'a mut self, values: Vec<&'a str>) -> &'a mut InsertBuilder<'a> {
        for v in values {
            self.values.push(v);
        }
        self
    }
    /// Sets the RETURNING clause
    ///
    /// # Example
    /// ```
    /// use squeal::*;
    /// let mut ib = I("users");
    /// let insert = ib.columns(vec!["name"]).values(vec!["'Charlie'"]).returning(Columns::Star).build();
    /// assert_eq!(insert.sql(), "INSERT INTO users (name) VALUES ('Charlie') RETURNING *");
    /// ```
    pub fn returning(&'a mut self, columns: Columns<'a>) -> &'a mut InsertBuilder<'a> {
        self.returning = Some(columns);
        self
    }
}

impl<'a> Parameterized for InsertBuilder<'a> {
    fn param(&mut self) -> String {
        self.params.seq()
    }
}
