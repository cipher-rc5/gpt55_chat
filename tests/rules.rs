// file: tests/rules.rs
// description: tests for config::load_rules and config::compose_instructions

use std::fs;
use std::path::PathBuf;

use gpt55_chat::config::{compose_instructions, load_rules};
use gpt55_chat::types::ChatError;

struct TempFile {
    path: PathBuf,
}

impl TempFile {
    fn new(suffix: &str, contents: &str) -> Self {
        let mut path = std::env::temp_dir();
        path.push(format!(
            "gpt55_chat_rules_{}_{}_{suffix}",
            std::process::id(),
            // nanoseconds for uniqueness within a single process
            std::time::SystemTime::now()
                .duration_since(std::time::UNIX_EPOCH)
                .map(|d| d.as_nanos())
                .unwrap_or(0)
        ));
        fs::write(&path, contents).expect("write tempfile");
        Self { path }
    }
}

impl Drop for TempFile {
    fn drop(&mut self) {
        let _ = fs::remove_file(&self.path);
    }
}

#[test]
fn load_rules_skips_empty_and_comments_and_strips_dash() {
    let contents = "\
# this is a comment
- be concise

   # indented comment
- avoid jargon
plain rule
   - trimmed dash rule
";
    let tf = TempFile::new("a", contents);
    let rules = load_rules(tf.path.to_str().unwrap()).expect("load_rules");
    assert_eq!(
        rules,
        vec![
            "be concise".to_string(),
            "avoid jargon".to_string(),
            "plain rule".to_string(),
            "trimmed dash rule".to_string(),
        ]
    );
}

#[test]
fn compose_none_and_empty_rules_is_none() {
    assert_eq!(compose_instructions(None, vec![]), None);
}

#[test]
fn compose_only_system_prompt() {
    assert_eq!(
        compose_instructions(Some("hi".into()), vec![]),
        Some("hi".into())
    );
}

#[test]
fn compose_only_rules() {
    assert_eq!(
        compose_instructions(None, vec!["a".into(), "b".into()]),
        Some("# Rules\n- a\n- b\n".into())
    );
}

#[test]
fn compose_system_prompt_and_rules() {
    assert_eq!(
        compose_instructions(Some("hi".into()), vec!["a".into()]),
        Some("hi\n\n# Rules\n- a\n".into())
    );
}

#[test]
fn compose_whitespace_only_system_prompt() {
    assert_eq!(compose_instructions(Some("   \t\n  ".into()), vec![]), None);
}

#[test]
fn load_rules_nonexistent_path_returns_config_error() {
    let bogus = "/nonexistent/path/that/should/not/exist.txt";
    let err = load_rules(bogus).expect_err("expected error for missing path");
    match err {
        ChatError::Config(msg) => {
            assert!(
                msg.contains("failed to read rules file"),
                "missing prefix in: {msg}"
            );
            assert!(msg.contains(bogus), "missing path in: {msg}");
        }
        other => panic!("expected ChatError::Config, got: {other:?}"),
    }
}
