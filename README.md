# osolar-sdk

`OSOLAR-LINK Open API`용 멀티 언어 클라이언트 SDK 모노레포입니다.

## 구조

- `specs/osolar-link-openapi.json`: 원본 OpenAPI 스펙 고정본
- `clients/ts`: JavaScript/TypeScript 클라이언트
- `clients/python`: Python 클라이언트
- `clients/go`: Go 클라이언트
- `clients/rust`: Rust 클라이언트
- `scripts/fetch-spec.sh`: 스펙 최신본 다시 가져오기

## 빠른 시작

```bash
# TypeScript
cd clients/ts && npm install && npm test && npm run build

# Python
cd clients/python && python3 -m pip install -e .[dev] && pytest

# Go
cd clients/go && go test ./...

# Rust
cd clients/rust && cargo test
```

## 실제 API 스모크 테스트

`OSOLAR_API_KEY` 환경 변수를 설정한 뒤 실행합니다.

```bash
export OSOLAR_API_KEY=...
make live
make live-all
```

## API 범위

현재 9개 엔드포인트를 공통으로 제공합니다.

1. `GET /v1/search`
2. `POST /v1/links`
3. `GET /v1/links`
4. `GET /v1/links/{link_id}`
5. `GET /v1/links/{link_id}/contract`
6. `GET /v1/links/{link_id}/documents`
7. `GET /v1/links/{link_id}/overview`
8. `GET /v1/links/{link_id}/generation/monthly`
9. `GET /v1/links/{link_id}/billing/monthly`
