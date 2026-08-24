import { MEDICAL_DISCLAIMER } from "@healthii/types";

export const disclaimer = MEDICAL_DISCLAIMER;

export type NavItem = {
  to: string;
  label: string;
  icon: string;
};

export const desktopNav: NavItem[] = [
  { to: "/", label: "Dashboard", icon: "home" },
  { to: "/timeline", label: "Timeline", icon: "timeline" },
  { to: "/measurements", label: "Measurements", icon: "pulse" },
  { to: "/labs", label: "Blood tests", icon: "flask" },
  { to: "/workouts", label: "Workouts", icon: "activity" },
  { to: "/documents", label: "Documents", icon: "file" },
  { to: "/medications", label: "Medications", icon: "pill" },
  { to: "/symptoms", label: "Symptoms", icon: "note" },
  { to: "/appointments", label: "Appointments", icon: "calendar" },
];

export const desktopSecondaryNav: NavItem[] = [
  { to: "/insights", label: "Insights", icon: "spark" },
  { to: "/reports", label: "Reports", icon: "chart" },
];

export const mobileNav: NavItem[] = [
  { to: "/", label: "Home", icon: "home" },
  { to: "/timeline", label: "Timeline", icon: "timeline" },
  { to: "/add", label: "Add", icon: "add" },
  { to: "/insights", label: "Insights", icon: "spark" },
  { to: "/profile", label: "Profile", icon: "user" },
];

export type QuickAddAction = {
  id: string;
  label: string;
  description: string;
};

export const quickAddActions: QuickAddAction[] = [
  { id: "weight", label: "Weight", description: "Log a weigh-in" },
  { id: "blood-pressure", label: "Blood pressure", description: "Systolic / diastolic" },
  { id: "heart-rate", label: "Heart rate", description: "Resting or current" },
  { id: "glucose", label: "Blood glucose", description: "Record a reading" },
  { id: "workout", label: "Workout", description: "Strength or cardio" },
  { id: "symptom", label: "Symptom", description: "How you feel" },
  { id: "medication", label: "Medication", description: "Dose taken" },
  { id: "lab", label: "Blood test", description: "Add a panel later" },
  { id: "document", label: "Document", description: "PDF or image" },
  { id: "note", label: "Note", description: "A private health note" },
];

export type TimelineKind =
  | "all"
  | "measurements"
  | "labs"
  | "documents"
  | "workouts"
  | "appointments"
  | "medications"
  | "symptoms";

export const timelineFilters: { id: TimelineKind; label: string }[] = [
  { id: "all", label: "All" },
  { id: "measurements", label: "Measurements" },
  { id: "labs", label: "Blood tests" },
  { id: "documents", label: "Documents" },
  { id: "workouts", label: "Workouts" },
  { id: "appointments", label: "Appointments" },
  { id: "medications", label: "Medications" },
  { id: "symptoms", label: "Symptoms" },
];

export type SampleWidget = {
  id: string;
  title: string;
  value: string;
  hint: string;
  sample: true;
};

export const sampleWidgets: SampleWidget[] = [
  { id: "weight", title: "Current weight", value: "—", hint: "No measurements yet", sample: true },
  { id: "trend", title: "Weight trend", value: "7 / 30 / 90 day", hint: "Trends appear after a few weigh-ins", sample: true },
  { id: "bp", title: "Blood pressure", value: "—", hint: "Latest reading will show here", sample: true },
  { id: "hr", title: "Resting heart rate", value: "—", hint: "Add a heart-rate sample", sample: true },
  { id: "labs", title: "Latest blood tests", value: "0 panels", hint: "Laboratory results stay in your archive", sample: true },
  { id: "workouts", title: "Recent workouts", value: "0 this week", hint: "Activity from Healthii or later integrations", sample: true },
  { id: "appointments", title: "Upcoming appointments", value: "None", hint: "Keep clinic visits in one place", sample: true },
  { id: "documents", title: "Recent documents", value: "0 files", hint: "PDF, PNG, JPEG or WEBP", sample: true },
];

export const sampleTimeline = [
  { id: "welcome", dateLabel: "Today", title: "Welcome to Healthii", detail: "Your timeline will collect measurements, labs, workouts and documents in one chronology.", kind: "note" as const, sample: true as const },
];

export const previewNotice =
  "Layout preview — this screen is not showing personal health data.";
