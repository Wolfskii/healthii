import { useEffect, useState } from "react";
import { Link } from "react-router-dom";
import { timelineFilters } from "@healthii/dashboard";
import type { DashboardResponse, DashboardWidget } from "@healthii/types";
import { Disclaimer } from "./Disclaimer";
import { useOptionalSession } from "./session";

const WIDGET_HREF: Record<string, string> = {
  weight: "/measurements",
  "weight-trend": "/insights",
  "weight-goal": "/settings",
  labs: "/labs",
  measurements: "/measurements",
  "blood-pressure": "/measurements",
  "heart-rate": "/measurements",
  sleep: "/measurements",
  workouts: "/workouts",
  medications: "/medications",
  appointments: "/appointments",
  documents: "/documents",
  timeline: "/timeline",
  notes: "/notes",
  symptoms: "/symptoms",
};

const GROUPS: { id: string; title: string; ids: string[] }[] = [
  {
    id: "body",
    title: "Body",
    ids: ["weight", "weight-goal", "weight-trend", "blood-pressure", "heart-rate", "sleep", "measurements"],
  },
  {
    id: "care",
    title: "Care",
    ids: ["medications", "appointments", "symptoms"],
  },
  {
    id: "records",
    title: "Records",
    ids: ["labs", "notes", "documents", "workouts", "timeline"],
  },
];

export function DashboardView() {
  const session = useOptionalSession();
  const [data, setData] = useState<DashboardResponse | null>(null);
  const [error, setError] = useState<string | null>(null);

  useEffect(() => {
    if (!session?.token) {
      return;
    }
    session.api
      .dashboard()
      .then(setData)
      .catch((caught: unknown) => {
        setError(caught instanceof Error ? caught.message : "Could not load dashboard");
      });
  }, [session]);

  const widgets = data?.widgets ?? [];
  const name = session?.user?.display_name?.trim();
  const heading = name ? `${name}, what is going on with your health?` : "What is going on with your health?";

  return (
    <>
      <div className="hii-page-header">
        <div>
          <p className="hii-chip">Personal health OS</p>
          <h1>{heading}</h1>
          <p className="hii-lede">
            One private place for measurements, laboratory results, documents and daily notes.
          </p>
        </div>
      </div>
      <Disclaimer />
      {error ? <p className="hii-error">{error}</p> : null}
      {GROUPS.map((group) => {
        const items = group.ids
          .map((id) => widgets.find((widget) => widget.id === id))
          .filter((widget): widget is DashboardWidget => Boolean(widget));
        if (items.length === 0) {
          return null;
        }
        return (
          <section key={group.id} className="hii-dash-group" aria-label={group.title}>
            <h2 className="hii-section-title">{group.title}</h2>
            <div className="hii-grid">
              {items.map((widget) => (
                <WidgetCard key={widget.id} widget={widget} />
              ))}
            </div>
          </section>
        );
      })}
      <section className="hii-grid" aria-label="Health overview">
        {!data ? (
          <article className="hii-card hii-span-4">
            <h2>Health overview</h2>
            <p className="hii-hint">
              Sign in to load your records. Empty widgets mean no data yet — never sample health values.
            </p>
          </article>
        ) : data.widgets.every((widget) => widget.empty) ? (
          <article className="hii-card hii-span-4">
            <h2>Start with one record</h2>
            <p className="hii-hint">
              Quick add a weight or blood pressure, upload a lab PDF, or import a previous export.
            </p>
            <div className="hii-filters">
              <Link to="/measurements">Add a measurement</Link>
              <Link to="/notes">Write a note</Link>
              <Link to="/labs">Add a blood test</Link>
              <Link to="/reports">Import Apple Health or JSON</Link>
            </div>
          </article>
        ) : null}
        <article className="hii-card hii-span-4">
          <h2>Health timeline</h2>
          <div className="hii-filters" role="toolbar" aria-label="Timeline shortcuts">
            {timelineFilters.map((filter) => (
              <Link key={filter.id} to={`/timeline?kind=${filter.id}`}>
                {filter.label}
              </Link>
            ))}
          </div>
        </article>
      </section>
    </>
  );
}

function WidgetCard({ widget }: { widget: DashboardWidget }) {
  const href = WIDGET_HREF[widget.id] ?? "/";
  const wide = widget.id === "weight-trend" || widget.id === "labs" || widget.id === "timeline";
  return (
    <Link to={href} className={`hii-card ${wide ? "hii-span-2" : ""}`}>
      <h2>{widget.title}</h2>
      <p className="hii-metric">{widget.summary ?? "—"}</p>
      <p className="hii-hint">{widget.hint ?? (widget.empty ? "Nothing recorded yet" : "")}</p>
    </Link>
  );
}
