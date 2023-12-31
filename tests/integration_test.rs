use squeal::*;

#[test]
fn test_select() {
    let result = Select::new(Columns::Star).sql();
    assert_eq!(result, "SELECT *");
}

// Integration tests exercising the complicated functionality of the
// squeal library for the Query object
#[test]
fn test_complicated_query_builder() {
    let result = Query {
        select: Select::new(Columns::Selected(vec!["a", "b",])),
        from: "table",
        where_clause: Some(
            Term::Condition(
                Box::new(Term::Atom("a")),
                Op::O("<>"),
                Box::new(Term::Atom("b")),
            )
        ),
        group_by: Some(vec!["a", "b"]),
        having: Some(Having::new(
            Term::Condition(
                Box::new(Term::Atom("a")),
                Op::O("<>"),
                Box::new(Term::Atom("b")),
            )
        )),
        order_by: Some(OrderBy{columns: vec![ OrderedColumn::Asc("a"),
                                              OrderedColumn::Desc("b")]}),
        limit: Some(19),
        offset: Some(10),
    }.sql();
    assert_eq!(result, "SELECT a, b FROM table WHERE a <> b GROUP BY a, b HAVING a <> b ORDER BY a ASC, b DESC LIMIT 19 OFFSET 10");
}
