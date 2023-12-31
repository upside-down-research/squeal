use squeal::*;
#[test]
fn test_select() {
    let result = Select::new(Columns::Star).sql();
    assert_eq!(result, "SELECT *");
}

// Integration tests exercizing the complicated functionality of the squeal library for the Query object
#[test]
fn test_complicated_query_builder() {
    let result = Query {
        select: Select::new(Columns::Selected(vec!["a".to_string(), "b".to_string()])),
        from: "table".to_string(),
        where_clause: Some(
            Term::Condition(
                Box::new(Term::Atom("a".to_string())),
                Op::O("<>".to_string()),
                Box::new(Term::Atom("b".to_string())),
            )
        ),
        group_by: Some(vec!["a".to_string(), "b".to_string()]),
        having: Some(Having::new(
            Term::Condition(
                Box::new(Term::Atom("a".to_string())),
                Op::O("<>".to_string()),
                Box::new(Term::Atom("b".to_string())),
            )
        )),
        order_by: Some(OrderBy{columns: vec![ OrderedColumn::Asc("a".to_string()),
                                              OrderedColumn::Desc("b".to_string())]}),
        limit: Some(19),
        offset: Some(10),
    }.sql();
    assert_eq!(result, "SELECT a, b FROM table WHERE a <> b GROUP BY a, b HAVING a <> b ORDER BY a ASC, b DESC LIMIT 19 OFFSET 10");
}