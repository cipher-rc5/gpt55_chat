// file: tests/tools_execute.rs
// description: integration tests for tools::execute

use std::fs;
use std::path::{Path, PathBuf};

use gpt55_chat::tools::execute;

struct TempDir {
    path: PathBuf,
}

impl TempDir {
    fn new(tag: &str) -> Self {
        let mut path = std::env::temp_dir();
        path.push(format!(
            "gpt55_chat_test_{}_{}_{tag}",
            std::process::id(),
            std::time::SystemTime::now()
                .duration_since(std::time::UNIX_EPOCH)
                .map(|d| d.as_nanos())
                .unwrap_or(0)
        ));
        fs::create_dir_all(&path).expect("create tempdir");
        // canonicalize so paths line up with read_file's canonicalisation
        let canonical = fs::canonicalize(&path).expect("canonicalize tempdir");
        Self { path: canonical }
    }

    fn path(&self) -> &Path {
        &self.path
    }
}

impl Drop for TempDir {
    fn drop(&mut self) {
        let _ = fs::remove_dir_all(&self.path);
    }
}

#[test]
fn get_time_returns_iso_and_unix() {
    let out = execute("get_time", "", None);
    assert!(out.contains("\"iso8601_utc\""), "got {out}");
    assert!(out.contains("\"unix_seconds\""), "got {out}");
}

#[test]
fn read_file_disabled_when_no_root() {
    let out = execute("read_file", "{\"path\":\"/tmp/whatever\"}", None);
    assert!(out.contains("OPENAI_TOOLS_READ_ROOT"), "got {out}");
}

#[test]
fn read_file_missing_path_argument() {
    let tmp = TempDir::new("missing_path");
    let out = execute("read_file", "{}", Some(tmp.path()));
    assert!(
        out.contains("missing required string argument: path"),
        "got {out}"
    );
}

#[test]
fn read_file_outside_sandbox() {
    let tmp = TempDir::new("sandbox");
    // The project's Cargo.toml lives well outside our tempdir.
    let outside = fs::canonicalize(env!("CARGO_MANIFEST_DIR"))
        .expect("canonicalize manifest dir")
        .join("Cargo.toml");
    let args = format!("{{\"path\":{}}}", serde_json::to_string(&outside).unwrap());
    let out = execute("read_file", &args, Some(tmp.path()));
    assert!(
        out.contains("outside the configured OPENAI_TOOLS_READ_ROOT sandbox"),
        "got {out}"
    );
}

#[test]
fn read_file_inside_sandbox() {
    let tmp = TempDir::new("inside");
    let file = tmp.path().join("hello.txt");
    fs::write(&file, "world").expect("write inside file");

    let args = format!("{{\"path\":{}}}", serde_json::to_string(&file).unwrap());
    let out = execute("read_file", &args, Some(tmp.path()));
    assert!(out.contains("\"contents\""), "got {out}");
    assert!(out.contains("world"), "got {out}");
}

#[test]
fn read_file_invalid_json() {
    let tmp = TempDir::new("invalid_json");
    let out = execute("read_file", "{not json}", Some(tmp.path()));
    assert!(out.contains("invalid tool arguments JSON"), "got {out}");
}

#[test]
fn unknown_tool_errors() {
    let out = execute("unknown_tool", "{}", None);
    assert!(out.contains("unknown tool: unknown_tool"), "got {out}");
}
