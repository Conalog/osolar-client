# @osolar-sdk/osolar-link-client

수동 구현된 TypeScript SDK입니다.

## 설치

```bash
npm install
```

## 사용 예시

```ts
import { OsolarLinkClient } from "@osolar-sdk/osolar-link-client";

const client = new OsolarLinkClient({ apiKey: process.env.OSOLAR_API_KEY! });
const plants = await client.listLinkedPlants();
console.log(plants.data);
```

## 실제 API 스모크 테스트

```bash
npm run build
OSOLAR_API_KEY=... npm run example:live
OSOLAR_API_KEY=... npm run example:live-all
```
