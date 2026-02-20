/** API 기본 응답 규격 */
export interface ApiResponse<T> {
  /** 요청 성공 여부 */
  success: boolean;
  /** 실패 시 에러 메시지 */
  message?: string | null;
  /** 성공 시 반환되는 데이터 */
  data?: T | null;
}

/** 지도 상의 좌표(경도, 위도) */
export interface GeoPoint {
  type?: string;
  /** [경도, 위도] 형식의 좌표 배열 */
  coordinates: number[];
}

/** 발전소 소유주(사업자) 정보 */
export interface PlantOwner {
  business_number: string;
  firm_name: string;
  firm_address: string;
  representative_name: string;
}

/** 발전소 Feature 속성 */
export interface PlantFeatureProperties {
  plant_uuid: string;
  plant_name: string;
  plant_address: string;
  plant_capacity: number;
  plant_owner: PlantOwner;
}

/** GeoJSON Feature */
export interface PlantFeature {
  type?: string;
  geometry: GeoPoint;
  properties: PlantFeatureProperties;
}

/** 발전소 검색 API 응답 (GeoJSON) */
export interface PlantGeoJSONResponse {
  type?: string;
  features: PlantFeature[];
}

/** 발전소 링크 연동 요청 파라미터 */
export interface PlantLinkRequest {
  /** 검색 API를 통해 얻은 발전소 고유 식별자 */
  plant_uuid: string;
  /** 클라이언트 시스템에서 사용하는 자체 발전소 ID (선택사항) */
  link_id?: string | null;
  /** 연동 요청에 대한 메모나 부가 설명 */
  remark: string;
}

/** 발전소 링크 연동 결과 */
export interface PlantLinkResponse {
  /** 연동된 외부 링크 식별자 (link_id 지정 시 그 값, 미지정 시 자동 생성) */
  link_id: string;
  /** 연동 생성 일시 (ISO 8601) */
  created_at: string;
}

/** 연동된 발전소 목록 항목 */
export interface PlantLinkListResponse {
  link_id: string;
  plant_name: string;
  plant_address?: string | null;
  remark?: string | null;
  created_at?: string | null;
}

/** 발전소 상세 기본 정보 */
export interface PlantInfoResponse {
  link_id: string;
  plant_name: string;
  plant_address: string;
  plant_capacity: string;
  plant_certified: boolean;
  plant_geometry: GeoPoint;
  plant_owner: PlantOwner;
}

/** REC 장기고정계약 규격 */
export interface RecFixedContractInfo {
  /** ESS 설비 포함 여부 */
  ess: boolean;
  /** 계약 대상처 (예: 한국수력원자력) */
  target: string;
  /** 계약 단가 유형 */
  price_type?: string | null;
  /** 계약 단가 */
  price?: number | null;
  /** 계약 시작일 */
  start_date?: string | null;
  /** 계약 종료일 */
  end_date?: string | null;
  /** 계약 기간(년) */
  contract_years?: number | null;
}

/** 발전소 계약 정보 (PPA 및 REC) */
export interface PlantContractResponse {
  /** PPA 유형 (예: 한전 PPA) */
  ppa_type: string;
  /** REC 거래 유형 (고정, 현물 등) */
  rec_trade_type: string;
  /** REC 계약 상세 리스트 */
  rec_contracts: RecFixedContractInfo[];
}

/** 발전소 관련 증빙 문서 목록 */
export interface DocumentResponse {
  document_id: string;
  document_code: string;
  document_name: string;
  file_name: string;
  /** 문서 다운로드 URL (만료 기한이 있는 서명된 URL) */
  download_url: string;
}

/** 월별 청구액 요약 */
export interface BillingAmountResponse {
  /** YYYY-MM 형식의 청구 월 */
  billing_month: string;
  /** SMP 정산액 */
  smp_billing_amount: number;
  /** REC 정산액 */
  rec_billing_amount: number | null;
}

/** 월별 발전량 요약 */
export interface GenerationAmountResponse {
  /** YYYY-MM 형식의 발전 월 */
  generation_month: string;
  /** 발전량 (kWh) */
  generation_amount: number;
  /** 인근 평균 발전량 (비교용) */
  nearby_generation_amount: number;
}

/** 작업 유형 */
export type TaskType = "REC_ISSUANCE" | "SMP_BILLING" | "REC_BILLING" | "REC_SPOT_TRADING";
/** 작업 상태 */
export type TaskStatus = "완료" | "진행중" | "실패" | "대기";

/** 최근 작업 이력 상세 */
export interface TaskDetail {
  /** 작업 기준일 */
  date?: string | null;
  /** 작업명 */
  name: string;
  /** 작업 구분 */
  type: TaskType;
  /** 작업 진행 결과 */
  status: TaskStatus;
  /** 작업 내용 요약 */
  summary: string;
  /** 작업 결과값 (금액, 거래량 등) */
  value: string;
}

/** 발전소 통합 대시보드 요약 정보 */
export interface PlantOverviewResponse {
  link_id: string;
  plant_name: string;
  /** 최근 청구 요약 목록 */
  billing_summary: BillingAmountResponse[];
  /** 최근 주요 업무 진행 내역 */
  recent_tasks: TaskDetail[];
}

/** 발전소 검색 질의 파라미터 */
export interface SearchPlantsParams {
  /** 검색어 (사업자번호 또는 주소) */
  q: string;
  /** 검색 기준 필드 */
  field: SearchPlantsField;
  /** 주소 검색 시 인접 반경(km) 제한 (단위: km) */
  distanceKm?: number;
}

export type SearchPlantsField = "business_number" | "address";

/** 월별 발전량 조회 조건 */
export interface MonthlyGenerationParams {
  /** 조회 시작 연도 (YYYY) */
  startYear?: number;
  /** 조회 종료 연도 (YYYY) */
  endYear?: number;
}

/** 월별 청구액 조회 조건 */
export interface MonthlyBillingParams {
  /** 조회 시작 연도 (YYYY) */
  startYear?: number;
  /** 조회 종료 연도 (YYYY) */
  endYear?: number;
}

/** 공통 API 요청 옵션 */
export interface RequestOptions {
  /** 요청을 취소할 수 있는 AbortSignal 객체 */
  signal?: AbortSignal;
  /** 
   * 이 요청에만 적용될 개별 타임아웃(밀리초).
   * 클라이언트 생성 시점의 기본 타임아웃 설정을 덮어씁니다.
   */
  timeout?: number;
}
