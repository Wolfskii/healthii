import type { ThemePreference } from "@healthii/design-tokens";
import { useTheme } from "./ThemeProvider";

const OPTIONS: { id: ThemePreference; label: string }[] = [
  { id: "system", label: "System" },
  { id: "light", label: "Light" },
  { id: "dark", label: "Dark" },
];

export function ThemeSwitch() {
  const { preference, setPreference, theme } = useTheme();
  return (
    <div className="hii-form">
      <div className="hii-theme-switch" role="radiogroup" aria-label="Appearance">
        {OPTIONS.map((option) => (
          <button
            key={option.id}
            type="button"
            role="radio"
            aria-checked={preference === option.id}
            className={preference === option.id ? "is-selected" : undefined}
            onClick={() => setPreference(option.id)}
          >
            {option.label}
          </button>
        ))}
      </div>
      {preference === "system" ? (
        <p className="hii-hint">This device is using {theme} mode right now.</p>
      ) : null}
    </div>
  );
}
