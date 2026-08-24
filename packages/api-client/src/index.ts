import type {
  ApiErrorBody,
  AuthResponse,
  DashboardResponse,
  HealthResponse,
  User,
} from "@healthii/types";

export class ApiError extends Error {
  constructor(
    public readonly status: number,
    public readonly code: string,
    message: string,
  ) {
    super(message);
    this.name = "ApiError";
  }
}

export interface ApiClientOptions {
  baseUrl: string;
  getToken?: () => string | null;
  fetch?: typeof fetch;
}

export function createApiClient(options: ApiClientOptions) {
  const fetchImpl = options.fetch ?? fetch;

  async function request<T>(path: string, init: RequestInit = {}): Promise<T> {
    const headers = new Headers(init.headers);
    headers.set("Accept", "application/json");
    if (init.body && !headers.has("Content-Type")) {
      headers.set("Content-Type", "application/json");
    }
    const token = options.getToken?.();
    if (token) {
      headers.set("Authorization", `Bearer ${token}`);
    }

    const response = await fetchImpl(`${options.baseUrl}${path}`, {
      ...init,
      headers,
      credentials: "include",
    });

    if (response.status === 204) {
      return undefined as T;
    }

    const data = (await response.json().catch(() => null)) as T | ApiErrorBody | null;
    if (!response.ok) {
      const error = (data as ApiErrorBody | null)?.error;
      throw new ApiError(
        response.status,
        error?.code ?? "REQUEST_FAILED",
        error?.message ?? "Something went wrong",
      );
    }
    return data as T;
  }

  return {
    health: () => request<HealthResponse>("/health"),
    ready: () => request<HealthResponse>("/ready"),
    register: (body: { email: string; password: string; display_name: string }) =>
      request<AuthResponse>("/api/v1/auth/register", {
        method: "POST",
        body: JSON.stringify(body),
      }),
    login: (body: { email: string; password: string }) =>
      request<AuthResponse>("/api/v1/auth/login", {
        method: "POST",
        body: JSON.stringify(body),
      }),
    logout: () => request<void>("/api/v1/auth/logout", { method: "POST" }),
    me: () => request<User>("/api/v1/me"),
    dashboard: () => request<DashboardResponse>("/api/v1/dashboard"),
  };
}

export type ApiClient = ReturnType<typeof createApiClient>;
