import { BrowserRouter, Navigate, Route, Routes } from "react-router-dom";
import { AppShell, DashboardView, PlaceholderPage, ThemeProvider } from "@healthii/ui";

export function App() {
  return (
    <ThemeProvider>
      <BrowserRouter>
        <AppShell>
          <Routes>
            <Route path="/" element={<DashboardView />} />
            <Route
              path="/timeline"
              element={
                <PlaceholderPage
                  title="Timeline"
                  body="A single chronology of measurements, labs, workouts, documents and notes."
                />
              }
            />
            <Route
              path="/measurements"
              element={
                <PlaceholderPage
                  title="Measurements"
                  body="Weight, blood pressure, heart rate, glucose and the rest of your vitals."
                />
              }
            />
            <Route
              path="/labs"
              element={
                <PlaceholderPage
                  title="Blood tests"
                  body="Flexible laboratory panels and biomarkers, never presented as a diagnosis."
                />
              }
            />
            <Route
              path="/workouts"
              element={
                <PlaceholderPage
                  title="Workouts"
                  body="Strength, endurance and mobility sessions with sets when you need them."
                />
              }
            />
            <Route
              path="/documents"
              element={
                <PlaceholderPage
                  title="Documents"
                  body="Doctor reports, imaging, prescriptions and lab PDFs in one private archive."
                />
              }
            />
            <Route
              path="/medications"
              element={
                <PlaceholderPage
                  title="Medications"
                  body="Medications and supplements you choose to track."
                />
              }
            />
            <Route
              path="/symptoms"
              element={
                <PlaceholderPage
                  title="Symptoms"
                  body="Private notes about how you feel, without judgment."
                />
              }
            />
            <Route
              path="/appointments"
              element={
                <PlaceholderPage
                  title="Appointments"
                  body="Upcoming clinic, lab and specialist visits."
                />
              }
            />
            <Route
              path="/insights"
              element={
                <PlaceholderPage
                  title="Insights"
                  body="Charts and trends will appear here once you have history to look at."
                />
              }
            />
            <Route
              path="/reports"
              element={
                <PlaceholderPage
                  title="Reports"
                  body="Exportable summaries of your own records. You should never be locked in."
                />
              }
            />
            <Route
              path="/settings"
              element={
                <PlaceholderPage
                  title="Settings"
                  body="Units, locale, appearance and privacy controls."
                />
              }
            />
            <Route path="*" element={<Navigate to="/" replace />} />
          </Routes>
        </AppShell>
      </BrowserRouter>
    </ThemeProvider>
  );
}
