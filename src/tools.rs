// file: rust/src/tools.rs
// description: built-in function tools exposed to the model
// reference: https://developers.openai.com/api/docs/api-reference/responses

use std::fs;
use std::path::Path;
use std::time::{SystemTime, UNIX_EPOCH};

use serde_json::{Value, json};

use crate::types::{FunctionTool, Tool};

const READ_FILE_MAX_BYTES: u64 = 64 * 1024;

pub fn builtin_tools() -> Vec<Tool> {
    vec![
        Tool::Function(FunctionTool {
            name: "get_time".into(),
            description: "Return the current UTC time as ISO 8601 and Unix seconds.".into(),
            parameters: json!({
                "type": "object",
                "properties": {},
                "additionalProperties": false,
            }),
        }),
        Tool::Function(FunctionTool {
            name: "read_file".into(),
            description: format!(
                "Read a UTF-8 text file from the local filesystem (max {} bytes). \
                 Returns the contents on success.",
                READ_FILE_MAX_BYTES
            ),
            parameters: json!({
                "type": "object",
                "properties": {
                    "path": {
                        "type": "string",
                        "description": "Absolute or working-directory-relative file path.",
                    }
                },
                "required": ["path"],
                "additionalProperties": false,
            }),
        }),
    ]
}

/// Execute a tool by name with raw JSON arguments. Always returns a JSON string
/// suitable for the `output` field of a `function_call_output` item.
///
/// `read_root`, when `Some`, is the canonicalised sandbox directory under which
/// the `read_file` tool is permitted to read; when `None`, `read_file` refuses.
pub fn execute(name: &str, arguments: &str, read_root: Option<&std::path::Path>) -> String {
    let args: Value = if arguments.is_empty() {
        Value::Null
    } else {
        match serde_json::from_str(arguments) {
            Ok(v) => v,
            Err(e) => {
                return json!({
                    "error": format!("invalid tool arguments JSON: {e}")
                })
                .to_string();
            }
        }
    };

    let result = match name {
        "get_time" => get_time(),
        "read_file" => read_file(&args, read_root),
        _ => Err(format!("unknown tool: {name}")),
    };

    match result {
        Ok(value) => value.to_string(),
        Err(msg) => json!({ "error": msg }).to_string(),
    }
}

fn get_time() -> Result<Value, String> {
    let secs = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .map_err(|e| e.to_string())?
        .as_secs();
    Ok(json!({
        "iso8601_utc": format_utc(secs),
        "unix_seconds": secs,
    }))
}

fn read_file(args: &Value, read_root: Option<&Path>) -> Result<Value, String> {
    let path = args
        .get("path")
        .and_then(Value::as_str)
        .ok_or_else(|| "missing required string argument: path".to_string())?;

    let root = read_root.ok_or_else(|| {
        "read_file disabled: set OPENAI_TOOLS_READ_ROOT to a directory to enable".to_string()
    })?;

    let canonical_root = fs::canonicalize(root)
        .map_err(|e| format!("read_root canonicalize failed: {e}"))?;
    let canonical_path = fs::canonicalize(Path::new(path))
        .map_err(|e| format!("path canonicalize failed: {e}"))?;

    if !canonical_path.starts_with(&canonical_root) {
        return Err(format!(
            "path '{path}' is outside the configured OPENAI_TOOLS_READ_ROOT sandbox"
        ));
    }

    let meta = fs::metadata(&canonical_path).map_err(|e| format!("stat failed: {e}"))?;
    if !meta.is_file() {
        return Err(format!("not a regular file: {path}"));
    }
    if meta.len() > READ_FILE_MAX_BYTES {
        return Err(format!(
            "file too large: {} bytes (limit {})",
            meta.len(),
            READ_FILE_MAX_BYTES
        ));
    }

    let contents =
        fs::read_to_string(&canonical_path).map_err(|e| format!("read failed: {e}"))?;
    Ok(json!({
        "path": path,
        "bytes": meta.len(),
        "contents": contents,
    }))
}

/// Format Unix seconds as `YYYY-MM-DDTHH:MM:SSZ` using Howard Hinnant's
/// civil_from_days algorithm. Valid for any year in i64 range.
pub fn format_utc(unix_secs: u64) -> String {
    let secs = unix_secs.min(i64::MAX as u64) as i64;
    let days = secs.div_euclid(86400);
    let sod = secs.rem_euclid(86400);

    let z = days + 719_468;
    let era = if z >= 0 { z } else { z - 146_096 } / 146_097;
    let doe = z - era * 146_097;
    let yoe = (doe - doe / 1460 + doe / 36_524 - doe / 146_096) / 365;
    let mut y = yoe + era * 400;
    let doy = doe - (365 * yoe + yoe / 4 - yoe / 100);
    let mp = (5 * doy + 2) / 153;
    let d = doy - (153 * mp + 2) / 5 + 1;
    let m = if mp < 10 { mp + 3 } else { mp - 9 };
    if m <= 2 {
        y += 1;
    }

    let hh = sod / 3600;
    let mm = (sod % 3600) / 60;
    let ss = sod % 60;

    format!("{y:04}-{m:02}-{d:02}T{hh:02}:{mm:02}:{ss:02}Z")
}
