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
  | "waist_circumference"
  | "sleep";

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
  hint?: string | null;
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

export interface Measurement {
  id: string;
  user_id: string;
  type: MeasurementType | string;
  value: number;
  unit: string;
  measured_at: string;
  source: string;
  notes: string | null;
  created_at: string;
  updated_at: string;
}

export interface LabResult {
  id: string;
  lab_test_id: string;
  biomarker_code: string;
  name: string | null;
  value: number | null;
  value_text: string | null;
  unit: string | null;
  reference_low: number | null;
  reference_high: number | null;
  reference_text: string | null;
  status: LabStatus;
}

export interface LabTest {
  id: string;
  user_id: string;
  tested_on: string;
  laboratory: string | null;
  notes: string | null;
  results: LabResult[];
  created_at: string;
  updated_at: string;
}

export interface BiomarkerDefinition {
  code: string;
  name: string;
  default_unit: string | null;
  category: string | null;
}

export interface Document {
  id: string;
  user_id: string;
  filename: string;
  mime_type: string;
  size: number;
  document_type: DocumentType | string;
  title: string | null;
  description: string | null;
  document_date: string | null;
  uploaded_at: string;
}

export interface ExerciseSet {
  repetitions?: number | null;
  weight?: number | null;
  weight_unit?: string | null;
  duration_seconds?: number | null;
}

export interface Exercise {
  name: string;
  sets: ExerciseSet[];
}

export interface Workout {
  id: string;
  user_id: string;
  workout_type: WorkoutType;
  started_at: string;
  duration_seconds: number | null;
  distance: number | null;
  distance_unit: string | null;
  calories: number | null;
  notes: string | null;
  exercises: Exercise[];
  created_at: string;
  updated_at: string;
}

export interface Medication {
  id: string;
  user_id: string;
  name: string;
  kind: "medication" | "supplement" | string;
  dosage: string | null;
  schedule: string | null;
  started_on: string | null;
  ended_on: string | null;
  notes: string | null;
}

export interface Symptom {
  id: string;
  user_id: string;
  name: string;
  severity: number | null;
  noted_at: string;
  notes: string | null;
}

export interface Note {
  id: string;
  user_id: string;
  title: string;
  body: string;
  noted_at: string;
  created_at: string;
  updated_at: string;
}

export interface ActivityEvent {
  id: string;
  action: string;
  resource_type: string | null;
  created_at: string;
}

export interface Appointment {
  id: string;
  user_id: string;
  title: string;
  location: string | null;
  provider: string | null;
  starts_at: string;
  ends_at: string | null;
  notes: string | null;
}

export interface Profile {
  user_id: string;
  height_cm: number | null;
  blood_type: string | null;
  allergies: string | null;
  medical_history: string | null;
  emergency_name: string | null;
  emergency_phone: string | null;
  unit_system: "metric" | "imperial" | string;
  weight_unit: "kg" | "lb" | string;
  goal_weight: number | null;
  goal_weight_unit: "kg" | "lb" | string | null;
  updated_at: string;
}

export interface SessionInfo {
  id: string;
  user_agent: string | null;
  created_at: string;
  expires_at: string;
  current: boolean;
}

export interface TimelineItem {
  id: string;
  kind: string;
  occurred_at: string;
  title: string;
  detail: string | null;
}

export interface ChartResponse {
  metric: string;
  unit: string | null;
  source: string;
  min: number | null;
  max: number | null;
  average: number | null;
  points: { t: string; v: number }[];
  notice: string;
}

export interface ImportResult {
  measurements: number;
  labs: number;
  workouts: number;
  medications: number;
  symptoms: number;
  appointments: number;
  notes: number;
  skipped: number;
  notice: string;
}

export const MEDICAL_DISCLAIMER =
  "Healthii is a personal health tracking and organization tool. It does not provide medical diagnosis or replace professional medical advice.";
