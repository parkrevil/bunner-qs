mod scenarios;

use bunner_qs_rs::parsing::api::parse;
use bunner_qs_rs::stringify::api::stringify;
use criterion::{Criterion, criterion_group, criterion_main};
use serde::Deserialize;
use serde_json::{Map, Value};
use std::collections::BTreeMap;
use std::hint::black_box;

use scenarios::{Scenario, scenario_extreme, scenario_high, scenario_medium, scenario_simple};

type ScenarioFactory = fn() -> Scenario;

#[derive(Clone, Debug, PartialEq, Deserialize)]
struct QsRoot {
    #[serde(flatten)]
    fields: BTreeMap<String, QsValue>,
}

#[derive(Clone, Debug, PartialEq, Deserialize)]
#[serde(untagged)]
enum QsValue {
    Map(BTreeMap<String, QsValue>),
    Seq(Vec<QsValue>),
    String(String),
    Bool(bool),
    I64(i64),
    U64(u64),
    F64(f64),
    Null,
}

impl From<QsRoot> for Value {
    fn from(root: QsRoot) -> Self {
        Value::Object(
            root.fields
                .into_iter()
                .map(|(k, v)| (k, Value::from(v)))
                .collect::<Map<String, Value>>(),
        )
    }
}

impl From<QsValue> for Value {
    fn from(value: QsValue) -> Self {
        match value {
            QsValue::Map(map) => Value::Object(
                map.into_iter()
                    .map(|(k, v)| (k, Value::from(v)))
                    .collect::<Map<String, Value>>(),
            ),
            QsValue::Seq(seq) => Value::Array(seq.into_iter().map(Value::from).collect()),
            QsValue::String(s) => Value::String(s),
            QsValue::Bool(b) => Value::Bool(b),
            QsValue::I64(n) => Value::Number(n.into()),
            QsValue::U64(n) => Value::Number(n.into()),
            QsValue::F64(n) => serde_json::Number::from_f64(n)
                .map(Value::Number)
                .unwrap_or(Value::Null),
            QsValue::Null => Value::Null,
        }
    }
}

fn bench_parse_compare(c: &mut Criterion) {
    for (label, scenario_fn) in scenarios_for_compare() {
        register_parse_benches(c, label, scenario_fn());
    }
}

fn bench_stringify_compare(c: &mut Criterion) {
    for (label, scenario_fn) in scenarios_for_compare() {
        register_stringify_benches(c, label, scenario_fn());
    }
}

fn scenarios_for_compare() -> [(&'static str, ScenarioFactory); 4] {
    [
        ("simple", scenario_simple),
        ("medium", scenario_medium),
        ("high", scenario_high),
        ("extreme", scenario_extreme),
    ]
}

fn serde_qs_config(max_depth: usize) -> serde_qs::Config {
    serde_qs::Config::new(max_depth, false)
}

fn register_parse_benches(c: &mut Criterion, label: &str, scenario: Scenario) {
    let Scenario {
        payload,
        query,
        parse_options,
        stringify_options: _,
        max_depth,
    } = scenario;

    let depth_limit = max_depth + 2;
    let serde_qs_cfg = serde_qs_config(depth_limit);

    let bunner_baseline: Value =
        parse(&query, &parse_options).expect("bunner parse baseline should succeed");
    assert_eq!(
        bunner_baseline, payload,
        "bunner baseline should equal payload"
    );

    let serde_qs_baseline: QsRoot = serde_qs_cfg
        .deserialize_str(&query)
        .expect("serde_qs baseline parse");
    let serde_qs_baseline_value: Value = serde_qs_baseline.into();
    assert_eq!(serde_qs_baseline_value, payload, "serde_qs baseline value");

    let bunner_query = query.clone();
    let bunner_opts = parse_options.clone();
    c.bench_function(&format!("bunner_qs_rs/parse/{}", label), move |b| {
        b.iter(|| {
            let parsed: Value =
                parse(black_box(bunner_query.as_str()), &bunner_opts).expect("parse");
            black_box(parsed);
        });
    });

    let serde_qs_query = query.clone();
    c.bench_function(&format!("serde_qs/parse/{}", label), move |b| {
        let cfg = serde_qs_config(depth_limit);
        b.iter(|| {
            let parsed: QsRoot = cfg
                .deserialize_str(black_box(serde_qs_query.as_str()))
                .expect("parse");
            black_box(parsed);
        });
    });
}

fn register_stringify_benches(c: &mut Criterion, label: &str, scenario: Scenario) {
    let Scenario {
        payload,
        query,
        parse_options,
        stringify_options,
        max_depth,
    } = scenario;

    let depth_limit = max_depth + 2;
    let serde_qs_cfg = serde_qs_config(depth_limit);

    let bunner_encoded = stringify(&payload, &stringify_options).expect("bunner stringify");
    assert_eq!(
        bunner_encoded, query,
        "bunner baseline encode should match query"
    );

    let bunner_roundtrip: Value =
        parse(&bunner_encoded, &parse_options).expect("bunner roundtrip parse");
    assert_eq!(bunner_roundtrip, payload, "bunner roundtrip value");

    let serde_qs_encoded = serde_qs::to_string(&payload).expect("serde_qs encode");
    let serde_qs_roundtrip: QsRoot = serde_qs_cfg
        .deserialize_str(&serde_qs_encoded)
        .expect("serde_qs roundtrip parse");
    let serde_qs_roundtrip_value: Value = serde_qs_roundtrip.into();
    assert_eq!(
        serde_qs_roundtrip_value, payload,
        "serde_qs roundtrip value"
    );

    let bunner_payload = payload.clone();
    let bunner_opts = stringify_options.clone();
    c.bench_function(&format!("bunner_qs_rs/stringify/{}", label), move |b| {
        b.iter(|| {
            let encoded = stringify(black_box(&bunner_payload), &bunner_opts).expect("stringify");
            black_box(encoded);
        });
    });

    let serde_qs_payload = payload.clone();
    c.bench_function(&format!("serde_qs/stringify/{}", label), move |b| {
        b.iter(|| {
            let encoded = serde_qs::to_string(black_box(&serde_qs_payload)).expect("stringify");
            black_box(encoded);
        });
    });
}

criterion_group!(ecosystem, bench_parse_compare, bench_stringify_compare);
criterion_main!(ecosystem);
