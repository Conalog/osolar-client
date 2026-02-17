# osolar-link-client (Python)

수동 구현된 Python SDK입니다.

## 설치

```bash
pip install -e .
```

## 사용 예시

```python
from osolar_link_client import OsolarLinkClient

client = OsolarLinkClient(api_key="YOUR_API_KEY")
result = client.list_linked_plants()
print(result["data"])
```

## 실제 API 스모크 테스트

```bash
python3 -m venv .venv
. .venv/bin/activate
pip install -e '.[dev]'
OSOLAR_API_KEY=... python examples/live_smoke.py
OSOLAR_API_KEY=... python examples/live_all.py
```
