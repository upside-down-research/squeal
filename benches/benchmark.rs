use criterion::{Criterion, criterion_group, criterion_main};
use squeal::*;

fn generate() -> String {
    Query {
        with_clause: None,
        select: Some(Select::new(Columns::Selected(vec!["a", "b"]), None)),
        from: Some(FromSource::Table("table")),
        joins: vec![],
        where_clause: Some(Term::Condition(
            Box::new(Term::Atom("a")),
            Op::O("<>"),
            Box::new(Term::Atom("b")),
        )),
        group_by: Some(vec!["a", "b"]),
        having: Some(Having::new(Term::Condition(
            Box::new(Term::Atom("a")),
            Op::O("<>"),
            Box::new(Term::Atom("b")),
        ))),
        order_by: Some(OrderBy {
            columns: vec![OrderedColumn::Asc("a"), OrderedColumn::Desc("b")],
        }),
        limit: Some(19),
        offset: Some(10),
        for_update: false,
    }
    .sql()
}

fn fluent_generation() -> String {
    let mut q = Q();

    let result = q
        .select(vec!["a", "sum(b)"])
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
        .offset(10)
        .build();
    result.sql()
}

fn criterion_benchmark(c: &mut Criterion) {
    c.bench_function("generate", |b| {
        b.iter(|| {
            generate();
        })
    });
    c.bench_function("fluent generation", |b| {
        b.iter(|| {
            fluent_generation();
        })
    });
}

criterion_group!(benches, criterion_benchmark);
criterion_main!(benches);
