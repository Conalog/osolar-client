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
  SearchPlantsParams,
} from "./types.js";

export interface OsolarLinkClientConfig {
  apiKey: string;
  baseUrl?: string;
  fetchFn?: typeof fetch;
}

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

export class OsolarLinkClient {
  private readonly apiKey: string;
  private readonly baseUrl: string;
  private readonly fetchFn: typeof fetch;

  constructor(config: OsolarLinkClientConfig) {
    this.apiKey = assertNonEmptyString(config.apiKey, "apiKey");
    this.baseUrl = (config.baseUrl ?? "https://openapi.osolar.io").replace(/\/$/, "");
    this.fetchFn = config.fetchFn ?? fetch;
  }

  async searchPlants(params: SearchPlantsParams): Promise<ApiResponse<PlantGeoJSONResponse>> {
    const q = assertNonEmptyString(params.q, "q");
    const field = assertNonEmptyString(params.field, "field");
    return this.request<ApiResponse<PlantGeoJSONResponse>>("GET", "/v1/search", {
      query: {
        q,
        field,
        distance_km: params.distanceKm,
      },
    });
  }

  async linkPlant(body: PlantLinkRequest): Promise<ApiResponse<PlantLinkResponse>> {
    const normalizedBody: PlantLinkRequest = {
      ...body,
      plant_uuid: assertNonEmptyString(body.plant_uuid, "plant_uuid"),
      remark: assertNonEmptyString(body.remark, "remark"),
    };
    return this.request<ApiResponse<PlantLinkResponse>>("POST", "/v1/links", { body: normalizedBody });
  }

  async listLinkedPlants(): Promise<ApiResponse<PlantLinkListResponse[]>> {
    return this.request<ApiResponse<PlantLinkListResponse[]>>("GET", "/v1/links");
  }

  async getPlantInfo(linkId: string): Promise<ApiResponse<PlantInfoResponse>> {
    const validatedLinkId = assertNonEmptyString(linkId, "linkId");
    return this.request<ApiResponse<PlantInfoResponse>>("GET", `/v1/links/${encodeURIComponent(validatedLinkId)}`);
  }

  async getPlantContract(linkId: string): Promise<ApiResponse<PlantContractResponse>> {
    const validatedLinkId = assertNonEmptyString(linkId, "linkId");
    const response = await this.request<ApiResponse<PlantContractResponse | LegacyPlantContractResponse>>(
      "GET",
      `/v1/links/${encodeURIComponent(validatedLinkId)}/contract`,
    );
    return normalizePlantContractResponse(response);
  }

  async getPlantDocuments(linkId: string): Promise<ApiResponse<DocumentResponse[]>> {
    const validatedLinkId = assertNonEmptyString(linkId, "linkId");
    return this.request<ApiResponse<DocumentResponse[]>>(
      "GET",
      `/v1/links/${encodeURIComponent(validatedLinkId)}/documents`,
    );
  }

  async getPlantOverview(linkId: string): Promise<ApiResponse<PlantOverviewResponse>> {
    const validatedLinkId = assertNonEmptyString(linkId, "linkId");
    return this.request<ApiResponse<PlantOverviewResponse>>(
      "GET",
      `/v1/links/${encodeURIComponent(validatedLinkId)}/overview`,
    );
  }

  async getMonthlyGeneration(
    linkId: string,
    params: MonthlyGenerationParams = {},
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
      },
    );
  }

  async getMonthlyBilling(
    linkId: string,
    params: MonthlyBillingParams = {},
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
      },
    );
  }

  private async request<T>(
    method: "GET" | "POST",
    path: string,
    options: {
      query?: Record<string, string | number | boolean | null | undefined>;
      body?: unknown;
    } = {},
  ): Promise<T> {
    const url = new URL(`${this.baseUrl}${path}`);
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

    const response = await this.fetchFn(url, {
      method,
      headers,
      body,
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
