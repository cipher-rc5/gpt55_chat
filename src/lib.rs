// file: rust/src/lib.rs
// description: library crate exposing internal modules for integration tests

//! `gpt55_chat` is a small client library for the OpenAI/Azure Responses API
//! that powers the bundled `gpt55-chat` terminal chat CLI. It wraps request
//! construction, the streaming/tool-call roundtrip loop, and a tiny built-in
//! function-tool registry behind an ergonomic async interface.
//!
//! # Modules
//!
//! - [`types`] carries the wire-format and configuration types shared across
//!   the crate.
//! - [`config`] loads env-driven runtime configuration for the chat client.
//! - [`client`] is the async HTTP client for the Responses API and the
//!   tool-roundtrip loop.
//! - [`tools`] is the built-in function-tool registry (`get_time`,
//!   `read_file`) and the tool dispatcher.
//!
//! The library is intended primarily as the engine for the `gpt55-chat`
//! binary; the public API is not yet semver-stable and may change without
//! notice.

/// Async HTTP client for the Responses API and the tool-roundtrip loop.
pub mod client;
/// Environment-driven configuration loader for the chat client.
pub mod config;
/// Image-generation client for the Azure OpenAI `images/generations` endpoint.
pub mod image;
/// PNG → SVG conversion via the Responses API vision input.
pub mod svg;
/// Built-in function tools (`get_time`, `read_file`) and the tool dispatcher.
pub mod tools;
/// Wire-format and configuration types shared across the crate.
pub mod types;
