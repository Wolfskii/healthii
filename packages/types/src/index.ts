export type MeasurementType =
  | "weight"
  | "height"
  | "bmi"
  | "body_fat"
  | "blood_pressure_systolic"
  | "blood_pressure_diastolic"
  | "resting_heart_rate"
  | "heart_rate"
  | "blood_glucose"
  | "body_temperature"
  | "oxygen_saturation"
  | "waist_circumference";

export type LabStatus = "low" | "normal" | "high" | "critical" | "unknown";

export type DocumentType =
  | "doctor_report"
  | "blood_test"
  | "prescription"
  | "imaging"
  | "vaccination"
  | "referral"
  | "discharge_summary"
  | "other";

export type WorkoutType =
  | "strength"
  | "running"
  | "cycling"
  | "walking"
  | "swimming"
  | "sports"
  | "mobility"
  | "other";

export interface User {
  id: string;
  email: string;
  display_name: string;
  timezone: string;
  locale: string;
  created_at: string;
  updated_at: string;
}

export interface AuthResponse {
  token: string;
  token_type: string;
  expires_in: number;
  user: User;
}

export interface ApiErrorBody {
  error: {
    code: string;
    message: string;
  };
}

export interface DashboardWidget {
  id: string;
  title: string;
  kind: string;
  empty: boolean;
  summary: string | null;
}

export interface DashboardResponse {
  mode: string;
  disclaimer: string;
  generated_at: string;
  widgets: DashboardWidget[];
}

export interface HealthResponse {
  status: string;
  service: string;
}

export const MEDICAL_DISCLAIMER =
  "Healthii is a personal health tracking and organization tool. It does not provide medical diagnosis or replace professional medical advice.";
