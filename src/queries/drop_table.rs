use crate::Sql;

/// DropTable is used to specify a drop table query.
pub struct DropTable<'a> {
    /// The name of the table to drop
    pub table: &'a str,
}

impl<'a> Sql for DropTable<'a> {
    fn sql(&self) -> String {
        let result = format!("DROP TABLE {}", self.table);
        result
    }
}
