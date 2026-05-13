// file: tests/extract_reply.rs
// description: unit tests for client::extract_reply against manually-built ResponsesResponse values

use gpt55_chat::client::extract_reply;
use gpt55_chat::types::{
    FunctionCall, MessageContent, MessageOutput, OutputItem, OutputTextContent, ReasoningOutput,
    ResponsesResponse, Role,
};

fn make_response(output: Vec<OutputItem>) -> ResponsesResponse {
    ResponsesResponse {
        id: "resp_test".to_owned(),
        status: "completed".to_owned(),
        model: "test-model".to_owned(),
        output,
        usage: None,
        incomplete_details: None,
    }
}

fn text(s: &str) -> MessageContent {
    MessageContent::OutputText(OutputTextContent { text: s.to_owned() })
}

fn message(id: &str, contents: Vec<MessageContent>) -> OutputItem {
    OutputItem::Message(MessageOutput {
        id: id.to_owned(),
        role: Role::Assistant,
        content: contents,
    })
}

fn reasoning(id: &str) -> OutputItem {
    OutputItem::Reasoning(ReasoningOutput {
        id: id.to_owned(),
        summary: vec![],
    })
}

#[test]
fn empty_output_returns_none() {
    let resp = make_response(vec![]);
    assert_eq!(extract_reply(&resp), None);
}

#[test]
fn single_message_single_fragment_returns_text() {
    let resp = make_response(vec![message("m1", vec![text("hello world")])]);
    assert_eq!(extract_reply(&resp), Some("hello world".to_owned()));
}

#[test]
fn single_message_two_fragments_concatenated() {
    let resp = make_response(vec![message("m1", vec![text("foo "), text("bar")])]);
    assert_eq!(extract_reply(&resp), Some("foo bar".to_owned()));
}

#[test]
fn two_messages_each_one_fragment_separated() {
    let resp = make_response(vec![
        message("m1", vec![text("alpha")]),
        message("m2", vec![text("beta")]),
    ]);
    assert_eq!(extract_reply(&resp), Some("alpha\n\nbeta".to_owned()));
}

#[test]
fn empty_message_between_text_messages_does_not_add_extra_separator() {
    let resp = make_response(vec![
        message("m1", vec![text("alpha")]),
        message("m2", vec![]),
        message("m3", vec![text("beta")]),
    ]);
    assert_eq!(extract_reply(&resp), Some("alpha\n\nbeta".to_owned()));
}

#[test]
fn multiple_fragments_then_second_message() {
    let resp = make_response(vec![
        message("m1", vec![text("hello "), text("world")]),
        message("m2", vec![text("second")]),
    ]);
    assert_eq!(
        extract_reply(&resp),
        Some("hello world\n\nsecond".to_owned())
    );
}

#[test]
fn reasoning_plus_message_ignores_reasoning() {
    let resp = make_response(vec![
        reasoning("r1"),
        message("m1", vec![text("only message text")]),
    ]);
    assert_eq!(extract_reply(&resp), Some("only message text".to_owned()));
}

#[test]
fn message_with_empty_content_returns_none() {
    let resp = make_response(vec![message("m1", vec![])]);
    assert_eq!(extract_reply(&resp), None);
}

#[test]
fn function_call_only_returns_none() {
    let resp = make_response(vec![OutputItem::FunctionCall(FunctionCall {
        id: "fc1".to_owned(),
        call_id: "call_1".to_owned(),
        name: "get_time".to_owned(),
        arguments: "{}".to_owned(),
    })]);
    assert_eq!(extract_reply(&resp), None);
}
