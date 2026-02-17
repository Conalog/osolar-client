# osolar-link-client (Python)

OSOLAR-LINK Open API용 수동 구현 Python SDK입니다.

## 무엇을 제공하나요?

- `httpx` 기반 동기 클라이언트
- 9개 엔드포인트 공통 호출 인터페이스
- `ApiError` 예외로 상태 코드/응답 본문 확인
- 실제 API 점검용 예제 (`examples/live_smoke.py`, `examples/live_all.py`)

## 설치 (이 레포 기준)

```bash
cd clients/python
python3 -m venv .venv
. .venv/bin/activate
pip install -e '.[dev]'
```

## GitHub에서 바로 설치 (다른 프로젝트에서 사용)

`pip`은 Git 저장소의 하위 폴더를 직접 지정할 수 있습니다.

```bash
pip install "osolar-link-client @ git+https://github.com/Conalog/osolar-sdk.git@codex/multi-sdk-packages#subdirectory=clients/python"
```

브랜치 대신 태그/커밋 SHA를 써도 됩니다.

```bash
pip install "osolar-link-client @ git+https://github.com/Conalog/osolar-sdk.git@<tag-or-sha>#subdirectory=clients/python"
```

## 환경 변수

```bash
export OSOLAR_API_KEY="..."
```

## 빠른 시작

```python
import os
from osolar_link_client import OsolarLinkClient

with OsolarLinkClient(api_key=os.environ["OSOLAR_API_KEY"]) as client:
    linked = client.list_linked_plants()
    print(linked.get("data"))
```

## 자주 쓰는 메서드

- `search_plants(q, field, distance_km=None)`: 발전소 검색
- `link_plant(body)`: 발전소 링크 생성 시도
- `list_linked_plants()`: 내 링크 목록 조회
- `get_plant_info(link_id)`: 발전소 기본 정보
- `get_plant_contract(link_id)`: 계약 정보
- `get_plant_documents(link_id)`: 문서 목록
- `get_plant_overview(link_id)`: 발전소 요약 정보
- `get_monthly_generation(link_id, start_year=None, end_year=None)`: 월별 발전량
- `get_monthly_billing(link_id, start_year=None, end_year=None)`: 월별 청구량

## 실전 예시 (링크 1건 기준 조회 흐름)

```python
import os
from osolar_link_client import ApiError, OsolarLinkClient

with OsolarLinkClient(api_key=os.environ["OSOLAR_API_KEY"]) as client:
    try:
        links = client.list_linked_plants().get("data") or []
        if not links:
            print("No linked plants")
            raise SystemExit(0)

        link_id = links[0]["link_id"]
        info = client.get_plant_info(link_id)
        overview = client.get_plant_overview(link_id)

        print(info.get("data", {}).get("plant_name"))
        print(overview.get("data", {}).get("contract_status"))
    except ApiError as err:
        print(err.status_code, err.response_body)
```

## 테스트 / 예제 실행

```bash
cd clients/python
pytest

# 실제 API 간단 점검
OSOLAR_API_KEY=... python examples/live_smoke.py

# 전체 경로 점검
OSOLAR_API_KEY=... python examples/live_all.py
```
