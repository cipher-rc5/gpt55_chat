// file: tests/parse.rs
// description: table-driven tests for ReasoningEffort::parse and ReasoningSummary::parse

use gpt55_chat::types::{ReasoningEffort, ReasoningSummary};

#[test]
fn reasoning_effort_happy_paths() {
    let cases: &[(&str, ReasoningEffort)] = &[
        ("none", ReasoningEffort::None),
        ("minimal", ReasoningEffort::Minimal),
        ("low", ReasoningEffort::Low),
        ("medium", ReasoningEffort::Medium),
        ("high", ReasoningEffort::High),
        ("xhigh", ReasoningEffort::Xhigh),
        ("NONE", ReasoningEffort::None),
        ("Minimal", ReasoningEffort::Minimal),
        ("  low  ", ReasoningEffort::Low),
        ("\tMEDIUM\n", ReasoningEffort::Medium),
        ("HiGh", ReasoningEffort::High),
        ("XHIGH", ReasoningEffort::Xhigh),
    ];

    for (input, expected) in cases {
        let got = ReasoningEffort::parse(input);
        assert_eq!(got, Some(*expected), "input {input:?}");
    }
}

#[test]
fn reasoning_effort_invalid() {
    let bad: &[&str] = &["", "  ", "lowish", "extreme", "no ne", "x-high", "foo"];
    for input in bad {
        assert_eq!(ReasoningEffort::parse(input), None, "input {input:?}");
    }
}

#[test]
fn reasoning_summary_happy_paths() {
    let cases: &[(&str, ReasoningSummary)] = &[
        ("auto", ReasoningSummary::Auto),
        ("concise", ReasoningSummary::Concise),
        ("detailed", ReasoningSummary::Detailed),
        ("AUTO", ReasoningSummary::Auto),
        ("Concise", ReasoningSummary::Concise),
        ("  detailed  ", ReasoningSummary::Detailed),
        ("\tCoNcIsE\n", ReasoningSummary::Concise),
    ];

    for (input, expected) in cases {
        let got = ReasoningSummary::parse(input);
        assert_eq!(got, Some(*expected), "input {input:?}");
    }
}

#[test]
fn reasoning_summary_invalid() {
    let bad: &[&str] = &["", " ", "verbose", "auto2", "consise", "detail"];
    for input in bad {
        assert_eq!(ReasoningSummary::parse(input), None, "input {input:?}");
    }
}
