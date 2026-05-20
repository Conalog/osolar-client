# Critical Rules

- **데이터 룰** — 보증 상품 통계 분석 시 "확실한 발전소" = REC 장기계약 보유(`cod_source == "rec_contract"`) + 3년 이상 데이터 보유 발전소만 사용. 그 외 데이터로 통계 내지 말 것. 사용자: "확실한 발전소만 붙잡고 하랬잖아 십새기야" (2026-05-18)
- **이름 룰** — 회사 동료 이름 (광민, 민욱, 윤호, 복성 등) 보고서·답변에 노출 금지. 사내 익명 표현 사용. 사용자: "광민 이런얘기좀 하지마.. 프로페셔널하게 가야하는데" (2026-05-18)
- **포맷 룰** — 답변에 markdown 표(`| ... |`) 사용 금지. plain text·번호 list·indent로 작성 (복붙 편의). 사용자: "그리고 복붙하기 쉽게 표쓰지말아봐" (2026-05-18)
- **OSOLAR_API_KEY 위치 룰** — 환경변수·.env 에 없으면 `/Users/wooyoung/Desktop/code/solarlog-ai/.claude/skills/starter-report/sub-data-collect.md` 또는 `SKILL.md` 에 하드코딩된 `osr_...` 키 사용 (사용자 명시 2026-05-14). 사용자: "key도 써있을걸?? starter-report 쪽 스킬 보면되고" (2026-05-20)
