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
    this.apiKey = config.apiKey;
    this.baseUrl = (config.baseUrl ?? "https://openapi.osolar.io").replace(/\/$/, "");
    this.fetchFn = config.fetchFn ?? fetch;
  }

  async searchPlants(params: SearchPlantsParams): Promise<ApiResponse<PlantGeoJSONResponse>> {
    return this.request<ApiResponse<PlantGeoJSONResponse>>("GET", "/v1/search", {
      query: {
        q: params.q,
        field: params.field,
        distance_km: params.distanceKm,
      },
    });
  }

  async linkPlant(body: PlantLinkRequest): Promise<ApiResponse<PlantLinkResponse>> {
    return this.request<ApiResponse<PlantLinkResponse>>("POST", "/v1/links", { body });
  }

  async listLinkedPlants(): Promise<ApiResponse<PlantLinkListResponse[]>> {
    return this.request<ApiResponse<PlantLinkListResponse[]>>("GET", "/v1/links");
  }

  async getPlantInfo(linkId: string): Promise<ApiResponse<PlantInfoResponse>> {
    return this.request<ApiResponse<PlantInfoResponse>>("GET", `/v1/links/${encodeURIComponent(linkId)}`);
  }

  async getPlantContract(linkId: string): Promise<ApiResponse<PlantContractResponse>> {
    return this.request<ApiResponse<PlantContractResponse>>(
      "GET",
      `/v1/links/${encodeURIComponent(linkId)}/contract`,
    );
  }

  async getPlantDocuments(linkId: string): Promise<ApiResponse<DocumentResponse[]>> {
    return this.request<ApiResponse<DocumentResponse[]>>(
      "GET",
      `/v1/links/${encodeURIComponent(linkId)}/documents`,
    );
  }

  async getPlantOverview(linkId: string): Promise<ApiResponse<PlantOverviewResponse>> {
    return this.request<ApiResponse<PlantOverviewResponse>>("GET", `/v1/links/${encodeURIComponent(linkId)}/overview`);
  }

  async getMonthlyGeneration(
    linkId: string,
    params: MonthlyGenerationParams = {},
  ): Promise<ApiResponse<GenerationAmountResponse[]>> {
    return this.request<ApiResponse<GenerationAmountResponse[]>>(
      "GET",
      `/v1/links/${encodeURIComponent(linkId)}/generation/monthly`,
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
    return this.request<ApiResponse<BillingAmountResponse[]>>(
      "GET",
      `/v1/links/${encodeURIComponent(linkId)}/billing/monthly`,
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
    const parsed = text.length > 0 ? safeParseJson(text) : null;

    if (!response.ok) {
      throw new ApiError(response.status, response.statusText, parsed);
    }

    return parsed as T;
  }
}

function safeParseJson(text: string): unknown {
  try {
    return JSON.parse(text);
  } catch {
    return text;
  }
}
