import {
  ApiResponse,
  BillingAmountResponse,
  DocumentResponse,
  GenerationAmountResponse,
  MonthlyBillingParams,
  MonthlyGenerationParams,
  PlantContractResponse,
  PlantGeoJSONResponse,
  PlantInfoResponse,
  PlantLinkListResponse,
  PlantLinkRequest,
  PlantLinkResponse,
  PlantOverviewResponse,
  RecFixedContractInfo,
  RequestOptions,
  SearchPlantsParams,
} from "./types.js";

/** 클라이언트 생성 설정 */
export interface OsolarLinkClientConfig {
  /** OSOLAR-LINK API 발급 키 */
  apiKey: string;
  /** 
   * API 서버 기본 URL 
   * @default "https://openapi.osolar.io"
   */
  baseUrl?: string;
  /** 사용할 fetch 함수 주입 (Node 18+ 내장 fetch 등) */
  fetchFn?: typeof fetch;
  /** 
   * 기본 타임아웃(밀리초).
   * 지정하지 않으면 제한 없이 요청을 기다립니다.
   */
  timeout?: number;
}

/**
 * OSOLAR-LINK API 에러 래퍼
 * API 요청이 2xx 이외의 상태 코드를 반환할 때 발생합니다.
 */
export class ApiError extends Error {
  readonly status: number;
  readonly statusText: string;
  readonly responseBody: unknown;

  constructor(status: number, statusText: string, responseBody: unknown) {
    super(`OSOLAR API error ${status} ${statusText}`);
    this.name = "ApiError";
    this.status = status;
    this.statusText = statusText;
    this.responseBody = responseBody;
  }
}

/**
 * OSOLAR-LINK API 연동 클라이언트
 * 발전소 검색, 연동, 정보 조회 및 발전/정산 데이터 조회를 지원합니다.
 */
export class OsolarLinkClient {
  private readonly apiKey: string;
  private readonly baseUrl: string;
  private readonly fetchFn: typeof fetch;
  private readonly timeout?: number;

  constructor(config: OsolarLinkClientConfig) {
    this.apiKey = assertNonEmptyString(config.apiKey, "apiKey");
    this.baseUrl = normalizeBaseUrl(config.baseUrl);
    this.fetchFn = resolveFetchFn(config.fetchFn);
    this.timeout = config.timeout;
  }

  /**
   * 발전소 검색
   * 사업자번호 또는 주소로 발전소를 검색합니다.
   *
   * @param params 검색 질의 파라미터
   * @param options 추가 요청 옵션 (Timeout, AbortSignal)
   * @returns 발전소 GeoJSON 피처 컬렉션
   */
  async searchPlants(
    params: SearchPlantsParams,
    options?: RequestOptions,
  ): Promise<ApiResponse<PlantGeoJSONResponse>> {
    const q = assertNonEmptyString(params.q, "q");
    const field = assertNonEmptyString(params.field, "field");
    return this.request<ApiResponse<PlantGeoJSONResponse>>("GET", "/v1/search", {
      query: {
        q,
        field,
        distance_km: params.distanceKm,
      },
      ...options,
    });
  }

  /**
   * 발전소 링크 연동
   * 검색된 발전소를 고객사의 시스템(RTU 등)과 연결합니다.
   *
   * @param body 연동 요청 정보
   * @param options 추가 요청 옵션
   * @returns 연동 결과 (link_id 반환)
   */
  async linkPlant(
    body: PlantLinkRequest,
    options?: RequestOptions,
  ): Promise<ApiResponse<PlantLinkResponse>> {
    const normalizedBody: PlantLinkRequest = {
      ...body,
      plant_uuid: assertNonEmptyString(body.plant_uuid, "plant_uuid"),
      remark: assertNonEmptyString(body.remark, "remark"),
    };
    return this.request<ApiResponse<PlantLinkResponse>>("POST", "/v1/links", {
      body: normalizedBody,
      ...options,
    });
  }

  /**
   * 연동된 발전소 목록 조회
   * 현재 API 키로 권한이 있는 링크 목록을 불러옵니다.
   *
   * @param options 추가 요청 옵션
   * @returns 링크된 발전소 요약 목록
   */
  async listLinkedPlants(options?: RequestOptions): Promise<ApiResponse<PlantLinkListResponse[]>> {
    return this.request<ApiResponse<PlantLinkListResponse[]>>("GET", "/v1/links", options);
  }

  /**
   * 발전소 상세 기본 정보 조회
   * 단일 발전소의 전체 용량, 인증 여부 등의 상세 정보를 반환합니다.
   *
   * @param linkId 연동된 링크 식별자
   * @param options 추가 요청 옵션
   * @returns 발전소 기본 정보
   */
  async getPlantInfo(linkId: string, options?: RequestOptions): Promise<ApiResponse<PlantInfoResponse>> {
    const validatedLinkId = assertNonEmptyString(linkId, "linkId");
    return this.request<ApiResponse<PlantInfoResponse>>(
      "GET",
      `/v1/links/${encodeURIComponent(validatedLinkId)}`,
      options,
    );
  }

  /**
   * 발전소 계약 정보 조회
   * PPA 및 REC 계약 유형과 장기고정계약 상세 내역을 반환합니다.
   *
   * @param linkId 연동된 링크 식별자
   * @param options 추가 요청 옵션
   * @returns 발전소의 최신 계약 구조
   */
  async getPlantContract(linkId: string, options?: RequestOptions): Promise<ApiResponse<PlantContractResponse>> {
    const validatedLinkId = assertNonEmptyString(linkId, "linkId");
    const response = await this.request<ApiResponse<PlantContractResponse | LegacyPlantContractResponse>>(
      "GET",
      `/v1/links/${encodeURIComponent(validatedLinkId)}/contract`,
      options,
    );
    return normalizePlantContractResponse(response);
  }

  /**
   * 발전소 증빙 문서 조회
   * 발전업허가증, 사업자등록증 등의 문서 리스트를 반환합니다.
   *
   * @param linkId 연동된 링크 식별자
   * @param options 추가 요청 옵션
   * @returns 다운로드 가능한 문서 메타데이터
   */
  async getPlantDocuments(linkId: string, options?: RequestOptions): Promise<ApiResponse<DocumentResponse[]>> {
    const validatedLinkId = assertNonEmptyString(linkId, "linkId");
    return this.request<ApiResponse<DocumentResponse[]>>(
      "GET",
      `/v1/links/${encodeURIComponent(validatedLinkId)}/documents`,
      options,
    );
  }

  /**
   * 발전소 통합 대시보드 데이터 조회
   * 발전소의 기본 정보 요약과 최근 3개월 청구액, 주요 업무 태스크 등을 결합하여 제공합니다.
   *
   * @param linkId 연동된 링크 식별자
   * @param options 추가 요청 옵션
   * @returns 통합 요약 정보
   */
  async getPlantOverview(linkId: string, options?: RequestOptions): Promise<ApiResponse<PlantOverviewResponse>> {
    const validatedLinkId = assertNonEmptyString(linkId, "linkId");
    return this.request<ApiResponse<PlantOverviewResponse>>(
      "GET",
      `/v1/links/${encodeURIComponent(validatedLinkId)}/overview`,
      options,
    );
  }

  /**
   * 월별 발전량 데이터 조회
   * 지정한 기간(예: 2023년~2024년)의 월 단위 누적 발전량을 불러옵니다.
   *
   * @param linkId 연동된 링크 식별자
   * @param params 연도 기반 조회 조건
   * @param options 추가 요청 옵션
   * @returns 월별 발전량 배열
   */
  async getMonthlyGeneration(
    linkId: string,
    params: MonthlyGenerationParams = {},
    options?: RequestOptions,
  ): Promise<ApiResponse<GenerationAmountResponse[]>> {
    const validatedLinkId = assertNonEmptyString(linkId, "linkId");
    this.validateYearRange(params, "getMonthlyGeneration");
    return this.request<ApiResponse<GenerationAmountResponse[]>>(
      "GET",
      `/v1/links/${encodeURIComponent(validatedLinkId)}/generation/monthly`,
      {
        query: {
          start_year: params.startYear,
          end_year: params.endYear,
        },
        ...options,
      },
    );
  }

  /**
   * 월별 청구/정산액 데이터 조회
   * REC 및 SMP 월별 정산 내역을 불러옵니다.
   *
   * @param linkId 연동된 링크 식별자
   * @param params 연도 기반 조회 조건
   * @param options 추가 요청 옵션
   * @returns 월별 정산액 배열
   */
  async getMonthlyBilling(
    linkId: string,
    params: MonthlyBillingParams = {},
    options?: RequestOptions,
  ): Promise<ApiResponse<BillingAmountResponse[]>> {
    const validatedLinkId = assertNonEmptyString(linkId, "linkId");
    this.validateYearRange(params, "getMonthlyBilling");
    return this.request<ApiResponse<BillingAmountResponse[]>>(
      "GET",
      `/v1/links/${encodeURIComponent(validatedLinkId)}/billing/monthly`,
      {
        query: {
          startYear: params.startYear,
          endYear: params.endYear,
        },
        ...options,
      },
    );
  }

  private async request<T>(
    method: "GET" | "POST",
    path: string,
    options: {
      query?: Record<string, string | number | boolean | null | undefined>;
      body?: unknown;
      signal?: AbortSignal;
      timeout?: number;
    } = {},
  ): Promise<T> {
    const url = new URL(path.replace(/^\/+/, ""), this.baseUrl);
    if (options.query) {
      for (const [key, value] of Object.entries(options.query)) {
        if (value !== undefined && value !== null) {
          url.searchParams.set(key, String(value));
        }
      }
    }

    const headers: Record<string, string> = {
      "x-api-key": this.apiKey,
    };

    let body: string | undefined;
    if (options.body !== undefined) {
      headers["content-type"] = "application/json";
      body = JSON.stringify(options.body);
    }

    let signal = options.signal;
    let timeoutId: ReturnType<typeof setTimeout> | undefined;

    const effectiveTimeout = options.timeout ?? this.timeout;

    if (!signal && effectiveTimeout !== undefined) {
      if (typeof AbortSignal !== "undefined" && "timeout" in AbortSignal && typeof AbortSignal.timeout === "function") {
        signal = AbortSignal.timeout(effectiveTimeout);
      } else if (typeof AbortController !== "undefined") {
        const controller = new AbortController();
        signal = controller.signal;
        timeoutId = setTimeout(() => controller.abort(new Error("Request timeout")), effectiveTimeout);
      }
    }

    try {
      const response = await this.fetchFn(url, {
        method,
        headers,
        body,
        signal,
        redirect: "manual",
      });

      const text = await response.text();
      const parsed = parseResponseBody(text);

      if (!response.ok) {
        throw new ApiError(
          response.status,
          response.statusText,
          parsed.kind === "invalid" ? parsed.rawText : parsed.value,
        );
      }

      if (parsed.kind === "empty") {
        throw new TypeError("Expected JSON response body but received empty body");
      }

      if (parsed.kind === "invalid") {
        throw new TypeError("Expected JSON response but received non-JSON body");
      }

      if (parsed.value === null || Array.isArray(parsed.value) || typeof parsed.value !== "object") {
        throw new TypeError("Expected JSON object response");
      }

      if (!("success" in parsed.value) || typeof parsed.value.success !== "boolean") {
        throw new TypeError("Expected ApiResponse envelope with boolean success");
      }

      return parsed.value as T;
    } finally {
      if (timeoutId !== undefined) {
        clearTimeout(timeoutId);
      }
    }
  }

  private validateYearRange(
    params: { startYear?: number; endYear?: number },
    methodName: "getMonthlyGeneration" | "getMonthlyBilling",
  ): void {
    if (
      params.startYear !== undefined &&
      params.endYear !== undefined &&
      params.startYear > params.endYear
    ) {
      throw new RangeError(
        `${methodName}: startYear (${params.startYear}) must be less than or equal to endYear (${params.endYear})`,
      );
    }
  }
}

type LegacyPlantContractResponse = Omit<PlantContractResponse, "rec_contracts"> & {
  rec_contracts?: RecFixedContractInfo[] | null;
  rec_fixed_contract?: RecFixedContractInfo | RecFixedContractInfo[] | null;
};

function normalizePlantContractResponse(
  response: ApiResponse<PlantContractResponse | LegacyPlantContractResponse>,
): ApiResponse<PlantContractResponse> {
  const rawData = response.data;
  if (rawData === null || rawData === undefined || typeof rawData !== "object" || Array.isArray(rawData)) {
    return response as ApiResponse<PlantContractResponse>;
  }

  const data = rawData as LegacyPlantContractResponse;
  if (typeof data.ppa_type !== "string" || typeof data.rec_trade_type !== "string") {
    return response as ApiResponse<PlantContractResponse>;
  }

  return {
    ...response,
    data: {
      ppa_type: data.ppa_type,
      rec_trade_type: data.rec_trade_type,
      rec_contracts: normalizeRecContracts(data),
    },
  };
}

function normalizeRecContracts(data: LegacyPlantContractResponse): RecFixedContractInfo[] {
  if (Array.isArray(data.rec_contracts)) {
    return data.rec_contracts.map(withDefaultEss);
  }
  if (Array.isArray(data.rec_fixed_contract)) {
    return data.rec_fixed_contract.map(withDefaultEss);
  }
  if (data.rec_fixed_contract && typeof data.rec_fixed_contract === "object") {
    return [withDefaultEss(data.rec_fixed_contract)];
  }
  return [];
}

function withDefaultEss(contract: RecFixedContractInfo): RecFixedContractInfo {
  return {
    ...contract,
    ess: typeof contract.ess === "boolean" ? contract.ess : false,
  };
}

function parseResponseBody(text: string):
  | { kind: "empty"; value: null }
  | { kind: "json"; value: unknown }
  | { kind: "invalid"; rawText: string } {
  if (text.length === 0) {
    return { kind: "empty", value: null };
  }
  try {
    return { kind: "json", value: JSON.parse(text) };
  } catch {
    return { kind: "invalid", rawText: text };
  }
}

function assertNonEmptyString(value: string, name: string): string {
  if (typeof value !== "string" || value.trim().length === 0) {
    throw new TypeError(`${name} must be a non-empty string`);
  }
  return value.trim();
}

function normalizeBaseUrl(baseUrl: string | undefined): string {
  const raw = baseUrl ?? "https://openapi.osolar.io";
  const parsed = new URL(raw);

  if (parsed.protocol !== "https:" && parsed.protocol !== "http:") {
    throw new RangeError(`baseUrl must be an absolute http(s) URL (got: ${parsed.protocol})`);
  }

  if (parsed.search || parsed.hash) {
    throw new RangeError("baseUrl must not include query parameters or a fragment");
  }

  if (parsed.protocol === "http:" && !isLocalhostHostname(parsed.hostname)) {
    throw new RangeError(
      "baseUrl must use https (http is only allowed for localhost) to avoid sending x-api-key over plaintext HTTP",
    );
  }

  if (!parsed.pathname.endsWith("/")) {
    parsed.pathname += "/";
  }
  parsed.search = "";
  parsed.hash = "";

  return parsed.toString();
}

function isLocalhostHostname(hostname: string): boolean {
  return hostname === "localhost" || hostname === "127.0.0.1" || hostname === "::1";
}

function resolveFetchFn(fetchFn: typeof fetch | undefined): typeof fetch {
  if (fetchFn) {
    return fetchFn;
  }

  const globalFetch = (globalThis as unknown as { fetch?: unknown }).fetch;
  if (typeof globalFetch === "function") {
    return globalFetch.bind(globalThis) as typeof fetch;
  }

  return (async () => {
    throw new ReferenceError(
      "fetch is not defined in this runtime. Pass config.fetchFn (for example, undici's fetch) when constructing OsolarLinkClient.",
    );
  }) as unknown as typeof fetch;
}
