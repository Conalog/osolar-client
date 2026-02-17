# osolar-link-client (Rust)

수동 구현된 Rust SDK입니다.

## 사용 예시

```rust
use osolar_link_client::OsolarLinkClient;

let client = OsolarLinkClient::new("YOUR_API_KEY");
let linked = client.list_linked_plants()?;
println!("{:?}", linked.data);
# Ok::<(), osolar_link_client::ApiError>(())
```

## 실제 API 스모크 테스트

```bash
OSOLAR_API_KEY=... cargo run --example live_smoke
OSOLAR_API_KEY=... cargo run --example live_all
```
