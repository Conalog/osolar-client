# osolar-client (Go)

OSOLAR-LINK Open API용 수동 구현 Go SDK입니다.

## 무엇을 제공하나요?

- `context.Context` 기반 동기 API 호출
- 제네릭 응답 타입(`ApiResponse[T]`)으로 타입 안정성 확보
- `APIError`로 상태 코드/응답 바디 추적
- 실제 API 점검용 예제 (`examples/live-smoke`, `examples/live-all`)

## 설치 (이 레포 기준)

```bash
cd clients/go
go mod tidy
```

## GitHub 소스에서 설치 (다른 프로젝트에서 사용)

```bash
git clone https://github.com/Conalog/osolar-client.git
```

소비 프로젝트 `go.mod`에서 로컬 clone 경로를 `replace`로 연결합니다.

```go
require github.com/conalog/osolar-client/clients/go v0.0.0

replace github.com/conalog/osolar-client/clients/go => /absolute/path/to/osolar-client/clients/go
```

## 환경 변수

```bash
export OSOLAR_API_KEY="..."
```

## 빠른 시작

```go
package main

import (
	"context"
	"fmt"
	"os"

	osolar "github.com/conalog/osolar-client/clients/go"
)

func main() {
	client := osolar.NewClient(os.Getenv("OSOLAR_API_KEY"), "", nil)

	linked, err := client.ListLinkedPlants(context.Background())
	if err != nil {
		panic(err)
	}

	fmt.Println(linked.Data)
}
```

## 자주 쓰는 메서드

- `SearchPlants(ctx, SearchPlantsParams)`: 발전소 검색
- `LinkPlant(ctx, PlantLinkRequest)`: 발전소 링크 생성 시도
- `ListLinkedPlants(ctx)`: 내 링크 목록 조회
- `GetPlantInfo(ctx, linkID)`: 발전소 기본 정보
- `GetPlantContract(ctx, linkID)`: 계약 정보
- `GetPlantDocuments(ctx, linkID)`: 문서 목록
- `GetPlantOverview(ctx, linkID)`: 발전소 요약 정보
- `GetMonthlyGeneration(ctx, linkID, MonthlyGenerationParams)`: 월별 발전량
- `GetMonthlyBilling(ctx, linkID, MonthlyBillingParams)`: 월별 청구량

## 실전 예시 (링크 1건 기준 조회 흐름)

```go
ctx := context.Background()
client := osolar.NewClient(os.Getenv("OSOLAR_API_KEY"), "", nil)

links, err := client.ListLinkedPlants(ctx)
if err != nil {
	panic(err)
}
if links.Data == nil || len(*links.Data) == 0 {
	fmt.Println("No linked plants")
	return
}

linkID := (*links.Data)[0].LinkID
info, err := client.GetPlantInfo(ctx, linkID)
if err != nil {
	panic(err)
}
overview, err := client.GetPlantOverview(ctx, linkID)
if err != nil {
	panic(err)
}

fmt.Println(info.Data)
fmt.Println(overview.Data)
```

## 테스트 / 예제 실행

```bash
cd clients/go
go test ./...

# 실제 API 간단 점검
OSOLAR_API_KEY=... go run ./examples/live-smoke

# 전체 경로 점검
OSOLAR_API_KEY=... go run ./examples/live-all
```
