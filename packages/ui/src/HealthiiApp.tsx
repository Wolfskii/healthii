import { useEffect, useState } from "react";
import { Navigate, Route, Routes } from "react-router-dom";
import { AppShell } from "./AppShell";
import { AuthScreen } from "./AuthScreen";
import { DashboardView } from "./DashboardView";
import {
  AppointmentsPage,
  DocumentsPage,
  InsightsPage,
  LabsPage,
  MeasurementsPage,
  MedicationsPage,
  NotesPage,
  ReportsPage,
  SearchPage,
  SettingsPage,
  SymptomsPage,
  TimelinePage,
  WorkoutsPage,
} from "./pages";
import { SessionProvider, useSession } from "./session";

export function HealthiiApp({ baseUrl }: { baseUrl: string }) {
  return (
    <SessionProvider baseUrl={baseUrl}>
      <Gate />
    </SessionProvider>
  );
}

function Gate() {
  const { token, api, setSession } = useSession();
  const [ready, setReady] = useState(!token);

  useEffect(() => {
    if (!token) {
      setReady(true);
      return;
    }
    let cancelled = false;
    api
      .me()
      .then((current) => {
        if (!cancelled) {
          setSession(token, current);
          setReady(true);
        }
      })
      .catch(() => {
        if (!cancelled) {
          setSession(null, null);
          setReady(true);
        }
      });
    return () => {
      cancelled = true;
    };
  }, [api, setSession, token]);

  if (!ready) {
    return (
      <div className="hii-auth">
        <p className="hii-hint">Loading your records…</p>
      </div>
    );
  }

  if (!token) {
    return <AuthScreen />;
  }

  return (
    <AppShell>
      <Routes>
        <Route path="/" element={<DashboardView />} />
        <Route path="/timeline" element={<TimelinePage />} />
        <Route path="/search" element={<SearchPage />} />
        <Route path="/measurements" element={<MeasurementsPage />} />
        <Route path="/labs" element={<LabsPage />} />
        <Route path="/workouts" element={<WorkoutsPage />} />
        <Route path="/documents" element={<DocumentsPage />} />
        <Route path="/medications" element={<MedicationsPage />} />
        <Route path="/symptoms" element={<SymptomsPage />} />
        <Route path="/notes" element={<NotesPage />} />
        <Route path="/appointments" element={<AppointmentsPage />} />
        <Route path="/insights" element={<InsightsPage />} />
        <Route path="/reports" element={<ReportsPage />} />
        <Route path="/settings" element={<SettingsPage />} />
        <Route path="*" element={<Navigate to="/" replace />} />
      </Routes>
    </AppShell>
  );
}
