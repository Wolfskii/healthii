import { useEffect, useState, type FormEvent, type ReactNode } from "react";
import { Link, useSearchParams } from "react-router-dom";
import { timelineFilters } from "@healthii/dashboard";
import type { ChartResponse, Medication, Profile, SessionInfo, TimelineItem } from "@healthii/types";
import { Button } from "./Button";
import { Disclaimer } from "./Disclaimer";
import { Sparkline } from "./Sparkline";
import { useSession } from "./session";
import { ThemeSwitch } from "./ThemeSwitch";

const KIND_HREF: Record<string, string> = {
  measurements: "/measurements",
  labs: "/labs",
  documents: "/documents",
  workouts: "/workouts",
  appointments: "/appointments",
  medications: "/medications",
  symptoms: "/symptoms",
  notes: "/notes",
};

export function SearchPage() {
  const { api } = useSession();
  const [params] = useSearchParams();
  const q = params.get("q") ?? "";
  const [items, setItems] = useState<TimelineItem[]>([]);

  useEffect(() => {
    if (q.trim().length < 2) {
      setItems([]);
      return;
    }
    api.search(q).then((response) => setItems(response.items)).catch(() => setItems([]));
  }, [api, q]);

  return (
    <Page title="Search" body="Find measurements, labs, documents and notes without leaving your account.">
      {q.trim().length < 2 ? (
        <p className="hii-hint">Type at least two characters in the search field.</p>
      ) : items.length === 0 ? (
        <p className="hii-hint">No matching records.</p>
      ) : (
        <TimelineList items={items} />
      )}
    </Page>
  );
}

export function TimelinePage() {
  const { api } = useSession();
  const [params, setParams] = useSearchParams();
  const kind = params.get("kind") ?? "all";
  const [items, setItems] = useState<TimelineItem[]>([]);

  useEffect(() => {
    api.timeline(kind).then((response) => setItems(response.items)).catch(() => setItems([]));
  }, [api, kind]);

  return (
    <Page title="Timeline" body="A single chronology of your records.">
      <div className="hii-filters" role="toolbar" aria-label="Timeline filters">
        {timelineFilters.map((filter) => (
          <button
            key={filter.id}
            type="button"
            aria-pressed={kind === filter.id}
            onClick={() => setParams({ kind: filter.id })}
          >
            {filter.label}
          </button>
        ))}
      </div>
      {items.length === 0 ? <p className="hii-hint">Nothing on the timeline yet.</p> : <TimelineList items={items} />}
    </Page>
  );
}

function TimelineList({ items }: { items: TimelineItem[] }) {
  return (
    <div className="hii-timeline">
      {items.map((item) => (
        <article key={`${item.kind}-${item.id}`} className="hii-timeline-item">
          <time>{new Date(item.occurred_at).toLocaleString()}</time>
          <div>
            <strong>
              {KIND_HREF[item.kind] ? <Link to={KIND_HREF[item.kind]}>{item.title}</Link> : item.title}
            </strong>
            <p className="hii-hint">
              {item.kind.replaceAll("_", " ")}
              {item.detail ? ` · ${item.detail}` : ""}
            </p>
          </div>
        </article>
      ))}
    </div>
  );
}

const DEFAULT_UNITS: Record<string, string> = {
  weight: "kg",
  resting_heart_rate: "bpm",
  heart_rate: "bpm",
  blood_glucose: "mmol/L",
  body_temperature: "°C",
  oxygen_saturation: "%",
  body_fat: "%",
  waist_circumference: "cm",
  height: "cm",
  sleep: "h",
};

export function MeasurementsPage() {
  const { api } = useSession();
  const [rows, setRows] = useState<Array<{ id: string; type: string; value: number; unit: string; measured_at: string }>>([]);
  const [kind, setKind] = useState("weight");
  const [value, setValue] = useState("");
  const [unit, setUnit] = useState("kg");
  const [systolic, setSystolic] = useState("");
  const [diastolic, setDiastolic] = useState("");
  const [filter, setFilter] = useState("all");
  const [measuredAt, setMeasuredAt] = useState("");

  async function refresh() {
    setRows(await api.listMeasurements());
  }
  useEffect(() => {
    refresh().catch(() => setRows([]));
  }, [api]);

  async function onSubmit(event: FormEvent) {
    event.preventDefault();
    await api.createMeasurement({
      type: kind,
      value: Number(value),
      unit,
      measured_at: fromLocalInput(measuredAt),
    });
    setValue("");
    await refresh();
  }

  async function onBloodPressure(event: FormEvent) {
    event.preventDefault();
    await api.createBloodPressure({
      systolic: Number(systolic),
      diastolic: Number(diastolic),
      measured_at: fromLocalInput(measuredAt),
    });
    setSystolic("");
    setDiastolic("");
    await refresh();
  }

  return (
    <Page title="Measurements" body="Weight, blood pressure, heart rate, sleep, glucose and the rest of your vitals.">
      <form className="hii-form" onSubmit={onSubmit}>
        <div className="hii-form-row">
          <label>
            Type
            <select
              value={kind}
              onChange={(event) => {
                const next = event.target.value;
                setKind(next);
                if (DEFAULT_UNITS[next]) {
                  setUnit(DEFAULT_UNITS[next]);
                }
              }}
            >
              <option value="weight">Weight</option>
              <option value="resting_heart_rate">Resting heart rate</option>
              <option value="heart_rate">Heart rate</option>
              <option value="blood_glucose">Blood glucose</option>
              <option value="sleep">Sleep</option>
              <option value="body_temperature">Temperature</option>
              <option value="oxygen_saturation">Oxygen saturation</option>
              <option value="body_fat">Body fat</option>
              <option value="waist_circumference">Waist</option>
              <option value="height">Height</option>
            </select>
          </label>
          <label>
            Value
            <input value={value} onChange={(event) => setValue(event.target.value)} required type="number" step="any" />
          </label>
          <label>
            Unit
            <input value={unit} onChange={(event) => setUnit(event.target.value)} required />
          </label>
          <label>
            When
            <input type="datetime-local" value={measuredAt} onChange={(event) => setMeasuredAt(event.target.value)} />
          </label>
        </div>
        <p className="hii-hint">Leave the time empty to use now. Past readings keep the original unit.</p>
        <Button type="submit">Save measurement</Button>
      </form>
      <form className="hii-form" onSubmit={onBloodPressure}>
        <div className="hii-form-row">
          <label>
            Systolic
            <input value={systolic} onChange={(event) => setSystolic(event.target.value)} required type="number" />
          </label>
          <label>
            Diastolic
            <input value={diastolic} onChange={(event) => setDiastolic(event.target.value)} required type="number" />
          </label>
        </div>
        <Button type="submit" variant="secondary">
          Save blood pressure
        </Button>
      </form>
      <label className="hii-field">
        Show
        <select value={filter} onChange={(event) => setFilter(event.target.value)}>
          <option value="all">All types</option>
          <option value="weight">Weight</option>
          <option value="blood_pressure_systolic">Blood pressure (systolic)</option>
          <option value="resting_heart_rate">Resting heart rate</option>
          <option value="blood_glucose">Blood glucose</option>
          <option value="sleep">Sleep</option>
        </select>
      </label>
      <List
        rows={rows
          .filter((row) => filter === "all" || row.type === filter || (filter === "blood_pressure_systolic" && row.type.startsWith("blood_pressure")))
          .map((row) => ({
          id: row.id,
          title: `${row.type.replaceAll("_", " ")} · ${row.value} ${row.unit}`,
          meta: new Date(row.measured_at).toLocaleString(),
          onDelete: () => api.deleteMeasurement(row.id).then(refresh),
        }))}
      />
    </Page>
  );
}

export function LabsPage() {
  const { api } = useSession();
  const [labs, setLabs] = useState<Array<{ id: string; tested_on: string; laboratory: string | null; results: Array<{ biomarker_code: string; value: number | null; unit: string | null; status: string }> }>>([]);
  const [testedOn, setTestedOn] = useState(new Date().toISOString().slice(0, 10));
  const [laboratory, setLaboratory] = useState("");
  const [code, setCode] = useState("vitamin_d");
  const [labValue, setLabValue] = useState("");
  const [labUnit, setLabUnit] = useState("nmol/L");
  const [low, setLow] = useState("50");
  const [high, setHigh] = useState("125");

  async function refresh() {
    setLabs(await api.listLabs());
  }
  useEffect(() => {
    refresh().catch(() => setLabs([]));
  }, [api]);

  async function onSubmit(event: FormEvent) {
    event.preventDefault();
    await api.createLab({
      tested_on: testedOn,
      laboratory,
      results: [
        {
          biomarker_code: code,
          value: Number(labValue),
          unit: labUnit,
          reference_low: Number(low),
          reference_high: Number(high),
        },
      ],
    });
    setLabValue("");
    await refresh();
  }

  return (
    <Page title="Blood tests" body="Flexible laboratory panels. Status compares a number to that report's range — it is not a diagnosis.">
      <form className="hii-form" onSubmit={onSubmit}>
        <div className="hii-form-row">
          <label>
            Date
            <input type="date" value={testedOn} onChange={(event) => setTestedOn(event.target.value)} required />
          </label>
          <label>
            Laboratory
            <input value={laboratory} onChange={(event) => setLaboratory(event.target.value)} />
          </label>
          <label>
            Biomarker
            <input value={code} onChange={(event) => setCode(event.target.value)} required />
          </label>
          <label>
            Value
            <input value={labValue} onChange={(event) => setLabValue(event.target.value)} required type="number" step="any" />
          </label>
          <label>
            Unit
            <input value={labUnit} onChange={(event) => setLabUnit(event.target.value)} />
          </label>
          <label>
            Range low
            <input value={low} onChange={(event) => setLow(event.target.value)} type="number" step="any" />
          </label>
          <label>
            Range high
            <input value={high} onChange={(event) => setHigh(event.target.value)} type="number" step="any" />
          </label>
        </div>
        <Button type="submit">Save panel</Button>
      </form>
      <LabHistory labs={labs} />
      <List
        rows={labs.map((lab) => ({
          id: lab.id,
          title: `${lab.tested_on} · ${lab.laboratory || "Lab"}`,
          meta: (
            <>
              {lab.results.map((result, index) => (
                <span key={`${result.biomarker_code}-${index}`}>
                  {result.biomarker_code} {result.value ?? "—"} {result.unit ?? ""}{" "}
                  <span className={`hii-status hii-status-${result.status}`}>{result.status}</span>
                </span>
              ))}
            </>
          ),
          onDelete: () => api.deleteLab(lab.id).then(refresh),
        }))}
      />
    </Page>
  );
}

export function DocumentsPage() {
  const { api } = useSession();
  const [rows, setRows] = useState<Array<{ id: string; filename: string; title: string | null; document_type: string }>>([]);
  const [title, setTitle] = useState("");
  const [documentType, setDocumentType] = useState("doctor_report");
  const [error, setError] = useState<string | null>(null);

  async function refresh() {
    setRows(await api.listDocuments());
  }
  useEffect(() => {
    refresh().catch(() => setRows([]));
  }, [api]);

  async function onSubmit(event: FormEvent<HTMLFormElement>) {
    event.preventDefault();
    setError(null);
    const form = new FormData(event.currentTarget);
    form.set("document_type", documentType);
    if (title) {
      form.set("title", title);
    }
    try {
      await api.uploadDocument(form);
      setTitle("");
      await refresh();
    } catch (caught) {
      setError(caught instanceof Error ? caught.message : "Upload failed");
    }
  }

  async function openFile(id: string, filename: string) {
    const { blob } = await api.downloadDocument(id);
    const url = URL.createObjectURL(blob);
    const link = document.createElement("a");
    link.href = url;
    link.download = filename;
    link.click();
    URL.revokeObjectURL(url);
  }

  return (
    <Page title="Documents" body="PDF and images stay private. Downloads use your session — files are never public links.">
      <form className="hii-form" onSubmit={onSubmit}>
        <div className="hii-form-row">
          <label>
            File
            <input name="file" type="file" accept="application/pdf,image/png,image/jpeg,image/webp" required />
          </label>
          <label>
            Type
            <select value={documentType} onChange={(event) => setDocumentType(event.target.value)}>
              <option value="doctor_report">Doctor report</option>
              <option value="blood_test">Blood test</option>
              <option value="prescription">Prescription</option>
              <option value="imaging">Imaging</option>
              <option value="vaccination">Vaccination</option>
              <option value="referral">Referral</option>
              <option value="discharge_summary">Discharge summary</option>
              <option value="other">Other</option>
            </select>
          </label>
          <label>
            Title
            <input value={title} onChange={(event) => setTitle(event.target.value)} />
          </label>
        </div>
        {error ? <p className="hii-error">{error}</p> : null}
        <Button type="submit">Upload</Button>
      </form>
      <List
        rows={rows.map((row) => ({
          id: row.id,
          title: row.title || row.filename,
          meta: row.document_type.replaceAll("_", " "),
          onOpen: () => openFile(row.id, row.filename),
          onDelete: () => api.deleteDocument(row.id).then(refresh),
        }))}
      />
    </Page>
  );
}

export function WorkoutsPage() {
  const { api } = useSession();
  const [rows, setRows] = useState<Array<{ id: string; workout_type: string; started_at: string; exercises: Array<{ name: string }> }>>([]);
  const [type, setType] = useState("strength");
  const [name, setName] = useState("Squat");
  const [reps, setReps] = useState("5");
  const [weight, setWeight] = useState("100");

  async function refresh() {
    setRows(await api.listWorkouts());
  }
  useEffect(() => {
    refresh().catch(() => setRows([]));
  }, [api]);

  async function onSubmit(event: FormEvent) {
    event.preventDefault();
    await api.createWorkout({
      workout_type: type,
      exercises: [{ name, sets: [{ repetitions: Number(reps), weight: Number(weight), weight_unit: "kg" }] }],
    });
    await refresh();
  }

  return (
    <Page title="Workouts" body="Strength, endurance and mobility sessions with sets when you need them.">
      <form className="hii-form" onSubmit={onSubmit}>
        <div className="hii-form-row">
          <label>
            Type
            <select value={type} onChange={(event) => setType(event.target.value)}>
              <option>strength</option>
              <option>running</option>
              <option>cycling</option>
              <option>walking</option>
              <option>swimming</option>
              <option>sports</option>
              <option>mobility</option>
              <option>other</option>
            </select>
          </label>
          <label>
            Exercise
            <input value={name} onChange={(event) => setName(event.target.value)} required />
          </label>
          <label>
            Reps
            <input value={reps} onChange={(event) => setReps(event.target.value)} />
          </label>
          <label>
            Weight
            <input value={weight} onChange={(event) => setWeight(event.target.value)} />
          </label>
        </div>
        <Button type="submit">Save workout</Button>
      </form>
      <List
        rows={rows.map((row) => ({
          id: row.id,
          title: `${row.workout_type} · ${row.exercises.map((exercise) => exercise.name).join(", ") || "Session"}`,
          meta: new Date(row.started_at).toLocaleString(),
          onDelete: () => api.deleteWorkout(row.id).then(refresh),
        }))}
      />
    </Page>
  );
}

export function MedicationsPage() {
  const { api } = useSession();
  const [rows, setRows] = useState<Array<{ id: string; name: string; kind: string; dosage: string | null }>>([]);
  const [name, setName] = useState("");
  const [kind, setKind] = useState("medication");
  const [dosage, setDosage] = useState("");
  async function refresh() {
    setRows(await api.listMedications());
  }
  useEffect(() => {
    refresh().catch(() => setRows([]));
  }, [api]);
  return (
    <Page title="Medications" body="Medications and supplements you choose to track.">
      <form
        className="hii-form"
        onSubmit={async (event) => {
          event.preventDefault();
          await api.createMedication({ name, kind, dosage });
          setName("");
          await refresh();
        }}
      >
        <div className="hii-form-row">
          <label>
            Name
            <input value={name} onChange={(event) => setName(event.target.value)} required />
          </label>
          <label>
            Kind
            <select value={kind} onChange={(event) => setKind(event.target.value)}>
              <option value="medication">Medication</option>
              <option value="supplement">Supplement</option>
            </select>
          </label>
          <label>
            Dosage
            <input value={dosage} onChange={(event) => setDosage(event.target.value)} />
          </label>
        </div>
        <Button type="submit">Save</Button>
      </form>
      <List
        rows={rows.map((row) => ({
          id: row.id,
          title: row.name,
          meta: `${row.kind}${row.dosage ? ` · ${row.dosage}` : ""}`,
          onDelete: () => api.deleteMedication(row.id).then(refresh),
        }))}
      />
    </Page>
  );
}

export function SymptomsPage() {
  const { api } = useSession();
  const [rows, setRows] = useState<Array<{ id: string; name: string; noted_at: string; severity: number | null }>>([]);
  const [name, setName] = useState("");
  const [severity, setSeverity] = useState("5");
  async function refresh() {
    setRows(await api.listSymptoms());
  }
  useEffect(() => {
    refresh().catch(() => setRows([]));
  }, [api]);
  return (
    <Page title="Symptoms" body="Private notes about how you feel, without judgment.">
      <form
        className="hii-form"
        onSubmit={async (event) => {
          event.preventDefault();
          await api.createSymptom({ name, severity: Number(severity) });
          setName("");
          await refresh();
        }}
      >
        <div className="hii-form-row">
          <label>
            Title
            <input value={name} onChange={(event) => setName(event.target.value)} required />
          </label>
          <label>
            Severity (1–10)
            <input type="number" min={1} max={10} value={severity} onChange={(event) => setSeverity(event.target.value)} />
          </label>
        </div>
        <Button type="submit">Save</Button>
      </form>
      <List
        rows={rows.map((row) => ({
          id: row.id,
          title: row.name,
          meta: `${new Date(row.noted_at).toLocaleString()}${row.severity ? ` · ${row.severity}/10` : ""}`,
          onDelete: () => api.deleteSymptom(row.id).then(refresh),
        }))}
      />
    </Page>
  );
}

export function NotesPage() {
  const { api } = useSession();
  const [rows, setRows] = useState<Array<{ id: string; title: string; body: string; noted_at: string }>>([]);
  const [title, setTitle] = useState("");
  const [body, setBody] = useState("");
  const [when, setWhen] = useState("");
  async function refresh() {
    setRows(await api.listNotes());
  }
  useEffect(() => {
    refresh().catch(() => setRows([]));
  }, [api]);
  return (
    <Page title="Notes" body="Private journal entries about visits, questions and context. Notes are not a medical record for a clinic.">
      <form
        className="hii-form"
        onSubmit={async (event) => {
          event.preventDefault();
          await api.createNote({ title, body, noted_at: fromLocalInput(when) });
          setTitle("");
          setBody("");
          await refresh();
        }}
      >
        <div className="hii-form-row">
          <label>
            Title
            <input value={title} onChange={(event) => setTitle(event.target.value)} required />
          </label>
          <label>
            When
            <input type="datetime-local" value={when} onChange={(event) => setWhen(event.target.value)} />
          </label>
        </div>
        <label>
          Note
          <textarea value={body} onChange={(event) => setBody(event.target.value)} required />
        </label>
        <Button type="submit">Save note</Button>
      </form>
      <List
        rows={rows.map((row) => ({
          id: row.id,
          title: row.title,
          meta: (
            <>
              {new Date(row.noted_at).toLocaleString()}
              {row.body ? ` · ${row.body}` : ""}
            </>
          ),
          onDelete: () => api.deleteNote(row.id).then(refresh),
        }))}
      />
    </Page>
  );
}

export function AppointmentsPage() {
  const { api } = useSession();
  const [rows, setRows] = useState<Array<{ id: string; title: string; starts_at: string; provider: string | null }>>([]);
  const [title, setTitle] = useState("");
  const [starts, setStarts] = useState("");
  const [provider, setProvider] = useState("");
  async function refresh() {
    setRows(await api.listAppointments());
  }
  useEffect(() => {
    refresh().catch(() => setRows([]));
  }, [api]);
  return (
    <Page title="Appointments" body="Upcoming clinic, lab and specialist visits.">
      <form
        className="hii-form"
        onSubmit={async (event) => {
          event.preventDefault();
          await api.createAppointment({
            title,
            starts_at: new Date(starts).toISOString(),
            provider,
          });
          setTitle("");
          await refresh();
        }}
      >
        <div className="hii-form-row">
          <label>
            Title
            <input value={title} onChange={(event) => setTitle(event.target.value)} required />
          </label>
          <label>
            Starts
            <input type="datetime-local" value={starts} onChange={(event) => setStarts(event.target.value)} required />
          </label>
          <label>
            Provider
            <input value={provider} onChange={(event) => setProvider(event.target.value)} />
          </label>
        </div>
        <Button type="submit">Save</Button>
      </form>
      <List
        rows={rows.map((row) => ({
          id: row.id,
          title: row.title,
          meta: `${new Date(row.starts_at).toLocaleString()}${row.provider ? ` · ${row.provider}` : ""}`,
          onDelete: () => api.deleteAppointment(row.id).then(refresh),
        }))}
      />
    </Page>
  );
}

const CHART_METRICS = [
  "weight",
  "blood_pressure_systolic",
  "heart_rate",
  "blood_glucose",
  "sleep",
  "hba1c",
  "total_cholesterol",
  "ldl",
  "hdl",
  "triglycerides",
  "vitamin_d",
  "ferritin",
  "body_fat",
];

export function InsightsPage() {
  const { api } = useSession();
  const [days, setDays] = useState(365);
  const [charts, setCharts] = useState<ChartResponse[]>([]);
  useEffect(() => {
    Promise.all(CHART_METRICS.map((metric) => api.charts(metric, days)))
      .then((results) => setCharts(results.filter((chart) => chart.points.length > 0)))
      .catch(() => setCharts([]));
  }, [api, days]);
  return (
    <Page title="Insights" body="Charts over your recorded history. Empty metrics stay hidden.">
      <div className="hii-filters" role="toolbar" aria-label="Chart range">
        {[30, 90, 365].map((range) => (
          <button
            key={range}
            type="button"
            aria-pressed={days === range}
            onClick={() => setDays(range)}
          >
            {range} days
          </button>
        ))}
      </div>
      {charts.length === 0 ? <p className="hii-hint">Add a few measurements or lab results to see trends.</p> : null}
      {charts.map((chart) => (
        <article key={chart.metric} className="hii-card" style={{ marginBottom: 16 }}>
          <h2>{chart.metric.replaceAll("_", " ")}</h2>
          <Sparkline chart={chart} />
        </article>
      ))}
    </Page>
  );
}

export function ReportsPage() {
  const { api, baseUrl, token } = useSession();
  const [notice, setNotice] = useState<string | null>(null);
  const [error, setError] = useState<string | null>(null);
  const [summary, setSummary] = useState<Array<{ title: string; summary: string | null }>>([]);

  useEffect(() => {
    api.dashboard()
      .then((data) => setSummary(data.widgets.map((widget) => ({ title: widget.title, summary: widget.summary }))))
      .catch(() => setSummary([]));
  }, [api]);

  async function download(format: "json" | "csv") {
    const response = await fetch(`${baseUrl}/api/v1/export?format=${format}`, {
      headers: token ? { Authorization: `Bearer ${token}` } : {},
    });
    const blob = await response.blob();
    const url = URL.createObjectURL(blob);
    const link = document.createElement("a");
    link.href = url;
    link.download = `healthii-export.${format}`;
    link.click();
    URL.revokeObjectURL(url);
  }

  async function onImport(event: FormEvent<HTMLFormElement>) {
    event.preventDefault();
    setError(null);
    setNotice(null);
    const file = new FormData(event.currentTarget).get("file");
    if (!(file instanceof File) || file.size === 0) {
      setError("Choose a JSON, CSV or Apple Health export.xml first.");
      return;
    }
    try {
      const text = await file.text();
      const name = file.name.toLowerCase();
      const result =
        name.endsWith(".xml") || text.includes("<HealthData")
          ? await api.importAppleHealth(text)
          : name.endsWith(".csv")
            ? await api.importCsv(text)
            : await api.importJson(JSON.parse(text));
      setNotice(
        `Imported ${result.measurements} measurements, ${result.labs} lab panels, ${result.workouts} workouts, ${result.notes} notes (${result.skipped} skipped). ${result.notice}`,
      );
    } catch (caught) {
      setError(caught instanceof Error ? caught.message : "Import failed");
    }
  }

  return (
    <Page title="Reports" body="Export, print, or bring records in from Healthii JSON/CSV, Health Connect / Fit CSV, or an Apple Health export.xml. Files stay in your account.">
      <div className="hii-split hii-no-print">
        <article className="hii-card">
          <h2>Export</h2>
          <p className="hii-hint">Take your data with you. Healthii should never lock you in.</p>
          <div className="hii-form-row">
            <Button type="button" onClick={() => download("json")}>
              Download JSON
            </Button>
            <Button type="button" variant="secondary" onClick={() => download("csv")}>
              Download CSV
            </Button>
            <Button type="button" variant="secondary" onClick={() => window.print()}>
              Print / PDF
            </Button>
          </div>
        </article>
        <article className="hii-card">
          <h2>Import</h2>
          <p className="hii-hint">
            Healthii JSON/CSV, a Health Connect / Fit vitals CSV, or Apple Health export.xml (unzip the export first). Steps, sleep stages and intraday heart rate are skipped. Sleep duration and time in bed are imported as hours.
          </p>
          <form className="hii-form" onSubmit={onImport}>
            <label>
              File
              <input name="file" type="file" accept=".json,.csv,.xml,application/json,text/csv,text/xml" required />
            </label>
            <Button type="submit">Import into this account</Button>
          </form>
        </article>
      </div>
      {notice ? <p className="hii-hint">{notice}</p> : null}
      {error ? <p className="hii-error">{error}</p> : null}
      <EmergencyCard />
      <section className="hii-print-summary">
        <h2>Print summary</h2>
        <p className="hii-hint">Empty lines mean nothing is recorded — this is not a medical report.</p>
        {summary.map((row) => (
          <div key={row.title} className="hii-list-row">
            <strong>{row.title}</strong>
            <span className="hii-hint">{row.summary ?? "—"}</span>
          </div>
        ))}
      </section>
    </Page>
  );
}

function EmergencyCard() {
  const { api, user } = useSession();
  const [profile, setProfile] = useState<Profile | null>(null);
  const [meds, setMeds] = useState<Medication[]>([]);
  useEffect(() => {
    Promise.all([api.profile(), api.listMedications()])
      .then(([nextProfile, nextMeds]) => {
        setProfile(nextProfile);
        setMeds(nextMeds.filter((row) => !row.ended_on));
      })
      .catch(() => undefined);
  }, [api]);
  const contact = [profile?.emergency_name, profile?.emergency_phone].filter(Boolean).join(" · ");
  const medicationLine = meds
    .map((row) => (row.dosage ? `${row.name} (${row.dosage})` : row.name))
    .join(", ");
  const rows = [
    { label: "Name", value: user?.display_name },
    { label: "Blood type", value: profile?.blood_type },
    { label: "Allergies", value: profile?.allergies },
    { label: "Emergency contact", value: contact },
    { label: "Medications", value: medicationLine },
  ];
  return (
    <article className="hii-card hii-emergency-card">
      <h2>In case of emergency</h2>
      <p className="hii-hint">
        A personal summary you can print — not an official medical ID and not a diagnosis.
      </p>
      <dl>
        {rows.map((row) => (
          <div key={row.label} className="hii-emergency-row">
            <dt>{row.label}</dt>
            <dd>{row.value?.trim() ? row.value : "—"}</dd>
          </div>
        ))}
      </dl>
    </article>
  );
}

export function SettingsPage() {
  const { api, setSession } = useSession();
  const [height, setHeight] = useState("");
  const [bloodType, setBloodType] = useState("");
  const [allergies, setAllergies] = useState("");
  const [history, setHistory] = useState("");
  const [emergencyName, setEmergencyName] = useState("");
  const [emergencyPhone, setEmergencyPhone] = useState("");
  const [unitSystem, setUnitSystem] = useState("metric");
  const [weightUnit, setWeightUnit] = useState("kg");
  const [goalWeight, setGoalWeight] = useState("");
  const [goalUnit, setGoalUnit] = useState("kg");
  const [saved, setSaved] = useState(false);
  useEffect(() => {
    api.profile().then((profile) => {
      setHeight(profile.height_cm?.toString() ?? "");
      setBloodType(profile.blood_type ?? "");
      setAllergies(profile.allergies ?? "");
      setHistory(profile.medical_history ?? "");
      setEmergencyName(profile.emergency_name ?? "");
      setEmergencyPhone(profile.emergency_phone ?? "");
      setUnitSystem(profile.unit_system);
      setWeightUnit(profile.weight_unit);
      setGoalWeight(profile.goal_weight?.toString() ?? "");
      setGoalUnit(profile.goal_weight_unit ?? profile.weight_unit ?? "kg");
    }).catch(() => undefined);
  }, [api]);
  return (
    <Page title="Settings" body="Account, units, appearance and a short health profile. Only store what is useful.">
      <AccountForm />
      <article className="hii-card">
        <h2>Appearance</h2>
        <p className="hii-hint">Dark mode uses the same layout. System follows this device.</p>
        <ThemeSwitch />
      </article>
      <article className="hii-card">
        <h2>Profile</h2>
        <p className="hii-hint">Units and a short health profile. Only store what is useful. A weight goal is a personal target, not a medical recommendation.</p>
      <form
        className="hii-form"
        onSubmit={async (event) => {
          event.preventDefault();
          await api.updateProfile({
            height_cm: height ? Number(height) : null,
            blood_type: bloodType || null,
            allergies: allergies || null,
            medical_history: history || null,
            emergency_name: emergencyName || null,
            emergency_phone: emergencyPhone || null,
            unit_system: unitSystem,
            weight_unit: weightUnit,
            goal_weight: goalWeight ? Number(goalWeight) : null,
            goal_weight_unit: goalWeight ? goalUnit : null,
          });
          setSaved(true);
        }}
      >
        <div className="hii-form-row">
          <label>
            Height (cm)
            <input value={height} onChange={(event) => setHeight(event.target.value)} />
          </label>
          <label>
            Blood type
            <input value={bloodType} onChange={(event) => setBloodType(event.target.value)} />
          </label>
          <label>
            Unit system
            <select value={unitSystem} onChange={(event) => setUnitSystem(event.target.value)}>
              <option value="metric">Metric</option>
              <option value="imperial">Imperial</option>
            </select>
          </label>
          <label>
            Weight unit
            <select value={weightUnit} onChange={(event) => setWeightUnit(event.target.value)}>
              <option value="kg">kg</option>
              <option value="lb">lb</option>
            </select>
          </label>
          <label>
            Goal weight
            <input value={goalWeight} onChange={(event) => setGoalWeight(event.target.value)} />
          </label>
          <label>
            Goal unit
            <select value={goalUnit} onChange={(event) => setGoalUnit(event.target.value)}>
              <option value="kg">kg</option>
              <option value="lb">lb</option>
            </select>
          </label>
        </div>
        <label>
          Allergies
          <textarea value={allergies} onChange={(event) => setAllergies(event.target.value)} />
        </label>
        <label>
          Medical history notes
          <textarea value={history} onChange={(event) => setHistory(event.target.value)} />
        </label>
        <div className="hii-form-row">
          <label>
            Emergency contact
            <input value={emergencyName} onChange={(event) => setEmergencyName(event.target.value)} />
          </label>
          <label>
            Emergency phone
            <input value={emergencyPhone} onChange={(event) => setEmergencyPhone(event.target.value)} />
          </label>
        </div>
        <Button type="submit">Save profile</Button>
        {saved ? <p className="hii-hint">Saved.</p> : null}
      </form>
      </article>
      <div className="hii-split">
        <article className="hii-card">
          <PasswordForm />
        </article>
        <article className="hii-card">
          <SessionsPanel />
        </article>
      </div>
      <ActivityPanel />
      <DeleteAccount />
      <Button
        variant="secondary"
        type="button"
        onClick={async () => {
          await api.logout().catch(() => undefined);
          setSession(null, null);
        }}
      >
        Sign out
      </Button>
    </Page>
  );
}

function AccountForm() {
  const { api, token, user, setSession } = useSession();
  const [displayName, setDisplayName] = useState(user?.display_name ?? "");
  const [timezone, setTimezone] = useState(user?.timezone ?? "UTC");
  const [saved, setSaved] = useState(false);
  const [error, setError] = useState<string | null>(null);
  useEffect(() => {
    if (!user) {
      return;
    }
    setDisplayName(user.display_name);
    setTimezone(user.timezone);
  }, [user]);
  return (
    <article className="hii-card">
      <h2>Account</h2>
      <p className="hii-hint">Your name appears on the dashboard. Timezone is stored for you — timestamps stay in UTC on the server.</p>
      <form
        className="hii-form"
        onSubmit={async (event) => {
          event.preventDefault();
          setError(null);
          setSaved(false);
          try {
            const updated = await api.updateMe({ display_name: displayName, timezone });
            setSession(token, updated);
            setSaved(true);
          } catch (caught) {
            setError(caught instanceof Error ? caught.message : "Could not update account");
          }
        }}
      >
        <div className="hii-form-row">
          <label>
            Display name
            <input
              value={displayName}
              onChange={(event) => setDisplayName(event.target.value)}
              required
              maxLength={120}
            />
          </label>
          <label>
            Timezone
            <input
              value={timezone}
              onChange={(event) => setTimezone(event.target.value)}
              required
              placeholder="Europe/Warsaw or UTC"
            />
          </label>
        </div>
        <Button type="submit">Save account</Button>
        {saved ? <p className="hii-hint">Saved.</p> : null}
        {error ? <p className="hii-error">{error}</p> : null}
      </form>
    </article>
  );
}

function PasswordForm() {
  const { api } = useSession();
  const [currentPassword, setCurrentPassword] = useState("");
  const [nextPassword, setNextPassword] = useState("");
  const [message, setMessage] = useState<string | null>(null);
  const [error, setError] = useState<string | null>(null);
  return (
    <form
      className="hii-form"
      onSubmit={async (event) => {
        event.preventDefault();
        setError(null);
        setMessage(null);
        try {
          await api.changePassword({
            current_password: currentPassword,
            new_password: nextPassword,
          });
          setCurrentPassword("");
          setNextPassword("");
          setMessage("Password updated. Other sessions were signed out.");
        } catch (caught) {
          setError(caught instanceof Error ? caught.message : "Could not update password");
        }
      }}
    >
      <h2>Password</h2>
      <div className="hii-form-row">
        <label>
          Current password
          <input
            type="password"
            autoComplete="current-password"
            required
            value={currentPassword}
            onChange={(event) => setCurrentPassword(event.target.value)}
          />
        </label>
        <label>
          New password
          <input
            type="password"
            autoComplete="new-password"
            required
            minLength={10}
            value={nextPassword}
            onChange={(event) => setNextPassword(event.target.value)}
          />
        </label>
      </div>
      <Button type="submit">Change password</Button>
      {message ? <p className="hii-hint">{message}</p> : null}
      {error ? <p className="hii-error">{error}</p> : null}
    </form>
  );
}

function SessionsPanel() {
  const { api, setSession } = useSession();
  const [rows, setRows] = useState<SessionInfo[]>([]);
  async function refresh() {
    setRows(await api.sessions());
  }
  useEffect(() => {
    refresh().catch(() => setRows([]));
  }, [api]);
  return (
    <div className="hii-form">
      <h2>Sessions</h2>
      <p className="hii-hint">Revoke a device you no longer use. Signing out of the current session returns you to the sign-in screen.</p>
      {rows.length === 0 ? <p className="hii-hint">No active sessions.</p> : null}
      {rows.map((row) => (
        <div key={row.id} className="hii-list-row">
          <div>
            <strong>{row.current ? "This device" : row.user_agent || "Unknown device"}</strong>
            <p className="hii-hint">Since {new Date(row.created_at).toLocaleString()}</p>
          </div>
          <Button
            variant="secondary"
            type="button"
            onClick={async () => {
              await api.revokeSession(row.id);
              if (row.current) {
                setSession(null, null);
              } else {
                await refresh();
              }
            }}
          >
            Revoke
          </Button>
        </div>
      ))}
    </div>
  );
}

const ACTIVITY_LABELS: Record<string, string> = {
  "auth.register": "Account created",
  "auth.login": "Signed in",
  "auth.logout": "Signed out",
  "auth.password_change": "Password changed",
  "auth.session_revoke": "Session revoked",
  "import.records": "Imported records",
  "user.update": "Account updated",
};

function ActivityPanel() {
  const { api } = useSession();
  const [rows, setRows] = useState<Array<{ id: string; action: string; created_at: string }>>([]);
  useEffect(() => {
    api.activity().then(setRows).catch(() => setRows([]));
  }, [api]);
  return (
    <article className="hii-card">
      <h2>Account activity</h2>
      <p className="hii-hint">Sign-in and import events for this account. Measurement values are never stored here.</p>
      {rows.length === 0 ? <p className="hii-hint">No events yet.</p> : null}
      <div className="hii-activity">
        {rows.map((row) => (
          <div key={row.id} className="hii-list-row">
            <strong>{ACTIVITY_LABELS[row.action] ?? row.action}</strong>
            <span className="hii-hint">{new Date(row.created_at).toLocaleString()}</span>
          </div>
        ))}
      </div>
    </article>
  );
}

function DeleteAccount() {
  const { api, setSession } = useSession();
  const [password, setPassword] = useState("");
  const [confirm, setConfirm] = useState("");
  const [error, setError] = useState<string | null>(null);
  return (
    <form
      className="hii-form hii-danger-zone"
      onSubmit={async (event) => {
        event.preventDefault();
        setError(null);
        if (confirm !== "DELETE") {
          setError("Type DELETE to confirm.");
          return;
        }
        try {
          await api.deleteAccount(password);
          setSession(null, null);
        } catch (caught) {
          setError(caught instanceof Error ? caught.message : "Could not delete account");
        }
      }}
    >
      <h2>Delete account</h2>
      <p className="hii-hint">
        Permanently removes your records, documents and sessions from this Healthii server. This cannot be undone.
      </p>
      <label>
        Current password
        <input type="password" required value={password} onChange={(event) => setPassword(event.target.value)} />
      </label>
      <label>
        Type DELETE
        <input value={confirm} onChange={(event) => setConfirm(event.target.value)} />
      </label>
      <Button type="submit" variant="secondary">
        Delete everything
      </Button>
      {error ? <p className="hii-error">{error}</p> : null}
    </form>
  );
}

function Page({ title, body, children }: { title: string; body: string; children: ReactNode }) {
  return (
    <>
      <div className="hii-page-header">
        <div>
          <h1>{title}</h1>
          <p className="hii-lede">{body}</p>
        </div>
      </div>
      <Disclaimer />
      {children}
    </>
  );
}

function List({
  rows,
}: {
  rows: Array<{
    id: string;
    title: string;
    meta: ReactNode;
    onDelete: () => void;
    onOpen?: () => void;
  }>;
}) {
  if (rows.length === 0) {
    return <p className="hii-hint">Nothing saved yet.</p>;
  }
  return (
    <div className="hii-list">
      {rows.map((row) => (
        <div key={row.id} className="hii-list-row">
          <div>
            <strong>{row.title}</strong>
            <p className="hii-hint">{row.meta}</p>
          </div>
          <div className="hii-form-row">
            {row.onOpen ? (
              <Button variant="secondary" type="button" onClick={row.onOpen}>
                File
              </Button>
            ) : null}
            <Button variant="secondary" type="button" onClick={row.onDelete}>
              Delete
            </Button>
          </div>
        </div>
      ))}
    </div>
  );
}

function LabHistory({
  labs,
}: {
  labs: Array<{
    tested_on: string;
    results: Array<{ biomarker_code: string; value: number | null; unit: string | null; status: string }>;
  }>;
}) {
  const groups = new Map<string, Array<{ date: string; value: number; unit: string; status: string }>>();
  for (const lab of labs) {
    for (const result of lab.results) {
      if (result.value == null) {
        continue;
      }
      const rows = groups.get(result.biomarker_code) ?? [];
      rows.push({
        date: lab.tested_on,
        value: result.value,
        unit: result.unit ?? "",
        status: result.status,
      });
      groups.set(result.biomarker_code, rows);
    }
  }
  const codes = [...groups.keys()].filter((code) => (groups.get(code)?.length ?? 0) >= 2);
  if (codes.length === 0) {
    return null;
  }
  return (
    <section className="hii-history">
      <h2>Biomarker history</h2>
      <p className="hii-hint">The same code across panels, compared to each report&apos;s own range — not a diagnosis.</p>
      {codes.map((code) => {
        const rows = groups.get(code) ?? [];
        return (
          <article key={code} className="hii-card">
            <h3>{code.replaceAll("_", " ")}</h3>
            <ol>
              {rows.map((row, index) => (
                <li key={`${code}-${row.date}-${index}`}>
                  {row.date} · {row.value} {row.unit}{" "}
                  <span className={`hii-status hii-status-${row.status}`}>{row.status}</span>
                </li>
              ))}
            </ol>
          </article>
        );
      })}
    </section>
  );
}

function fromLocalInput(value: string): string | undefined {
  if (!value) {
    return undefined;
  }
  const parsed = new Date(value);
  if (Number.isNaN(parsed.getTime())) {
    return undefined;
  }
  return parsed.toISOString();
}
