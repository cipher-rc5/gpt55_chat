// file: tests/format_utc.rs
// description: table-driven and property tests for tools::format_utc

use gpt55_chat::tools::format_utc;
use proptest::prelude::*;

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

proptest! {
    /// For any 4-digit-year-range input, `format_utc` produces a 20-char
    /// `YYYY-MM-DDTHH:MM:SSZ` string with sane field bounds.
    #[test]
    fn output_shape_is_iso8601_z(secs in 0u64..253_402_300_799) {
        let s = format_utc(secs);
        prop_assert_eq!(s.len(), 20);
        prop_assert!(s.ends_with('Z'));
        prop_assert!(s.is_ascii());
        prop_assert_eq!(&s[4..5], "-");
        prop_assert_eq!(&s[7..8], "-");
        prop_assert_eq!(&s[10..11], "T");
        prop_assert_eq!(&s[13..14], ":");
        prop_assert_eq!(&s[16..17], ":");
        let month: u32 = s[5..7].parse().unwrap();
        let day: u32 = s[8..10].parse().unwrap();
        let hour: u32 = s[11..13].parse().unwrap();
        let minute: u32 = s[14..16].parse().unwrap();
        let second: u32 = s[17..19].parse().unwrap();
        prop_assert!((1..=12).contains(&month), "month out of range: {}", month);
        prop_assert!((1..=31).contains(&day), "day out of range: {}", day);
        prop_assert!(hour < 24, "hour out of range: {}", hour);
        prop_assert!(minute < 60, "minute out of range: {}", minute);
        prop_assert!(second < 60, "second out of range: {}", second);
    }

    /// `format_utc` is monotonically non-decreasing in lexicographic order
    /// for monotonically non-decreasing inputs (within ISO 8601's year range).
    #[test]
    fn monotonic_lex_order(
        a in 0u64..253_402_300_799,
        delta in 0u64..86_400,
    ) {
        let b = a.saturating_add(delta);
        prop_assert!(format_utc(a) <= format_utc(b));
    }
}
