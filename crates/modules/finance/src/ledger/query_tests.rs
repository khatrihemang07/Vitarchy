use chrono::NaiveDate;

use super::{Flow, LedgerQuery, Sort, query::QueryError};

fn parse(query: &str) -> Result<LedgerQuery, QueryError> {
    LedgerQuery::from_raw(Some(query))
}

#[test]
fn an_empty_query_keeps_everything_newest_first() {
    let q = parse("").unwrap();
    assert_eq!(q, LedgerQuery::default());
    assert_eq!(q.sort, Sort::DateDesc);
}

#[test]
fn account_id_may_repeat() {
    let q = parse("account_id=3&account_id=7").unwrap();
    assert_eq!(q.filter.account_ids, [3, 7]);
}

#[test]
fn every_direction_and_sort_is_understood() {
    for (text, flow) in [
        ("in", Flow::In),
        ("out", Flow::Out),
        ("transfer", Flow::Transfer),
    ] {
        assert_eq!(
            parse(&format!("direction={text}")).unwrap().filter.flow,
            Some(flow)
        );
    }
    for (text, sort) in [
        ("date_desc", Sort::DateDesc),
        ("date_asc", Sort::DateAsc),
        ("amount_desc", Sort::AmountDesc),
        ("amount_asc", Sort::AmountAsc),
    ] {
        assert_eq!(parse(&format!("sort={text}")).unwrap().sort, sort);
    }
}

#[test]
fn the_search_is_escaped_and_a_blank_one_is_none() {
    assert_eq!(
        parse("q=50%25").unwrap().filter.pattern.as_deref(),
        Some("%50\\%%")
    );
    assert_eq!(parse("q=").unwrap().filter.pattern, None);
    assert_eq!(parse("q=%20%20").unwrap().filter.pattern, None);
}

#[test]
fn dates_limit_and_offset_are_read() {
    let q = parse("from=2026-07-01&to=2026-07-31&limit=5&offset=10").unwrap();
    assert_eq!(q.filter.from, NaiveDate::from_ymd_opt(2026, 7, 1));
    assert_eq!(q.filter.to, NaiveDate::from_ymd_opt(2026, 7, 31));
    assert_eq!((q.limit, q.offset), (Some(5), Some(10)));
}

#[test]
fn bad_values_are_refused_naming_the_parameter() {
    for (query, name) in [
        ("direction=sideways", "direction"),
        ("sort=random", "sort"),
        ("account_id=x", "account_id"),
        ("from=July", "from"),
        ("limit=many", "limit"),
        ("from=2026-07-02&to=2026-07-01", "to"),
    ] {
        assert_eq!(parse(query), Err(QueryError(name)), "{query}");
    }
}
