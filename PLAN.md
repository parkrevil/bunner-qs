# bunner_qs 후속 계획

현재 저장소는 RFC 기반 파싱/직렬화, 입력 제한 옵션, 중첩 구조 해석, HPP 대비 중복 키 차단, Serde 연계 등 1차 목표를 이미 달성했습니다. 이 문서는 앞으로 고려할 확장 과제만을 기록합니다.

## 추가 기능 후보
- `form-mode` 기능 플래그: 기본 `space_as_plus = true` 설정과 폼 데이터 헬퍼 제공
- `query-types` 프로필: 정수/불리언 등 스칼라 타입 변환 유틸리티
- 선택적 정렬 콜백 API (`FnMut(&str, &str) -> Ordering`) 도입
- 단일 값 맵/스트럭트를 위한 고수준 변환 헬퍼 강화

## 시험 및 품질 과제
- 무작위 입력 기반 property 테스트(Proptest 등) 추가
- 대용량 입력에 대한 벤치마크 및 성능 회귀 감시

## 문서화 TODO
- `form-mode`, `serde` 등 기능 플래그별 사용 가이드 작성
- 다른 Rust 크레이트 및 JS `URLSearchParams`와의 비교 표 보강
- 보안 주의사항(HPP 등)에 대한 별도 섹션 추가

## 커스텀 Serde 라운드트립 계획
### 배경 및 목표
- `serde_roundtrip.rs`의 모든 테스트에서 `parse().to_struct()`와 `stringify(QueryMap | struct)` 흐름이 완전히 지원되도록, 외부 의존성인 `serde_urlencoded`를 제거하고 내부 전용 직렬화/역직렬화 로직을 구현한다.
- 기능 플래그 `serde`는 유지하되, 활성화 시 커스텀 구현이 빌드에 포함되도록 구조를 재편한다.

### 전제 조건
1. 현재 `serde_roundtrip.rs` 테스트가 기대하는 키/값 규칙, 중첩 구조, 배열 표현(`[]` suffix) semantics를 신규 구현에서도 그대로 유지한다.
2. 기존 `QueryMap` 및 `Value` 타입의 내부 표현(IndexMap 기반)을 변경하지 않고, serde 브리지가 이를 활용하도록 설계한다.
3. 퍼블릭 API (`parse`, `stringify`, `ParseOptions`, `StringifyOptions`, `QueryMap::from_struct`, `QueryMap::to_struct`)의 함수 시그니처를 파손하지 않는다.

### 단계별 실행 계획
1. **현황 점검 및 요구 사항 확정**
	- `serde_roundtrip.rs` 기능별 테스트 케이스를 분류하여 입력/출력 패턴을 문서화한다.
	- `serde_urlencoded`가 현재 제공하는 encode/decode 동작, 에러 종류, 옵션 의존성을 정리한다.
	- `serde` 기능 플래그가 제공하는 API(예: `QueryMap::from_struct`, `to_struct`)의 호출 경로와 기대치를 명확히 기록한다.

2. **빌드/의존성 정리**
	- `Cargo.toml`에서 `serde_urlencoded` 의존성을 제거하고, `serde` 기능 플래그에서 해당 항목을 삭제한다.
	- 필요 시 새 헬퍼 모듈(예: `crate::serde_impl`)을 추가할 준비를 한다.

3. **커스텀 직렬화(Serialize) 설계**
	- `serde::Serializer` 트레이트를 충족하는 구조체(예: `FormSerializer`)를 정의하고, URL 쿼리 규칙(키 조합, 배열/객체 인덱싱, percent-encoding)을 재현한다.
	- 스칼라 타입(`str`, `bool`, `i64` 등), 시퀀스, map, struct, tuple variants에 대한 직렬화 경로를 구현하고 공통 헬퍼(`KeyPath`)를 설계한다.
	- `StringifyOptions`와 동일한 옵션 (특히 `space_as_plus`)을 serializer 옵션으로 전달할 수 있도록 API를 정의한다.
	- 직렬화 결과를 중간 구조(`QueryMap`) 혹은 문자열 버퍼로 축적하는 두 가지 모드를 검토하고, `QueryMap::from_struct`는 Map으로, 최종 `stringify`는 문자열로 이어지도록 처리한다.

4. **커스텀 역직렬화(Deserialize) 설계**
	- `serde::Deserializer` 구현체(예: `FormDeserializer`)를 만들고, 내부적으로 `QueryMap`을 순회하며 `Visitor` 패턴에 맞춰 값을 재구성한다.
	- 스칼라 파싱, 배열/맵 재구성, 중첩 키(`foo[bar][0]`) 해석 규칙을 명시하고 재사용 가능한 파서 헬퍼를 구축한다.
	- `ParseOptions`(특히 `max_*` 제한과 `space_as_plus`)를 역직렬화 로직에 통합한다.
	- 잘못된 입력에 대해 기존 `ParseError`와 호환되는 에러를 발생시키고, 이를 `serde` 오류(`serde::de::Error`)로 변환하는 어댑터를 구현한다.

5. **API 라우팅 및 통합**
	- 기존 `serde_bridge` 모듈을 대체/갱신하여, `QueryMap::from_struct`, `QueryMap::to_struct`, `parse().to_struct()`, `stringify(QueryMap | struct)` 경로가 커스텀 구현을 사용하도록 연결한다.
	- 필요 시 `to_string_with` 등 기존 함수에 serde 옵션 전달 경로를 보강한다.
	- `to_struct()`는 기본적으로 구조체에 정의되지 않은 키를 발견하면 실패하도록 하는 엄격 모드로 동작시키고, 추후 요청 시 관대한 변형(예: 허용 후 누락 목록 반환)을 옵션으로 제공할 수 있도록 훅을 마련한다.

6. **테스트 및 회귀 보강**
	- `serde_roundtrip.rs` 외에 신규 회귀 테스트를 추가하여, 에러 경로(잘못된 percent encoding, 중복 키 등)도 serde 경로에서 재현되도록 한다.
	- property 기반 테스트(proptest)를 활용하여 struct <-> query 간 round-trip이 보장되는지 검증한다.

7. **성능 및 메모리 검증**
	- 커스텀 구현이 기존 `serde_urlencoded` 대비 성능 저하를 초래하지 않는지 벤치마크(간단한 criterion 혹은 micro-bench)를 준비한다.
	- 대용량 쿼리에서 할당 횟수/복사 횟수를 추적하고, 필요 시 최적화 포인트를 문서화한다.

8. **문서/마이그레이션 가이드 작성**
	- README 및 API 문서에서 `serde_urlencoded` 제거와 새로운 커스텀 구현에 대한 설명을 추가한다.
	- 변경된 기능 플래그 안내, 옵션 전달 방법, 예제 코드를 업데이트한다.
	- 주요 Breaking Change 유무(없을 예정이지만, 내부 동작 변경 사항)를 릴리스 노트 초안으로 정리한다.

9. **릴리스 준비**
	- `cargo fmt`, `cargo clippy`, `cargo test` 전 범위 실행으로 최종 검증한다.
	- 버전 업 필요 시 `Cargo.toml` 패치 버전 증가 및 CHANGELOG 업데이트를 준비한다.

### 리스크 및 대응
- **복잡한 키 경로 처리**: 배열/객체 지점에서 인덱싱 규칙 불일치 위험 → 키 경로 유틸리티를 단일 소스로 두고, 통합 테스트에서 다양한 패턴을 커버.
- **Serde trait 구현 실수**: `Serializer`/`Deserializer` 구현이 광범위하므로, 최소 기능부터 단계적으로 확장하고 각 단계마다 테스트.
- **성능 회귀**: 이전 의존성 대비 성능 저하 가능 → 벤치마크를 통한 회귀 감시와 Hot path 최적화를 병행.
