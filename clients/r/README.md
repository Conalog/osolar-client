# osolarclient (R)

OSOLAR-LINK Open API용 수동 구현 R 클라이언트 패키지입니다.

## 무엇을 제공하나요?

- 9개 엔드포인트 공통 호출 함수
- `osolar_api_error` 조건으로 상태 코드/응답 바디 확인
- 테스트 가능하도록 HTTP 요청 함수 주입(`request_fn`) 지원

## 설치 (이 레포 기준)

```bash
cd clients/r
R -q -e "install.packages(c('httr', 'jsonlite', 'testthat'))"
R CMD INSTALL .
```

## 환경 변수

```bash
export OSOLAR_API_KEY="..."
```

## 빠른 시작

```r
library(osolarclient)

client <- osolar_client(api_key = Sys.getenv("OSOLAR_API_KEY"))
links <- list_linked_plants(client)
print(links$data)
```

## 자주 쓰는 함수

- `search_plants(client, q, field, distance_km = NULL)`: 발전소 검색
- `link_plant(client, plant_uuid = ..., remark = ..., link_id = NULL)`: 발전소 링크 생성 시도
- `list_linked_plants(client)`: 내 링크 목록 조회
- `get_plant_info(client, link_id)`: 발전소 기본 정보
- `get_plant_contract(client, link_id)`: 계약 정보
- `get_plant_documents(client, link_id)`: 문서 목록
- `get_plant_overview(client, link_id)`: 발전소 요약 정보
- `get_monthly_generation(client, link_id, start_year = NULL, end_year = NULL)`: 월별 발전량
- `get_monthly_billing(client, link_id, start_year = NULL, end_year = NULL)`: 월별 청구량

## 테스트

```bash
R -q -e "testthat::test_local('clients/r')"
```
