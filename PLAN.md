# bunner_qs Port Plan

A structured plan to port the JavaScript `qs` library to Rust with high fidelity, performance, and safe API ergonomics.

## 1. Goals
- Functional parity (configurable) with `qs` for parsing and stringifying query strings.
- Memory safety, predictable performance, and allocation minimization.
- Extensible architecture for future/optional features (charsets, custom encoders, duplicate strategies).
- Idiomatic Rust API (Result types, iterators, enums) while preserving conceptual mapping to original behaviors.

## 2. Core Feature Domains (Revised)
1. Parsing nested structures (bracket/dot notation)
2. Arrays handling (indices, brackets, repeated keys, comma format)
3. Duplicate key strategies (combine, first, last)
4. Limits: depth, parameterLimit, arrayLimit, strictDepth, throwOnLimitExceeded
5. Character encoding / decoding (UTF-8 default, iso-8859-1, sentinel detection)
6. Null/empty handling (strictNullHandling, skipNulls)
7. Dots vs brackets (allowDots, decodeDotInKeys, encodeDotInKeys)
8. Sparse arrays (allowSparse)
9. Empty arrays (`allowEmptyArrays`)
10. Charset sentinel `utf8=✓` handling (and its priority over `charset` option)
11. Filter & sort hooks (custom ordering / inclusion by key list or function)
12. Custom encoder / decoder functions (strategy injection)
13. Array format variants (indices, brackets, repeat, comma + commaRoundTrip)
14. Plain objects mode (`plainObjects`) -> In Rust, `IndexMap` is already safe from prototype pollution, so this is a conceptual mapping.
15. Prototype pollution defense (`allowPrototypes` option, default `false`)
16. Interpret numeric entities (`interpretNumericEntities`)
17. Numeric / boolean / null primitive interpretation (external in JS; optional feature in Rust)
18. Delimiter customization (default `&`, support for single char alternatives. Regex deferred.)
19. Parameter prefix ignoring (`ignoreQueryPrefix`)
20. Add query prefix on stringify (`addQueryPrefix`)
21. Strict array index limit for large indices
22. Date serialization customization (`serializeDate`)
23. Filter function key-based transformation (supports array indices)
24. Skip null, keep empty string semantics
25. Comma parsing for arrays (`comma: true`)
26. Special handling for `depth: 0` or `depth: false` (disables nesting)

## 3. Phased Delivery
### Phase 1 (MVP)
- Parse basic bracket nesting + arrays (indices + implicit push)
- Stringify basic map/array structure
- Options: depth, parameter_limit, array_limit, parse_arrays, allow_dots (basic), add_query_prefix, ignore_query_prefix, encode (on/off)
- Duplicate strategy: combine default (Vec accumulation)
- Basic percent-decoding (UTF-8) / percent-encoding

### Phase 2
- Duplicate strategy variants (first, last)
- Array formats: brackets, repeat, comma
- skip_nulls, strict_null_handling
- allow_empty_arrays
- encode_values_only, encode_dot_in_keys
- sort keys (stable) with user callback or ordering enum
- filter (list-based inclusion) (function variant maybe later)

### Phase 3
- strict_depth, throw_on_limit_exceeded errors
- allow_sparse arrays
- decode_dot_in_keys
- interpret_numeric_entities
- charset & charsetSentinel (iso-8859-1 detection via feature)
- plain_objects mode (map without default trait impl differences)
- large index guard

### Phase 4
- Custom encoder / decoder injection traits
- serialize_date strategy (Formatter closure)
- plugin-like extension traits
- Performance tuning (arena alloc, smallvec usage for path segments)

## 4. Data Model
```rust
use indexmap::IndexMap;

#[derive(Debug, Clone, PartialEq)]
pub enum QsValue {
    Null,              // explicit null (distinguished if strict_null_handling)
    String(String),    // scalar
    Array(Vec<QsValue>),
    Object(IndexMap<String, QsValue>), // ordered map for stable stringify
}
```
Rationale:  
- `IndexMap` preserves insertion order (qs stable output)  
- Null vs Empty string distinction needed when strict_null_handling enabled  
- Booleans / numbers are kept as String (parity) unless an optional feature `coerce_primitives` is enabled later.

### Path Representation
During parsing, keys like `foo[bar][0][baz]` are tokenized into segments:  
`[Key("foo"), Key("bar"), Index(0), Key("baz")]`  

Internal enum:
```rust
enum PathSegment {
    Key(String),
    Push,        // represents [] with no index
    Index(usize),
}
```

Segments built by scanning with a small state machine.  

## 5. Options Struct (Revised)
```rust
// Note: Box<dyn Fn... for filter/sort/serialize will be added in later phases.
pub enum Charset {
    Utf8,
    Iso88591,
}

pub struct ParseOptions {
    pub depth: Option<usize>,      // default Some(5). None or Some(0) disables nesting.
    pub parameter_limit: usize,    // default 1000
    pub array_limit: usize,        // default 20 (index guard)
    pub parse_arrays: bool,
    pub allow_dots: bool,
    pub strict_depth: bool,
    pub throw_on_limit_exceeded: bool,
    pub ignore_query_prefix: bool,
    pub allow_prototypes: bool,    // default false
    pub allow_sparse: bool,
    pub plain_objects: bool,       // semantic flag for parity; IndexMap already safe
    pub interpret_numeric_entities: bool,
    pub charset: Charset,
    pub charset_sentinel: bool,
    pub duplicates: DuplicateStrategy, // combine | first | last
    pub decode_dot_in_keys: bool,
    pub allow_empty_arrays: bool,
    pub comma: bool,               // for parsing 'a=b,c'
    pub delimiter: Option<Delimiter>,
    pub decoder: Option<DecodeFn>, // hook for custom decoding (phase 4)
    pub strict_null_handling: bool, // default false; true => null for bare keys, false => ''
}

pub enum Delimiter {
    Str(String),
    Regex(String), // compiled form behind optional "regex" feature
}

pub type DecodeFn = Box<dyn Fn(&str, Charset, ValueKind) -> String + Send + Sync>;

pub enum ValueKind { Key, Value }

pub enum DuplicateStrategy { Combine, First, Last }

impl Default for ParseOptions { /* ... */ }

pub struct StringifyOptions {
    pub add_query_prefix: bool,
    pub array_format: ArrayFormat, // Indices | Brackets | Repeat | Comma
    pub comma_round_trip: bool,
    pub skip_nulls: bool,
    pub strict_null_handling: bool,
    pub encode: bool,
    pub encode_values_only: bool,
    pub encode_dot_in_keys: bool,
    pub charset: Charset,
    pub charset_sentinel: bool,
    pub format: Format,
    pub serialize_date: Option<SerializeDateFn>,
    pub sort: Option<SortFn>,
    pub filter: Option<Filter>,
    pub delimiter: Option<char>,
    pub encoder: Option<EncodeFn>,
    pub allow_dots: bool,          // default false; true => use dot notation for nesting
    pub allow_empty_arrays: bool,  // default false; true => emit [] for empty arrays
}

pub enum ArrayFormat { Indices, Brackets, Repeat, Comma }

pub enum Format { Rfc3986, Rfc1738 }

pub type SerializeDateFn = Box<dyn Fn(&DateTime<Utc>) -> String + Send + Sync>;
pub type SortFn = Box<dyn Fn(&str, &str) -> std::cmp::Ordering + Send + Sync>;
pub type EncodeFn = Box<dyn Fn(&str, Charset, ValueKind) -> String + Send + Sync>;

pub enum Filter {
    Keys(Vec<String>),
    Function(Box<dyn Fn(&str, &QsValue) -> Option<QsValue> + Send + Sync>),
}
```

## 6. Parsing Algorithm (Revised)
1. Optionally strip leading '?' if `ignore_query_prefix`.
2. Split by delimiter (`delimiter` option or default '&'). Hard cap at `parameter_limit`.
3. For each pair: split on first '='.
   - Determine active charset: when `charset_sentinel` is enabled and a `utf8=✓` (or `utf8=%E2%9C%93`) parameter appears, switch to UTF-8 regardless of configured charset.
   - Decode key/value with `decoder` if provided; otherwise run built-in percent-decoder with numeric-entity handling controlled by `interpret_numeric_entities`.
   - If no '=' found and `strict_null_handling` is true, treat as key with null value; otherwise use empty string.
    - Sentinel pairs (`utf8=✓`, `utf8=&#10003;`) are consumed for charset detection and **never** inserted into the resulting map (skip entirely after detection).
4. If `depth` is `None` (or 0), treat keys literally and do not parse nesting.
5. Tokenize key respecting options:
    - Scan base segment until '[' or '.' (provided `allow_dots` is true and `decode_dot_in_keys` controls literal handling).
    - For `[...]` groups: empty => `Push`; numeric => `Index(n)` (subject to `array_limit` guard); other => `Key(string)`.
    - When `parse_arrays` is false, treat `[]` and numeric segments as object keys rather than array semantics.
6. Apply comma handling before insertion when `comma` is enabled and the value is a UTF-8 string: split by ',' with optional trimming; respect `allow_empty_arrays` when producing empty vectors.
7. Insert into tree using iterative merge logic (`utils::merge`):
    - Start with root `IndexMap<String, QsValue>`.
    - Walk segments, creating `Object`/`Array` nodes as needed, observing `plain_objects` and `allow_sparse` behaviors.
    - Reject prototype pollution attempts (`allow_prototypes: false`).
8. At leaf nodes, apply `duplicates` strategy (`Combine`, `First`, `Last`).
9. After building the tree:
    - If `allow_sparse` is false, compact arrays by removing holes and re-index sequentially.
    - If `allow_empty_arrays` is false, collapse empty arrays back to empty strings.
    - Convert array-like objects to plain objects when index keys exceed `array_limit` or are non-sequential (mirrors `utils.merge` coercion logic) unless `allow_sparse` preserves holes.

Errors: depth overflow, parameter limit, array index overflow, comma splitting overflow, invalid encoding, prototype key rejection.

## 7. Stringify Algorithm (Phase 1)
1. Traverse structure depth-first to produce `(key_path, value)` pairs.
    - Apply `filter` early: key whitelist (`Filter::Keys`) or function transforming/removing entries.
    - Apply `sort` on object keys prior to recursion when provided.
    - Detect cycles (side-channel pattern in JS). Rust implementation will track raw pointer identity or `Rc`/`Weak` IDs and return an error (`QsError::CyclicValue`) instead of panic.
2. Determine array serialization per `array_format`:
    - `Indices`: `a[0]=...&a[1]=...`
    - `Brackets`: `a[]=...`
    - `Repeat`: `a=...&a=...`
    - `Comma`: join values with `,`; when `comma_round_trip` is true, also emit explicit array indexes to ensure parse fidelity.
3. Format key paths using bracket notation. When `allow_dots` is true, use dot notation for nesting instead of brackets.
4. Render null/undefined values:
   - `strict_null_handling`: output bare key (`foo` instead of `foo=`) when value is `Null`.
   - Otherwise treat `Null` as empty string unless `skip_nulls` removes it.
5. When `allow_empty_arrays` is true, emit `foo[]` for empty arrays; otherwise omit.
6. Skip entries when `skip_nulls` is true and value is null; omit undefined entirely.
7. Encode strings:
    - Select charset per options (`charset`, overridden by sentinel emission).
    - Invoke custom `encoder` when supplied; otherwise use built-in RFC3986 encoder or RFC1738 replacement for spaces based on `format` and `encode` flags.
    - Respect `encode_values_only` by leaving key path untouched.
8. When emitting charset sentinel, include `utf8=✓` as first parameter.
9. Join encoded pairs with configured delimiter (default `&`) and prefix '?' when `add_query_prefix` is true.

Edge nuances:
- Filter array form: when `filter` is a list, only listed keys/indices survive traversal (mirrors JS behavior—indices and nested segment names included).
- Back-compat `indices` flag in JS maps to `array_format` in Rust; we will not expose a deprecated alias—documented as a migration note.
- `comma_round_trip` with a single-element array emits `[]` suffix to force parse back into array; included in algorithm.
- Cycle detection triggers error early before partial serialization.
- Binary values (`Vec<u8>` or byte slices): attempt UTF-8 decoding first; on failure, percent-encode each byte to mirror Buffer behavior in JS. Explicit opt-in feature later for raw binary passthrough.

## 8. Module Layout (Revised)
```
src/
  lib.rs
  value.rs        (QsValue, PathSegment)
  options.rs      (ParseOptions, StringifyOptions, enums)
  parser.rs       (parse_query, tokenizer, insertion logic)
  stringify.rs    (stringify_internal)
  error.rs        (Error enum)
  utils.rs        (merge, compact, and other helper logic)
```

## 9. Error Type
```rust
#[derive(Debug)]
pub enum QsError {
    DepthExceeded { max: usize },
    ParameterLimitExceeded { limit: usize },
    ArrayIndexTooLarge { index: usize, limit: usize },
    InvalidEncoding(String),
    CyclicValue,
}
pub type Result<T> = std::result::Result<T, QsError>;
```

## 10. Public API (Phase 1)
```rust
pub fn parse(input: &str, opts: &ParseOptions) -> Result<QsValue>;

pub fn stringify(value: &QsValue, opts: &StringifyOptions) -> Result<String>;
```

Convenience wrappers with `Default` options.

## 11. Performance Considerations
- Reuse buffers for percent-decoding (small stack buffer + fallback to String).
- SmallVec for path segments (common depth <= 5).
- Avoid cloning strings by slicing & copying only when needed.
- Iterative instead of recursive building to avoid deep call overhead.

## 12. Testing Strategy
- Snapshot tests for stringify forms.
- Round trip for nested objects, arrays, null handling.
- Limits: depth, parameterLimit, arrayLimit edge cases.
- Duplicate strategies matrix.
- Compare outputs to known `qs` JavaScript output fixtures (manually curated expected strings).
- Charset sentinel scenarios: verify UTF-8/ISO-8859-1 switching and confirm sentinel params are absent from parsed result.
- Binary payloads: assert UTF-8 buffers round-trip as strings and non-UTF-8 bytes are percent-encoded to match JS Buffer behavior.

## 13. Future Extensions
- Feature flags: `coerce_primitives`, `charset`, `audit`, `serde` interop.
- Serde integration: implement `Deserialize`/`Serialize` bridging with `QsValue`.
- Streaming parser for very large query strings.
- Regex delimiter support (parity with `utils.isRegExp` handling in JS parse options) — likely via `regex` crate behind feature.
- Optional deprecated alias `indices` (boolean) that maps to `ArrayFormat::Indices` vs `Repeat` for migration help.

## 14. Implementation Order Summary
1. Data model + options + error
2. Basic parser + tests
3. Basic stringify + tests
4. Duplicate strategies
5. Array formats
6. Null / skip behaviors
7. Sorting / filtering
8. Charset + sentinel
9. Sparse arrays / strict depth / throw behaviors
10. Custom hooks & serde integration

## 15. Open Questions
- Should numeric entities be behind a feature? (Yes, low demand)  
- Provide both `IndexMap` and `BTreeMap` option? (Maybe overkill initially)  
- Multi-delimiter regex support? (Defer)

---
Generated initial plan; will refine as implementation progresses.
