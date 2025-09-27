# 통합 성능 최적화 로드맵

## 0. 목표와 측정 지표
- **핵심 지표**: ① 문자열화/파싱 벤치마크(`benches/`) 평균 실행 시간, ② 메모리 할당 횟수(`jemalloc`/`cargo instruments` 등), ③ `serde` roundtrip 대기 시간.
- **목표치(1차)**: `serde_qs` 대비 stringify 2× 이내, parse 1.5× 이내, 대형 시나리오(`benches/scenarios.rs`)에서 peak allocation 30% 절감.
- **추적 방법**: Criterion 벤치 + `cargo instruments`/`heaptrack` 결과를 이 문서 하단에 날짜와 함께 기록.

## 1. 최우선 개선 영역 (즉시 효과, 난이도 낮음~중간)
### 1-1. 문자열화 경로 최적화 (`stringify.rs`, `encoding.rs`)
- `Vec<String> + join("&")` 제거 → 단일 출력 버퍼(`String::with_capacity`)에 `write!`/`push_str`로 직접 기록.
- `flatten_value` 재귀 제거: 명시적 스택(예: `SmallVec<Frame>`)으로 객체/배열 순회를 반복 구조로 전환하고, `format!` 대신 수동으로 키 경로를 조립.
- `encode_with_set` 개선: per-char `to_string()` 호출 대신 `encode_into(buf, &str, space_as_plus)` 헬퍼에서 `%HH`를 직접 출력. 공백-플러스와 나머지 문자 분기를 분리.
- 기대 효과: Grok/GPT 피드백 기준 stringify 2~4× 향상, `alloc::String` 생성 수 급감.

### 1-2. 파싱 경로의 디코딩 경량화 (`parse.rs`)
- `percent_encoding::percent_decode_str`를 사용해 디코딩이 필요 없는 입력은 `Cow::Borrowed`로 처리.
- 함수 외부에서 재사용 가능한 `Vec<u8>` 스크래치 버퍼를 전달받아 퍼센트 디코딩 시 복사를 최소화.
- `decode_component` 내 제어 문자 검증은 디코딩 완료 후 한 번만 수행하도록 분기 정리.

### 1-3. 제어 문자 및 용량 관리 공통 유틸
- `ensure_no_control` + `decode_component`의 중복 검사를 통합한 바이트 스캔 유틸 도입 (`memchr`/SIMD 옵션 검토).
- `QueryMap::with_capacity_hint(max_params)` 추가하여 대량 파라미터 시 리해시 방지. `Entry` API로 중복 키 검사.

## 2. 구조적 리팩토링 (난이도 중간~높음)
### 2-1. 단일 패스 파서 & 중첩 삽입 정비 (`parse.rs`, `nested.rs`)
- `parse_query_map`를 단일 상태 머신으로 재작성해 `find('&')`/`find('=')` 반복 호출 제거.
- 키 경로 파싱을 `Vec<String>` 대신 `KeyPathIter<'a>`(슬라이스 기반) 또는 `SmallVec<[&str; N]>`로 유지. 
- `PatternState` HashMap 키를 `SmallVec<[u8; N]>` 혹은 해시 가능한 경로 구조체로 교체해 해시 비용 감소.
- `insert_nested_value`를 재귀 → 반복 스택으로 변환, 컨테이너 타입(`Array/Object`) 캐시를 적극 활용.

### 2-2. 문자열화·중첩의 일관성 확보
- `flatten_value`와 `insert_nested_value` 모두에서 같은 키 빌더/스택 자료구조 사용 → 문자열화/파싱 간 구조적 대칭 유지.
- 깊은 중첩에 대비해 placeholder (`Value::String("")`) 처리 로직을 반복 구조에서도 안전하게 보강.

### 2-3. Serde 브리지 경량화 (`serde_impl/ser.rs`, `serde_impl/de.rs`)
- `FormValue` 중간 enum 제거 후 `ValueBuilder` 패턴으로 직접 `Value` 구성.
- 역직렬화에서 `Value::Object(map.clone())` 대신 `ValueRef<'a>`(borrowed view)를 도입해 전체 복사 제거.
- HashSet 기반 중복 필드 추적을 비트마스크 또는 재사용 가능한 구조로 대체.

## 3. 중장기 지원 과제
- **버퍼/메모리 풀링**: 파싱/문자열화 전반에 걸쳐 `SmallVec`, thread-local 버퍼, `bumpalo` 등 도입 검토.
- **IndexMap 대안 평가**: `hashbrown::HashMap` + insertion order 보존 전략 비교, 혹은 사용자 요구에 따라 옵션화.
- **옵션 전처리**: `ParseOptions`/`StringifyOptions`의 분기(예: `space_as_plus`)를 핫 루프 진입 전 계산해 분기 예측 정확도 향상.

## 4. 검증 및 관측 도구
- **Criterion 벤치**: `stringify_flat_array`, `parse_deep_nested`, `parse_sparse_percent`, `serde_roundtrip_large` 등 세분화된 벤치를 추가하고, 각 개선 단계 후 리포트.
- **단위/프로퍼티 테스트**: 제어 문자, 잘못된 퍼센트 인코딩, 빈 키, 깊은 배열/객체 혼합 구조를 포함한 회귀 테스트 강화.
- **프로파일링**: `perf`, `dhat`, `heaptrack` 등을 활용한 before/after 스냅샷을 문서에 기록.
- **추적 방식**: 본 문서 상단 로드맵 아래에 “실행 이벤트 로그” 표를 추가해 단계별 착수/완료/측정 수치를 누적.

## 5. 실행 순서 제안 (예시)
1. **Step A**: 문자열화 경로 개선 (`stringify.rs`, `encoding.rs`) → 벤치 갱신.
2. **Step B**: 파싱 디코딩 경량화 (`decode_component`) → 컨트롤 문자 유틸 통합.
3. **Step C**: 단일 패스 파서 + 반복 중첩 삽입 구현.
4. **Step D**: Serde 브리지 경량화 및 역직렬화 복사 제거.
5. **Step E**: 중장기 과제(버퍼 풀링, IndexMap 대체) 탐색.
