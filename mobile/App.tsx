import { StatusBar } from "expo-status-bar";
import { useEffect, useMemo, useState } from "react";
import {
  Platform,
  Pressable,
  SafeAreaView,
  ScrollView,
  Text,
  TextInput,
  View,
} from "react-native";
import { createApiClient, type ApiClient } from "@healthii/api-client";
import { disclaimer, mobileNav, quickAddActions } from "@healthii/dashboard";
import type { ThemePreference } from "@healthii/design-tokens";
import type { DashboardResponse, TimelineItem, User } from "@healthii/types";
import { AppThemeProvider, useAppTheme } from "./theme";

type Tab = "/" | "/timeline" | "/add" | "/insights" | "/profile";

const TOKEN_KEY = "healthii.token";
let memoryToken: string | null = null;

function apiBaseUrl() {
  return (
    process.env.EXPO_PUBLIC_API_BASE_URL ??
    (Platform.OS === "android" ? "http://10.0.2.2:8080" : "http://localhost:8080")
  );
}

function readToken() {
  if (memoryToken) {
    return memoryToken;
  }
  try {
    if (typeof localStorage !== "undefined") {
      return localStorage.getItem(TOKEN_KEY);
    }
  } catch {
    /* native has no localStorage */
  }
  return null;
}

function writeToken(token: string | null) {
  memoryToken = token;
  try {
    if (typeof localStorage !== "undefined") {
      if (token) {
        localStorage.setItem(TOKEN_KEY, token);
      } else {
        localStorage.removeItem(TOKEN_KEY);
      }
    }
  } catch {
    /* ignore */
  }
}

export function App() {
  return (
    <AppThemeProvider>
      <AppMain />
    </AppThemeProvider>
  );
}

function AppMain() {
  const { styles, theme } = useAppTheme();
  const [token, setToken] = useState<string | null>(readToken);
  const [user, setUser] = useState<User | null>(null);
  const [tab, setTab] = useState<Tab>("/");
  const [error, setError] = useState<string | null>(null);

  const api = useMemo(
    () =>
      createApiClient({
        baseUrl: apiBaseUrl(),
        getToken: readToken,
      }),
    [],
  );

  useEffect(() => {
    if (!token) {
      return;
    }
    api
      .me()
      .then(setUser)
      .catch(() => {
        writeToken(null);
        setToken(null);
      });
  }, [api, token]);

  function signedIn(nextToken: string, nextUser: User) {
    writeToken(nextToken);
    setToken(nextToken);
    setUser(nextUser);
    setError(null);
  }

  async function signOut() {
    await api.logout().catch(() => undefined);
    writeToken(null);
    setToken(null);
    setUser(null);
    setTab("/");
  }

  const title = useMemo(() => {
    switch (tab) {
      case "/timeline":
        return "Timeline";
      case "/insights":
        return "Insights";
      case "/profile":
        return "Profile";
      case "/add":
        return "Add";
      default:
        return "Healthii";
    }
  }, [tab]);

  if (!token) {
    return (
      <SafeAreaView style={styles.safe}>
        <StatusBar style={theme === "dark" ? "light" : "dark"} />
        <AuthForm api={api} error={error} setError={setError} onSignedIn={signedIn} />
      </SafeAreaView>
    );
  }

  return (
    <SafeAreaView style={styles.safe}>
      <StatusBar style={theme === "dark" ? "light" : "dark"} />
      <View style={styles.header}>
        <Text style={styles.wordmark}>Healthii</Text>
        <Text style={styles.kicker}>{user?.display_name ?? title}</Text>
      </View>
      <ScrollView contentContainerStyle={styles.content}>
        {tab === "/" ? <Home api={api} /> : null}
        {tab === "/timeline" ? <Timeline api={api} /> : null}
        {tab === "/add" ? <QuickAdd api={api} onDone={() => setTab("/")} /> : null}
        {tab === "/insights" ? <Insights api={api} /> : null}
        {tab === "/profile" ? <Profile user={user} onSignOut={signOut} /> : null}
      </ScrollView>
      <View style={styles.tabbar} accessibilityRole="tablist">
        {mobileNav.map((item) => {
          const selected = tab === item.to;
          const isAdd = item.to === "/add";
          return (
            <Pressable
              key={item.to}
              accessibilityRole="tab"
              accessibilityState={{ selected }}
              onPress={() => setTab(item.to as Tab)}
              style={[styles.tab, isAdd && styles.addTab, selected && !isAdd && styles.tabSelected]}
            >
              <Text style={[styles.tabLabel, isAdd && styles.addLabel]}>{item.label}</Text>
            </Pressable>
          );
        })}
      </View>
    </SafeAreaView>
  );
}

function AuthForm({
  api,
  error,
  setError,
  onSignedIn,
}: {
  api: ApiClient;
  error: string | null;
  setError: (value: string | null) => void;
  onSignedIn: (token: string, user: User) => void;
}) {
  const { styles, colors, theme } = useAppTheme();
  const [mode, setMode] = useState<"login" | "register">("login");
  const [email, setEmail] = useState("");
  const [password, setPassword] = useState("");
  const [displayName, setDisplayName] = useState("");
  const [busy, setBusy] = useState(false);

  async function submit() {
    setBusy(true);
    setError(null);
    try {
      const result =
        mode === "login"
          ? await api.login({ email, password })
          : await api.register({ email, password, display_name: displayName || email });
      onSignedIn(result.token, result.user);
    } catch (caught) {
      setError(caught instanceof Error ? caught.message : "Could not sign in");
    } finally {
      setBusy(false);
    }
  }

  return (
    <ScrollView contentContainerStyle={styles.auth}>
      <Text style={styles.wordmark}>Healthii</Text>
      <Text style={styles.heading}>Your health, privately held</Text>
      {mode === "register" ? (
        <TextInput
          style={styles.input}
          placeholder="Display name"
          placeholderTextColor={colors.muted}
          value={displayName}
          onChangeText={setDisplayName}
        />
      ) : null}
      <TextInput
        style={styles.input}
        placeholder="Email"
        placeholderTextColor={colors.muted}
        keyboardAppearance={theme === "dark" ? "dark" : "light"}
        autoCapitalize="none"
        keyboardType="email-address"
        value={email}
        onChangeText={setEmail}
      />
      <TextInput
        style={styles.input}
        placeholder="Password (10+ characters)"
        placeholderTextColor={colors.muted}
        keyboardAppearance={theme === "dark" ? "dark" : "light"}
        secureTextEntry
        value={password}
        onChangeText={setPassword}
      />
      {error ? <Text style={styles.error}>{error}</Text> : null}
      <Pressable style={styles.primary} onPress={submit} disabled={busy} accessibilityRole="button">
        <Text style={styles.primaryLabel}>{mode === "login" ? "Sign in" : "Create account"}</Text>
      </Pressable>
      <Pressable onPress={() => setMode(mode === "login" ? "register" : "login")}>
        <Text style={styles.link}>{mode === "login" ? "Need an account?" : "Already have an account?"}</Text>
      </Pressable>
      <AppearancePicker />
      <Text style={styles.disclaimer}>{disclaimer}</Text>
    </ScrollView>
  );
}

function Home({ api }: { api: ApiClient }) {
  const { styles } = useAppTheme();
  const [data, setData] = useState<DashboardResponse | null>(null);
  useEffect(() => {
    api.dashboard().then(setData).catch(() => setData(null));
  }, [api]);
  return (
    <View style={styles.stack}>
      <Text style={styles.disclaimer}>{disclaimer}</Text>
      {(data?.widgets ?? []).map((widget) => (
        <View key={widget.id} style={styles.card}>
          <Text style={styles.cardTitle}>{widget.title}</Text>
          <Text style={styles.metric}>{widget.summary ?? "—"}</Text>
          <Text style={styles.hint}>{widget.hint ?? (widget.empty ? "Nothing recorded yet" : "")}</Text>
        </View>
      ))}
      {!data ? <Text style={styles.hint}>Loading your overview…</Text> : null}
    </View>
  );
}

function Timeline({ api }: { api: ApiClient }) {
  const { styles } = useAppTheme();
  const [items, setItems] = useState<TimelineItem[]>([]);
  useEffect(() => {
    api.timeline().then((response) => setItems(response.items)).catch(() => setItems([]));
  }, [api]);
  if (items.length === 0) {
    return <Text style={styles.hint}>Nothing on the timeline yet.</Text>;
  }
  return (
    <View style={styles.stack}>
      {items.map((item) => (
        <View key={`${item.kind}-${item.id}`} style={styles.card}>
          <Text style={styles.cardTitle}>{item.kind}</Text>
          <Text style={styles.metric}>{item.title}</Text>
          <Text style={styles.hint}>
            {new Date(item.occurred_at).toLocaleString()}
            {item.detail ? ` · ${item.detail}` : ""}
          </Text>
        </View>
      ))}
    </View>
  );
}

function Insights({ api }: { api: ApiClient }) {
  const { styles } = useAppTheme();
  const [summary, setSummary] = useState("Add a few measurements to see trends.");
  useEffect(() => {
    api
      .charts("weight", 90)
      .then((chart) => {
        if (chart.points.length === 0) {
          return;
        }
        setSummary(
          `Weight ${chart.points.length} points · min ${chart.min?.toFixed(1) ?? "—"} · max ${chart.max?.toFixed(1) ?? "—"}`,
        );
      })
      .catch(() => undefined);
  }, [api]);
  return (
    <View style={styles.stack}>
      <Text style={styles.heading}>Insights</Text>
      <Text style={styles.hint}>{summary}</Text>
      <Text style={styles.hint}>Full charts live on desktop and web.</Text>
    </View>
  );
}

function Profile({ user, onSignOut }: { user: User | null; onSignOut: () => void }) {
  const { styles } = useAppTheme();
  return (
    <View style={styles.stack}>
      <Text style={styles.heading}>{user?.display_name ?? "You"}</Text>
      <Text style={styles.hint}>{user?.email}</Text>
      <Text style={styles.hint}>Appearance</Text>
      <AppearancePicker />
      <Text style={styles.hint}>HealthKit, Health Connect and Withings adapters come later.</Text>
      <Pressable style={styles.card} onPress={onSignOut} accessibilityRole="button">
        <Text style={styles.cardTitle}>Session</Text>
        <Text style={styles.metric}>Sign out</Text>
      </Pressable>
    </View>
  );
}

function QuickAdd({ api, onDone }: { api: ApiClient; onDone: () => void }) {
  const { styles, colors } = useAppTheme();
  const [action, setAction] = useState<string | null>(null);
  const [value, setValue] = useState("");
  const [extra, setExtra] = useState("");
  const [error, setError] = useState<string | null>(null);

  async function save() {
    setError(null);
    try {
      if (action === "weight") {
        await api.createMeasurement({ type: "weight", value: Number(value), unit: extra || "kg" });
      } else if (action === "heart-rate") {
        await api.createMeasurement({
          type: "resting_heart_rate",
          value: Number(value),
          unit: extra || "bpm",
        });
      } else if (action === "glucose") {
        await api.createMeasurement({
          type: "blood_glucose",
          value: Number(value),
          unit: extra || "mmol/L",
        });
      } else if (action === "blood-pressure") {
        const [systolic, diastolic] = value.split("/").map((part) => Number(part.trim()));
        await api.createBloodPressure({ systolic, diastolic });
      } else if (action === "symptom") {
        await api.createSymptom({ name: value, notes: extra || undefined });
      } else if (action === "medication") {
        await api.createMedication({ name: value, dosage: extra || undefined });
      } else if (action === "workout") {
        await api.createWorkout({ workout_type: extra || "other", notes: value });
      } else if (action === "note") {
        await api.createNote({ title: value, body: extra || "" });
      } else if (action === "sleep") {
        await api.createMeasurement({ type: "sleep", value: Number(value), unit: extra || "h" });
      } else if (action === "appointment") {
        if (!extra) {
          setError("Add a start time, or use desktop.");
          return;
        }
        await api.createAppointment({ title: value, starts_at: new Date(extra).toISOString() });
      } else {
        setError("Use desktop or web for labs and documents.");
        return;
      }
      setAction(null);
      setValue("");
      onDone();
    } catch (caught) {
      setError(caught instanceof Error ? caught.message : "Could not save");
    }
  }

  if (action) {
    return (
      <View style={styles.stack}>
        <Text style={styles.heading}>{action.replace("-", " ")}</Text>
        <TextInput
          style={styles.input}
          placeholder={
            action === "blood-pressure"
              ? "120 / 80"
              : action === "appointment"
                ? "Title"
                : action === "sleep"
                  ? "Hours"
                  : "Value"
          }
          placeholderTextColor={colors.muted}
          value={value}
          onChangeText={setValue}
        />
        <TextInput
          style={styles.input}
          placeholder={action === "appointment" ? "2026-08-26T10:00" : action === "sleep" ? "h" : "Optional"}
          placeholderTextColor={colors.muted}
          value={extra}
          onChangeText={setExtra}
        />
        {error ? <Text style={styles.error}>{error}</Text> : null}
        <Pressable style={styles.primary} onPress={save} accessibilityRole="button">
          <Text style={styles.primaryLabel}>Save</Text>
        </Pressable>
        <Pressable onPress={() => setAction(null)}>
          <Text style={styles.link}>Back</Text>
        </Pressable>
      </View>
    );
  }

  return (
    <View style={styles.stack}>
      <Text style={styles.heading}>Add in seconds</Text>
      {quickAddActions.map((item) => (
        <Pressable
          key={item.id}
          style={styles.card}
          onPress={() => setAction(item.id)}
          accessibilityRole="button"
        >
          <Text style={styles.cardTitle}>{item.label}</Text>
          <Text style={styles.hint}>{item.description}</Text>
        </Pressable>
      ))}
    </View>
  );
}

function AppearancePicker() {
  const { styles, preference, setPreference } = useAppTheme();
  const options: { id: ThemePreference; label: string }[] = [
    { id: "system", label: "System" },
    { id: "light", label: "Light" },
    { id: "dark", label: "Dark" },
  ];
  return (
    <View style={styles.themeRow}>
      {options.map((option) => (
        <Pressable
          key={option.id}
          accessibilityRole="button"
          accessibilityState={{ selected: preference === option.id }}
          onPress={() => setPreference(option.id)}
          style={[styles.themeChip, preference === option.id && styles.themeChipSelected]}
        >
          <Text style={styles.tabLabel}>{option.label}</Text>
        </Pressable>
      ))}
    </View>
  );
}
