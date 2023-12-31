use criterion::{black_box, criterion_group, criterion_main, Criterion};
use squeal::*;

fn generate() -> String {
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
    result
}

fn criterion_benchmark(c: &mut Criterion) {
    c.bench_function("generate", |b| b.iter(|| { generate(); } ));
}

criterion_group!(benches, criterion_benchmark);
criterion_main!(benches);
