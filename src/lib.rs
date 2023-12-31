
trait Sql {
    fn sql(&self) -> String;
}
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


}

impl Sql for Select {
    fn sql(&self) -> String {
        match &self.cols {
            Columns::Star => "SELECT *".to_string(),
            Columns::Selected(v) => format!("SELECT {}", v.join(", "))
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
            }.to_string()
    }
}
pub enum Term {
    Atom(String),     
    Condition(Box<Term>, Op, Box<Term>),
        
}
    
pub enum Logical {
    Term,
    Null
}   

impl Logical {
    fn new() -> Logical {
       Logical::Null 
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
