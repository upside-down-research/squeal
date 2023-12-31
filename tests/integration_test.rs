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
        select: Select::new(Columns::Selected(vec!["a", "b"])),
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
        order_by: Some(OrderBy {
            columns: vec![OrderedColumn::Asc("a"),
                          OrderedColumn::Desc("b")]
        }),
        limit: Some(19),
        offset: Some(10),
    }.sql();
    assert_eq!(result, "SELECT a, b FROM table WHERE a <> b GROUP BY a, b HAVING a <> b ORDER BY a ASC, b DESC LIMIT 19 OFFSET 10");
}

#[test]
fn test_fluent_query() {
    let mut q = Q("the table");

    let result = q.select(vec!["a", "sum(b)"])
        .from("the_table")
        .where_(Term::Condition(
            Box::new(Term::Atom("a")),
            Op::O("<>"),
            Box::new(Term::Atom("b")),
        ))
        .group_by(vec!["a"])
        .having(Term::Condition(
            Box::new(Term::Atom("a")),
            Op::O("<>"),
            Box::new(Term::Atom("b")),
        ))
        .order_by(vec![OrderedColumn::Asc("a")])
        .limit(19)
        .offset(10);

    let r = result.build();
    assert_eq!(r.sql(), "SELECT a, sum(b) FROM the_table WHERE a <> b GROUP BY a HAVING a <> b ORDER BY a ASC LIMIT 19 OFFSET 10");
}