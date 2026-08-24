import { useEffect, useId, useState } from "react";
import { quickAddActions } from "@healthii/dashboard";
import { Button } from "./Button";

export function QuickAdd({
  open,
  onClose,
}: {
  open: boolean;
  onClose: () => void;
}) {
  const titleId = useId();

  useEffect(() => {
    if (!open) {
      return;
    }
    const onKey = (event: KeyboardEvent) => {
      if (event.key === "Escape") {
        onClose();
      }
    };
    window.addEventListener("keydown", onKey);
    return () => window.removeEventListener("keydown", onKey);
  }, [open, onClose]);

  if (!open) {
    return null;
  }

  return (
    <div className="hii-dialog-backdrop" onClick={onClose}>
      <div
        className="hii-dialog"
        role="dialog"
        aria-modal="true"
        aria-labelledby={titleId}
        onClick={(event) => event.stopPropagation()}
      >
        <div className="hii-page-header">
          <div>
            <h2 id={titleId}>Quick add</h2>
            <p className="hii-lede">Capture a measurement in seconds. Saving comes in the next milestone.</p>
          </div>
          <Button variant="secondary" onClick={onClose}>
            Close
          </Button>
        </div>
        <div className="hii-quick-grid">
          {quickAddActions.map((action) => (
            <button key={action.id} type="button">
              <strong>{action.label}</strong>
              <span>{action.description}</span>
            </button>
          ))}
        </div>
      </div>
    </div>
  );
}

export function useQuickAdd() {
  const [open, setOpen] = useState(false);
  return {
    open,
    openQuickAdd: () => setOpen(true),
    closeQuickAdd: () => setOpen(false),
  };
}
