import { HashRouter } from "react-router-dom";
import { HealthiiApp, ThemeProvider } from "@healthii/ui";

const baseUrl = import.meta.env.VITE_API_BASE_URL ?? "http://localhost:8080";

export function App() {
  return (
    <ThemeProvider>
      <HashRouter>
        <HealthiiApp baseUrl={baseUrl} />
      </HashRouter>
    </ThemeProvider>
  );
}
