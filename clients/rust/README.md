# osolar-client (Rust)

OSOLAR Open API용 수동 구현 Rust SDK입니다.

## 무엇을 제공하나요?

- `reqwest::blocking` 기반 동기 클라이언트
- serde 모델 타입으로 응답 구조체 매핑
- `ApiError`로 HTTP/파싱 오류를 분리 처리
- 실제 API 점검용 예제 (`examples/live_smoke.rs`, `examples/live_all.rs`)

## 설치 (이 레포 기준)

```bash
cd clients/rust
cargo build
```

## GitHub 소스에서 설치 (다른 프로젝트에서 사용)

```bash
git clone https://github.com/Conalog/osolar-client.git
```

소비 프로젝트의 `Cargo.toml`에 path dependency로 연결합니다.

```toml
[dependencies]
osolar-client = { path = "/absolute/path/to/osolar-client/clients/rust" }
```

## 환경 변수

```bash
export OSOLAR_API_KEY="..."
```

## 빠른 시작

```rust
use osolar_client::OsolarClient;

fn main() -> Result<(), osolar_client::ApiError> {
    let api_key = std::env::var("OSOLAR_API_KEY").expect("OSOLAR_API_KEY is required");
    let client = OsolarClient::new(api_key);

    let connected = client.list_connected_plants()?;
    println!("{:?}", connected.data);
    Ok(())
}
```

## 자주 쓰는 메서드

- `search_plants(SearchPlantsParams)`: 발전소 검색
- `connect_plant(&PlantConnectionRequest)`: 발전소 연결 생성 시도
- `list_connected_plants()`: 내 연결 목록 조회
- `get_plant_info(connection_id)`: 발전소 기본 정보
- `get_plant_contract(connection_id)`: 계약 정보
- `get_plant_documents(connection_id)`: 문서 목록
- `get_plant_overview(connection_id)`: 발전소 요약 정보
- `get_monthly_generation(connection_id, MonthlyGenerationParams)`: 월별 발전량
- `get_monthly_billing(connection_id, MonthlyBillingParams)`: 월별 청구량

## 실전 예시 (연결 1건 기준 조회 흐름)

```rust
use osolar_client::OsolarClient;

fn main() -> Result<(), osolar_client::ApiError> {
    let api_key = std::env::var("OSOLAR_API_KEY").expect("OSOLAR_API_KEY is required");
    let client = OsolarClient::new(api_key);

    let connections = client.list_connected_plants()?;
    let Some(first) = connections.data.as_ref().and_then(|items| items.first()) else {
        println!("No connected plants");
        return Ok(());
    };

    let info = client.get_plant_info(&first.connection_id)?;
    let overview = client.get_plant_overview(&first.connection_id)?;

    println!("{:?}", info.data);
    println!("{:?}", overview.data);
    Ok(())
}
```

## 테스트 / 예제 실행

```bash
cd clients/rust
cargo test

# 실제 API 간단 점검
OSOLAR_API_KEY=... cargo run --example live_smoke

# 전체 경로 점검
OSOLAR_API_KEY=... cargo run --example live_all
```
