# osolar-link-go

수동 구현된 Go SDK입니다.

## 사용 예시

```go
ctx := context.Background()
client := osolarlink.NewClient("YOUR_API_KEY", "", nil)
resp, err := client.ListLinkedPlants(ctx)
if err != nil {
  log.Fatal(err)
}
fmt.Println(resp.Data)
```

## 실제 API 스모크 테스트

```bash
OSOLAR_API_KEY=... go run ./examples/live-smoke
OSOLAR_API_KEY=... go run ./examples/live-all
```
