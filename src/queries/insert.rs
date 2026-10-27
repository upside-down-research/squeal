use crate::{BuildError, Columns, Parameterized, PgParams, Query, Sql};

/// Represents the source of data for an INSERT statement
#[derive(Clone)]
pub enum InsertSource<'a> {
    /// Insert from literal values: VALUES (val1, val2, ...), (val3, val4, ...), ...
    /// Each inner Vec represents one row of values
    Values(Vec<Vec<&'a str>>),
    /// Insert from a SELECT query: SELECT ...
    Select(Box<Query<'a>>),
}

/// Represents the ON CONFLICT clause for INSERT statements (PostgreSQL UPSERT)
#[derive(Clone)]
pub enum OnConflict<'a> {
    /// ON CONFLICT (columns) DO NOTHING
    DoNothing(Vec<&'a str>),
    /// ON CONFLICT (columns) DO UPDATE SET col1 = val1, col2 = val2, ...
    DoUpdate(Vec<&'a str>, Vec<(&'a str, &'a str)>),
}

impl<'a> Sql for OnConflict<'a> {
    fn sql(&self) -> String {
        match self {
            OnConflict::DoNothing(columns) => {
                format!("ON CONFLICT ({}) DO NOTHING", columns.join(", "))
            }
            OnConflict::DoUpdate(columns, updates) => {
                let mut result = format!("ON CONFLICT ({}) DO UPDATE SET ", columns.join(", "));
                let mut first = true;
                for (col, val) in updates {
                    if !first {
                        result.push_str(", ");
                    }
                    first = false;
                    result.push_str(&format!("{} = {}", col, val));
                }
                result
            }
        }
    }
}

/// The Insert struct is used to specify an insert query.
/// The user is expect to construct the Insert object and then call the sql() method to
/// get the SQL string.
///
///  # Examples
/// ```
/// use squeal::*;
/// // Single row insert
/// let result = Insert {
///    table: "table",
///    columns: vec!["a", "b"],
///    source: InsertSource::Values(vec![vec!["1", "2"]]),
///    on_conflict: None,
///    returning: None,
/// }.sql();
/// assert_eq!(result, "INSERT INTO table (a, b) VALUES (1, 2)");
/// ```
///
/// ```
/// use squeal::*;
/// // Multiple row insert
/// let result = Insert {
///    table: "table",
///    columns: vec!["a", "b"],
///    source: InsertSource::Values(vec![vec!["1", "2"], vec!["3", "4"]]),
///    on_conflict: None,
///    returning: None,
/// }.sql();
/// assert_eq!(result, "INSERT INTO table (a, b) VALUES (1, 2), (3, 4)");
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
    /// The source of data (VALUES or SELECT)
    pub source: InsertSource<'a>,
    /// Optional ON CONFLICT clause for handling unique constraint violations
    pub on_conflict: Option<OnConflict<'a>>,
    /// Optional RETURNING clause columns
    pub returning: Option<Columns<'a>>,
}

impl<'a> Insert<'a> {
    /// Checks that the insert can produce valid SQL.
    ///
    /// Returns `BuildError::NoColumns` if `columns` is empty. For `InsertSource::Values`,
    /// returns `BuildError::NoRows` if there are no rows, and `BuildError::RowWidthMismatch`
    /// if any row does not have one value per column. The width of an
    /// `InsertSource::Select` is not checked.
    pub fn validate(&self) -> Result<(), BuildError> {
        if self.columns.is_empty() {
            return Err(BuildError::NoColumns);
        }
        if let InsertSource::Values(rows) = &self.source {
            if rows.is_empty() {
                return Err(BuildError::NoRows);
            }
            for (i, row) in rows.iter().enumerate() {
                if row.len() != self.columns.len() {
                    return Err(BuildError::RowWidthMismatch {
                        row: i,
                        columns: self.columns.len(),
                        values: row.len(),
                    });
                }
            }
        }
        Ok(())
    }
}

impl<'a> Sql for Insert<'a> {
    /// # Panics
    /// Panics if `validate()` fails.
    fn sql(&self) -> String {
        if let Err(e) = self.validate() {
            panic!("{}", e);
        }
        let mut result = format!("INSERT INTO {} (", self.table);
        let mut first = true;
        for c in &self.columns {
            if !first {
                result.push_str(", ");
            }
            first = false;
            result.push_str(c.as_ref());
        }
        result.push_str(") ");

        // Handle source (VALUES or SELECT)
        match &self.source {
            InsertSource::Values(rows) => {
                result.push_str("VALUES ");
                let mut first_row = true;
                for row in rows {
                    if !first_row {
                        result.push_str(", ");
                    }
                    first_row = false;
                    result.push('(');
                    let mut first_val = true;
                    for v in row {
                        if !first_val {
                            result.push_str(", ");
                        }
                        first_val = false;
                        result.push_str(v.as_ref());
                    }
                    result.push(')');
                }
            }
            InsertSource::Select(query) => {
                result.push_str(&query.sql());
            }
        }

        if let Some(on_conflict) = &self.on_conflict {
            result.push_str(&format!(" {}", on_conflict.sql()));
        }

        if let Some(returning) = &self.returning {
            result.push_str(&format!(" RETURNING {}", returning.sql()));
        }

        result
    }
}

/// Builder for constructing INSERT statements with a fluent interface
pub struct InsertBuilder<'a> {
    table: &'a str,
    columns: Vec<&'a str>,
    source: Option<InsertSource<'a>>,
    on_conflict: Option<OnConflict<'a>>,
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
        source: None,
        on_conflict: None,
        returning: None,
        params: PgParams::new(),
    }
}

impl<'a> InsertBuilder<'a> {
    /// Builds the final Insert statement
    ///
    /// # Panics
    /// Panics if `Insert::validate()` fails: no columns, no values, or a row whose width
    /// differs from the column count. Use `build_checked()` to get a `Result` instead.
    ///
    /// # Example
    /// ```
    /// use squeal::*;
    /// let mut ib = I("users");
    /// let insert = ib.columns(vec!["name"]).values(vec!["'Alice'"]).build();
    /// assert_eq!(insert.sql(), "INSERT INTO users (name) VALUES ('Alice')");
    /// ```
    pub fn build(&self) -> Insert<'a> {
        self.build_checked().unwrap_or_else(|e| panic!("{}", e))
    }

    /// Builds the final Insert statement, returning an error if it is invalid.
    ///
    /// # Example
    /// ```
    /// use squeal::*;
    /// let mut ib = I("t");
    /// let result = ib.columns(vec!["a", "b"]).values(vec!["1"]).build_checked();
    /// assert_eq!(
    ///     result.err(),
    ///     Some(BuildError::RowWidthMismatch { row: 0, columns: 2, values: 1 })
    /// );
    /// ```
    pub fn build_checked(&self) -> Result<Insert<'a>, BuildError> {
        let insert = Insert {
            table: self.table,
            columns: self.columns.clone(),
            source: self
                .source
                .clone()
                .unwrap_or(InsertSource::Values(vec![Vec::new()])),
            on_conflict: self.on_conflict.clone(),
            returning: self.returning.clone(),
        };
        insert.validate()?;
        Ok(insert)
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
    /// Sets the values to insert (single row)
    ///
    /// # Example
    /// ```
    /// use squeal::*;
    /// let mut ib = I("users");
    /// let insert = ib.columns(vec!["name"]).values(vec!["'Bob'"]).build();
    /// assert_eq!(insert.sql(), "INSERT INTO users (name) VALUES ('Bob')");
    /// ```
    pub fn values(&'a mut self, values: Vec<&'a str>) -> &'a mut InsertBuilder<'a> {
        self.source = Some(InsertSource::Values(vec![values]));
        self
    }

    /// Sets multiple rows of values to insert
    ///
    /// # Example
    /// ```
    /// use squeal::*;
    /// let mut ib = I("users");
    /// let insert = ib.columns(vec!["name", "age"])
    ///     .rows(vec![
    ///         vec!["'Alice'", "30"],
    ///         vec!["'Bob'", "25"],
    ///         vec!["'Charlie'", "35"]
    ///     ])
    ///     .build();
    /// assert_eq!(insert.sql(), "INSERT INTO users (name, age) VALUES ('Alice', 30), ('Bob', 25), ('Charlie', 35)");
    /// ```
    pub fn rows(&'a mut self, rows: Vec<Vec<&'a str>>) -> &'a mut InsertBuilder<'a> {
        self.source = Some(InsertSource::Values(rows));
        self
    }

    /// Sets a SELECT query as the data source
    ///
    /// # Example
    /// ```
    /// use squeal::*;
    /// let subquery = Query {
    ///     with_clause: None,
    ///     select: Some(Select::new(Columns::Selected(vec!["name", "email"]), None)),
    ///     from: Some(FromSource::Table("active_users")),
    ///     joins: vec![],
    ///     where_clause: None,
    ///     group_by: None,
    ///     having: None,
    ///     order_by: None,
    ///     limit: None,
    ///     offset: None,
    ///     for_update: false,
    /// };
    /// let mut ib = I("archived_users");
    /// let insert = ib.columns(vec!["name", "email"]).select(subquery).build();
    /// assert_eq!(insert.sql(), "INSERT INTO archived_users (name, email) SELECT name, email FROM active_users");
    /// ```
    pub fn select(&'a mut self, query: Query<'a>) -> &'a mut InsertBuilder<'a> {
        self.source = Some(InsertSource::Select(Box::new(query)));
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

    /// Sets the ON CONFLICT DO NOTHING clause
    ///
    /// # Example
    /// ```
    /// use squeal::*;
    /// let mut ib = I("users");
    /// let insert = ib.columns(vec!["email", "name"])
    ///     .values(vec!["'alice@example.com'", "'Alice'"])
    ///     .on_conflict_do_nothing(vec!["email"])
    ///     .build();
    /// assert_eq!(insert.sql(), "INSERT INTO users (email, name) VALUES ('alice@example.com', 'Alice') ON CONFLICT (email) DO NOTHING");
    /// ```
    pub fn on_conflict_do_nothing(
        &'a mut self,
        columns: Vec<&'a str>,
    ) -> &'a mut InsertBuilder<'a> {
        self.on_conflict = Some(OnConflict::DoNothing(columns));
        self
    }

    /// Sets the ON CONFLICT DO UPDATE clause
    ///
    /// # Example
    /// ```
    /// use squeal::*;
    /// let mut ib = I("users");
    /// let insert = ib.columns(vec!["email", "name"])
    ///     .values(vec!["'alice@example.com'", "'Alice'"])
    ///     .on_conflict_do_update(vec!["email"], vec![("name", "'Alice Updated'")])
    ///     .build();
    /// assert_eq!(insert.sql(), "INSERT INTO users (email, name) VALUES ('alice@example.com', 'Alice') ON CONFLICT (email) DO UPDATE SET name = 'Alice Updated'");
    /// ```
    pub fn on_conflict_do_update(
        &'a mut self,
        conflict_columns: Vec<&'a str>,
        updates: Vec<(&'a str, &'a str)>,
    ) -> &'a mut InsertBuilder<'a> {
        self.on_conflict = Some(OnConflict::DoUpdate(conflict_columns, updates));
        self
    }
}

impl<'a> Parameterized for InsertBuilder<'a> {
    fn param(&mut self) -> String {
        self.params.seq()
    }
}
