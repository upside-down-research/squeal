pub enum Columns {
    Star,
    Selected(Vec<String>)
}

pub struct Select {
    pub cols: Columns
}

impl Select {
    pub fn new(c: Columns) -> Select {
        Select {cols: c}
    }

    pub fn sql(&self) -> String {
        match &self.cols {
            Columns::Star => "SELECT *".to_string(),
            Columns::Selected(v) => format!("SELECT {}", v.join(", "))
        }
    }
}

pub enum Condition  {
    Op{left: String,
       op: String,
       right: String},
    Leg{
}

pub enum Logical {
    And(Condition, Box<Logical>),
    Or(Condition, Box<Logical>),
    Atom(Condition),
    None
}

impl Logical {
    fn new() -> Logical {
        None
    }

    pub fn cond(&self, Condition) -> {

    }
}

pub struct Query {
    pub select: Select,
    pub table: String,
    pub conditions: Logical,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn select_star() {
        let result = Select::new(Columns::Star).sql();
        assert_eq!(result, "SELECT *");
    }
}
