#!/usr/bin/env python3
"""Generate large query-string corpora for seed-based tests.

The cases draw inspiration from public test suites and guidance:
- WHATWG URL Standard & HTML form encoding algorithms
- WPT urltestdata fixtures
- RFC 3986 / 3987 examples
- Node.js URLSearchParams and querystring edge cases
- OWASP HTTP Parameter Pollution guidance
- PayloadsAllTheThings URL payload samples

Running this script rewrites the JSON corpora consumed by tests in
`tests/data/query_allow.json`, `tests/data/query_reject.json`, and
`tests/data/query_roundtrip.json`.
"""
from __future__ import annotations

import json
import itertools
from pathlib import Path
from typing import Dict, List, Optional

ROOT = Path(__file__).resolve().parents[1]
ALLOW_PATH = ROOT / "tests" / "data" / "query_allow.json"
REJECT_PATH = ROOT / "tests" / "data" / "query_reject.json"
ROUNDTRIP_PATH = ROOT / "tests" / "data" / "query_roundtrip.json"
DATA_DIR = ALLOW_PATH.parent


Case = Dict[str, object]
RoundTripCase = Dict[str, object]


def add_case(collection: List[Case], name: str, query: str, expect: str,
             *, options: Optional[Dict[str, object]] = None) -> None:
    case: Case = {"name": name, "input": query, "expect": expect}
    if options:
        case["options"] = options
    collection.append(case)


def add_roundtrip_case(
    collection: List[RoundTripCase],
    name: str,
    query: str,
    *,
    parse_options: Optional[Dict[str, object]] = None,
    stringify_options: Optional[Dict[str, object]] = None,
    normalized: Optional[str] = None,
) -> None:
    case: RoundTripCase = {"name": name, "query": query}
    if parse_options:
        case["parse_options"] = parse_options
    if stringify_options:
        case["stringify_options"] = stringify_options
    if normalized is not None:
        case["normalized"] = normalized
    collection.append(case)


def extend_complex_allow_cases(cases: List[Case]) -> None:
    names = ["홍길동", "이몽룡", "성춘향", "임꺽정", "장보고"]
    cities = ["Seoul", "Busan", "Incheon", "Daegu", "Daejeon"]
    hobbies = [["coding", "music"], ["reading", "travel"], ["gaming", "hiking"], ["photography", "cooking"], ["swimming", "climbing"]]

    for idx in range(5):
        segments = [
            (f"profile{idx}[name]", names[idx]),
            (f"profile{idx}[address][city]", cities[idx]),
            (f"profile{idx}[address][postal]", f"0{idx}234"),
            (f"profile{idx}[preferences][newsletter]", "true"),
            (f"profile{idx}[links][github]", "https%3A%2F%2Fexample.com"),
        ]
        for hobby in hobbies[idx]:
            segments.append((f"profile{idx}[hobbies][]", hobby))
        query = "&".join(f"{key}={value}" for key, value in segments)
        add_case(cases, f"complex_profile_object_array_{idx}", query, "ok")

    for idx in range(4):
        segments = [
            (f"order{idx}[customer][name]", names[idx]),
            (f"order{idx}[customer][email]", f"user{idx}%40example.com"),
            (f"order{idx}[meta][currency]", "KRW"),
            (f"order{idx}[meta][notes]", "priority%20delivery"),
        ]
        for item_idx, suffix in enumerate(["A", "B"]):
            segments.append((f"order{idx}[items][{item_idx}][sku]", f"SKU{idx}{suffix}"))
            segments.append((f"order{idx}[items][{item_idx}][qty]", str(item_idx + 1)))
            segments.append((f"order{idx}[items][{item_idx}][attributes][color]", ["red", "blue"][item_idx]))
        query = "&".join(f"{key}={value}" for key, value in segments)
        add_case(cases, f"complex_order_nested_objects_{idx}", query, "ok")

    for idx in range(3):
        matrix_segments = []
        for row in range(2):
            for col in range(2):
                matrix_segments.append((f"matrix{idx}[data][{row}][{col}]", str((row + 1) * (col + 2) * (idx + 1))))
        matrix_segments.extend([
            (f"matrix{idx}[meta][determinant]", str((idx + 2) * 3)),
            (f"matrix{idx}[meta][symmetric]", "false" if idx % 2 else "true"),
        ])
        query = "&".join(f"{key}={value}" for key, value in matrix_segments)
        add_case(cases, f"complex_matrix_numeric_{idx}", query, "ok")

    for idx in range(4):
        segments = [
            (f"config{idx}[services][0][name]", "auth"),
            (f"config{idx}[services][0][url]", "https%3A%2F%2Fauth.example.com"),
            (f"config{idx}[services][0][retries]", "3"),
            (f"config{idx}[services][1][name]", "billing"),
            (f"config{idx}[services][1][url]", "https%3A%2F%2Fbill.example.com"),
            (f"config{idx}[limits][max_connections]", str(10 * (idx + 1))),
            (f"config{idx}[limits][timeout]", str(30 + idx)),
            (f"config{idx}[features][beta][]", "new-dashboard"),
            (f"config{idx}[features][beta][]", "ai-suggest"),
            (f"config{idx}[flags][strict_mode]", "1" if idx % 2 == 0 else "0"),
        ]
        query = "&".join(f"{key}={value}" for key, value in segments)
        add_case(cases, f"complex_config_services_{idx}", query, "ok")


def extend_complex_reject_cases(cases: List[Case]) -> None:
    add_case(
        cases,
        "conflict_array_object_0",
        "mix_0[]=on&mix_0[flag]=true",
        "duplicate_key",
    )

    add_case(
        cases,
        "conflict_numeric_scalar_0",
        "node_0[0]=a&node_0[key]=b",
        "duplicate_key",
    )

    add_case(
        cases,
        "nested_scalar_override_0",
        "profile_0[name]=A&profile_0=override",
        "duplicate_key",
    )

    add_case(
        cases,
        "deep_non_contiguous_index_0",
        "grid_0[0][0]=a&grid_0[0][2]=c",
        "duplicate_key",
    )

    add_case(
        cases,
        "array_index_reset_conflict_0",
        "items_0[]=a&items_0[1]=b",
        "duplicate_key",
    )


def build_allow_cases() -> List[Case]:
    cases: List[Case] = []

    # Keep historical seed coverage (hand-crafted smoke tests)
    base_allow = [
        {"name": "simple_ascii", "input": "foo=bar", "expect": "ok"},
        {"name": "unicode_literal", "input": "emoji=😀", "expect": "ok"},
        {"name": "percent_encoded_unicode", "input": "name=%E6%9D%8E%E9%9B%84", "expect": "ok"},
        {"name": "nested_arrays_unicode", "input": "items[0]=苹果&items[1]=バナナ", "expect": "ok"},
        {
            "name": "form_plus_spaces",
            "input": "note=hello+world",
            "expect": "ok",
            "options": {"space_as_plus": True},
        },
        {"name": "empty_value", "input": "empty=", "expect": "ok"},
        {
            "name": "mixed_percent_and_literal",
            "input": "greeting=%E3%81%AF%E3%82%8D&planet=world",
            "expect": "ok",
        },
    ]
    cases.extend(base_allow)

    key_patterns = [
        {"slug": "rfc3986_alpha", "template": "alpha{idx}"},
        {"slug": "rfc3986_unreserved", "template": "token_{idx}-_.~"},
        {"slug": "rfc3986_subdelim", "template": "sub!$&'()*{idx}"},
        {"slug": "whatwg_form_simple", "template": "field_{idx}"},
    {"slug": "whatwg_form_array", "template": "items_{idx}[0]"},
    {"slug": "php_parse_str_nested", "template": "user_{idx}[name]"},
        {"slug": "node_urlsearchparams", "template": "nodeParam{idx}"},
    {"slug": "qs_js_style", "template": "qs{idx}[]"},
        {"slug": "unicode_hangul_key", "template": "키{idx}"},
        {"slug": "unicode_cjk_key", "template": "键{idx}"},
        {"slug": "emoji_key", "template": "emoji{idx}🚀"},
        {"slug": "owasp_security_key", "template": "sec-{idx}"},
    ]

    value_patterns = [
        {"slug": "ascii_word", "value": "value"},
        {"slug": "ascii_dash", "value": "foo-bar"},
        {"slug": "ascii_tilde", "value": "tilde~value"},
        {"slug": "numeric", "value": "1234567890"},
        {"slug": "percent_utf8_tokyo", "value": "%E6%9D%B1%E4%BA%AC"},
        {"slug": "percent_utf8_hangul", "value": "%ED%95%9C%EA%B8%80"},
        {"slug": "percent_reserved", "value": "%26%3D%23%3F"},
        {"slug": "unicode_cjk", "value": "東京"},
        {"slug": "unicode_hangul", "value": "한글"},
        {"slug": "unicode_emoji", "value": "🚀"},
        {"slug": "form_plus", "value": "hello+world", "options": {"space_as_plus": True}},
        {"slug": "long_ascii", "value": "l" * 32},
    ]

    fanout = 10  # 12 key patterns * 12 value patterns * 10 = 1,440 cases

    for idx in range(fanout):
        for key_pat in key_patterns:
            key = key_pat["template"].format(idx=idx)
            for val_pat in value_patterns:
                options = {}
                if "options" in val_pat:
                    options.update(val_pat["options"])
                if "options" in key_pat:
                    options.update(key_pat["options"])
                query = f"{key}={val_pat['value']}"
                name = f"{key_pat['slug']}__{val_pat['slug']}__{idx}"
                add_case(cases, name, query, "ok", options=options or None)

    # Multi-parameter scenarios inspired by WHATWG & Node.js fixtures
    for param_count in [2, 3, 4, 5, 8, 16, 32]:
        segments = [f"param{i}=v{i}" for i in range(param_count)]
        query = "&".join(segments)
        add_case(
            cases,
            f"whatwg_multi_param_exact_{param_count}",
            query,
            "ok",
            options={"max_params": param_count},
        )

    # Max length boundaries (allow branch)
    for limit in [16, 32, 64, 128, 256, 512, 1024]:
        value = "a" * (limit - 5 if limit > 5 else limit)
        query = f"len={value}"
        add_case(
            cases,
            f"rfc3986_length_exact_{limit}",
            query,
            "ok",
            options={"max_length": len(query)},
        )

    # Max depth boundaries just within limits
    add_case(cases, "whatwg_depth_flat_default", "flat=ok", "ok")
    for limit in [1, 2, 3, 4]:
        key = "root" + "".join("[branch]" for _ in range(limit))
        query = f"{key}=ok"
        add_case(
            cases,
            f"whatwg_depth_within_{limit}",
            query,
            "ok",
            options={"max_depth": limit},
        )

    # Space-plus behaviour for both modes
    add_case(
        cases,
        "form_mode_plus_decodes",
        "comment=form+value",
        "ok",
        options={"space_as_plus": True},
    )
    add_case(
        cases,
        "strict_mode_plus_literal",
        "comment=form+value",
        "ok",
        options={"space_as_plus": False},
    )

    extend_complex_allow_cases(cases)

    cases.sort(key=lambda item: item["name"])
    return cases


def build_reject_cases() -> List[Case]:
    cases: List[Case] = []

    base_reject = [
        {"name": "duplicate_scalar", "input": "foo=1&foo=2", "expect": "duplicate_key"},
        {
            "name": "mixed_scalar_and_nested",
            "input": "user=name&user[age]=30",
            "expect": "duplicate_key",
        },
        {
            "name": "mixed_append_numeric",
            "input": "items[]=1&items[0]=2",
            "expect": "duplicate_key",
        },
        {
            "name": "non_contiguous_index",
            "input": "items[0]=x&items[2]=z",
            "expect": "duplicate_key",
        },
        {
            "name": "invalid_percent_encoding",
            "input": "broken=%E4%ZZ",
            "expect": "invalid_percent_encoding",
        },
        {
            "name": "control_character",
            "input": "bad=%00",
            "expect": "invalid_character",
        },
    ]
    cases.extend(base_reject)

    # Too many parameters
    for limit in [1, 2, 4, 8, 16, 32]:
        actual = limit + 1
        segments = [f"p{i}=v{i}" for i in range(actual)]
        query = "&".join(segments)
        add_case(
            cases,
            f"too_many_params_limit_{limit}",
            query,
            "too_many_parameters",
            options={"max_params": limit},
        )

    # Input length exceeded
    for limit in [8, 16, 32, 64, 128, 256, 512]:
        query = "len=" + ("x" * (limit + 1))
        add_case(
            cases,
            f"input_too_long_{limit}",
            query,
            "input_too_long",
            options={"max_length": limit},
        )

    # Depth exceeded
    for limit in [1, 2, 3, 4]:
        key = "root" + "".join("[branch]" for _ in range(limit + 1))
        query = f"{key}=deep"
        add_case(
            cases,
            f"depth_exceeded_limit_{limit}",
            query,
            "depth_exceeded",
            options={"max_depth": limit},
        )

    # Unmatched brackets from WPT / Node oddities
    unmatched_keys = ["user]", "[broken", "arr[0", "key]extra"]
    for idx, key in enumerate(unmatched_keys):
        add_case(
            cases,
            f"unmatched_bracket_{idx}",
            f"{key}=1",
            "unmatched_bracket",
        )

    # Unexpected question mark inside query
    add_case(
        cases,
        "unexpected_question_mark",
        "foo?bar=baz",
        "unexpected_question_mark",
    )

    # Invalid UTF-8 sequences (valid percent-encoding but invalid UTF-8)
    invalid_utf8_payloads = ["%80", "%C1%BF", "%F0%80%80%80"]
    for idx, payload in enumerate(invalid_utf8_payloads):
        add_case(
            cases,
            f"invalid_utf8_{idx}",
            f"bin={payload}",
            "invalid_utf8",
        )

    # OWASP / PayloadsAllTheThings style control payloads
    suspicious_inputs = [
        ("owasp_null_mix", "a=1&b=%00%00&c=2", "invalid_character"),
        ("payloads_control_newline", "inject=foo%0Abar", "invalid_character"),
        ("payloads_control_tab", "inject=foo%09bar", "invalid_character"),
    ]
    for name, query, expect in suspicious_inputs:
        add_case(cases, name, query, expect)

    # Duplicate detection with mixed array syntax
    add_case(
        cases,
        "duplicate_append_and_index",
        "arr[]=1&arr[]=2&arr[0]=override",
        "duplicate_key",
    )

    extend_complex_reject_cases(cases)

    cases.sort(key=lambda item: item["name"])
    return cases


def build_roundtrip_cases() -> List[RoundTripCase]:
    cases: List[RoundTripCase] = []

    base_cases = [
        {"name": "simple_roundtrip", "query": "a=1&b=2"},
        {
            "name": "space_plus_roundtrip",
            "query": "note=hello+world",
            "parse_options": {"space_as_plus": True},
            "stringify_options": {"space_as_plus": True},
        },
        {
            "name": "unicode_roundtrip",
            "query": "name=%E6%9D%8E%E9%9B%84",
        },
        {
            "name": "nested_array_roundtrip",
            "query": "items[0]=alpha&items[1]=beta",
        },
        {
            "name": "config_roundtrip",
            "query": "settings[flags][strict]=true&settings[limits][max]=10",
        },
        {
            "name": "sorted_alpha_roundtrip",
            "query": "alpha=1&beta=2&gamma=3",
        },
    ]

    for case in base_cases:
        add_roundtrip_case(
            cases,
            case["name"],
            case["query"],
            parse_options=case.get("parse_options"),
            stringify_options=case.get("stringify_options"),
            normalized=case.get("normalized", case["query"]),
        )

    key_patterns = [
        {"slug": "alpha", "template": "alpha{idx}"},
        {"slug": "token", "template": "token_{idx}"},
        {"slug": "flat_dash", "template": "flat-key-{idx}"},
        {"slug": "nested_user", "template": "user{idx}[name]"},
        {"slug": "nested_order", "template": "order{idx}[items][0][sku]"},
        {"slug": "array_index", "template": "items{idx}[0]"},
        {"slug": "config_limit", "template": "config{idx}[limits][max]"},
        {"slug": "matrix_value", "template": "matrix{idx}[data][0][0]"},
        {"slug": "env_var", "template": "env_{idx}"},
        {"slug": "profile_pref", "template": "profile{idx}[preferences][newsletter]"},
    ]

    value_patterns = [
        {"slug": "simple", "value": "value"},
        {"slug": "numeric", "value": "123456"},
        {"slug": "dash", "value": "foo-bar"},
        {"slug": "tilde", "value": "tilde~value"},
        {"slug": "percent_tokyo", "value": "%E6%9D%B1%E4%BA%AC"},
        {"slug": "percent_hangul", "value": "%ED%95%9C%EA%B8%80"},
        {"slug": "percent_reserved", "value": "%26%3D%23%3F"},
        {"slug": "long16", "value": "l" * 16},
        {
            "slug": "space_plus",
            "value": "hello+world",
            "space_as_plus": True,
        },
        {"slug": "snake", "value": "snake_value"},
    ]

    fanout = 10  # 10 key patterns * 10 value patterns * 10 fanout = 1,000 cases

    for idx in range(fanout):
        for key_pat in key_patterns:
            key = key_pat["template"].format(idx=idx)
            for val_pat in value_patterns:
                parse_options: Dict[str, object] = {}
                stringify_options: Dict[str, object] = {}
                if val_pat.get("space_as_plus"):
                    parse_options["space_as_plus"] = val_pat["space_as_plus"]
                    stringify_options["space_as_plus"] = val_pat["space_as_plus"]
                query = f"{key}={val_pat['value']}"
                add_roundtrip_case(
                    cases,
                    f"pair_{key_pat['slug']}__{val_pat['slug']}__{idx}",
                    query,
                    parse_options=parse_options or None,
                    stringify_options=stringify_options or None,
                    normalized=query,
                )

    cases.sort(key=lambda item: item["name"])
    return cases


def main() -> None:
    allow_cases = build_allow_cases()
    reject_cases = build_reject_cases()
    roundtrip_cases = build_roundtrip_cases()

    DATA_DIR.mkdir(parents=True, exist_ok=True)

    ALLOW_PATH.write_text(json.dumps(allow_cases, indent=2, ensure_ascii=False) + "\n", encoding="utf-8")
    REJECT_PATH.write_text(json.dumps(reject_cases, indent=2, ensure_ascii=False) + "\n", encoding="utf-8")
    ROUNDTRIP_PATH.write_text(json.dumps(roundtrip_cases, indent=2, ensure_ascii=False) + "\n", encoding="utf-8")
    print(f"Wrote {len(allow_cases)} allow cases -> {ALLOW_PATH}")
    print(f"Wrote {len(reject_cases)} reject cases -> {REJECT_PATH}")
    print(f"Wrote {len(roundtrip_cases)} roundtrip cases -> {ROUNDTRIP_PATH}")


if __name__ == "__main__":
    main()
