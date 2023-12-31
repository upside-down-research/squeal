use criterion::{black_box, criterion_group, criterion_main, Criterion};
use squeal::*;

fn generate() -> String {
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
        order_by: Some(OrderBy{columns: vec![ OrderedColumn::Asc("a"),
                                              OrderedColumn::Desc("b")]}),
        limit: Some(19),
        offset: Some(10),
        }.sql();
    result
}

fn criterion_benchmark(c: &mut Criterion) {
    c.bench_function("generate", |b| b.iter(|| { generate(); } ));
}

criterion_group!(benches, criterion_benchmark);
criterion_main!(benches);
