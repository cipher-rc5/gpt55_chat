// file: tests/cli.rs
// description: unit tests for the slash-command parsers in gpt55_chat::cli

use std::path::PathBuf;

use gpt55_chat::cli::{
    ImageArgs, PromptSource, SlashCommand, SvgArgs, classify_prompt, classify_slash,
    parse_image_args, parse_svg_args, truncate_for_display,
};
use gpt55_chat::svg::SvgStyle;

#[test]
fn classify_slash_recognises_known_commands() {
    assert_eq!(classify_slash("/help"), SlashCommand::Help);
    assert_eq!(classify_slash("/?"), SlashCommand::Help);
    assert_eq!(
        classify_slash("/image a red fox"),
        SlashCommand::Image { rest: "a red fox" }
    );
    assert_eq!(
        classify_slash("/svg --style=editable logo.png"),
        SlashCommand::Svg {
            rest: "--style=editable logo.png",
        }
    );
}

#[test]
fn classify_slash_unknown_returns_name() {
    assert_eq!(
        classify_slash("/wat foo"),
        SlashCommand::Unknown { name: "/wat" }
    );
}

#[test]
fn classify_slash_no_argument() {
    assert_eq!(classify_slash("/image"), SlashCommand::Image { rest: "" });
}

#[test]
fn parse_image_args_collects_all_flags() {
    let args = parse_image_args("--size=512x512 --quality=high --n=3 --format=jpeg a red fox")
        .expect("parse");
    assert_eq!(
        args,
        ImageArgs {
            size: Some("512x512".into()),
            quality: Some("high".into()),
            n: Some(3),
            format: Some("jpeg".into()),
            remainder: "a red fox".into(),
        }
    );
}

#[test]
fn parse_image_args_stops_at_first_non_flag() {
    let args = parse_image_args("--quality=low a fox --size=ignored").expect("parse");
    assert_eq!(args.quality.as_deref(), Some("low"));
    assert_eq!(args.size, None);
    assert_eq!(args.remainder, "a fox --size=ignored");
}

#[test]
fn parse_image_args_rejects_unknown_flag() {
    let err = parse_image_args("--mystery=1 a fox").expect_err("must fail");
    assert!(err.contains("unknown flag"));
}

#[test]
fn parse_image_args_rejects_bad_n() {
    let err = parse_image_args("--n=banana a fox").expect_err("must fail");
    assert!(err.contains("--n"));
}

#[test]
fn parse_image_args_empty_input() {
    let args = parse_image_args("").expect("parse");
    assert_eq!(args, ImageArgs::default());
}

#[test]
fn classify_prompt_branches() {
    assert_eq!(classify_prompt(""), PromptSource::MultiLine);
    assert_eq!(
        classify_prompt("@./prompt.txt"),
        PromptSource::File(PathBuf::from("./prompt.txt"))
    );
    assert_eq!(
        classify_prompt("just inline text"),
        PromptSource::Inline("just inline text".into())
    );
}

#[test]
fn parse_svg_args_default_style() {
    let SvgArgs { style, path } = parse_svg_args("logo.png").expect("parse");
    assert_eq!(style, SvgStyle::Combined);
    assert_eq!(path, PathBuf::from("logo.png"));
}

#[test]
fn parse_svg_args_explicit_style() {
    let SvgArgs { style, path } = parse_svg_args("--style=editable logo.png").expect("parse");
    assert_eq!(style, SvgStyle::Editable);
    assert_eq!(path, PathBuf::from("logo.png"));
}

#[test]
fn parse_svg_args_missing_path() {
    let err = parse_svg_args("--style=compact").expect_err("must fail");
    assert!(err.contains("usage:"));
}

#[test]
fn parse_svg_args_bad_style() {
    let err = parse_svg_args("--style=wrong logo.png").expect_err("must fail");
    assert!(err.contains("--style"));
}

#[test]
fn parse_svg_args_unknown_flag() {
    let err = parse_svg_args("--mystery logo.png").expect_err("must fail");
    assert!(err.contains("unknown flag"));
}

#[test]
fn truncate_for_display_short_input_unchanged() {
    assert_eq!(truncate_for_display("hi", 10), "hi");
}

#[test]
fn truncate_for_display_long_input_truncated() {
    let body = "x".repeat(300);
    let out = truncate_for_display(&body, 256);
    assert_eq!(out.chars().count(), 257); // 256 chars + '…'
    assert!(out.ends_with('…'));
}

#[test]
fn truncate_for_display_trims_whitespace() {
    assert_eq!(truncate_for_display("   hello   ", 10), "hello");
}

#[test]
fn truncate_for_display_does_not_split_multibyte() {
    let body = "日本語テキスト".repeat(20);
    let out = truncate_for_display(&body, 5);
    assert!(out.ends_with('…'));
    // Should be valid UTF-8 — calling .chars() must succeed without panicking.
    let _ = out.chars().count();
}
