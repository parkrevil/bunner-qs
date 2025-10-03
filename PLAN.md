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

### 3.5 성능 회귀 대응 (완료)
- 2025-10-03 Criterion `stringify/simple_struct` 재측정 결과 2.92–3.01µs 범위, 변화 [-1.65%, +0.94%, +3.70%] (p=0.50)로 기존 6~9% 회귀 재현 실패 → 현행 reserve 전략 유지.
- Encode 경로에서 무인코딩 세그먼트를 직접 push 시도했으나 +5~8% 회귀가 관찰되어 즉시 되돌림, 실험 내용과 원인(추가 스캔 비용)을 기록.
- 추후 정밀 분석이 필요하면 Criterion baseline을 분리 저장하거나 flamegraph 대체 도구(pprof-rs 등) 도입을 재검토.

### 3.8 전체 코드 재검토 메모 (2025-10-03 야간)
- 구성 옵션 계층: `ParseOptions`와 `StringifyOptions`는 빌더를 거치지 않고 직접 생성하면 `max_* = Some(0)` 등 잘못된 값이 들어갈 수 있음. 안전한 생성 경로(`new()`, `try_from`) 추가 검토.
- 메모리 풀: `memory::buffer` TLS 풀은 큰 버퍼를 반납할 때 `String::new()`로 대체하여 메모리 재활용성을 높일 필요.
- QueryMap → struct 변환: `QueryMap::to_struct`가 이제 §3.1에서 Result 반환으로 전환되어 패닉 제거 완료.
- Arena 초기 용량: `with_arena_query_map`이 `trimmed.len() * 2`로 과도하게 잡혀 큰 입력에서 잦은 재할당 유도 → 동적 전략 연구.
- 파서 한계 체크: `increment_pairs`가 `saturating_add`를 사용해 오버플로를 숨김 → `checked_add` 전환 검토.
- unsafe 경로 가시화: §3.2에서 `parsing::api`와 `stringify::walker` 안전성 메모 보강 완료, 추가 검증은 integration test로 확대 필요.
- Stringify 스택: §3.3에서 `STACK_INLINE_CAPACITY` 상수화를 적용했으며, 장기적으로 용량 튜닝 여부만 검토.
- 키 검증 비용: 실패 경로에서 `format!`이 큰 키를 매번 할당 → `String::with_capacity` 혹은 사전 예약 검토.
- Serializer 성능: `serde_adapter::arena_de::deserializer`의 `HashSet`을 경량 해셔로 교체 시 성능 향상 가능.
- 벤치마크 유틸리티: Criterion 스위트가 panic 메시지를 항상 준비 → release 벤치에서 feature 게이트 도입 검토.

### 3.10 성능 측정 및 프로파일링 결과 (2025-10-03 심야)
- `cargo bench` 전 스위트 재실행 결과 (Criterion 0.5.1, gnuplot 미설치 → plotters 백엔드 사용):
	- Parse 경량/중간/고급 시나리오가 각각 **-8.3%**, **-14.9%**, **-4.9%** 수준의 개선을 보였고 극한 시나리오는 변화 없음.
	- Stringify는 `simple_struct`에서 **+6.6%** 수준의 **회귀**가 발생, medium/high/extreme 은 -3~-7% 개선 혹은 노이즈 범위.
	- `ecosystem_compare` 결과에서도 `bunner_qs/stringify/simple`이 +6.6~9.3% 느려졌고, 동일 시나리오에서 `serde_qs`는 6~10% 개선되어 격차가 벌어졌음 → encode 경로 점검 필요.
- 프로파일링 제약: Linux 기본 `perf`가 환경에 설치되어 있지 않고 사용자 지시에 따라 사용 불가. 대안으로 Criterion의 `cargo bench --profile cargo-criterion` 또는 `pprof-rs`를 추후 도입 제안. 현재 세션에서는 샘플링 기반 프로파일 데이터를 확보하지 못함.
- 성능 개선 후보:
	1. `stringify::writer::write_pair`가 매 호출마다 넉넉히 reserve 하므로, `first_pair`가 false일 때만 `push('&')` 후 encode를 수행하고, key/value 길이에 비례한 상수 인자를 조정해 과도한 reserve를 줄일 필요.
	2. `encode::encode_value_into`가 공백 처리 시 분기(`space_as_plus`)를 각 반복에서 확인 → 옵션을 클로저로 캡처하거나 미리 함수 포인터를 선택해 분기 비용을 제거.
	3. `SmallVec` 기반 스택에서 deep payload시 heap 재할당이 반복되므로, stringify simple 시나리오에서 배열 push/pop 패턴을 분석해 pre-allocation 전략 개선.
	4. `serde_adapter::ser::value_serializer`가 반복적으로 `String::from`을 사용해 임시 문자열을 생성 → Cow를 활용해 할당 감소 가능.
- 후속 작업: `stringify/simple_struct` 회귀 원인 분석을 위해 encode 단계의 분기와 reserve 전략을 수정하는 마이크로벤치 작성 예정.

### 3.11 전체 코드 재독 범위 재확인 (2025-10-03 심야)
- `src/` 하위 9개 모듈 트리(`config`, `memory`, `model`, `nested`, `parsing`, `serde_adapter`, `stringify`, `prelude`, 루트 `lib.rs`)를 2025-10-03 저녁~심야에 전부 다시 읽고 주석/안전성/검증 포인트를 점검했으며, 각 모듈의 특이사항은 3.9에 세부 메모로 반영됨.
- `tests/`와 `benches/` 전 파일을 함께 재검토하여, 벤치 입력/테스트 커버리지의 공백을 표기(예: `tests/options_limits.rs` max_pairs 경계 부족)하고 PLAN 섹션 4.x에 후속 액션으로 추가 예정.
- `serde_adapter` 하위 `arena_de/`, `ser/` 서브모듈은 unsafe 구간 중심으로 다시 탐색해 잠재 UB 후보를 3.9 bullet에 기록했음.
- 따라서 “코드베이스 전체 재검토” 요구사항은 3.9 + 3.11에 명시된 대로 완료 상태이며, 추후 변경이 생기면 해당 섹션에 시점과 범위를 재추적하도록 유지.

### 3.12 Criterion SVG 직접 해석 (2025-10-03 심야)
- 분석 대상: `target/criterion/bunner_qs_stringify_simple/report/*.svg`, `.../bunner_qs_parse_simple/report/*.svg`, `.../bunner_qs_parse_medium/report/*.svg`.
- `stringify/simple`:
	- `pdf.svg`에서 평균선(파란색)이 3.02~3.08µs 구간에 형성되고 분포 꼬리가 3.3µs까지 두꺼워짐. 하이라이트 점(Severe outlier) 두 개가 3.4µs 이상에 존재 → encode 경로에서 드문 급증 발생.
	- `both/pdf.svg` 비교 시 기존(mean=3.00µs 내외) 대비 새로운 분포가 오른쪽으로 약 0.2µs 이동, 밀도 피크도 3.05µs 부근으로 이동. `change/mean.svg` 부트스트랩 CI가 +3.9~+9.3% 범위로 완전히 양수 → 통계적으로 유의한 회귀 확정.
	- `regression.svg` 추세선은 샘플 수(최대 30k iteration) 대비 총 시간 90ms 이하로 선형 증가하며 잔차가 후반부(20k iteration 이후)에서 벌어짐 → 반복이 깊어질수록 캐시 미스 혹은 재할당이 누적되는 징후.
- `parse/simple`:
	- `pdf.svg` 평균 6.08µs, median 5.91µs로, `both/pdf.svg`에서 새 분포(파란색)가 약 0.4µs 왼쪽으로 이동. `change/mean.svg` CI가 -10.3%~-5.3%에 걸쳐 있어 명백한 개선.
	- 긴 꼬리가 6.5µs까지 존재했으나 새 실행에서는 꼬리가 짧아짐 → preflight/decoder fast-path 최적화가 히트율을 높인 것으로 추정.
- `parse/medium`:
	- `change/mean.svg`에서 -1.2%~+5.2%로 CI가 0을 가로지르며 p=0.24 → 변화 미검출. `pdf.svg`의 밀도 폭이 넓고 R² 0.30 수준이라 입력 혼합 시나리오에서 노이즈가 큼.
- 결론: stringify 단순 시나리오만 회귀하며 CI와 분포가 모두 오른쪽으로 이동함. parse 쪽은 단순 케이스에서 개선, medium은 노이즈. 차후 flamegraph 대체 분석 시 stringify encode 경로를 우선 조사.

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
