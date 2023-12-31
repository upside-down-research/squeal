pub trait Sql {
    fn sql(&self) -> String;
}
pub enum Columns {
    Star,
    Selected(Vec<String>),
}

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
pub enum Term {
    Atom(String),
    Condition(Box<Term>, Op, Box<Term>),
    Null,
}
impl Sql for Term {
    fn sql(&self) -> String {
        match &self {
            Term::Atom(s) => s.to_string(),
            Term::Condition(t1, op, t2) => format!("{} {} {}", t1.sql(), op.sql(), t2.sql()),
            Term::Null => "".to_string(),
        }
    }
}
pub struct Query {
    pub select: Select,
    pub table: String,
    pub conditions: Option<Term>,
}
impl Sql for Query {
    fn sql(&self) -> String {
        let mut result = self.select.sql();
        result.push_str(&format!(" FROM {}", self.table));
        if let Some(conditions) = &self.conditions {
            result.push_str(&format!(" WHERE {}", conditions.sql()));
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
            table: "table".to_string(),
            conditions: Some(Term::Condition(
                Box::new(Term::Atom("a".to_string())),
                Op::O("<>".to_string()),
                Box::new(Term::Atom("b".to_string())),
            )),
        }
        .sql();
        assert_eq!(result, "SELECT * FROM table WHERE a <> b");
    }
    #[test]
    fn query2() {
        let result = Query {
            select: Select::new(Columns::Selected(vec!["a".to_string(), "b".to_string()])),
            table: "table".to_string(),
            conditions: Some(Term::Condition(
                Box::new(Term::Atom("a".to_string())),
                Op::O("<>".to_string()),
                Box::new(Term::Atom("b".to_string())),
            )),
        }
        .sql();
        assert_eq!(result, "SELECT a, b FROM table WHERE a <> b");
    }
    #[test]
    fn query3() {
        let result = Query {
            select: Select::new(Columns::Selected(vec!["a".to_string(), "b".to_string()])),
            table: "table".to_string(),
            conditions: None,
        }
        .sql();
        assert_eq!(result, "SELECT a, b FROM table");
    }
}
