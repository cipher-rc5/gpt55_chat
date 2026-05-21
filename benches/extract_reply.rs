// file: benches/extract_reply.rs
// description: criterion benchmark for client::extract_reply

use criterion::{Criterion, black_box, criterion_group, criterion_main};
use gpt55_chat::client::extract_reply;
use gpt55_chat::types::{
    MessageContent, MessageOutput, OutputItem, OutputTextContent, ResponsesResponse, Role,
};

fn make_response(fragments_per_message: usize, message_count: usize) -> ResponsesResponse {
    let output: Vec<OutputItem> = (0..message_count)
        .map(|i| {
            let content = (0..fragments_per_message)
                .map(|j| {
                    MessageContent::OutputText(OutputTextContent {
                        text: format!("msg{i}-fragment{j} "),
                    })
                })
                .collect();
            OutputItem::Message(MessageOutput {
                id: format!("m{i}"),
                role: Role::Assistant,
                content,
            })
        })
        .collect();
    ResponsesResponse {
        id: "resp_bench".into(),
        status: "completed".into(),
        model: "test".into(),
        output,
        usage: None,
        incomplete_details: None,
    }
}

fn bench_extract_reply(c: &mut Criterion) {
    let small = make_response(1, 1);
    let medium = make_response(8, 4);
    let large = make_response(32, 16);

    c.bench_function("extract_reply/small", |b| {
        b.iter(|| extract_reply(black_box(&small)));
    });
    c.bench_function("extract_reply/medium", |b| {
        b.iter(|| extract_reply(black_box(&medium)));
    });
    c.bench_function("extract_reply/large", |b| {
        b.iter(|| extract_reply(black_box(&large)));
    });
}

criterion_group!(benches, bench_extract_reply);
criterion_main!(benches);
