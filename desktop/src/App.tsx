import { HashRouter, Navigate, Route, Routes } from "react-router-dom";
import { AppShell, DashboardView, PlaceholderPage, ThemeProvider } from "@healthii/ui";

export function App() {
  return (
    <ThemeProvider>
      <HashRouter>
        <AppShell>
          <Routes>
            <Route path="/" element={<DashboardView />} />
            <Route
              path="/timeline"
              element={
                <PlaceholderPage
                  title="Timeline"
                  body="Desktop is the primary Healthii workspace. Your chronology will live here."
                />
              }
            />
            <Route
              path="/measurements"
              element={
                <PlaceholderPage
                  title="Measurements"
                  body="Local cache is ready for later sync. Records themselves arrive in Phase 2."
                />
              }
            />
            <Route
              path="/labs"
              element={
                <PlaceholderPage
                  title="Blood tests"
                  body="Panels and biomarkers without presenting them as a diagnosis."
                />
              }
            />
            <Route
              path="/workouts"
              element={<PlaceholderPage title="Workouts" body="Training history with sets, distance and notes." />}
            />
            <Route
              path="/documents"
              element={
                <PlaceholderPage
                  title="Documents"
                  body="Secure local previews of reports you upload, with ownership checks on the API."
                />
              }
            />
            <Route
              path="/medications"
              element={<PlaceholderPage title="Medications" body="Medications and supplements you choose to track." />}
            />
            <Route
              path="/symptoms"
              element={<PlaceholderPage title="Symptoms" body="Private notes about how you feel." />}
            />
            <Route
              path="/appointments"
              element={<PlaceholderPage title="Appointments" body="Clinic and lab visits on one calendar." />}
            />
            <Route
              path="/insights"
              element={<PlaceholderPage title="Insights" body="Charts and trends over years of history." />}
            />
            <Route
              path="/reports"
              element={<PlaceholderPage title="Reports" body="Export JSON, CSV and PDF when the export service lands." />}
            />
            <Route
              path="/settings"
              element={<PlaceholderPage title="Settings" body="Units, locale, appearance and the API endpoint." />}
            />
            <Route path="*" element={<Navigate to="/" replace />} />
          </Routes>
        </AppShell>
      </HashRouter>
    </ThemeProvider>
  );
}
