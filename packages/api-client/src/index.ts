import type {
  ApiErrorBody,
  Appointment,
  AuthResponse,
  BiomarkerDefinition,
  ChartResponse,
  DashboardResponse,
  Document,
  HealthResponse,
    ImportResult,
    LabTest,
    SessionInfo,
  Measurement,
  Medication,
  Note,
  ActivityEvent,
  Profile,
  Symptom,
  TimelineItem,
  User,
  Workout,
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
    if (!headers.has("Accept")) {
      headers.set("Accept", "application/json");
    }
    if (init.body && !(init.body instanceof FormData) && !headers.has("Content-Type")) {
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

    const contentType = response.headers.get("content-type") ?? "";
    if (!contentType.includes("application/json")) {
      if (!response.ok) {
        throw new ApiError(response.status, "REQUEST_FAILED", "Something went wrong");
      }
      return response as T;
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
    changePassword: (body: { current_password: string; new_password: string }) =>
      request<void>("/api/v1/auth/password", { method: "PUT", body: JSON.stringify(body) }),
    sessions: () => request<SessionInfo[]>("/api/v1/sessions"),
    revokeSession: (id: string) => request<void>(`/api/v1/sessions/${id}`, { method: "DELETE" }),
    deleteAccount: (password: string) =>
      request<void>("/api/v1/me", { method: "DELETE", body: JSON.stringify({ password }) }),
    me: () => request<User>("/api/v1/me"),
    updateMe: (body: { display_name?: string; timezone?: string }) =>
      request<User>("/api/v1/me", { method: "PUT", body: JSON.stringify(body) }),
    profile: () => request<Profile>("/api/v1/profile"),
    updateProfile: (body: Record<string, unknown>) =>
      request<Profile>("/api/v1/profile", { method: "PUT", body: JSON.stringify(body) }),
    dashboard: () => request<DashboardResponse>("/api/v1/dashboard"),
    timeline: (kind = "all") =>
      request<{ items: TimelineItem[] }>(`/api/v1/timeline?kind=${encodeURIComponent(kind)}`),
    search: (q: string) =>
      request<{ items: TimelineItem[] }>(`/api/v1/search?q=${encodeURIComponent(q)}`),
    downloadDocument: async (id: string) => {
      const headers = new Headers();
      const token = options.getToken?.();
      if (token) {
        headers.set("Authorization", `Bearer ${token}`);
      }
      const response = await fetchImpl(`${options.baseUrl}/api/v1/documents/${id}/file`, {
        headers,
        credentials: "include",
      });
      if (!response.ok) {
        throw new ApiError(response.status, "REQUEST_FAILED", "Could not download the file");
      }
      const blob = await response.blob();
      const disposition = response.headers.get("content-disposition") ?? "";
      const match = /filename="([^"]+)"/.exec(disposition);
      return { blob, filename: match?.[1] ?? "document" };
    },
    charts: (metric: string, days = 90) =>
      request<ChartResponse>(`/api/v1/charts?metric=${encodeURIComponent(metric)}&days=${days}`),
    exportUrl: (format: "json" | "csv") => `/api/v1/export?format=${format}`,
    importJson: (body: unknown) =>
      request<ImportResult>("/api/v1/import", { method: "POST", body: JSON.stringify(body) }),
    importCsv: (csv: string) =>
      request<ImportResult>("/api/v1/import/csv", {
        method: "POST",
        headers: { "Content-Type": "text/csv" },
        body: csv,
      }),
    importAppleHealth: (xml: string) =>
      request<ImportResult>("/api/v1/import/apple-health", {
        method: "POST",
        headers: { "Content-Type": "application/xml" },
        body: xml,
      }),
    listMeasurements: (type?: string) =>
      request<Measurement[]>(
        type ? `/api/v1/measurements?type=${encodeURIComponent(type)}` : "/api/v1/measurements",
      ),
    createMeasurement: (body: {
      type: string;
      value: number;
      unit: string;
      measured_at?: string;
      notes?: string;
    }) =>
      request<Measurement>("/api/v1/measurements", {
        method: "POST",
        body: JSON.stringify(body),
      }),
    createBloodPressure: (body: {
      systolic: number;
      diastolic: number;
      unit?: string;
      measured_at?: string;
    }) =>
      request<{ systolic: Measurement; diastolic: Measurement }>(
        "/api/v1/measurements/blood-pressure",
        { method: "POST", body: JSON.stringify(body) },
      ),
    deleteMeasurement: (id: string) =>
      request<void>(`/api/v1/measurements/${id}`, { method: "DELETE" }),
    listLabs: () => request<LabTest[]>("/api/v1/labs"),
    biomarkers: () => request<BiomarkerDefinition[]>("/api/v1/labs/biomarkers"),
    createLab: (body: unknown) =>
      request<LabTest>("/api/v1/labs", { method: "POST", body: JSON.stringify(body) }),
    deleteLab: (id: string) => request<void>(`/api/v1/labs/${id}`, { method: "DELETE" }),
    listDocuments: () => request<Document[]>("/api/v1/documents"),
    uploadDocument: (form: FormData) =>
      request<Document>("/api/v1/documents", { method: "POST", body: form }),
    documentFileUrl: (id: string) => `/api/v1/documents/${id}/file`,
    deleteDocument: (id: string) =>
      request<void>(`/api/v1/documents/${id}`, { method: "DELETE" }),
    listWorkouts: () => request<Workout[]>("/api/v1/workouts"),
    createWorkout: (body: unknown) =>
      request<Workout>("/api/v1/workouts", { method: "POST", body: JSON.stringify(body) }),
    deleteWorkout: (id: string) =>
      request<void>(`/api/v1/workouts/${id}`, { method: "DELETE" }),
    listMedications: () => request<Medication[]>("/api/v1/medications"),
    createMedication: (body: unknown) =>
      request<Medication>("/api/v1/medications", { method: "POST", body: JSON.stringify(body) }),
    deleteMedication: (id: string) =>
      request<void>(`/api/v1/medications/${id}`, { method: "DELETE" }),
    listSymptoms: () => request<Symptom[]>("/api/v1/symptoms"),
    createSymptom: (body: unknown) =>
      request<Symptom>("/api/v1/symptoms", { method: "POST", body: JSON.stringify(body) }),
    deleteSymptom: (id: string) =>
      request<void>(`/api/v1/symptoms/${id}`, { method: "DELETE" }),
    listAppointments: () => request<Appointment[]>("/api/v1/appointments"),
    createAppointment: (body: unknown) =>
      request<Appointment>("/api/v1/appointments", { method: "POST", body: JSON.stringify(body) }),
    deleteAppointment: (id: string) =>
      request<void>(`/api/v1/appointments/${id}`, { method: "DELETE" }),
    listNotes: () => request<Note[]>("/api/v1/notes"),
    createNote: (body: { title: string; body?: string; noted_at?: string }) =>
      request<Note>("/api/v1/notes", { method: "POST", body: JSON.stringify(body) }),
    deleteNote: (id: string) => request<void>(`/api/v1/notes/${id}`, { method: "DELETE" }),
    activity: () => request<ActivityEvent[]>("/api/v1/activity"),
  };
}

export type ApiClient = ReturnType<typeof createApiClient>;
