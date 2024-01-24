use testcontainers_modules::{postgres::Postgres, testcontainers::clients::Cli};
use testcontainers::{RunnableImage};
use squeal::*;

#[test]
fn test_select() {
    let result = Select::new(Columns::Star).sql();
    assert_eq!(result, "*");
}

// Integration tests exercising the complicated functionality of the
// squeal library for the Query object
#[test]
fn test_complicated_query_builder() {
    let result = Query {
        select: Some(Select::new(Columns::Selected(vec!["a", "b"]))),
        from: Some("table"),
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
        for_update: true,
    }.sql();
    assert_eq!(result, "SELECT a, b FROM table WHERE a <> b GROUP BY a, b HAVING a <> b ORDER BY a ASC, b DESC LIMIT 19 OFFSET 10 FOR UPDATE");
}

#[test]
fn test_fluent_query() {
    let mut q = Q();

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

#[test] 
fn test_fluent_update() {
    let mut u = U("the_table");

    let result = u.set(vec![("a", "b"), ("c", "d")])
        .where_(Term::Condition(
            Box::new(Term::Atom("a")),
            Op::O("<>"),
            Box::new(Term::Atom("b")),
        ))
        .returning(vec!["a", "b"]);

    let r = result.build();
    assert_eq!(r.sql(), "UPDATE the_table SET a = b, c = d WHERE a <> b RETURNING a, b");

}


struct DockerTests {
    cli: testcontainers::clients::Cli,
}

impl DockerTests {
    fn new() -> DockerTests{
        let cli = Cli::default();

        let result = DockerTests {
            cli: cli,

        };
        result
    }
    fn get_new_node_and_connection(&mut self) -> (testcontainers::Container<Postgres>, postgres::Client) {
        let image = RunnableImage::from(Postgres::default()).with_tag("13.3-alpine");

        let node = self.cli.run(image);
        // prepare connection string
        let connection_string = &format!(
            "postgres://postgres:postgres@localhost:{}/postgres",
            node.get_host_port_ipv4(5432)
        );
        // container is up, you can use it
        let conn = postgres::Client::connect(connection_string, postgres::NoTls).unwrap();

        (node, conn)
    }
}

#[test]
#[cfg_attr(not(feature = "postgres-docker"), ignore)]
fn verify_postgres() -> Result<(), String> {
    let mut harness = DockerTests::new();
    let (node, mut conn) = harness.get_new_node_and_connection();
    let rows = conn.query("SELECT 1 + 1", &[]).unwrap();
    assert_eq!(rows.len(), 1);

    let first_row = &rows[0];
    let first_column: i32 = first_row.get(0);
    assert_eq!(first_column, 2);

    println!("{}", node.id());
    Ok(())
}

#[test]
#[cfg_attr(not(feature = "postgres-docker"), ignore)]
fn simple_query() -> Result<(), String> {
    let mut harness = DockerTests::new();
    let (node, mut conn) = harness.get_new_node_and_connection();

    let result = Q().select( vec!["1 + 1"]).build().sql();
    let rows = conn.query(&result, &[]).unwrap();
    assert_eq!(rows.len(), 1);

    let first_row = &rows[0];
    let first_column: i32 = first_row.get(0);
    assert_eq!(first_column, 2);

    println!("{}", node.id());
    Ok(())
}

#[test]
#[cfg_attr(not(feature = "postgres-docker"), ignore)]
fn create_and_drop_table() -> Result<(), String> {
    let mut harness = DockerTests::new();
    let (node, mut conn) = harness.get_new_node_and_connection();

    let result = T("test_table")
        .column("id", "serial", vec![])
        .column("name", "text", vec![])
        .build_create_table().sql();
    let code = conn.execute(&result, &[]).unwrap();
    assert_eq!(code, 0);


    let result = T("test_table").build_drop_table().sql();
    let code = conn.execute(&result, &[]).unwrap();
    assert_eq!(code, 0);

    println!("{}", node.id());
    Ok(())
}

use std::hash::{Hash, Hasher};
use std::collections::hash_map::DefaultHasher;
use std::time::{SystemTime, UNIX_EPOCH};

fn generate_random_string(len: usize) -> String {
    let mut hasher = DefaultHasher::new();
    SystemTime::now().duration_since(UNIX_EPOCH).unwrap().hash(&mut hasher);
    let hash = hasher.finish();

    let characters: Vec<char> = "ABCDEFGHIJKLMNOPQRSTUVWXYZabcdefghijklmnopqrstuvwxyz0123456789".chars().collect();
    let mut random_string = String::new();
    let string_length = len;

    for _ in 0..string_length {
        let num = hash as usize;
        let index = num % characters.len();
        random_string.push(characters[index]);
    }

    random_string
}

#[test]
#[cfg_attr(not(feature = "postgres-docker"), ignore)]
fn create_table_insert_data_query_it() -> Result<(), String> {
    let mut harness = DockerTests::new();
    let (_, mut conn) = harness.get_new_node_and_connection();

    // randomly generated tablename
    let tablename = format!("test_table_{}",generate_random_string(8));

    let result = T(&tablename)
        .column("id", "serial", vec![])
        .column("name", "text", vec![])
        .build_create_table().sql();
    let code = conn.execute(&result, &[]).unwrap();
    assert_eq!(code, 0);

    let result = I(&tablename)
        .columns(vec!["name"])
        .values(vec!["'test'"])
        .build().sql();
    let code = conn.execute(&result, &[]).unwrap();
    assert_eq!(code, 1);

    let result = Q().select( vec!["name"]).from(&tablename).build().sql();
    let rows = conn.query(&result, &[]).unwrap();
    assert_eq!(rows.len(), 1);

    for row in &rows {
        let first_column: String = row.get(0);
        assert_eq!(first_column, "test");
    }

    let result = T(&tablename).build_drop_table().sql();
    let code = conn.execute(&result, &[]).unwrap();
    assert_eq!(code, 0);

    Ok(())
}