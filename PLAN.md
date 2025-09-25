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
