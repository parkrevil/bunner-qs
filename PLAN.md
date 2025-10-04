# bunner_qs 개선 계획

> **검토 범위**: 102개 전체 Rust 파일 직접 스캔 완료
> **검토일**: 2025-10-03

## 1. 오픈소스 필수 인프라 부재 (심각)

### 1.1 Cargo.toml 메타데이터 누락
crates.io 게시 불가능:
- `description`, `repository`, `readme` 필드 없음
- `keywords`, `categories` 없음
- `homepage`, `documentation` 권장

### 1.2 프로젝트 문서 부재
- `CHANGELOG.md` 없음
- `CONTRIBUTING.md` 없음
- `CODE_OF_CONDUCT.md` 없음
- `.github/workflows/` CI/CD 없음
- `.github/ISSUE_TEMPLATE/` 없음

### 1.3 API 문서 완전 부재
모든 공개 API에 `///` 문서 주석 없음:
- `parse()`, `parse_with()`, `stringify()`, `stringify_with()`
- `ParseOptions`, `StringifyOptions`, `DuplicateKeyBehavior`
- `Value`, `QueryMap`, `ParseError`
- 모든 `mod.rs`에 `//!` 모듈 문서 없음

### 1.4 예제 부재
`examples/` 디렉토리 없음

### 1.5 README.md 개선 필요
- 배지(badge) 없음
- MSRV 명시 없음
- Security Considerations 섹션 없음

## 2. 표준 문서화 부족

### 2.1 RFC/WHATWG 참조 주석 없음
구현은 표준 준수하나 코드에 RFC 섹션 번호 참조 없음:
- `src/parsing/decoder.rs`: RFC 3986 § 2.1 참조 필요
- `src/stringify/encode.rs`: RFC 3986 § 2.2, 2.3 참조 필요
- `src/parsing/preflight.rs`: RFC 3986 § 3.4 참조 필요

## 3. 코딩 품질 및 성능 개선 (경미)

- [ ] `ParseOptions`/`StringifyOptions` 기본 생성 시 잘못된 한계를 막기 위한 안전 생성자(`new()`, `try_from`) 도입 검토
- [ ] TLS 버퍼 풀(`memory::buffer`)에서 대형 버퍼 회수 전략 개선 및 재사용성 측정
- [ ] `with_arena_query_map` 초기 용량 산정 방식을 입력 특성 기반으로 재설계
- [ ] 파서 `increment_pairs`를 `checked_add` 기반 한계 검증으로 교체해 오버플로 관측 강화
- [ ] `StringifyRuntime` 스택/`format!` 실패 경로 최적화 여부 평가 및 필요 시 SmallVec 용량 튜닝
- [ ] encode 경로 성능 모니터링: Criterion baseline 저장, `stringify/simple_struct` 주기적 재측정, 회귀 감지 워크플로 구축
- [ ] `serde_adapter::arena_de::deserializer`의 `HashSet`을 경량 해셔 또는 커스텀 자료구조로 교체 검토
- [ ] Criterion 벤치 + 커버리지 리포트 자동화(예: CI 업로드, regression 알림)

## 4. 테스트 개선 (미미)

### 4.1 테스트 문서화
- `tests/README.md` 생성 권장
- 통합 테스트 파일에 `//!` 모듈 문서 추가

### 4.2 CI 자동화
- 커버리지 자동 업로드 (codecov) 권장
- 벤치마크 regression 추적 권장
