import { createContext, useCallback, useContext, useMemo, useState, type ReactNode } from "react";
import { StyleSheet, useColorScheme } from "react-native";
import { palettes, type ColorPalette, type ThemeName, type ThemePreference } from "@healthii/design-tokens";

const STORAGE_KEY = "healthii-theme";

type AppTheme = {
  theme: ThemeName;
  preference: ThemePreference;
  setPreference: (preference: ThemePreference) => void;
  colors: ColorPalette;
  styles: ReturnType<typeof createStyles>;
};

const AppThemeContext = createContext<AppTheme | null>(null);

function readPreference(): ThemePreference {
  try {
    const stored = globalThis.localStorage?.getItem(STORAGE_KEY);
    if (stored === "light" || stored === "dark" || stored === "system") {
      return stored;
    }
  } catch {
    /* native has no localStorage */
  }
  return "system";
}

function persistPreference(preference: ThemePreference) {
  try {
    globalThis.localStorage?.setItem(STORAGE_KEY, preference);
  } catch {
    /* ignore */
  }
}

export function AppThemeProvider({ children }: { children: ReactNode }) {
  const system = useColorScheme() === "dark" ? "dark" : "light";
  const [preference, setPreferenceState] = useState<ThemePreference>(readPreference);
  const theme: ThemeName = preference === "system" ? system : preference;
  const colors = palettes[theme];
  const styles = useMemo(() => createStyles(colors), [colors]);
  const setPreference = useCallback((next: ThemePreference) => {
    setPreferenceState(next);
    persistPreference(next);
  }, []);
  const value = useMemo(
    () => ({ theme, preference, setPreference, colors, styles }),
    [theme, preference, setPreference, colors, styles],
  );
  return <AppThemeContext.Provider value={value}>{children}</AppThemeContext.Provider>;
}

export function useAppTheme() {
  const value = useContext(AppThemeContext);
  if (!value) {
    throw new Error("useAppTheme must be used within AppThemeProvider");
  }
  return value;
}

export function createStyles(color: ColorPalette) {
  return StyleSheet.create({
    safe: { flex: 1, backgroundColor: color.paper },
    header: { paddingHorizontal: 24, paddingTop: 12, paddingBottom: 8 },
    wordmark: {
      fontSize: 28,
      fontWeight: "700",
      color: color.ink,
      letterSpacing: -0.6,
    },
    kicker: { color: color.muted, marginTop: 2 },
    content: { padding: 20, paddingBottom: 120, gap: 12 },
    stack: { gap: 12 },
    disclaimer: { color: color.muted, fontSize: 13, lineHeight: 18 },
    card: {
      backgroundColor: color.surface,
      borderRadius: 22,
      padding: 16,
      borderWidth: 1,
      borderColor: color.line,
    },
    cardTitle: {
      color: color.muted,
      textTransform: "uppercase",
      letterSpacing: 0.6,
      fontSize: 12,
      fontWeight: "600",
    },
    metric: { fontSize: 28, color: color.ink, marginTop: 6 },
    hint: { color: color.muted, marginTop: 6, lineHeight: 20 },
    heading: { fontSize: 28, fontWeight: "700", color: color.ink },
    error: { color: color.clay, marginTop: 8 },
    input: {
      backgroundColor: color.surface,
      borderColor: color.line,
      borderWidth: 1,
      borderRadius: 14,
      paddingHorizontal: 14,
      paddingVertical: 12,
      color: color.ink,
    },
    primary: {
      backgroundColor: color.sage,
      borderRadius: 999,
      paddingVertical: 14,
      alignItems: "center",
    },
    primaryLabel: { color: color.onSage, fontWeight: "700" },
    link: { color: color.sage, fontWeight: "600", textAlign: "center", marginTop: 8 },
    auth: { padding: 24, gap: 12, flexGrow: 1, justifyContent: "center" },
    tabbar: {
      position: "absolute",
      left: 16,
      right: 16,
      bottom: 16,
      flexDirection: "row",
      backgroundColor: color.surface,
      borderRadius: 28,
      padding: 8,
      gap: 4,
      borderWidth: 1,
      borderColor: color.line,
    },
    tab: { flex: 1, alignItems: "center", paddingVertical: 10, borderRadius: 20 },
    tabSelected: { backgroundColor: color.sageSoft },
    addTab: { backgroundColor: color.sage, flex: 1.1 },
    tabLabel: { color: color.ink, fontWeight: "600", fontSize: 12 },
    addLabel: { color: color.onSage },
    themeRow: { flexDirection: "row", gap: 8 },
    themeChip: {
      flex: 1,
      alignItems: "center",
      paddingVertical: 10,
      borderRadius: 16,
      borderWidth: 1,
      borderColor: color.line,
      backgroundColor: color.surface,
    },
    themeChipSelected: { backgroundColor: color.sageSoft, borderColor: color.sage },
  });
}
