use bunner_qs_rs::{ParseOptions, Qs, StringifyOptions};
use serde_json::Value;
use std::thread;

fn make_query() -> &'static str {
    "profile[name]=Ada&profile[contacts][email]=ada@example.com&tags[]=rust&tags[]=serde"
}

#[test]
fn should_roundtrip_parse_and_stringify_concurrently_without_panics() {
    let query = make_query().to_string();
    let threads = 8;
    let iterations = 100;

    let handles: Vec<_> = (0..threads)
        .map(|_| {
            let query = query.clone();
            thread::spawn(move || {
                let qs = Qs::new()
                    .with_parse(ParseOptions::default())
                    .expect("parse options should validate")
                    .with_stringify(StringifyOptions::default())
                    .expect("stringify options should validate");

                for _ in 0..iterations {
                    let parsed: Value = qs.parse(&query).expect("parse should succeed");
                    assert_eq!(parsed["profile"]["name"], "Ada");

                    let encoded = qs.stringify(&parsed).expect("stringify should succeed");
                    let reparsed: Value =
                        qs.parse(&encoded).expect("roundtrip parse should succeed");
                    assert_eq!(parsed, reparsed);
                }
            })
        })
        .collect();

    for handle in handles {
        handle.join().expect("thread should not panic");
    }
}
