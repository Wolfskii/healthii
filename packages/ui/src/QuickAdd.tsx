import { useCallback, useEffect, useId, useRef, useState, type FormEvent } from "react";
import { useNavigate } from "react-router-dom";
import { quickAddActions } from "@healthii/dashboard";
import { Button } from "./Button";
import { useOptionalSession } from "./session";

const FOCUSABLE =
  'a[href], button:not([disabled]), input:not([disabled]), select:not([disabled]), textarea:not([disabled]), [tabindex]:not([tabindex="-1"])';

export function QuickAdd({
  open,
  onClose,
}: {
  open: boolean;
  onClose: () => void;
}) {
  const titleId = useId();
  const dialogRef = useRef<HTMLDivElement>(null);
  const session = useOptionalSession();
  const navigate = useNavigate();
  const [action, setAction] = useState<string | null>(null);
  const [value, setValue] = useState("");
  const [extra, setExtra] = useState("");
  const [error, setError] = useState<string | null>(null);
  const [busy, setBusy] = useState(false);

  useEffect(() => {
    if (!open) {
      return;
    }
    const root = dialogRef.current;
    const nodes = () =>
      Array.from(root?.querySelectorAll<HTMLElement>(FOCUSABLE) ?? []).filter(
        (element) => element.tabIndex !== -1,
      );
    nodes()[0]?.focus();
    function onKey(event: KeyboardEvent) {
      if (event.key === "Escape") {
        onClose();
        return;
      }
      if (event.key !== "Tab") {
        return;
      }
      const list = nodes();
      if (list.length === 0) {
        return;
      }
      const first = list[0];
      const last = list[list.length - 1];
      const active = document.activeElement;
      if (event.shiftKey && active === first) {
        event.preventDefault();
        last.focus();
      } else if (!event.shiftKey && active === last) {
        event.preventDefault();
        first.focus();
      }
    }
    window.addEventListener("keydown", onKey);
    return () => window.removeEventListener("keydown", onKey);
  }, [open, action, onClose]);

  useEffect(() => {
    if (!open) {
      setAction(null);
      setValue("");
      setExtra("");
      setError(null);
    }
  }, [open]);

  if (!open) {
    return null;
  }

  async function submit(event: FormEvent) {
    event.preventDefault();
    if (!session || !action) {
      return;
    }
    if (action === "lab") {
      onClose();
      navigate("/labs");
      return;
    }
    if (action === "document") {
      onClose();
      navigate("/documents");
      return;
    }
    setBusy(true);
    setError(null);
    try {
      const { api } = session;
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
      } else if (action === "sleep") {
        await api.createMeasurement({ type: "sleep", value: Number(value), unit: extra || "h" });
      } else if (action === "blood-pressure") {
        const [systolic, diastolic] = value.split("/").map((part) => Number(part.trim()));
        await api.createBloodPressure({ systolic, diastolic, unit: extra || "mmHg" });
      } else if (action === "symptom") {
        await api.createSymptom({ name: value, notes: extra || undefined });
      } else if (action === "medication") {
        await api.createMedication({ name: value, dosage: extra || undefined });
      } else if (action === "workout") {
        await api.createWorkout({ workout_type: extra || "other", notes: value });
      } else if (action === "note") {
        await api.createNote({ title: value, body: extra || "" });
      } else if (action === "appointment") {
        if (!extra) {
          throw new Error("Choose a start time");
        }
        await api.createAppointment({ title: value, starts_at: new Date(extra).toISOString() });
      }
      onClose();
    } catch (caught) {
      setError(caught instanceof Error ? caught.message : "Could not save");
    } finally {
      setBusy(false);
    }
  }

  const valueLabel =
    action === "blood-pressure"
      ? "Systolic / diastolic"
      : action === "note" || action === "appointment"
        ? "Title"
        : action === "sleep"
          ? "Hours"
          : "Value";
  const extraLabel =
    action === "weight" || action === "sleep"
      ? "Unit"
      : action === "workout"
        ? "Type"
        : action === "note"
          ? "Note"
          : action === "appointment"
            ? "Starts"
            : "Optional";

  return (
    <div className="hii-dialog-backdrop" onClick={onClose}>
      <div
        ref={dialogRef}
        className="hii-dialog"
        role="dialog"
        aria-modal="true"
        aria-labelledby={titleId}
        onClick={(event) => event.stopPropagation()}
      >
        <div className="hii-page-header">
          <div>
            <h2 id={titleId}>Quick add</h2>
            <p className="hii-lede">Capture a measurement in seconds. Press Escape to close.</p>
          </div>
          <Button variant="secondary" onClick={onClose}>
            Close
          </Button>
        </div>
        {action ? (
          <form className="hii-form" onSubmit={submit}>
            <label>
              {valueLabel}
              <input
                value={value}
                onChange={(event) => setValue(event.target.value)}
                required
                placeholder={action === "blood-pressure" ? "120 / 80" : undefined}
              />
            </label>
            <label>
              {extraLabel}
              <input
                type={action === "appointment" ? "datetime-local" : "text"}
                value={extra}
                onChange={(event) => setExtra(event.target.value)}
                required={action === "appointment"}
              />
            </label>
            {error ? <p className="hii-error">{error}</p> : null}
            <Button type="submit" disabled={busy}>
              Save
            </Button>
          </form>
        ) : (
          <div className="hii-quick-grid">
            {quickAddActions.map((item) => (
              <button key={item.id} type="button" onClick={() => setAction(item.id)}>
                <strong>{item.label}</strong>
                <span>{item.description}</span>
              </button>
            ))}
          </div>
        )}
      </div>
    </div>
  );
}

export function useQuickAdd() {
  const [open, setOpen] = useState(false);
  const openQuickAdd = useCallback(() => setOpen(true), []);
  const closeQuickAdd = useCallback(() => setOpen(false), []);
  return {
    open,
    openQuickAdd,
    closeQuickAdd,
  };
}
