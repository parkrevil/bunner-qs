use bunner_qs_rs::{StringifyOptions, stringify_with};
use criterion::{Criterion, criterion_group, criterion_main};
use serde::Serialize;
use std::hint::black_box;

#[derive(Serialize)]
struct SimplePair<'a> {
    username: &'a str,
    token: &'a str,
}

#[derive(Serialize)]
struct SpacedValue<'a> {
    query: &'a str,
}

fn bench_stringify_pair_ascii(c: &mut Criterion) {
    let input = SimplePair {
        username: "alice",
        token: "s3cret",
    };
    let options = StringifyOptions::default();

    c.bench_function("stringify_pair_ascii", |b| {
        b.iter(|| {
            let output = stringify_with(black_box(&input), &options).unwrap();
            black_box(output);
        });
    });
}

fn bench_stringify_pair_spaces_plus(c: &mut Criterion) {
    let input = SpacedValue {
        query: "alpha beta gamma delta",
    };
    let options = StringifyOptions::builder()
        .space_as_plus(true)
        .build()
        .expect("builder should succeed");

    c.bench_function("stringify_pair_spaces_plus", |b| {
        b.iter(|| {
            let output = stringify_with(black_box(&input), &options).unwrap();
            black_box(output);
        });
    });
}

criterion_group!(
    stringify_micro,
    bench_stringify_pair_ascii,
    bench_stringify_pair_spaces_plus
);
criterion_main!(stringify_micro);
