# @conalog/osolar-client

OSOLAR-LINK Open API용 수동 구현 TypeScript SDK입니다.

## 무엇을 제공하나요?

- 타입이 포함된 9개 엔드포인트 클라이언트
- `ApiError` 기반 공통 에러 처리
- 실제 API 키로 바로 실행 가능한 예제 (`examples/live-smoke.js`, `examples/live-all.js`)

## 설치 (이 레포 기준)

```bash
cd clients/ts
npm install
npm run build
```

## GitHub 소스에서 설치 (다른 프로젝트에서 사용)

`npm`은 Git URL에서 모노레포 하위 폴더(`clients/ts`)를 직접 패키지로 설치하지 못합니다.
그래서 GitHub에서 소스를 받아 로컬 경로 의존성으로 연결하는 방식이 가장 안정적입니다.

```bash
# 1) SDK 소스 가져오기
git clone https://github.com/Conalog/osolar-client.git
cd osolar-client/clients/ts
npm install
npm run build

# 2) 내 프로젝트에서 로컬 경로로 설치
npm install /absolute/path/to/osolar-client/clients/ts
```

개발 중에는 `npm link` 방식도 사용할 수 있습니다.

```bash
cd /absolute/path/to/osolar-client/clients/ts
npm link

cd /absolute/path/to/your-project
npm link @conalog/osolar-client
```

## 환경 변수

```bash
export OSOLAR_API_KEY="..."
```

## 빠른 시작

```ts
import { OsolarLinkClient } from "@conalog/osolar-client";

async function main() {
  const client = new OsolarLinkClient({
    apiKey: process.env.OSOLAR_API_KEY!,
    // baseUrl: "https://openapi.osolar.io", // 기본값
  });

  const linked = await client.listLinkedPlants();
  console.log(linked.data);
}

main().catch(console.error);
```

### 주의사항 (런타임 / baseUrl)

- Node.js 18 미만처럼 `fetch`가 없는 런타임에서는 `fetchFn`을 직접 주입해야 합니다.
- `baseUrl`은 `https://...`만 허용됩니다. (로컬 개발용 `http://localhost...`만 예외)
- `baseUrl`에 `?query`나 `#fragment`를 붙이지 마세요. (요청 URL이 깨질 수 있습니다)

## 자주 쓰는 메서드

- `searchPlants({ q, field, distanceKm? })`: 발전소 검색
- `linkPlant({ plant_uuid, link_id?, remark })`: 발전소 링크 생성 시도
- `listLinkedPlants()`: 내 링크 목록 조회
- `getPlantInfo(linkId)`: 발전소 기본 정보
- `getPlantContract(linkId)`: 계약 정보
- `getPlantDocuments(linkId)`: 문서 목록
- `getPlantOverview(linkId)`: 발전소 요약 정보
- `getMonthlyGeneration(linkId, { startYear?, endYear? })`: 월별 발전량
- `getMonthlyBilling(linkId, { startYear?, endYear? })`: 월별 청구량

## 실전 예시 (링크 1건 기준 조회 흐름)

```ts
import { ApiError, OsolarLinkClient } from "@conalog/osolar-client";

async function main() {
  const client = new OsolarLinkClient({ apiKey: process.env.OSOLAR_API_KEY! });

  try {
    const links = await client.listLinkedPlants();
    const first = links.data?.[0];
    if (!first) {
      console.log("No linked plants");
      return;
    }

    const info = await client.getPlantInfo(first.link_id);
    const overview = await client.getPlantOverview(first.link_id);

    console.log(info.data?.plant_name);
    console.log(overview.data?.billing_summary?.[0]);
  } catch (error) {
    if (error instanceof ApiError) {
      console.error(error.status, error.responseBody);
      return;
    }
    console.error(error);
  }
}

main();
```

## 테스트 / 예제 실행

```bash
cd clients/ts
npm test

# 실제 API 간단 점검
OSOLAR_API_KEY=... npm run example:live

# 전체 경로 점검
OSOLAR_API_KEY=... npm run example:live-all
```
