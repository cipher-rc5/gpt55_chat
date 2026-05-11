// file: tests/format_utc.rs
// description: table-driven tests for tools::format_utc

use gpt55_chat::tools::format_utc;

#[test]
fn known_timestamps() {
    let cases: &[(u64, &str)] = &[
        (0, "1970-01-01T00:00:00Z"),
        (1_700_000_000, "2023-11-14T22:13:20Z"),
        (253_402_300_799, "9999-12-31T23:59:59Z"),
    ];

    for (input, expected) in cases {
        assert_eq!(format_utc(*input), *expected, "input {input}");
    }
}

#[test]
fn saturating_cast_does_not_panic() {
    let out = format_utc(u64::MAX);
    assert!(!out.is_empty());
}
