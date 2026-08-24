import { createApiClient } from "@healthii/api-client";

const TOKEN_KEY = "healthii.token";

export const api = createApiClient({
  baseUrl: import.meta.env.VITE_API_BASE_URL ?? "http://localhost:8080",
  getToken: () => {
    try {
      return window.localStorage.getItem(TOKEN_KEY);
    } catch {
      return null;
    }
  },
});
