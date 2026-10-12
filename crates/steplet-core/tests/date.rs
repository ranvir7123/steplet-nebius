use steplet_core::date::Date;

#[test]
fn reads_and_writes_dates() {
    assert_eq!(Date::parse("2026-10-25").unwrap().to_string(), "2026-10-25");
    assert!(Date::parse("2028-02-29").is_some());
    assert!(Date::parse("2000-02-29").is_some());
}

#[test]
fn refuses_anything_else() {
    for s in [
        "2026-02-29",
        "1900-02-29",
        "2026-13-01",
        "2026-00-10",
        "2026-10-32",
        "2026-10-00",
        "26-10-25",
        "2026/10/25",
        "2026-1-05",
        "2026-10-25 ",
        " 2026-10-25",
        "\u{ff12}026-10-25",
        "1999-12-31",
        "+026-10-25",
        "2026-10-2a",
        "",
    ] {
        assert_eq!(Date::parse(s), None, "{s:?}");
    }
}

#[test]
fn counts_days_from_1970() {
    let d = Date::parse("2026-10-11").unwrap();
    assert_eq!(d.days_since_epoch(), 20_737);
    assert_eq!(Date::from_days_since_epoch(20_737), Some(d));
    assert_eq!(Date::parse("2000-01-01").unwrap().days_since_epoch(), 10_957);
}

#[test]
fn a_day_ends_at_midnight_utc() {
    let d = Date::parse("2026-10-11").unwrap();
    assert_eq!(d.end_unix(), 1_791_763_200);
    assert_eq!(Date::from_unix(1_791_763_199), Some(d));
    assert_eq!(Date::from_unix(1_791_763_200), Date::parse("2026-10-12"));
}

#[test]
fn adds_days_across_months_and_years() {
    let d = Date::parse("2026-10-11").unwrap();
    assert_eq!(d.add_days(14), Date::parse("2026-10-25"));
    assert_eq!(d.add_days(365), Date::parse("2027-10-11"));
    assert_eq!(Date::parse("2028-02-28").unwrap().add_days(1), Date::parse("2028-02-29"));
    assert_eq!(Date::parse("9999-12-31").unwrap().add_days(1), None);
}
