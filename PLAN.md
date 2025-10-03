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

## 3. 코드 품질 개선 (경미)

### 3.1 매직 넘버 상수화
- `src/nested/segment.rs:17`: `SmallVec<[u8; 24]>` → 상수화
- `src/nested/key_path.rs:8`: `SmallVec<[&str; 16]>` → 상수화
- 제어 문자 범위 `\u{0000}`~`\u{001F}`, `\u{007F}` 하드코딩 → 상수화

### 3.2 .expect() 제거
라이브러리 코드 2곳:
- `src/parsing/pair_inserter.rs:53`: `unwrap_or_else()` + `debug_assert!` 권장
- `src/model/value.rs:99`: release 빌드 처리 필요

### 3.3 에러 메시지 일관성
일부 메시지에 마침표/콜론 불일치 → 통일 권장

## 4. 테스트 개선 (미미)

### 4.1 테스트 문서화
- `tests/README.md` 생성 권장
- 통합 테스트 파일에 `//!` 모듈 문서 추가

### 4.2 CI 자동화
- 커버리지 자동 업로드 (codecov) 권장
- 벤치마크 regression 추적 권장

## 5. 우선순위별 로드맵

### Phase 1: crates.io 게시 준비 (필수, 1-2일)
1. Cargo.toml 메타데이터 추가
2. CHANGELOG.md 작성
3. 기본 CI 구축 (test, clippy, fmt)

### Phase 2: 문서화 (필수, 3-4일)
1. 공개 API `///` 문서 주석
2. 모듈 `//!` 문서
3. README.md 배지, MSRV, Security 섹션 추가

### Phase 3: 커뮤니티 인프라 (중요, 2-3일)
1. CONTRIBUTING.md, CODE_OF_CONDUCT.md
2. GitHub 템플릿 (.github/)
3. CI 확장 (크로스 플랫폼, 커버리지, MSRV)

### Phase 4: 예제 및 표준 문서 (중요, 2일)
1. examples/ 디렉토리 생성
2. RFC 참조 주석 추가

### Phase 5: 코드 품질 (선택, 1-2일)
1. 매직 넘버 상수화
2. .expect() 제거
3. 에러 메시지 통일

### Phase 6: 테스트 개선 (선택)
1. tests/README.md
2. CI 자동화 확장

## 결론

**현재 상태**: 코드 품질 ⭐⭐⭐⭐⭐, 테스트 ⭐⭐⭐⭐⭐, 문서 ⭐☆☆☆☆

**Phase 1~2 완료 후 crates.io 게시 가능**
**추정 작업 시간**: 필수 4-6일, 전체 10-14일
