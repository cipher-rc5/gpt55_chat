// file: tests/load_config.rs
// description: tests for env-driven config loading

use std::sync::{Mutex, MutexGuard};

use gpt55_chat::config::load_config;
use gpt55_chat::types::{ChatError, LogLevel, Provider, Tool};

static ENV_LOCK: Mutex<()> = Mutex::new(());

const ENV_KEYS: &[&str] = &[
    "AZURE_API_KEY",
    "AZURE_RESOURCE",
    "API_VERSION",
    "MODEL",
    "OPENAI_API_KEY",
    "OPENAI_ENDPOINT",
    "OPENAI_PROVIDER",
    "OPENAI_LOG",
    "OPENAI_RULES_FILE",
    "OPENAI_SYSTEM_PROMPT",
    "OPENAI_TOOLS",
    "OPENAI_TOOLS_READ_ROOT",
    "REASONING_EFFORT",
    "REASONING_SUMMARY",
    "MAX_OUTPUT_TOKENS",
    "AZURE_IMAGE_DEPLOYMENT",
    "AZURE_IMAGE_API_VERSION",
    "OPENAI_IMAGE_OUT_DIR",
];

fn env_guard() -> MutexGuard<'static, ()> {
    let guard = ENV_LOCK.lock().expect("env lock");
    for key in ENV_KEYS {
        remove_env(key);
    }
    guard
}

fn set_env(key: &str, value: &str) {
    // Tests serialize process-env mutation with ENV_LOCK.
    unsafe { std::env::set_var(key, value) }
}

fn remove_env(key: &str) {
    // Tests serialize process-env mutation with ENV_LOCK.
    unsafe { std::env::remove_var(key) }
}

fn set_required_azure() {
    set_env("AZURE_API_KEY", "test-key");
    set_env("AZURE_RESOURCE", "https://example.openai.azure.com");
    set_env("API_VERSION", "2025-test");
    set_env("MODEL", "test-model");
    for key in [
        "OPENAI_PROVIDER",
        "OPENAI_LOG",
        "OPENAI_RULES_FILE",
        "OPENAI_SYSTEM_PROMPT",
        "OPENAI_TOOLS",
        "OPENAI_TOOLS_READ_ROOT",
        "REASONING_EFFORT",
        "REASONING_SUMMARY",
        "MAX_OUTPUT_TOKENS",
        "AZURE_IMAGE_DEPLOYMENT",
        "AZURE_IMAGE_API_VERSION",
        "OPENAI_IMAGE_OUT_DIR",
    ] {
        set_env(key, "");
    }
}

#[test]
fn load_config_defaults_to_azure_provider() {
    let _guard = env_guard();
    set_required_azure();

    let config = load_config().expect("load config");
    assert_eq!(config.provider(), Provider::Azure);
    assert_eq!(
        config.endpoint(),
        "https://example.openai.azure.com/openai/responses"
    );
    assert_eq!(config.api_version(), "2025-test");
    assert_eq!(config.max_output_tokens(), 16384);
    assert_eq!(config.log_level(), LogLevel::Normal);
    assert_eq!(config.tools().len(), 1);
}

#[test]
fn load_config_rejects_empty_required_var() {
    let _guard = env_guard();
    set_required_azure();
    set_env("AZURE_API_KEY", " ");

    let err = load_config().expect_err("expected config error");
    match err {
        ChatError::Config(msg) => assert!(msg.contains("AZURE_API_KEY must not be empty")),
        other => panic!("expected config error, got {other:?}"),
    }
}

#[test]
fn load_config_parses_optional_values_and_read_file_tool() {
    let _guard = env_guard();
    set_required_azure();
    set_env("REASONING_EFFORT", "medium");
    set_env("REASONING_SUMMARY", "auto");
    set_env("MAX_OUTPUT_TOKENS", "4096");
    set_env("OPENAI_LOG", "quiet");
    set_env("OPENAI_TOOLS_READ_ROOT", env!("CARGO_MANIFEST_DIR"));

    let config = load_config().expect("load config");
    assert_eq!(config.max_output_tokens(), 4096);
    assert_eq!(config.log_level(), LogLevel::Quiet);
    assert!(config.tools().iter().any(|tool| match tool {
        Tool::Function(function) => function.name == "read_file",
    }));
}

#[test]
fn load_config_rejects_zero_max_output_tokens() {
    let _guard = env_guard();
    set_required_azure();
    set_env("MAX_OUTPUT_TOKENS", "0");

    let err = load_config().expect_err("expected config error");
    match err {
        ChatError::Config(msg) => assert!(msg.contains("MAX_OUTPUT_TOKENS must be between")),
        other => panic!("expected config error, got {other:?}"),
    }
}

#[test]
fn load_config_disables_tools_when_requested() {
    let _guard = env_guard();
    set_required_azure();
    set_env("OPENAI_TOOLS", "off");
    set_env("OPENAI_TOOLS_READ_ROOT", env!("CARGO_MANIFEST_DIR"));

    let config = load_config().expect("load config");
    assert!(config.tools().is_empty());
}

#[test]
fn load_config_parses_openai_compatible_provider() {
    let _guard = env_guard();
    set_env("OPENAI_PROVIDER", "openai-compatible");
    set_env("OPENAI_API_KEY", "test-key");
    set_env("OPENAI_ENDPOINT", "https://api.example.test/v1/responses");
    set_env("API_VERSION", "");
    set_env("MODEL", "test-model");

    let config = load_config().expect("load config");
    assert_eq!(config.provider(), Provider::OpenAiCompatible);
    assert_eq!(config.endpoint(), "https://api.example.test/v1/responses");
    assert!(config.api_version().is_empty());
}

#[test]
fn load_config_rejects_duplicate_azure_responses_path() {
    let _guard = env_guard();
    set_required_azure();
    set_env(
        "AZURE_RESOURCE",
        "https://example.openai.azure.com/openai/responses",
    );

    let err = load_config().expect_err("expected config error");
    match err {
        ChatError::Config(msg) => assert!(msg.contains("resource root")),
        other => panic!("expected config error, got {other:?}"),
    }
}

#[test]
fn load_config_image_defaults_when_unset() {
    let _guard = env_guard();
    set_required_azure();

    let config = load_config().expect("load config");
    assert!(config.image_deployment().is_none());
    assert_eq!(config.image_api_version(), "2024-02-01");
    assert_eq!(config.image_out_dir(), std::path::Path::new("images"));
}

#[test]
fn load_config_image_overrides_from_env() {
    let _guard = env_guard();
    set_required_azure();
    set_env("AZURE_IMAGE_DEPLOYMENT", "gpt-image-2");
    set_env("AZURE_IMAGE_API_VERSION", "2024-05-13");
    set_env("OPENAI_IMAGE_OUT_DIR", "/tmp/gpt55-images");

    let config = load_config().expect("load config");
    assert_eq!(config.image_deployment(), Some("gpt-image-2"));
    assert_eq!(config.image_api_version(), "2024-05-13");
    assert_eq!(
        config.image_out_dir(),
        std::path::Path::new("/tmp/gpt55-images")
    );
}

#[test]
fn client_config_debug_redacts_api_key() {
    let _guard = env_guard();
    set_required_azure();

    let config = load_config().expect("load config");
    let debug = format!("{config:?}");
    assert!(debug.contains("<redacted>"));
    assert!(!debug.contains("test-key"));
}
