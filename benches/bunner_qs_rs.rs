mod scenarios;

use bunner_qs_rs::parsing::api::parse;
use bunner_qs_rs::stringify::api::stringify;
use criterion::{Criterion, criterion_group, criterion_main};
use serde_json::Value;
use std::hint::black_box;

use scenarios::{
    Scenario, max_bracket_depth, scenario_extreme, scenario_high, scenario_medium, scenario_simple,
};

fn bench_parse_simple(c: &mut Criterion) {
    run_parse_bench(c, "parse/simple_struct", scenario_simple());
}

fn bench_parse_medium(c: &mut Criterion) {
    run_parse_bench(c, "parse/medium_struct", scenario_medium());
}

fn bench_parse_high(c: &mut Criterion) {
    run_parse_bench(c, "parse/high_struct", scenario_high());
}

fn bench_parse_extreme(c: &mut Criterion) {
    run_parse_bench(c, "parse/extreme_struct", scenario_extreme());
}
fn bench_stringify_simple(c: &mut Criterion) {
    run_stringify_bench(c, "stringify/simple_struct", scenario_simple());
}

fn bench_stringify_medium(c: &mut Criterion) {
    run_stringify_bench(c, "stringify/medium_struct", scenario_medium());
}

fn bench_stringify_high(c: &mut Criterion) {
    run_stringify_bench(c, "stringify/high_struct", scenario_high());
}

fn bench_stringify_extreme(c: &mut Criterion) {
    run_stringify_bench(c, "stringify/extreme_struct", scenario_extreme());
}
fn run_parse_bench(c: &mut Criterion, name: &str, scenario: Scenario) {
    let Scenario {
        payload,
        query,
        parse_options,
        stringify_options,
        max_depth,
    } = scenario;

    let _stringify_options = stringify_options;

    let baseline: Value = parse(&query, &parse_options).expect("baseline parse should succeed");
    assert_eq!(
        baseline, payload,
        "baseline parse output should match payload"
    );

    let depth = max_bracket_depth(&query);
    assert!(
        depth <= max_depth,
        "scenario depth {} exceeded expected max {}",
        depth,
        max_depth
    );

    let query_owned = query;
    let options_owned = parse_options;

    c.bench_function(name, move |b| {
        b.iter(|| {
            let parsed: Value =
                parse(black_box(query_owned.as_str()), &options_owned).expect("parse");
            black_box(parsed);
        });
    });
}
fn run_stringify_bench(c: &mut Criterion, name: &str, scenario: Scenario) {
    let Scenario {
        payload,
        query,
        parse_options,
        stringify_options,
        max_depth,
    } = scenario;

    let encoded = stringify(&payload, &stringify_options).expect("stringify baseline");
    assert_eq!(
        encoded, query,
        "baseline stringify should match calibrated query"
    );

    let reparsed: Value = parse(&encoded, &parse_options).expect("roundtrip parse");
    assert_eq!(reparsed, payload, "roundtrip parse should match payload");

    let depth = max_bracket_depth(&encoded);
    assert!(
        depth <= max_depth,
        "stringify depth {} exceeded expected max {}",
        depth,
        max_depth
    );

    let payload_for_bench = payload;
    let options_owned = stringify_options;

    c.bench_function(name, move |b| {
        b.iter(|| {
            let encoded =
                stringify(black_box(&payload_for_bench), &options_owned).expect("stringify");
            black_box(encoded);
        });
    });
}
criterion_group!(
    benches,
    bench_parse_simple,
    bench_parse_medium,
    bench_parse_high,
    bench_parse_extreme,
    bench_stringify_simple,
    bench_stringify_medium,
    bench_stringify_high,
    bench_stringify_extreme
);
criterion_main!(benches);
