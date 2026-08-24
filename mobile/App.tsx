import { StatusBar } from "expo-status-bar";
import { useMemo, useState } from "react";
import {
  Pressable,
  SafeAreaView,
  ScrollView,
  StyleSheet,
  Text,
  View,
} from "react-native";
import {
  disclaimer,
  mobileNav,
  previewNotice,
  quickAddActions,
  sampleTimeline,
  sampleWidgets,
} from "@healthii/dashboard";
import { tokens } from "@healthii/design-tokens";

type Tab = "/" | "/timeline" | "/add" | "/insights" | "/profile";

export function App() {
  const [tab, setTab] = useState<Tab>("/");
  const title = useMemo(() => {
    switch (tab) {
      case "/timeline":
        return "Timeline";
      case "/insights":
        return "Insights";
      case "/profile":
        return "Profile";
      default:
        return "Healthii";
    }
  }, [tab]);

  return (
    <SafeAreaView style={styles.safe}>
      <StatusBar style="dark" />
      <View style={styles.header}>
        <Text style={styles.wordmark}>Healthii</Text>
        <Text style={styles.kicker}>{title}</Text>
      </View>
      <ScrollView contentContainerStyle={styles.content}>
        {tab === "/" ? <Home /> : null}
        {tab === "/timeline" ? <Simple title="Timeline" body={sampleTimeline[0]?.detail ?? ""} /> : null}
        {tab === "/add" ? <QuickAdd onDone={() => setTab("/")} /> : null}
        {tab === "/insights" ? (
          <Simple title="Insights" body="Charts stay on desktop and web until there is history to show." />
        ) : null}
        {tab === "/profile" ? (
          <Simple title="You" body="Units, locale and integrations (HealthKit, Health Connect, Withings) will live here." />
        ) : null}
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

function Home() {
  return (
    <View style={styles.stack}>
      <View style={styles.banner}>
        <Text style={styles.bannerText}>{previewNotice}</Text>
      </View>
      <Text style={styles.disclaimer}>{disclaimer}</Text>
      {sampleWidgets.slice(0, 6).map((widget) => (
        <View key={widget.id} style={styles.card}>
          <Text style={styles.cardTitle}>{widget.title}</Text>
          <Text style={styles.metric}>{widget.value}</Text>
          <Text style={styles.hint}>{widget.hint}</Text>
        </View>
      ))}
    </View>
  );
}

function QuickAdd({ onDone }: { onDone: () => void }) {
  return (
    <View style={styles.stack}>
      <Text style={styles.heading}>Add in seconds</Text>
      <Text style={styles.hint}>Saving to your account arrives with measurements in the next milestone.</Text>
      {quickAddActions.map((action) => (
        <Pressable key={action.id} style={styles.card} onPress={onDone} accessibilityRole="button">
          <Text style={styles.cardTitle}>{action.label}</Text>
          <Text style={styles.hint}>{action.description}</Text>
        </Pressable>
      ))}
    </View>
  );
}

function Simple({ title, body }: { title: string; body: string }) {
  return (
    <View style={styles.stack}>
      <Text style={styles.heading}>{title}</Text>
      <Text style={styles.hint}>{body}</Text>
    </View>
  );
}

const styles = StyleSheet.create({
  safe: { flex: 1, backgroundColor: tokens.color.paper },
  header: { paddingHorizontal: 24, paddingTop: 12, paddingBottom: 8 },
  wordmark: {
    fontSize: 28,
    fontWeight: "700",
    color: tokens.color.ink,
    letterSpacing: -0.6,
  },
  kicker: { color: tokens.color.muted, marginTop: 2 },
  content: { padding: 20, paddingBottom: 120, gap: 12 },
  stack: { gap: 12 },
  banner: {
    backgroundColor: "#d9ece4",
    borderRadius: 18,
    padding: 14,
  },
  bannerText: { color: tokens.color.ink, fontWeight: "600" },
  disclaimer: { color: tokens.color.muted, fontSize: 13, lineHeight: 18 },
  card: {
    backgroundColor: tokens.color.surface,
    borderRadius: 22,
    padding: 16,
    borderWidth: 1,
    borderColor: tokens.color.line,
  },
  cardTitle: {
    color: tokens.color.muted,
    textTransform: "uppercase",
    letterSpacing: 0.6,
    fontSize: 12,
    fontWeight: "600",
  },
  metric: { fontSize: 28, color: tokens.color.ink, marginTop: 6 },
  hint: { color: tokens.color.muted, marginTop: 6, lineHeight: 20 },
  heading: { fontSize: 28, fontWeight: "700", color: tokens.color.ink },
  tabbar: {
    position: "absolute",
    left: 16,
    right: 16,
    bottom: 16,
    flexDirection: "row",
    backgroundColor: tokens.color.surface,
    borderRadius: 28,
    padding: 8,
    gap: 4,
    borderWidth: 1,
    borderColor: tokens.color.line,
  },
  tab: { flex: 1, alignItems: "center", paddingVertical: 10, borderRadius: 20 },
  tabSelected: { backgroundColor: "#d9ece4" },
  addTab: { backgroundColor: tokens.color.sage, flex: 1.1 },
  tabLabel: { color: tokens.color.ink, fontWeight: "600", fontSize: 12 },
  addLabel: { color: "#fffbf4" },
});
