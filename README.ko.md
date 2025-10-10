<h1 align="center">bunner-qs-rs</h1>

<p align="center">
    <a href="https://crates.io/crates/bunner_qs_rs"><img src="https://img.shields.io/crates/v/bunner_qs_rs.svg" alt="Crates.io"></a>
    <a href="https://github.com/parkrevil/bunner-qs-rs/releases"><img src="https://img.shields.io/github/v/release/parkrevil/bunner-qs-rs?sort=semver" alt="version"></a>
    <a href="https://github.com/parkrevil/bunner-qs-rs/actions/workflows/tests.yml"><img src="https://github.com/parkrevil/bunner-qs-rs/actions/workflows/tests.yml/badge.svg?branch=main" alt="tests"></a>
    <a href="https://codecov.io/gh/parkrevil/bunner-qs-rs"><img src="https://codecov.io/gh/parkrevil/bunner-qs-rs/branch/main/graph/badge.svg" alt="coverage"></a>
    <a href="LICENSE.md"><img src="https://img.shields.io/badge/license-MIT-blue.svg" alt="License"></a>
</p>

<p align="center">
  <a href="README.md">English</a> | <strong>한국어</strong>
</p>

---

<a id="소개"></a>
## ✨ 소개

`bunner-qs-rs`는 쿼리 문자열을 빠르고 안전하게 파싱/직렬화하는 라이브러리입니다.

- **Serde 통합**: 임의의 `Deserialize`/`Serialize` 타입과 자연스러운 라운드트립
- **중첩 구조 지원**: 브래킷 표기법(`a[0]`, `a[b][c]`)을 배열과 맵으로 변환
- **강력한 가드레일**: 길이·파라미터·깊이 제한 옵션 제공과 문법 검사
- **정책 제어**: 공백 처리, 중복 키 정책, 허용 한도 옵션
- **표준 준수**: RFC 3986 퍼센트 인코딩을 완전하게 지원

### 미리보기

#### ✅ 지원
```
a=1&b=two                           → {"a": "1", "b": "two"}
flag                                → {"flag": ""}
flag=                               → {"flag": ""}
name=J%C3%BCrgen                    → {"name": "Jürgen"}
키=값                               → {"키": "값"}
a[b][c]=value                       → {"a": {"b": {"c": "value"}}}
a[0]=x&a[1]=y                       → {"a": ["x", "y"]}
a[1]=x                              → {"a": ["", "x"]}
a[0]=x&a[2]=y                       → {"a": ["x", "", "y"]}
a[]=x&a[]=y                         → {"a": ["x", "y"]}
a[][b]=1                            → {"a": [{"b": "1"}]}

# space_as_plus 옵션
(space_as_plus=false) a=hello+world → {"a": "hello+world"}
(space_as_plus=true)  a=hello+world → {"a": "hello world"}
```

❌ 미지원
```
a[b=1                               → ParseError::UnmatchedBracket
a]                                  → ParseError::UnmatchedBracket
a=1?b=2                             → ParseError::UnexpectedQuestionMark
a=%ZZ                               → ParseError::InvalidPercentEncoding
a=%01                               → ParseError::InvalidCharacter
a=%FF                               → ParseError::InvalidUtf8
a=1&a=2                             → ParseError::DuplicateKey
a[0]=x&a[b]=y                       → ParseError::DuplicateKey
a[b]=1&a[]=2                        → ParseError::DuplicateKey
a=1&a[b]=2                          → ParseError::DuplicateKey

# max_depth 옵션
(max_depth=1)   a[b][c]=1           → ParseError::DepthExceeded

# max_depth 옵션
(max_params=1)  a=1&b=2             → ParseError::TooManyParameters

# max_depth 옵션
(max_length=3)  aaaa                → ParseError::InputTooLong
```

> [!IMPORTANT]
> 이 라이브러리는 HTTP 서버나 미들웨어 기능은 제공하지 않으므로 사용 중인 프레임워크에 맞춰 통합 코드를 작성해야 합니다.

---

## 📚 목차
*   [**소개**](#소개)
*   [**시작하기**](#시작하기)
    *   [설치](#설치)
    *   [빠른 시작](#빠른-시작)
*   [**파싱**](#parseoptions)
    *   [ParseOptions](#parseoptions)
    *   [결과](#파싱-결과)
*   [**직렬화**](#parseoptions)
    *   [StringifyOptions](#stringifyoptions)
    *   [결과](#직렬화-결과)
*   [**오류**](#오류)
    *   [검증 오류](#검증-오류)
    *   [런타임 오류](#런타임-오류)
*   [**예제**](#예제)
*   [**기여하기**](#기여하기)
*   [**라이선스**](#라이선스)

---

<a id="시작하기"></a>
## 🚀 시작하기

<a id="설치"></a>
### 설치

`cargo add`로 라이브러리를 추가하세요:

```bash
cargo add bunner_qs_rs
```

또는 `Cargo.toml`에 직접 명시할 수 있습니다:

```toml
[dependencies]
bunner_qs_rs = "0.1.0"
```

<a id="빠른-시작"></a>
### 빠른 시작

아래 예제는 `http` 크레이트를 사용해 요청에서 쿼리를 추출해 Pagination 구조체로 파싱합니다.

```rust
use bunner_qs_rs::{ParseOptions, Qs};
use http::Request;
use serde::Deserialize;

#[derive(Debug, Deserialize, PartialEq)]
struct Pagination {
    page: u32,
    per_page: u32,
}

let request = Request::builder()
    .uri("https://api.example.com/items?page=2&per_page=50")
    .body(())
    .unwrap();

let query = request.uri().query().unwrap_or("");

let qs = Qs::new().with_parse(
    ParseOptions::new()
        .max_params(10)
        .max_depth(2),
)?;

let pagination: Pagination = qs.parse(query)?;
assert_eq!(pagination, Pagination { page: 2, per_page: 50 });
```

> [!TIP]
> `Qs` 인스턴스는 애플리케이션 시작 시 한 번 생성하고 재사용하세요.

---

<a id="parseoptions"></a>
## ⚙️ ParseOptions

주요 옵션과 기본값은 다음과 같습니다.

| 옵션 | 기본값 | 설명 |
|------|--------|------|
| `space_as_plus` | `false` | `+` 기호를 공백으로 해석합니다. |
| `duplicate_keys` | `DuplicateKeyBehavior::Reject` | 동일 키가 반복될 때 동작을 지정합니다. |
| `max_params` | `None` | 파라미터 쌍 개수 제한입니다. |
| `max_length` | `None` | 입력 문자열 길이(바이트) 제한입니다. |
| `max_depth` | `None` | 중첩 브래킷 깊이 제한입니다. |

<a id="space_as_plus"></a>
### `space_as_plus`

`www-form-urlencoded` 스타일 쿼리를 그대로 받아야 할 경우 활성화합니다.

```rust
// 기본 동작 (비활성화)
let default_options = ParseOptions::new();
// "a=hello+world" → {"a": "hello+world"} (+ 기호 그대로)

// 활성화
let form_options = ParseOptions::new().space_as_plus(true);
// "a=hello+world" → {"a": "hello world"} (+ → 공백 변환)
```

> [!TIP]
> 브라우저 폼 제출 데이터를 처리할 때 이 옵션을 활성화하세요.

<a id="duplicate_keys"></a>
### `duplicate_keys`

동일 키가 반복될 때 정책을 선택합니다.

- `Reject` (기본): 중복 키가 등장하면 즉시 오류를 반환합니다.
- `FirstWins`: 첫 번째 값을 유지하고 나머지를 무시합니다.
- `LastWins`: 마지막으로 등장한 값을 採用합니다.

```rust
// 중복 키 거부
let strict = ParseOptions::new()
    .duplicate_keys(DuplicateKeyBehavior::Reject);
// "a=1&a=2" → ParseError::DuplicateKey

// 마지막 값 채택
let last_wins = ParseOptions::new()
    .duplicate_keys(DuplicateKeyBehavior::LastWins);
// "a=1&a=2" → {"a": "2"}

// 첫 번째 값 채택
let first_wins = ParseOptions::new()
    .duplicate_keys(DuplicateKeyBehavior::FirstWins);
// "a=1&a=2" → {"a": "1"}
```

> [!NOTE]
> 브래킷 인덱스가 명시된 경우(`a[0]=x&a[1]=y`)는 중복으로 간주하지 않습니다.

<a id="max_params"></a>
### `max_params`

허용할 파라미터 수의 상한을 지정합니다. `Some(0)`은 허용되지 않습니다.

```rust
let options = ParseOptions::new().max_params(128);
// "a=1&b=2&...&c=200" (200개) → ParseError::TooManyParameters
```

> [!WARNING]
> 제한 없이 사용하면 악의적인 대량 파라미터 공격에 취약할 수 있습니다. 프로덕션에서는 적절한 값을 설정하세요.

<a id="max_length"></a>
### `max_length`

입력 전체 길이 한도를 정의합니다. 초과 시 `ParseError::InputTooLong`이 반환됩니다.

```rust
let options = ParseOptions::new().max_length(8 * 1024); // 8KB 제한
// 8KB 초과 입력 → ParseError::InputTooLong
```

> [!TIP]
> 일반적인 웹 요청의 경우 8KB~16KB 제한을 권장합니다.

<a id="max_depth"></a>
### `max_depth`

브래킷 중첩 깊이를 제한하여 악성 입력을 방지합니다.

```rust
let options = ParseOptions::new().max_depth(10);
// "a[b][c][d][e][f][g][h][i][j][k]" (깊이 11) → ParseError::DepthExceeded
```

**깊이 계산 예시:**
```
a=1                 깊이 0
a[0]=1             깊이 1
a[b][c]=1          깊이 2
a[b][c][d][e]=1    깊이 4
```

> [!WARNING]
> 제한 없이 사용하면 스택 오버플로우 공격에 취약합니다. 권장값은 5~20 사이입니다.

---

<a id="stringifyoptions"></a>
## 🧰 StringifyOptions

직렬화 시 추가적인 제어가 필요한 경우 활용합니다.

| 옵션 | 기본값 | 설명 |
|------|--------|------|
| `space_as_plus` | `false` | 공백을 `+`로 인코딩합니다. |

<a id="stringify-space_as_plus"></a>
### `space_as_plus`

브라우저 호환 쿼리 문자열이 필요할 때 활성화하세요.

```rust
use bunner_qs_rs::StringifyOptions;

// 기본 동작 (비활성화)
let default_options = StringifyOptions::new();
// {"text": "hello world"} → "text=hello%20world"

// 활성화
let form_options = StringifyOptions::new().space_as_plus(true);
// {"text": "hello world"} → "text=hello+world"
```

> [!NOTE]
> 폼 제출과의 일관성을 위해 파싱과 직렬화에서 같은 `space_as_plus` 값을 사용하세요.

---

<a id="오류"></a>
## 🚨 오류

<a id="검증-오류"></a>
### 검증 오류

`ParseOptions::validate()` 또는 `StringifyOptions::validate()`는 잘못된 조합 시 `OptionsValidationError`를 반환합니다.

| 오류 | 설명 |
|------|------|
| `NonZeroRequired { field }` | `max_params`, `max_length`, `max_depth`는 0보다 커야 합니다. |

```rust
// 잘못된 설정 예시
let invalid = ParseOptions::new().max_depth(0);
match invalid.validate() {
    Err(OptionsValidationError::NonZeroRequired { field }) => {
        eprintln!("Invalid option: {}", field); // "max_depth"
    }
    _ => {}
}
```

<a id="런타임-오류"></a>
### 런타임 오류

런타임에서는 구성 누락과 파서·직렬화기에서 발생하는 오류를 확인하세요.

#### 구성 관련 오류

| 오류 | 설명 |
|------|------|
| `QsParseError::MissingParseOptions` | `Qs`에 파싱 옵션이 설정되어 있지 않습니다. |
| `QsStringifyError::MissingStringifyOptions` | 문자열화 옵션이 설정되지 않았습니다. |

```rust
let qs = Qs::new(); // 옵션 미설정
match qs.parse::<Value>("a=1") {
    Err(QsParseError::MissingParseOptions) => {
        eprintln!("파싱 옵션을 먼저 설정하세요");
    }
    _ => {}
}
```

#### 파싱 오류

| 오류 | 설명 | 예시 |
|------|------|------|
| `ParseError::InputTooLong` | 입력이 `max_length`를 초과했습니다. | `max_length(10)` 설정에 20바이트 입력 |
| `ParseError::TooManyParameters` | `max_params` 제한을 초과했습니다. | `max_params(5)` 설정에 10개 파라미터 |
| `ParseError::DuplicateKey` | 중복 키가 정책에 위배됩니다. | `duplicate_keys(Reject)` 설정에 `a=1&a=2` |
| `ParseError::InvalidPercentEncoding` | 잘못된 퍼센트 인코딩이 발견되었습니다. | `a=%ZZ` (유효하지 않은 hex) |
| `ParseError::InvalidCharacter` | 허용되지 않은 문자가 포함되어 있습니다. | `a=<script>` (제어 문자) |
| `ParseError::UnexpectedQuestionMark` | 쿼리 내부에서 `?`가 발견되었습니다. | `a=1?b=2` (중간에 `?`) |
| `ParseError::UnmatchedBracket` | 브래킷 구조가 불완전합니다. | `a[b=1` (닫는 브래킷 누락) |
| `ParseError::DepthExceeded` | 중첩 깊이 제한을 초과했습니다. | `max_depth(3)` 설정에 `a[b][c][d]` |
| `ParseError::InvalidUtf8` | UTF-8 디코딩에 실패했습니다. | 잘못된 바이트 시퀀스 |
| `ParseError::Serde` | 타깃 타입으로 역직렬화에 실패했습니다. | 숫자 필드에 문자열 값 |

```rust
// 오류 처리 예시
use bunner_qs_rs::{ParseError, QsParseError};

match qs.parse::<UserInfo>("name=Alice&age=invalid") {
    Ok(user) => println!("Parsed: {:?}", user),
    Err(QsParseError::Parse(ParseError::Serde(e))) => {
        eprintln!("타입 변환 실패: {}", e);
    }
    Err(e) => eprintln!("파싱 오류: {}", e),
}
```

#### 직렬화 오류

| 오류 | 설명 | 예시 |
|------|------|------|
| `StringifyError::Serialize` | `Serialize` 구현이 실패했습니다. | 직렬화 불가능한 타입 |
| `StringifyError::InvalidKey` | 키에 제어 문자가 포함되어 있습니다. | 키에 NULL 문자 포함 |
| `StringifyError::InvalidValue` | 값에 제어 문자가 포함되어 있습니다. | 값에 제어 문자 포함 |

```rust
// 직렬화 오류 처리
use bunner_qs_rs::{QsStringifyError, StringifyError};

match qs.stringify(&data) {
    Ok(query) => println!("Encoded: {}", query),
    Err(QsStringifyError::Stringify(StringifyError::InvalidKey { key })) => {
        eprintln!("유효하지 않은 키: {}", key);
    }
    Err(e) => eprintln!("직렬화 오류: {}", e),
}
```

---

<a id="파싱과-직렬화-흐름"></a>
## 📋 파싱과 직렬화 흐름

<a id="입력-준비"></a>
### 입력 준비

쿼리 문자열 입력 형식:
- `?`가 있든 없든 그대로 전달하면 됩니다. (`"a=1&b=2"` 또는 `"?a=1&b=2"` 모두 허용)
- 빈 문자열이나 `"?"`만 있는 경우 기본 값을 반환합니다.
- `ParseOptions::validate()`를 통해 구성 오류를 미리 차단하세요.

#### `Qs` 래퍼 사용

```rust
use bunner_qs_rs::{ParseOptions, Qs};

let qs = Qs::new()
    .with_parse(ParseOptions::new().max_depth(8))?;

let result: serde_json::Value = qs.parse("a=1&b[0]=x&b[1]=y")?;
```

#### 직접 파싱 API 호출

`Qs`를 사용하지 않는 경우 `parsing::parse`를 직접 호출할 수 있습니다:

```rust
use bunner_qs_rs::{ParseOptions, parsing};

let options = ParseOptions::new().max_params(100);
let result: serde_json::Value = parsing::parse("key=value", &options)?;
```

<a id="결과-활용"></a>
### 결과 활용

<a id="맵으로-받기"></a>
#### 맵으로 받기

`serde_json::Value`로 받아 키-값 구조를 그대로 확인할 수 있습니다:

```rust
use serde_json::Value;

let result: Value = qs.parse("name=Alice&age=30")?;
// result = {"name": "Alice", "age": "30"}
```

<a id="serde-구조체로-받기"></a>
#### Serde 구조체로 받기

`Deserialize`를 구현한 도메인 구조체로 직접 매핑해 애플리케이션 로직에 바로 사용하세요:

```rust
use serde::Deserialize;

#[derive(Deserialize)]
struct UserInfo {
    name: String,
    age: u32,
}

let user: UserInfo = qs.parse("name=Alice&age=30")?;
```

<a id="json으로-받기"></a>
#### JSON으로 받기

`serde_json::Value`로 요청을 검증한 뒤 다른 시스템과 JSON으로 교환할 수 있습니다:

```rust
let json_value: serde_json::Value = qs.parse("items[0]=a&items[1]=b")?;
// json_value = {"items": ["a", "b"]}
```

<a id="문자열로-직렬화"></a>
#### 문자열로 직렬화

`Qs::stringify` 또는 `stringify::stringify`로 임의의 `Serialize` 데이터를 쿼리 문자열로 변환합니다:

```rust
use serde::Serialize;

#[derive(Serialize)]
struct Query {
    search: String,
    page: u32,
}

let query = Query { search: "Rust".into(), page: 1 };
let encoded = qs.stringify(&query)?;
// encoded = "search=Rust&page=1"
```

#### 직접 직렬화 API 호출

```rust
use bunner_qs_rs::{StringifyOptions, stringify};

let options = StringifyOptions::new().space_as_plus(true);
let data = serde_json::json!({"key": "hello world"});
let result = stringify::stringify(&data, &options)?;
// result = "key=hello+world"
```

<a id="맞춤-옵션-적용"></a>
#### 맞춤 옵션 적용

같은 데이터에 서로 다른 정책을 적용해야 한다면 별도 `Qs` 인스턴스나 옵션 세트를 구성하세요:

```rust
// 엄격한 파싱용
let strict_qs = Qs::new()
    .with_parse(ParseOptions::new()
        .duplicate_keys(DuplicateKeyBehavior::Reject)
        .max_depth(5))?;

// 관대한 파싱용
let lenient_qs = Qs::new()
    .with_parse(ParseOptions::new()
        .duplicate_keys(DuplicateKeyBehavior::LastWins))?;
```

---

<a id="예제"></a>
## 📝 예제

공식 예제는 준비 중입니다. 동작을 확인하려면 `tests/` 디렉터리의 시나리오와 `benches/`의 벤치마크를 참고하세요.

---

<a id="기여하기"></a>
## ❤️ 기여하기

기여는 현재 받지 않습니다. 의견이나 버그 리포트는 이슈로 남겨주세요.

---

<a id="라이선스"></a>
## 📜 라이선스

MIT License. 자세한 내용은 [LICENSE.md](LICENSE.md)를 참고하세요.
