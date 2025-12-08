use crate::{DropTable, Sql};

/// CreateTable is used to specify a create table query.
pub struct CreateTable<'a> {
    /// The name of the table to create
    pub table: &'a str,
    /// The columns to create. Note that they must be syntactically correct.
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

/// The TableBuilder struct is a fluent interface for building a Table.
/// Tables can be built into DROP or CREATE forms.
pub struct TableBuilder<'a> {
    /// The table name
    pub table: &'a str,
    /// Column definitions (each inner Vec represents one column definition)
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
    /// Builds a CREATE TABLE statement
    ///
    /// # Example
    /// ```
    /// use squeal::*;
    /// let mut tb = T("users");
    /// let create = tb.column("id", "serial", vec![]).build_create_table();
    /// assert_eq!(create.sql(), "CREATE TABLE users (id serial)");
    /// ```
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
    /// Builds a DROP TABLE statement
    ///
    /// # Example
    /// ```
    /// use squeal::*;
    /// let mut tb = T("users");
    /// let drop = tb.build_drop_table();
    /// assert_eq!(drop.sql(), "DROP TABLE users");
    /// ```
    pub fn build_drop_table(&self) -> DropTable<'a> {
        DropTable { table: self.table }
    }
    /// Changes the table name
    ///
    /// # Example
    /// ```
    /// use squeal::*;
    /// let mut builder = T("old_name");
    /// builder.table("new_name");
    /// let create = builder.build_create_table();
    /// assert_eq!(create.sql(), "CREATE TABLE new_name ()");
    /// ```
    pub fn table(&mut self, table: &'a str) -> &mut TableBuilder<'a> {
        self.table = table;
        self
    }
    /// Adds a column definition
    ///
    /// # Example
    /// ```
    /// use squeal::*;
    /// let mut tb = T("users");
    /// let create = tb.column("id", "serial", vec!["PRIMARY KEY"])
    ///     .build_create_table();
    /// assert_eq!(create.sql(), "CREATE TABLE users (id serial PRIMARY KEY)");
    /// ```
    pub fn column(
        &mut self,
        column: &str,
        datatype: &str,
        other: Vec<&str>,
    ) -> &mut TableBuilder<'a> {
        let mut col = vec![column, datatype];
        col.extend(other);
        let str_cols = col.iter().map(|s| s.to_string()).collect();
        self.columns.push(str_cols);
        self
    }
}
