use std::collections::hash_map::DefaultHasher;
use std::hash::{Hash, Hasher};
use std::time::{SystemTime, UNIX_EPOCH};

use testcontainers::RunnableImage;
use testcontainers_modules::{postgres::Postgres, testcontainers::clients::Cli};

use squeal::*;

#[test]
fn test_select() {
    let result = Select::new(Columns::Star, None).sql();
    assert_eq!(result, "*");
}

// Integration tests exercising the complicated functionality of the
// squeal library for the Query object
#[test]
fn test_complicated_query_builder() {
    let result = Query {
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
        for_update: true,
    }
    .sql();
    assert_eq!(result, "SELECT a, b FROM table WHERE a <> b GROUP BY a, b HAVING a <> b ORDER BY a ASC, b DESC LIMIT 19 OFFSET 10 FOR UPDATE");
}

#[test]
fn test_fluent_query() {
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
        .offset(10);

    let r = result.build();
    assert_eq!(r.sql(), "SELECT a, sum(b) FROM the_table WHERE a <> b GROUP BY a HAVING a <> b ORDER BY a ASC LIMIT 19 OFFSET 10");
}

#[test]
fn test_fluent_update() {
    let mut u = U("table_table");
    let result = u
        .columns(vec!["last_read", "last_write"])
        .values(vec!["now()", "now()"])
        .where_(Term::Condition(
            Box::new(Term::Atom("id")),
            Op::Equals,
            Box::new(Term::Atom("100")),
        ))
        .build()
        .sql();

    assert_eq!(
        result,
        "UPDATE table_table SET last_read = now(), last_write = now() WHERE id = 100"
    );
}

#[test]
fn test_fluent_delete() {
    let mut d = D("table_table");
    let result = d
        .where_(Term::Condition(
            Box::new(Term::Atom("id")),
            Op::Equals,
            Box::new(Term::Atom("100")),
        ))
        .build()
        .sql();

    assert_eq!(result, "DELETE FROM table_table WHERE id = 100");
}

struct DockerTests {
    cli: testcontainers::clients::Cli,
}

impl DockerTests {
    fn new() -> DockerTests {
        let cli = Cli::default();

        DockerTests { cli }
    }
    fn get_new_node_and_connection(
        &mut self,
    ) -> (testcontainers::Container<'_, Postgres>, postgres::Client) {
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

    let result = Q().select(vec!["1 + 1"]).build().sql();
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
        .build_create_table()
        .sql();
    let code = conn.execute(&result, &[]).unwrap();
    assert_eq!(code, 0);

    let result = T("test_table").build_drop_table().sql();
    let code = conn.execute(&result, &[]).unwrap();
    assert_eq!(code, 0);

    println!("{}", node.id());
    Ok(())
}

fn generate_random_string(len: usize) -> String {
    let mut hasher = DefaultHasher::new();
    SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap()
        .hash(&mut hasher);
    let hash = hasher.finish();

    let characters: Vec<char> = "ABCDEFGHIJKLMNOPQRSTUVWXYZabcdefghijklmnopqrstuvwxyz0123456789"
        .chars()
        .collect();
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
    // Note: node must be kept alive to prevent Docker container cleanup
    let (_node, mut conn) = harness.get_new_node_and_connection();

    // randomly generated tablename
    let tablename = format!("test_table_{}", generate_random_string(8));

    let result = T(&tablename)
        .column("id", "serial", vec![])
        .column("name", "text", vec![])
        .build_create_table()
        .sql();
    let code = conn.execute(&result, &[]).unwrap();
    assert_eq!(code, 0);

    let result = I(&tablename)
        .columns(vec!["name"])
        .values(vec!["'test'"])
        .build()
        .sql();
    let code = conn.execute(&result, &[]).unwrap();
    assert_eq!(code, 1);

    let result = Q().select(vec!["name"]).from(&tablename).build().sql();
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

#[test]
fn test_subquery_in_where_with_in() {
    let subquery = Query {
        with_clause: None,
        select: Some(Select::new(Columns::Selected(vec!["user_id"]), None)),
        from: Some(FromSource::Table("orders")),
        joins: vec![],
        where_clause: None,
        group_by: None,
        having: None,
        order_by: None,
        limit: None,
        offset: None,
        for_update: false,
    };
    let result = in_subquery("id", subquery).sql();
    assert_eq!(result, "id IN (SELECT user_id FROM orders)");
}

#[test]
fn test_subquery_in_where_direct() {
    let subquery = Query {
        with_clause: None,
        select: Some(Select::new(Columns::Selected(vec!["user_id"]), None)),
        from: Some(FromSource::Table("orders")),
        joins: vec![],
        where_clause: None,
        group_by: None,
        having: None,
        order_by: None,
        limit: None,
        offset: None,
        for_update: false,
    };
    let result = Term::Subquery(Box::new(subquery)).sql();
    assert_eq!(result, "(SELECT user_id FROM orders)");
}

#[test]
fn test_exists_operator() {
    let subquery = Query {
        with_clause: None,
        select: Some(Select::new(Columns::Selected(vec!["1"]), None)),
        from: Some(FromSource::Table("orders")),
        joins: vec![],
        where_clause: Some(eq("orders.user_id", "users.id")),
        group_by: None,
        having: None,
        order_by: None,
        limit: None,
        offset: None,
        for_update: false,
    };
    let result = exists(subquery).sql();
    assert_eq!(
        result,
        "EXISTS (SELECT 1 FROM orders WHERE orders.user_id = users.id)"
    );
}

#[test]
fn test_not_exists_operator() {
    let subquery = Query {
        with_clause: None,
        select: Some(Select::new(Columns::Selected(vec!["1"]), None)),
        from: Some(FromSource::Table("banned_users")),
        joins: vec![],
        where_clause: Some(eq("banned_users.id", "users.id")),
        group_by: None,
        having: None,
        order_by: None,
        limit: None,
        offset: None,
        for_update: false,
    };
    let result = not_exists(subquery).sql();
    assert_eq!(
        result,
        "NOT EXISTS (SELECT 1 FROM banned_users WHERE banned_users.id = users.id)"
    );
}

#[test]
fn test_any_operator() {
    let subquery = Query {
        with_clause: None,
        select: Some(Select::new(Columns::Selected(vec!["price"]), None)),
        from: Some(FromSource::Table("competitor_prices")),
        joins: vec![],
        where_clause: None,
        group_by: None,
        having: None,
        order_by: None,
        limit: None,
        offset: None,
        for_update: false,
    };
    let result = any("our_price", Op::LessThan, subquery).sql();
    assert_eq!(
        result,
        "our_price < ANY (SELECT price FROM competitor_prices)"
    );
}

#[test]
fn test_all_operator() {
    let subquery = Query {
        with_clause: None,
        select: Some(Select::new(Columns::Selected(vec!["price"]), None)),
        from: Some(FromSource::Table("competitor_prices")),
        joins: vec![],
        where_clause: None,
        group_by: None,
        having: None,
        order_by: None,
        limit: None,
        offset: None,
        for_update: false,
    };
    let result = all("our_price", Op::LessThan, subquery).sql();
    assert_eq!(
        result,
        "our_price < ALL (SELECT price FROM competitor_prices)"
    );
}

#[test]
fn test_subquery_in_from_clause() {
    let subquery = Query {
        with_clause: None,
        select: Some(Select::new(Columns::Star, None)),
        from: Some(FromSource::Table("users")),
        joins: vec![],
        where_clause: Some(eq("active", "true")),
        group_by: None,
        having: None,
        order_by: None,
        limit: None,
        offset: None,
        for_update: false,
    };
    let result = FromSource::Subquery(Box::new(subquery), "active_users").sql();
    assert_eq!(
        result,
        "(SELECT * FROM users WHERE active = true) AS active_users"
    );
}

#[test]
fn test_subquery_in_from_with_builder() {
    let subquery = Query {
        with_clause: None,
        select: Some(Select::new(Columns::Star, None)),
        from: Some(FromSource::Table("users")),
        joins: vec![],
        where_clause: Some(eq("active", "true")),
        group_by: None,
        having: None,
        order_by: None,
        limit: None,
        offset: None,
        for_update: false,
    };
    let mut qb = Q();
    let result = qb
        .select(vec!["*"])
        .from_subquery(subquery, "active_users")
        .build()
        .sql();
    assert_eq!(
        result,
        "SELECT * FROM (SELECT * FROM users WHERE active = true) AS active_users"
    );
}

#[test]
fn test_subquery_in_select_clause() {
    let subquery = Query {
        with_clause: None,
        select: Some(Select::new(Columns::Selected(vec!["COUNT(*)"]), None)),
        from: Some(FromSource::Table("orders")),
        joins: vec![],
        where_clause: Some(eq("orders.user_id", "users.id")),
        group_by: None,
        having: None,
        order_by: None,
        limit: None,
        offset: None,
        for_update: false,
    };
    let expr = SelectExpression::Subquery(Box::new(subquery), Some("order_count"));
    assert_eq!(
        expr.sql(),
        "(SELECT COUNT(*) FROM orders WHERE orders.user_id = users.id) AS order_count"
    );
}

#[test]
fn test_subquery_in_select_clause_no_alias() {
    let subquery = Query {
        with_clause: None,
        select: Some(Select::new(Columns::Selected(vec!["COUNT(*)"]), None)),
        from: Some(FromSource::Table("orders")),
        joins: vec![],
        where_clause: None,
        group_by: None,
        having: None,
        order_by: None,
        limit: None,
        offset: None,
        for_update: false,
    };
    let expr = SelectExpression::Subquery(Box::new(subquery), None);
    assert_eq!(expr.sql(), "(SELECT COUNT(*) FROM orders)");
}

#[test]
fn test_full_query_with_select_subquery() {
    let subquery = Query {
        with_clause: None,
        select: Some(Select::new(Columns::Selected(vec!["COUNT(*)"]), None)),
        from: Some(FromSource::Table("orders")),
        joins: vec![],
        where_clause: Some(eq("orders.user_id", "users.id")),
        group_by: None,
        having: None,
        order_by: None,
        limit: None,
        offset: None,
        for_update: false,
    };
    let mut qb = Q();
    let result = qb
        .select_expressions(vec![
            SelectExpression::Column("id"),
            SelectExpression::Column("name"),
            SelectExpression::Subquery(Box::new(subquery), Some("order_count")),
        ])
        .from("users")
        .build()
        .sql();
    assert_eq!(result, "SELECT id, name, (SELECT COUNT(*) FROM orders WHERE orders.user_id = users.id) AS order_count FROM users");
}

#[test]
fn test_complex_nested_query() {
    let exists_subquery = Query {
        with_clause: None,
        select: Some(Select::new(Columns::Selected(vec!["1"]), None)),
        from: Some(FromSource::Table("orders")),
        joins: vec![],
        where_clause: Some(eq("orders.user_id", "u.id")),
        group_by: None,
        having: None,
        order_by: None,
        limit: None,
        offset: None,
        for_update: false,
    };

    let from_subquery = Query {
        with_clause: None,
        select: Some(Select::new(Columns::Star, None)),
        from: Some(FromSource::Table("users")),
        joins: vec![],
        where_clause: Some(eq("active", "true")),
        group_by: None,
        having: None,
        order_by: None,
        limit: None,
        offset: None,
        for_update: false,
    };

    let mut qb = Q();
    let result = qb
        .select(vec!["*"])
        .from_subquery(from_subquery, "u")
        .where_(exists(exists_subquery))
        .build()
        .sql();
    assert_eq!(result, "SELECT * FROM (SELECT * FROM users WHERE active = true) AS u WHERE EXISTS (SELECT 1 FROM orders WHERE orders.user_id = u.id)");
}

#[test]
fn test_nested_subquery_in_where() {
    let inner_subquery = Query {
        with_clause: None,
        select: Some(Select::new(Columns::Selected(vec!["category_id"]), None)),
        from: Some(FromSource::Table("popular_categories")),
        joins: vec![],
        where_clause: None,
        group_by: None,
        having: None,
        order_by: None,
        limit: None,
        offset: None,
        for_update: false,
    };

    let outer_subquery = Query {
        with_clause: None,
        select: Some(Select::new(Columns::Selected(vec!["product_id"]), None)),
        from: Some(FromSource::Table("products")),
        joins: vec![],
        where_clause: Some(in_subquery("category_id", inner_subquery)),
        group_by: None,
        having: None,
        order_by: None,
        limit: None,
        offset: None,
        for_update: false,
    };

    let result = in_subquery("id", outer_subquery).sql();
    assert_eq!(result, "id IN (SELECT product_id FROM products WHERE category_id IN (SELECT category_id FROM popular_categories))");
}

#[test]
fn test_op_exists() {
    assert_eq!(Op::Exists.sql(), "EXISTS");
}

#[test]
fn test_op_not_exists() {
    assert_eq!(Op::NotExists.sql(), "NOT EXISTS");
}

#[test]
fn test_op_any() {
    assert_eq!(Op::Any.sql(), "ANY");
}

#[test]
fn test_op_all() {
    assert_eq!(Op::All.sql(), "ALL");
}

#[test]
fn test_coalesce() {
    let result = coalesce(vec![
        Term::Atom("col1"),
        Term::Atom("col2"),
        Term::Atom("'default'"),
    ])
    .sql();
    assert_eq!(result, "COALESCE(col1, col2, 'default')");
}

#[test]
fn test_nullif() {
    let result = nullif(Term::Atom("col1"), Term::Atom("col2")).sql();
    assert_eq!(result, "NULLIF(col1, col2)");
}

#[test]
fn test_concat() {
    let result = concat(vec![Term::Atom("col1"), Term::Atom("col2")]).sql();
    assert_eq!(result, "CONCAT(col1, col2)");
}

#[test]
fn test_substring() {
    let result = substring(
        Term::Atom("col1"),
        Some(Term::Atom("1")),
        Some(Term::Atom("3")),
    )
    .sql();
    assert_eq!(result, "SUBSTRING(col1 FROM 1 FOR 3)");
}

#[test]
fn test_upper() {
    let result = upper(Term::Atom("col1")).sql();
    assert_eq!(result, "UPPER(col1)");
}

#[test]
fn test_lower() {
    let result = lower(Term::Atom("col1")).sql();
    assert_eq!(result, "LOWER(col1)");
}

#[test]
fn test_now() {
    let result = now().sql();
    assert_eq!(result, "NOW()");
}

#[test]
fn test_current_date() {
    let result = current_date().sql();
    assert_eq!(result, "CURRENT_DATE");
}

#[test]
fn test_interval() {
    let result = interval("1 day").sql();
    assert_eq!(result, "INTERVAL '1 day'");
}

#[test]
fn test_date_add() {
    let result = date_add(Term::Atom("col1"), interval("1 day")).sql();
    assert_eq!(result, "col1 + INTERVAL '1 day'");
}

#[test]
fn test_date_sub() {
    let result = date_sub(Term::Atom("col1"), interval("1 day")).sql();
    assert_eq!(result, "col1 - INTERVAL '1 day'");
}

// Tests for Op enum variants
#[test]
fn test_op_not_equals() {
    assert_eq!(Op::NotEquals.sql(), "!=");
}

#[test]
fn test_op_greater_than() {
    assert_eq!(Op::GreaterThan.sql(), ">");
}

#[test]
fn test_op_less_than() {
    assert_eq!(Op::LessThan.sql(), "<");
}

#[test]
fn test_op_greater_or_equal() {
    assert_eq!(Op::GreaterOrEqual.sql(), ">=");
}

#[test]
fn test_op_less_or_equal() {
    assert_eq!(Op::LessOrEqual.sql(), "<=");
}

#[test]
fn test_op_like() {
    assert_eq!(Op::Like.sql(), "LIKE");
}

#[test]
fn test_op_in() {
    assert_eq!(Op::In.sql(), "IN");
}

#[test]
fn test_op_and() {
    assert_eq!(Op::And.sql(), "AND");
}

#[test]
fn test_op_or() {
    assert_eq!(Op::Or.sql(), "OR");
}

#[test]
fn test_op_equals() {
    assert_eq!(Op::Equals.sql(), "=");
}

#[test]
fn test_op_custom() {
    assert_eq!(Op::O("@@").sql(), "@@");
}

// Tests for helper functions
#[test]
fn test_ne_helper() {
    let result = ne("col1", "col2").sql();
    assert_eq!(result, "col1 != col2");
}

#[test]
fn test_gt_helper() {
    let result = gt("col1", "col2").sql();
    assert_eq!(result, "col1 > col2");
}

#[test]
fn test_lt_helper() {
    let result = lt("col1", "col2").sql();
    assert_eq!(result, "col1 < col2");
}

#[test]
fn test_gte_helper() {
    let result = gte("col1", "col2").sql();
    assert_eq!(result, "col1 >= col2");
}

#[test]
fn test_lte_helper() {
    let result = lte("col1", "col2").sql();
    assert_eq!(result, "col1 <= col2");
}

#[test]
fn test_like_helper() {
    let result = like("col1", "'%test%'").sql();
    assert_eq!(result, "col1 LIKE '%test%'");
}

#[test]
fn test_and_helper() {
    let result = and(eq("col1", "1"), eq("col2", "2")).sql();
    assert_eq!(result, "col1 = 1 AND col2 = 2");
}

#[test]
fn test_or_helper() {
    let result = or(eq("col1", "1"), eq("col2", "2")).sql();
    assert_eq!(result, "col1 = 1 OR col2 = 2");
}

#[test]
fn test_not_helper() {
    let result = not(eq("col1", "1")).sql();
    assert_eq!(result, "NOT col1 = 1");
}

#[test]
fn test_cast_helper() {
    let result = cast(Term::Atom("col1"), "integer").sql();
    assert_eq!(result, "CAST(col1 AS integer)");
}

#[test]
fn test_pg_cast_helper() {
    let result = pg_cast(Term::Atom("col1"), "integer").sql();
    assert_eq!(result, "col1::integer");
}

#[test]
fn test_case_helper() {
    let result = case(
        vec![
            WhenThen {
                when: eq("status", "'active'"),
                then: Term::Atom("1"),
            },
            WhenThen {
                when: eq("status", "'inactive'"),
                then: Term::Atom("0"),
            },
        ],
        Some(Term::Atom("-1")),
    )
    .sql();
    assert_eq!(
        result,
        "CASE WHEN status = 'active' THEN 1 WHEN status = 'inactive' THEN 0 ELSE -1 END"
    );
}

#[test]
fn test_case_helper_no_else() {
    let result = case(
        vec![WhenThen {
            when: eq("status", "'active'"),
            then: Term::Atom("1"),
        }],
        None,
    )
    .sql();
    assert_eq!(result, "CASE WHEN status = 'active' THEN 1 END");
}

#[test]
fn test_parens_helper() {
    let result = parens(eq("col1", "1")).sql();
    assert_eq!(result, "(col1 = 1)");
}

#[test]
fn test_in_helper() {
    let result = in_("status", vec!["'active'", "'pending'"]).sql();
    assert_eq!(result, "status IN ('active', 'pending')");
}

#[test]
fn test_between_helper() {
    let result = between("age", "18", "65").sql();
    assert_eq!(result, "age BETWEEN 18 AND 65");
}

#[test]
fn test_is_null_helper() {
    let result = is_null("deleted_at").sql();
    assert_eq!(result, "deleted_at IS NULL");
}

#[test]
fn test_is_not_null_helper() {
    let result = is_not_null("created_at").sql();
    assert_eq!(result, "created_at IS NOT NULL");
}

// Tests for PgParams
#[test]
fn test_pg_params_new_and_seq() {
    let mut pg = PgParams::new();
    assert_eq!(pg.seq(), "$1");
    assert_eq!(pg.seq(), "$2");
    assert_eq!(pg.seq(), "$3");
}

#[test]
fn test_pg_params_default() {
    let mut pg = PgParams::default();
    assert_eq!(pg.seq(), "$1");
}

#[test]
fn test_p_function() {
    assert_eq!(p(1), "$1");
    assert_eq!(p(2), "$2");
    assert_eq!(p(100), "$100");
}

// Tests for QueryBuilder methods
#[test]
fn test_query_builder_distinct() {
    let mut qb = Q();
    let query = qb
        .select(vec!["name", "email"])
        .from("users")
        .distinct()
        .build();
    assert_eq!(query.sql(), "SELECT DISTINCT name, email FROM users");
}

#[test]
fn test_query_builder_distinct_on() {
    let mut qb = Q();
    let query = qb
        .select(vec!["name", "email", "created_at"])
        .from("users")
        .distinct_on(vec!["name"])
        .build();
    assert_eq!(
        query.sql(),
        "SELECT DISTINCT ON (name) name, email, created_at FROM users"
    );
}

#[test]
fn test_query_builder_where_opt_some() {
    let mut qb = Q();
    let query = qb
        .select(vec!["*"])
        .from("users")
        .where_opt(Some(eq("active", "true")))
        .build();
    assert_eq!(query.sql(), "SELECT * FROM users WHERE active = true");
}

#[test]
fn test_query_builder_where_opt_none() {
    let mut qb = Q();
    let query = qb.select(vec!["*"]).from("users").where_opt(None).build();
    assert_eq!(query.sql(), "SELECT * FROM users");
}

#[test]
fn test_query_builder_and_where_first() {
    let mut qb = Q();
    let query = qb
        .select(vec!["*"])
        .from("users")
        .and_where(eq("active", "true"))
        .build();
    assert_eq!(query.sql(), "SELECT * FROM users WHERE active = true");
}

#[test]
fn test_query_builder_and_where_chained() {
    let mut qb = Q();
    let query = qb
        .select(vec!["*"])
        .from("users")
        .and_where(eq("active", "true"))
        .and_where(eq("verified", "true"))
        .build();
    assert_eq!(
        query.sql(),
        "SELECT * FROM users WHERE active = true AND verified = true"
    );
}

#[test]
fn test_query_builder_param() {
    let mut qb = Q();
    let p1 = qb.param();
    let p2 = qb.param();
    let query = qb
        .select(vec!["*"])
        .from("users")
        .where_(and(eq("id", &p1), eq("status", &p2)))
        .build();
    assert_eq!(
        query.sql(),
        "SELECT * FROM users WHERE id = $1 AND status = $2"
    );
}

// Tests for Insert
#[test]
fn test_insert_direct() {
    let insert = Insert {
        table: "users",
        columns: vec!["name", "email"],
        source: InsertSource::Values(vec![vec!["'John'", "'john@example.com'"]]),
        on_conflict: None,
        returning: None,
    };
    assert_eq!(
        insert.sql(),
        "INSERT INTO users (name, email) VALUES ('John', 'john@example.com')"
    );
}

#[test]
fn test_insert_with_returning() {
    let insert = Insert {
        table: "users",
        columns: vec!["name"],
        source: InsertSource::Values(vec![vec!["'Alice'"]]),
        on_conflict: None,
        returning: Some(Columns::Star),
    };
    assert_eq!(
        insert.sql(),
        "INSERT INTO users (name) VALUES ('Alice') RETURNING *"
    );
}

#[test]
fn test_insert_with_returning_columns() {
    let insert = Insert {
        table: "users",
        columns: vec!["name"],
        source: InsertSource::Values(vec![vec!["'Bob'"]]),
        on_conflict: None,
        returning: Some(Columns::Selected(vec!["id", "name"])),
    };
    assert_eq!(
        insert.sql(),
        "INSERT INTO users (name) VALUES ('Bob') RETURNING id, name"
    );
}

#[test]
fn test_insert_builder_basic() {
    let mut ib = I("users");
    let insert = ib.columns(vec!["name"]).values(vec!["'Charlie'"]).build();
    assert_eq!(insert.sql(), "INSERT INTO users (name) VALUES ('Charlie')");
}

#[test]
fn test_insert_builder_returning() {
    let mut ib = I("users");
    let insert = ib
        .columns(vec!["name"])
        .values(vec!["'David'"])
        .returning(Columns::Star)
        .build();
    assert_eq!(
        insert.sql(),
        "INSERT INTO users (name) VALUES ('David') RETURNING *"
    );
}

#[test]
fn test_insert_builder_param() {
    let mut ib = I("users");
    let p1 = ib.param();
    let p2 = ib.param();
    let insert = ib
        .columns(vec!["name", "email"])
        .values(vec![&p1, &p2])
        .build();
    assert_eq!(
        insert.sql(),
        "INSERT INTO users (name, email) VALUES ($1, $2)"
    );
}

#[test]
fn test_insert_select_direct() {
    let select_query = Query {
        with_clause: None,
        select: Some(Select::new(Columns::Selected(vec!["name", "email"]), None)),
        from: Some(FromSource::Table("active_users")),
        joins: vec![],
        where_clause: Some(eq("status", "'active'")),
        group_by: None,
        having: None,
        order_by: None,
        limit: None,
        offset: None,
        for_update: false,
    };
    let insert = Insert {
        table: "archived_users",
        columns: vec!["name", "email"],
        source: InsertSource::Select(Box::new(select_query)),
        on_conflict: None,
        returning: None,
    };
    assert_eq!(insert.sql(), "INSERT INTO archived_users (name, email) SELECT name, email FROM active_users WHERE status = 'active'");
}

#[test]
fn test_insert_select_builder() {
    let select_query = Query {
        with_clause: None,
        select: Some(Select::new(Columns::Star, None)),
        from: Some(FromSource::Table("old_data")),
        joins: vec![],
        where_clause: Some(eq("archived", "true")),
        group_by: None,
        having: None,
        order_by: None,
        limit: Some(100),
        offset: None,
        for_update: false,
    };
    let mut ib = I("archive");
    let insert = ib.columns(vec!["*"]).select(select_query).build();
    assert_eq!(
        insert.sql(),
        "INSERT INTO archive (*) SELECT * FROM old_data WHERE archived = true LIMIT 100"
    );
}

#[test]
fn test_insert_select_with_returning() {
    let select_query = Query {
        with_clause: None,
        select: Some(Select::new(
            Columns::Selected(vec!["user_id", "amount"]),
            None,
        )),
        from: Some(FromSource::Table("pending_transactions")),
        joins: vec![],
        where_clause: None,
        group_by: None,
        having: None,
        order_by: None,
        limit: None,
        offset: None,
        for_update: false,
    };
    let mut ib = I("completed_transactions");
    let insert = ib
        .columns(vec!["user_id", "amount"])
        .select(select_query)
        .returning(Columns::Selected(vec!["id", "user_id"]))
        .build();
    assert_eq!(insert.sql(), "INSERT INTO completed_transactions (user_id, amount) SELECT user_id, amount FROM pending_transactions RETURNING id, user_id");
}

// Tests for CreateTable and DropTable
#[test]
fn test_create_table_direct() {
    let create = CreateTable {
        table: "users",
        columns: vec!["id serial PRIMARY KEY".to_string(), "name text".to_string()],
    };
    assert_eq!(
        create.sql(),
        "CREATE TABLE users (id serial PRIMARY KEY, name text)"
    );
}

#[test]
fn test_create_table_builder() {
    let mut tb = T("users");
    let create = tb
        .column("id", "serial", vec!["PRIMARY KEY"])
        .column("name", "text", vec![])
        .build_create_table();
    assert_eq!(
        create.sql(),
        "CREATE TABLE users (id serial PRIMARY KEY, name text)"
    );
}

#[test]
fn test_create_table_builder_table_method() {
    let mut tb = T("old_name");
    tb.table("new_name");
    let create = tb.build_create_table();
    assert_eq!(create.sql(), "CREATE TABLE new_name ()");
}

#[test]
fn test_drop_table() {
    let drop = DropTable { table: "users" };
    assert_eq!(drop.sql(), "DROP TABLE users");
}

#[test]
fn test_drop_table_builder() {
    let tb = T("users");
    let drop = tb.build_drop_table();
    assert_eq!(drop.sql(), "DROP TABLE users");
}

// Tests for Delete
#[test]
fn test_delete_direct() {
    let delete = Delete {
        table: "users",
        where_clause: Some(eq("id", "10")),
        returning: None,
    };
    assert_eq!(delete.sql(), "DELETE FROM users WHERE id = 10");
}

#[test]
fn test_delete_no_where() {
    let delete = Delete {
        table: "users",
        where_clause: None,
        returning: None,
    };
    assert_eq!(delete.sql(), "DELETE FROM users");
}

#[test]
fn test_delete_with_returning() {
    let delete = Delete {
        table: "users",
        where_clause: Some(eq("id", "10")),
        returning: Some(Columns::Star),
    };
    assert_eq!(delete.sql(), "DELETE FROM users WHERE id = 10 RETURNING *");
}

#[test]
fn test_delete_with_returning_columns() {
    let delete = Delete {
        table: "users",
        where_clause: Some(eq("active", "false")),
        returning: Some(Columns::Selected(vec!["id", "name", "email"])),
    };
    assert_eq!(
        delete.sql(),
        "DELETE FROM users WHERE active = false RETURNING id, name, email"
    );
}

#[test]
fn test_delete_builder_returning() {
    let mut db = D("users");
    let delete = db.where_(eq("id", "10")).returning(Columns::Star).build();
    assert_eq!(delete.sql(), "DELETE FROM users WHERE id = 10 RETURNING *");
}

#[test]
fn test_delete_builder_param() {
    let mut db = D("users");
    let p1 = db.param();
    let delete = db.where_(eq("id", &p1)).build();
    assert_eq!(delete.sql(), "DELETE FROM users WHERE id = $1");
}

// Tests for Update
#[test]
fn test_update_direct() {
    let update = Update {
        table: "users",
        columns: vec!["name", "email"],
        values: vec!["'John'", "'john@example.com'"],
        from: None,
        where_clause: None,
        returning: None,
    };
    assert_eq!(
        update.sql(),
        "UPDATE users SET name = 'John', email = 'john@example.com'"
    );
}

#[test]
fn test_update_with_from() {
    let update = Update {
        table: "users",
        columns: vec!["active"],
        values: vec!["false"],
        from: Some("banned"),
        where_clause: Some(eq("users.id", "banned.user_id")),
        returning: None,
    };
    assert_eq!(
        update.sql(),
        "UPDATE users SET active = false FROM banned WHERE users.id = banned.user_id"
    );
}

#[test]
fn test_update_with_returning() {
    let update = Update {
        table: "users",
        columns: vec!["status"],
        values: vec!["'active'"],
        from: None,
        where_clause: None,
        returning: Some(Columns::Selected(vec!["id", "status"])),
    };
    assert_eq!(
        update.sql(),
        "UPDATE users SET status = 'active' RETURNING id, status"
    );
}

#[test]
fn test_update_builder_set() {
    let mut ub = U("users");
    let update = ub
        .set(vec![("name", "'Eve'"), ("email", "'eve@example.com'")])
        .build();
    assert_eq!(
        update.sql(),
        "UPDATE users SET name = 'Eve', email = 'eve@example.com'"
    );
}

#[test]
fn test_update_builder_from() {
    let mut ub = U("users");
    let update = ub
        .set(vec![("active", "false")])
        .from("banned")
        .where_(eq("users.id", "banned.user_id"))
        .build();
    assert_eq!(
        update.sql(),
        "UPDATE users SET active = false FROM banned WHERE users.id = banned.user_id"
    );
}

#[test]
fn test_update_builder_returning() {
    let mut ub = U("users");
    let update = ub
        .set(vec![("status", "'active'")])
        .returning(Columns::Star)
        .build();
    assert_eq!(
        update.sql(),
        "UPDATE users SET status = 'active' RETURNING *"
    );
}

#[test]
fn test_update_builder_param() {
    let mut ub = U("users");
    let p1 = ub.param();
    let p2 = ub.param();
    let update = ub.set(vec![("name", &p1)]).where_(eq("id", &p2)).build();
    assert_eq!(update.sql(), "UPDATE users SET name = $1 WHERE id = $2");
}

// Tests for SelectExpression::Column
#[test]
fn test_select_expression_column() {
    let expr = SelectExpression::Column("id");
    assert_eq!(expr.sql(), "id");
}

// Additional tests for 100% coverage

#[test]
fn test_term_null() {
    let result = Term::Null.sql();
    assert_eq!(result, "");
}

#[test]
fn test_case_multiple_when() {
    let case_expr = CaseExpression {
        when_thens: vec![
            WhenThen {
                when: eq("x", "1"),
                then: Term::Atom("'one'"),
            },
            WhenThen {
                when: eq("x", "2"),
                then: Term::Atom("'two'"),
            },
            WhenThen {
                when: eq("x", "3"),
                then: Term::Atom("'three'"),
            },
        ],
        else_term: Some(Box::new(Term::Atom("'other'"))),
    };
    // Test CaseExpression::sql() directly
    let result1 = case_expr.clone().sql();
    assert_eq!(
        result1,
        "CASE WHEN x = 1 THEN 'one' WHEN x = 2 THEN 'two' WHEN x = 3 THEN 'three' ELSE 'other' END"
    );

    // Also test via Term::Case
    let result2 = Term::Case(case_expr).sql();
    assert_eq!(
        result2,
        "CASE WHEN x = 1 THEN 'one' WHEN x = 2 THEN 'two' WHEN x = 3 THEN 'three' ELSE 'other' END"
    );
}

#[test]
fn test_substring_from_only() {
    let result = substring(Term::Atom("col1"), Some(Term::Atom("5")), None).sql();
    assert_eq!(result, "SUBSTRING(col1 FROM 5)");
}

#[test]
fn test_substring_for_only() {
    let result = substring(Term::Atom("col1"), None, Some(Term::Atom("3"))).sql();
    assert_eq!(result, "SUBSTRING(col1 FOR 3)");
}

#[test]
fn test_substring_no_params() {
    let result = substring(Term::Atom("col1"), None, None).sql();
    assert_eq!(result, "SUBSTRING(col1)");
}

#[test]
fn test_create_table_multiple_columns() {
    let create = CreateTable {
        table: "users",
        columns: vec![
            "id serial PRIMARY KEY".to_string(),
            "name text NOT NULL".to_string(),
            "email text UNIQUE".to_string(),
        ],
    };
    assert_eq!(
        create.sql(),
        "CREATE TABLE users (id serial PRIMARY KEY, name text NOT NULL, email text UNIQUE)"
    );
}

#[test]
fn test_create_table_builder_multiple_columns() {
    let mut tb = T("products");
    let create = tb
        .column("id", "serial", vec!["PRIMARY KEY"])
        .column("name", "text", vec!["NOT NULL"])
        .column("price", "numeric", vec!["DEFAULT", "0"])
        .build_create_table();
    assert_eq!(create.sql(), "CREATE TABLE products (id serial PRIMARY KEY, name text NOT NULL, price numeric DEFAULT 0)");
}

#[test]
fn test_insert_multiple_columns_direct() {
    let insert = Insert {
        table: "users",
        columns: vec!["name", "email", "age"],
        source: InsertSource::Values(vec![vec!["'John'", "'john@example.com'", "30"]]),
        on_conflict: None,
        returning: None,
    };
    assert_eq!(
        insert.sql(),
        "INSERT INTO users (name, email, age) VALUES ('John', 'john@example.com', 30)"
    );
}

#[test]
fn test_insert_multiple_rows_direct() {
    let insert = Insert {
        table: "users",
        columns: vec!["name", "age"],
        source: InsertSource::Values(vec![
            vec!["'Alice'", "30"],
            vec!["'Bob'", "25"],
            vec!["'Charlie'", "35"],
        ]),
        on_conflict: None,
        returning: None,
    };
    assert_eq!(
        insert.sql(),
        "INSERT INTO users (name, age) VALUES ('Alice', 30), ('Bob', 25), ('Charlie', 35)"
    );
}

#[test]
fn test_insert_multiple_rows_builder() {
    let mut ib = I("products");
    let insert = ib
        .columns(vec!["name", "price"])
        .rows(vec![
            vec!["'Widget'", "9.99"],
            vec!["'Gadget'", "19.99"],
            vec!["'Doohickey'", "14.99"],
        ])
        .build();
    assert_eq!(insert.sql(), "INSERT INTO products (name, price) VALUES ('Widget', 9.99), ('Gadget', 19.99), ('Doohickey', 14.99)");
}

#[test]
fn test_update_columns_and_values() {
    let mut ub = U("users");
    let update = ub
        .columns(vec!["name", "email", "age"])
        .values(vec!["'Alice'", "'alice@example.com'", "25"])
        .build();
    assert_eq!(
        update.sql(),
        "UPDATE users SET name = 'Alice', email = 'alice@example.com', age = 25"
    );
}

#[test]
fn test_update_multiple_columns() {
    let update = Update {
        table: "users",
        columns: vec!["name", "email", "status"],
        values: vec!["'Bob'", "'bob@example.com'", "'active'"],
        from: None,
        where_clause: None,
        returning: None,
    };
    assert_eq!(
        update.sql(),
        "UPDATE users SET name = 'Bob', email = 'bob@example.com', status = 'active'"
    );
}

#[test]
fn test_query_empty() {
    let query = Query {
        with_clause: None,
        select: None,
        from: None,
        joins: vec![],
        where_clause: None,
        group_by: None,
        having: None,
        order_by: None,
        limit: None,
        offset: None,
        for_update: false,
    };
    assert_eq!(query.sql(), "");
}

#[test]
fn test_query_select_only() {
    let query = Query {
        with_clause: None,
        select: Some(Select::new(Columns::Star, None)),
        from: None,
        joins: vec![],
        where_clause: None,
        group_by: None,
        having: None,
        order_by: None,
        limit: None,
        offset: None,
        for_update: false,
    };
    assert_eq!(query.sql(), "SELECT *");
}

#[test]
fn test_query_from_only() {
    let query = Query {
        with_clause: None,
        select: None,
        from: Some(FromSource::Table("users")),
        joins: vec![],
        where_clause: None,
        group_by: None,
        having: None,
        order_by: None,
        limit: None,
        offset: None,
        for_update: false,
    };
    assert_eq!(query.sql(), " FROM users");
}

#[test]
fn test_query_where_only() {
    let query = Query {
        with_clause: None,
        select: None,
        from: None,
        joins: vec![],
        where_clause: Some(eq("active", "true")),
        group_by: None,
        having: None,
        order_by: None,
        limit: None,
        offset: None,
        for_update: false,
    };
    assert_eq!(query.sql(), " WHERE active = true");
}

#[test]
fn test_query_group_by_only() {
    let query = Query {
        with_clause: None,
        select: None,
        from: None,
        joins: vec![],
        where_clause: None,
        group_by: Some(vec!["category", "status"]),
        having: None,
        order_by: None,
        limit: None,
        offset: None,
        for_update: false,
    };
    assert_eq!(query.sql(), " GROUP BY category, status");
}

#[test]
fn test_query_having_only() {
    let query = Query {
        with_clause: None,
        select: None,
        from: None,
        joins: vec![],
        where_clause: None,
        group_by: None,
        having: Some(Having::new(gt("count", "5"))),
        order_by: None,
        limit: None,
        offset: None,
        for_update: false,
    };
    assert_eq!(query.sql(), " HAVING count > 5");
}

#[test]
fn test_query_order_by_only() {
    let query = Query {
        with_clause: None,
        select: None,
        from: None,
        joins: vec![],
        where_clause: None,
        group_by: None,
        having: None,
        order_by: Some(OrderBy {
            columns: vec![OrderedColumn::Desc("created_at")],
        }),
        limit: None,
        offset: None,
        for_update: false,
    };
    assert_eq!(query.sql(), " ORDER BY created_at DESC");
}

#[test]
fn test_query_limit_only() {
    let query = Query {
        with_clause: None,
        select: None,
        from: None,
        joins: vec![],
        where_clause: None,
        group_by: None,
        having: None,
        order_by: None,
        limit: Some(10),
        offset: None,
        for_update: false,
    };
    assert_eq!(query.sql(), " LIMIT 10");
}

#[test]
fn test_query_offset_only() {
    let query = Query {
        with_clause: None,
        select: None,
        from: None,
        joins: vec![],
        where_clause: None,
        group_by: None,
        having: None,
        order_by: None,
        limit: None,
        offset: Some(20),
        for_update: false,
    };
    assert_eq!(query.sql(), " OFFSET 20");
}

#[test]
fn test_query_for_update_only() {
    let query = Query {
        with_clause: None,
        select: None,
        from: None,
        joins: vec![],
        where_clause: None,
        group_by: None,
        having: None,
        order_by: None,
        limit: None,
        offset: None,
        for_update: true,
    };
    assert_eq!(query.sql(), " FOR UPDATE");
}

#[test]
fn test_order_by_multiple_columns() {
    let order_by = OrderBy {
        columns: vec![
            OrderedColumn::Asc("name"),
            OrderedColumn::Desc("created_at"),
            OrderedColumn::Asc("id"),
        ],
    };
    assert_eq!(order_by.sql(), "ORDER BY name ASC, created_at DESC, id ASC");
}

#[test]
fn test_query_builder_for_update() {
    let mut qb = Q();
    let query = qb.select(vec!["*"]).from("accounts").for_update().build();
    assert_eq!(query.sql(), "SELECT * FROM accounts FOR UPDATE");
}

#[test]
fn test_distinct_before_select() {
    let mut qb = Q();
    // Call distinct before select (should not affect query since select isn't set yet)
    let query = qb.distinct().select(vec!["name"]).from("users").build();
    // Since distinct was called before select, it should be applied
    assert_eq!(query.sql(), "SELECT name FROM users");
}

#[test]
fn test_distinct_on_before_select() {
    let mut qb = Q();
    // Call distinct_on before select (should not affect query since select isn't set yet)
    let query = qb
        .distinct_on(vec!["name"])
        .select(vec!["name", "email"])
        .from("users")
        .build();
    // Since distinct_on was called before select, it should be applied
    assert_eq!(query.sql(), "SELECT name, email FROM users");
}

#[test]
fn test_distinct_with_select() {
    let result = Select::new(Columns::Selected(vec!["name"]), Some(Distinct::All)).sql();
    assert_eq!(result, "DISTINCT name");
}

#[test]
fn test_distinct_on_with_select() {
    let result = Select::new(
        Columns::Selected(vec!["name", "email"]),
        Some(Distinct::On(vec!["name", "city"])),
    )
    .sql();
    assert_eq!(result, "DISTINCT ON (name, city) name, email");
}

// JOIN tests
#[test]
fn test_inner_join() {
    let mut qb = Q();
    let query = qb
        .select(vec!["users.name", "orders.total"])
        .from("users")
        .inner_join("orders", eq("users.id", "orders.user_id"))
        .build();
    assert_eq!(
        query.sql(),
        "SELECT users.name, orders.total FROM users INNER JOIN orders ON users.id = orders.user_id"
    );
}

#[test]
fn test_left_join() {
    let mut qb = Q();
    let query = qb
        .select(vec!["users.name", "orders.total"])
        .from("users")
        .left_join("orders", eq("users.id", "orders.user_id"))
        .build();
    assert_eq!(
        query.sql(),
        "SELECT users.name, orders.total FROM users LEFT JOIN orders ON users.id = orders.user_id"
    );
}

#[test]
fn test_right_join() {
    let mut qb = Q();
    let query = qb
        .select(vec!["users.name", "orders.total"])
        .from("users")
        .right_join("orders", eq("users.id", "orders.user_id"))
        .build();
    assert_eq!(
        query.sql(),
        "SELECT users.name, orders.total FROM users RIGHT JOIN orders ON users.id = orders.user_id"
    );
}

#[test]
fn test_full_join() {
    let mut qb = Q();
    let query = qb
        .select(vec!["users.name", "orders.total"])
        .from("users")
        .full_join("orders", eq("users.id", "orders.user_id"))
        .build();
    assert_eq!(
        query.sql(),
        "SELECT users.name, orders.total FROM users FULL JOIN orders ON users.id = orders.user_id"
    );
}

#[test]
fn test_cross_join() {
    let mut qb = Q();
    let query = qb
        .select(vec!["users.name", "colors.name"])
        .from("users")
        .cross_join("colors")
        .build();
    assert_eq!(
        query.sql(),
        "SELECT users.name, colors.name FROM users CROSS JOIN colors"
    );
}

#[test]
fn test_multiple_joins() {
    let mut qb = Q();
    let query = qb
        .select(vec!["users.name", "orders.total", "products.name"])
        .from("users")
        .inner_join("orders", eq("users.id", "orders.user_id"))
        .left_join("products", eq("orders.product_id", "products.id"))
        .build();
    assert_eq!(query.sql(), "SELECT users.name, orders.total, products.name FROM users INNER JOIN orders ON users.id = orders.user_id LEFT JOIN products ON orders.product_id = products.id");
}

#[test]
fn test_join_with_where_clause() {
    let mut qb = Q();
    let query = qb
        .select(vec!["users.name", "orders.total"])
        .from("users")
        .inner_join("orders", eq("users.id", "orders.user_id"))
        .where_(gt("orders.total", "100"))
        .build();
    assert_eq!(query.sql(), "SELECT users.name, orders.total FROM users INNER JOIN orders ON users.id = orders.user_id WHERE orders.total > 100");
}

#[test]
fn test_join_with_complex_on_condition() {
    let mut qb = Q();
    let query = qb
        .select(vec!["users.name", "orders.total"])
        .from("users")
        .inner_join(
            "orders",
            and(
                eq("users.id", "orders.user_id"),
                eq("orders.status", "'active'"),
            ),
        )
        .build();
    assert_eq!(query.sql(), "SELECT users.name, orders.total FROM users INNER JOIN orders ON users.id = orders.user_id AND orders.status = 'active'");
}

#[test]
fn test_join_with_order_by_and_limit() {
    let mut qb = Q();
    let query = qb
        .select(vec!["users.name", "orders.total"])
        .from("users")
        .left_join("orders", eq("users.id", "orders.user_id"))
        .order_by(vec![OrderedColumn::Desc("orders.total")])
        .limit(10)
        .build();
    assert_eq!(query.sql(), "SELECT users.name, orders.total FROM users LEFT JOIN orders ON users.id = orders.user_id ORDER BY orders.total DESC LIMIT 10");
}

#[test]
fn test_join_type_sql() {
    assert_eq!(JoinType::Inner.sql(), "INNER JOIN");
    assert_eq!(JoinType::Left.sql(), "LEFT JOIN");
    assert_eq!(JoinType::Right.sql(), "RIGHT JOIN");
    assert_eq!(JoinType::Full.sql(), "FULL JOIN");
    assert_eq!(JoinType::Cross.sql(), "CROSS JOIN");
}

#[test]
fn test_join_struct_with_table() {
    let join = Join {
        join_type: JoinType::Inner,
        source: FromSource::Table("orders"),
        on: Some(eq("users.id", "orders.user_id")),
    };
    assert_eq!(join.sql(), "INNER JOIN orders ON users.id = orders.user_id");
}

#[test]
fn test_join_struct_cross_join_no_on() {
    let join = Join {
        join_type: JoinType::Cross,
        source: FromSource::Table("colors"),
        on: None,
    };
    assert_eq!(join.sql(), "CROSS JOIN colors");
}

#[test]
fn test_join_subquery() {
    let subquery = Query {
        with_clause: None,
        select: Some(Select::new(
            Columns::Selected(vec!["user_id", "COUNT(*) as order_count"]),
            None,
        )),
        from: Some(FromSource::Table("orders")),
        joins: vec![],
        where_clause: None,
        group_by: Some(vec!["user_id"]),
        having: None,
        order_by: None,
        limit: None,
        offset: None,
        for_update: false,
    };
    let mut qb = Q();
    let query = qb
        .select(vec!["users.name", "oc.order_count"])
        .from("users")
        .join_subquery(JoinType::Left, subquery, "oc", eq("users.id", "oc.user_id"))
        .build();
    assert_eq!(query.sql(), "SELECT users.name, oc.order_count FROM users LEFT JOIN (SELECT user_id, COUNT(*) as order_count FROM orders GROUP BY user_id) AS oc ON users.id = oc.user_id");
}

#[test]
fn test_join_with_group_by_and_having() {
    let mut qb = Q();
    let query = qb
        .select(vec!["users.name", "COUNT(orders.id) as order_count"])
        .from("users")
        .left_join("orders", eq("users.id", "orders.user_id"))
        .group_by(vec!["users.name"])
        .having(gt("COUNT(orders.id)", "5"))
        .build();
    assert_eq!(query.sql(), "SELECT users.name, COUNT(orders.id) as order_count FROM users LEFT JOIN orders ON users.id = orders.user_id GROUP BY users.name HAVING COUNT(orders.id) > 5");
}

#[test]
fn test_direct_join_struct_construction() {
    let query = Query {
        with_clause: None,
        select: Some(Select::new(
            Columns::Selected(vec!["users.name", "orders.total"]),
            None,
        )),
        from: Some(FromSource::Table("users")),
        joins: vec![Join {
            join_type: JoinType::Inner,
            source: FromSource::Table("orders"),
            on: Some(eq("users.id", "orders.user_id")),
        }],
        where_clause: None,
        group_by: None,
        having: None,
        order_by: None,
        limit: None,
        offset: None,
        for_update: false,
    };
    assert_eq!(
        query.sql(),
        "SELECT users.name, orders.total FROM users INNER JOIN orders ON users.id = orders.user_id"
    );
}

// CTE / WITH clause tests
#[test]
fn test_simple_cte() {
    let cte_query = Query {
        with_clause: None,
        select: Some(Select::new(Columns::Selected(vec!["id", "name"]), None)),
        from: Some(FromSource::Table("users")),
        joins: vec![],
        where_clause: Some(eq("active", "true")),
        group_by: None,
        having: None,
        order_by: None,
        limit: None,
        offset: None,
        for_update: false,
    };
    let mut qb = Q();
    let query = qb
        .with("active_users", cte_query)
        .select(vec!["*"])
        .from("active_users")
        .build();
    assert_eq!(query.sql(), "WITH active_users AS (SELECT id, name FROM users WHERE active = true) SELECT * FROM active_users");
}

#[test]
fn test_multiple_ctes() {
    let cte1 = Query {
        with_clause: None,
        select: Some(Select::new(Columns::Selected(vec!["id", "name"]), None)),
        from: Some(FromSource::Table("users")),
        joins: vec![],
        where_clause: Some(eq("active", "true")),
        group_by: None,
        having: None,
        order_by: None,
        limit: None,
        offset: None,
        for_update: false,
    };
    let cte2 = Query {
        with_clause: None,
        select: Some(Select::new(
            Columns::Selected(vec!["user_id", "COUNT(*) as order_count"]),
            None,
        )),
        from: Some(FromSource::Table("orders")),
        joins: vec![],
        where_clause: None,
        group_by: Some(vec!["user_id"]),
        having: None,
        order_by: None,
        limit: None,
        offset: None,
        for_update: false,
    };
    let mut qb = Q();
    let query = qb
        .with("active_users", cte1)
        .with("user_orders", cte2)
        .select(vec!["au.name", "uo.order_count"])
        .from("active_users au")
        .inner_join("user_orders uo", eq("au.id", "uo.user_id"))
        .build();
    assert_eq!(query.sql(), "WITH active_users AS (SELECT id, name FROM users WHERE active = true), user_orders AS (SELECT user_id, COUNT(*) as order_count FROM orders GROUP BY user_id) SELECT au.name, uo.order_count FROM active_users au INNER JOIN user_orders uo ON au.id = uo.user_id");
}

#[test]
fn test_cte_with_join() {
    let cte_query = Query {
        with_clause: None,
        select: Some(Select::new(Columns::Selected(vec!["id", "name"]), None)),
        from: Some(FromSource::Table("users")),
        joins: vec![],
        where_clause: Some(gt("created_at", "'2023-01-01'")),
        group_by: None,
        having: None,
        order_by: None,
        limit: None,
        offset: None,
        for_update: false,
    };
    let mut qb = Q();
    let query = qb
        .with("recent_users", cte_query)
        .select(vec!["ru.name", "orders.total"])
        .from("recent_users ru")
        .left_join("orders", eq("ru.id", "orders.user_id"))
        .build();
    assert_eq!(query.sql(), "WITH recent_users AS (SELECT id, name FROM users WHERE created_at > '2023-01-01') SELECT ru.name, orders.total FROM recent_users ru LEFT JOIN orders ON ru.id = orders.user_id");
}

#[test]
fn test_cte_with_where_clause() {
    let cte_query = Query {
        with_clause: None,
        select: Some(Select::new(
            Columns::Selected(vec!["category", "SUM(amount) as total"]),
            None,
        )),
        from: Some(FromSource::Table("transactions")),
        joins: vec![],
        where_clause: None,
        group_by: Some(vec!["category"]),
        having: None,
        order_by: None,
        limit: None,
        offset: None,
        for_update: false,
    };
    let mut qb = Q();
    let query = qb
        .with("category_totals", cte_query)
        .select(vec!["*"])
        .from("category_totals")
        .where_(gt("total", "1000"))
        .build();
    assert_eq!(query.sql(), "WITH category_totals AS (SELECT category, SUM(amount) as total FROM transactions GROUP BY category) SELECT * FROM category_totals WHERE total > 1000");
}

#[test]
fn test_cte_struct_sql() {
    let cte_query = Query {
        with_clause: None,
        select: Some(Select::new(Columns::Selected(vec!["id"]), None)),
        from: Some(FromSource::Table("users")),
        joins: vec![],
        where_clause: None,
        group_by: None,
        having: None,
        order_by: None,
        limit: None,
        offset: None,
        for_update: false,
    };
    let cte = Cte {
        name: "my_cte",
        query: Box::new(cte_query),
    };
    assert_eq!(cte.sql(), "my_cte AS (SELECT id FROM users)");
}

#[test]
fn test_cte_with_order_and_limit() {
    let cte_query = Query {
        with_clause: None,
        select: Some(Select::new(Columns::Selected(vec!["*"]), None)),
        from: Some(FromSource::Table("users")),
        joins: vec![],
        where_clause: None,
        group_by: None,
        having: None,
        order_by: Some(OrderBy {
            columns: vec![OrderedColumn::Desc("created_at")],
        }),
        limit: Some(10),
        offset: None,
        for_update: false,
    };
    let mut qb = Q();
    let query = qb
        .with("top_users", cte_query)
        .select(vec!["name", "email"])
        .from("top_users")
        .build();
    assert_eq!(query.sql(), "WITH top_users AS (SELECT * FROM users ORDER BY created_at DESC LIMIT 10) SELECT name, email FROM top_users");
}

#[test]
fn test_direct_cte_construction() {
    let query = Query {
        with_clause: Some(vec![Cte {
            name: "cte1",
            query: Box::new(Query {
                with_clause: None,
                select: Some(Select::new(Columns::Selected(vec!["id"]), None)),
                from: Some(FromSource::Table("users")),
                joins: vec![],
                where_clause: None,
                group_by: None,
                having: None,
                order_by: None,
                limit: None,
                offset: None,
                for_update: false,
            }),
        }]),
        select: Some(Select::new(Columns::Star, None)),
        from: Some(FromSource::Table("cte1")),
        joins: vec![],
        where_clause: None,
        group_by: None,
        having: None,
        order_by: None,
        limit: None,
        offset: None,
        for_update: false,
    };
    assert_eq!(
        query.sql(),
        "WITH cte1 AS (SELECT id FROM users) SELECT * FROM cte1"
    );
}

// ON CONFLICT / UPSERT tests
#[test]
fn test_on_conflict_do_nothing() {
    let mut ib = I("users");
    let insert = ib
        .columns(vec!["email", "name"])
        .values(vec!["'alice@example.com'", "'Alice'"])
        .on_conflict_do_nothing(vec!["email"])
        .build();
    assert_eq!(insert.sql(), "INSERT INTO users (email, name) VALUES ('alice@example.com', 'Alice') ON CONFLICT (email) DO NOTHING");
}

#[test]
fn test_on_conflict_do_update() {
    let mut ib = I("users");
    let insert = ib
        .columns(vec!["email", "name"])
        .values(vec!["'alice@example.com'", "'Alice'"])
        .on_conflict_do_update(vec!["email"], vec![("name", "'Alice Updated'")])
        .build();
    assert_eq!(insert.sql(), "INSERT INTO users (email, name) VALUES ('alice@example.com', 'Alice') ON CONFLICT (email) DO UPDATE SET name = 'Alice Updated'");
}

#[test]
fn test_on_conflict_multiple_columns() {
    let mut ib = I("products");
    let insert = ib
        .columns(vec!["sku", "name", "price"])
        .values(vec!["'ABC123'", "'Widget'", "19.99"])
        .on_conflict_do_nothing(vec!["sku", "name"])
        .build();
    assert_eq!(insert.sql(), "INSERT INTO products (sku, name, price) VALUES ('ABC123', 'Widget', 19.99) ON CONFLICT (sku, name) DO NOTHING");
}

#[test]
fn test_on_conflict_do_update_multiple_columns() {
    let mut ib = I("products");
    let insert = ib
        .columns(vec!["sku", "name", "price"])
        .values(vec!["'ABC123'", "'Widget'", "19.99"])
        .on_conflict_do_update(
            vec!["sku"],
            vec![("name", "'Widget Updated'"), ("price", "24.99")],
        )
        .build();
    assert_eq!(insert.sql(), "INSERT INTO products (sku, name, price) VALUES ('ABC123', 'Widget', 19.99) ON CONFLICT (sku) DO UPDATE SET name = 'Widget Updated', price = 24.99");
}

#[test]
fn test_on_conflict_with_returning() {
    let mut ib = I("users");
    let insert = ib
        .columns(vec!["email", "name"])
        .values(vec!["'bob@example.com'", "'Bob'"])
        .on_conflict_do_update(vec!["email"], vec![("name", "'Bob Updated'")])
        .returning(Columns::Star)
        .build();
    assert_eq!(insert.sql(), "INSERT INTO users (email, name) VALUES ('bob@example.com', 'Bob') ON CONFLICT (email) DO UPDATE SET name = 'Bob Updated' RETURNING *");
}

#[test]
fn test_on_conflict_enum_do_nothing() {
    let on_conflict = OnConflict::DoNothing(vec!["email"]);
    assert_eq!(on_conflict.sql(), "ON CONFLICT (email) DO NOTHING");
}

#[test]
fn test_on_conflict_enum_do_update() {
    let on_conflict = OnConflict::DoUpdate(
        vec!["email"],
        vec![("name", "'Updated'"), ("status", "'active'")],
    );
    assert_eq!(
        on_conflict.sql(),
        "ON CONFLICT (email) DO UPDATE SET name = 'Updated', status = 'active'"
    );
}

#[test]
fn test_direct_on_conflict_construction() {
    let insert = Insert {
        table: "users",
        columns: vec!["email", "name"],
        source: InsertSource::Values(vec![vec!["'test@example.com'", "'Test'"]]),
        on_conflict: Some(OnConflict::DoNothing(vec!["email"])),
        returning: None,
    };
    assert_eq!(insert.sql(), "INSERT INTO users (email, name) VALUES ('test@example.com', 'Test') ON CONFLICT (email) DO NOTHING");
}

#[test]
fn test_on_conflict_with_multiple_rows() {
    let mut ib = I("users");
    let insert = ib
        .columns(vec!["email", "name"])
        .rows(vec![
            vec!["'alice@example.com'", "'Alice'"],
            vec!["'bob@example.com'", "'Bob'"],
        ])
        .on_conflict_do_nothing(vec!["email"])
        .build();
    assert_eq!(insert.sql(), "INSERT INTO users (email, name) VALUES ('alice@example.com', 'Alice'), ('bob@example.com', 'Bob') ON CONFLICT (email) DO NOTHING");
}

#[test]
fn test_insert_single_column() {
    let insert = Insert {
        table: "users",
        columns: vec!["name"],
        source: InsertSource::Values(vec![vec!["'Alice'"]]),
        on_conflict: None,
        returning: None,
    };
    assert_eq!(insert.sql(), "INSERT INTO users (name) VALUES ('Alice')");
}

#[test]
fn test_insert_builder_columns_method() {
    let mut ib = I("users");
    let insert = ib
        .columns(vec!["name", "email", "age"])
        .values(vec!["'Test'", "'test@example.com'", "25"])
        .build();
    assert_eq!(
        insert.sql(),
        "INSERT INTO users (name, email, age) VALUES ('Test', 'test@example.com', 25)"
    );
}

#[test]
fn test_on_conflict_single_update() {
    let on_conflict = OnConflict::DoUpdate(vec!["id"], vec![("status", "'active'")]);
    assert_eq!(
        on_conflict.sql(),
        "ON CONFLICT (id) DO UPDATE SET status = 'active'"
    );
}

#[test]
fn test_insert_with_select_and_on_conflict() {
    let select_query = Query {
        with_clause: None,
        select: Some(Select::new(Columns::Selected(vec!["id", "name"]), None)),
        from: Some(FromSource::Table("temp_users")),
        joins: vec![],
        where_clause: None,
        group_by: None,
        having: None,
        order_by: None,
        limit: None,
        offset: None,
        for_update: false,
    };
    let insert = Insert {
        table: "users",
        columns: vec!["id", "name"],
        source: InsertSource::Select(Box::new(select_query)),
        on_conflict: Some(OnConflict::DoNothing(vec!["id"])),
        returning: None,
    };
    assert_eq!(
        insert.sql(),
        "INSERT INTO users (id, name) SELECT id, name FROM temp_users ON CONFLICT (id) DO NOTHING"
    );
}

#[test]
fn test_query_with_cte_and_joins() {
    let cte_query = Query {
        with_clause: None,
        select: Some(Select::new(
            Columns::Selected(vec!["user_id", "total"]),
            None,
        )),
        from: Some(FromSource::Table("orders")),
        joins: vec![],
        where_clause: None,
        group_by: Some(vec!["user_id"]),
        having: None,
        order_by: None,
        limit: None,
        offset: None,
        for_update: false,
    };
    let mut qb = Q();
    let query = qb
        .with("user_totals", cte_query)
        .select(vec!["u.name", "ut.total"])
        .from("users u")
        .inner_join("user_totals ut", eq("u.id", "ut.user_id"))
        .where_(gt("ut.total", "100"))
        .order_by(vec![
            OrderedColumn::Desc("ut.total"),
            OrderedColumn::Asc("u.name"),
        ])
        .limit(10)
        .build();
    assert_eq!(query.sql(), "WITH user_totals AS (SELECT user_id, total FROM orders GROUP BY user_id) SELECT u.name, ut.total FROM users u INNER JOIN user_totals ut ON u.id = ut.user_id WHERE ut.total > 100 ORDER BY ut.total DESC, u.name ASC LIMIT 10");
}

#[test]
fn test_case_expression_with_else() {
    let case_expr = Term::Case(CaseExpression {
        when_thens: vec![
            WhenThen {
                when: eq("status", "'active'"),
                then: Term::Atom("1"),
            },
            WhenThen {
                when: eq("status", "'pending'"),
                then: Term::Atom("2"),
            },
        ],
        else_term: Some(Box::new(Term::Atom("0"))),
    });
    assert_eq!(
        case_expr.sql(),
        "CASE WHEN status = 'active' THEN 1 WHEN status = 'pending' THEN 2 ELSE 0 END"
    );
}

#[test]
fn test_substring_with_from_and_for() {
    let result = substring(
        Term::Atom("name"),
        Some(Term::Atom("1")),
        Some(Term::Atom("5")),
    )
    .sql();
    assert_eq!(result, "SUBSTRING(name FROM 1 FOR 5)");
}
