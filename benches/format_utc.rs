// file: benches/format_utc.rs
// description: criterion benchmark for tools::format_utc

use criterion::{Criterion, black_box, criterion_group, criterion_main};
use gpt55_chat::tools::format_utc;

fn bench_format_utc(c: &mut Criterion) {
    c.bench_function("format_utc/epoch", |b| {
        b.iter(|| format_utc(black_box(0)));
    });
    c.bench_function("format_utc/2023", |b| {
        b.iter(|| format_utc(black_box(1_700_000_000)));
    });
    c.bench_function("format_utc/max_year", |b| {
        b.iter(|| format_utc(black_box(253_402_300_799)));
    });
}

criterion_group!(benches, bench_format_utc);
criterion_main!(benches);
