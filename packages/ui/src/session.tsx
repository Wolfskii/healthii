import { createContext, useCallback, useContext, useMemo, useState, type ReactNode } from "react";
import { createApiClient, type ApiClient } from "@healthii/api-client";
import type { User } from "@healthii/types";

const TOKEN_KEY = "healthii.token";

type Session = {
  api: ApiClient;
  baseUrl: string;
  token: string | null;
  user: User | null;
  setSession: (token: string | null, user: User | null) => void;
};

const SessionContext = createContext<Session | null>(null);

export function SessionProvider({
  baseUrl,
  children,
}: {
  baseUrl: string;
  children: ReactNode;
}) {
  const [token, setToken] = useState<string | null>(() => {
    try {
      return window.localStorage.getItem(TOKEN_KEY);
    } catch {
      return null;
    }
  });
  const [user, setUser] = useState<User | null>(null);

  const api = useMemo(
    () =>
      createApiClient({
        baseUrl,
        getToken: () => {
          try {
            return window.localStorage.getItem(TOKEN_KEY);
          } catch {
            return null;
          }
        },
      }),
    [baseUrl],
  );

  const setSession = useCallback((nextToken: string | null, nextUser: User | null) => {
    setToken(nextToken);
    setUser(nextUser);
    try {
      if (nextToken) {
        window.localStorage.setItem(TOKEN_KEY, nextToken);
      } else {
        window.localStorage.removeItem(TOKEN_KEY);
      }
    } catch {
      /* ignore */
    }
  }, []);

  const value = useMemo(
    () => ({ api, baseUrl, token, user, setSession }),
    [api, baseUrl, token, user, setSession],
  );

  return <SessionContext.Provider value={value}>{children}</SessionContext.Provider>;
}

export function useSession() {
  const value = useContext(SessionContext);
  if (!value) {
    throw new Error("useSession must be used within SessionProvider");
  }
  return value;
}

export function useOptionalSession() {
  return useContext(SessionContext);
}
