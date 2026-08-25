import { fireEvent, render, screen } from "@testing-library/react";
import { afterEach, beforeEach, describe, expect, it, vi } from "vitest";
import { THEME_STORAGE_KEY, ThemeProvider, readPreference, resolveTheme, useTheme } from "./ThemeProvider";

function Probe() {
  const { theme, preference, setPreference, toggle } = useTheme();
  return (
    <div>
      <p>theme:{theme}</p>
      <p>preference:{preference}</p>
      <button type="button" onClick={toggle}>
        Toggle
      </button>
      <button type="button" onClick={() => setPreference("dark")}>
        Dark
      </button>
      <button type="button" onClick={() => setPreference("system")}>
        System
      </button>
    </div>
  );
}

function mockMatchMedia(dark: boolean) {
  Object.defineProperty(window, "matchMedia", {
    writable: true,
    value: (query: string) => ({
      matches: dark && query.includes("dark"),
      media: query,
      addEventListener: vi.fn(),
      removeEventListener: vi.fn(),
    }),
  });
}

describe("ThemeProvider", () => {
  beforeEach(() => {
    window.localStorage.clear();
    document.documentElement.removeAttribute("data-theme");
    mockMatchMedia(false);
  });

  afterEach(() => {
    window.localStorage.clear();
  });

  it("defaults to system and does not lock a resolved theme in storage", () => {
    mockMatchMedia(true);
    expect(readPreference()).toBe("system");
    expect(resolveTheme("system")).toBe("dark");
    render(
      <ThemeProvider>
        <Probe />
      </ThemeProvider>,
    );
    expect(screen.getByText("theme:dark")).toBeInTheDocument();
    expect(window.localStorage.getItem(THEME_STORAGE_KEY)).toBeNull();
  });

  it("honours a stored dark preference", () => {
    window.localStorage.setItem(THEME_STORAGE_KEY, "dark");
    render(
      <ThemeProvider>
        <Probe />
      </ThemeProvider>,
    );
    expect(screen.getByText("theme:dark")).toBeInTheDocument();
    expect(document.documentElement.dataset.theme).toBe("dark");
  });

  it("toggle persists an explicit light or dark choice", () => {
    render(
      <ThemeProvider>
        <Probe />
      </ThemeProvider>,
    );
    fireEvent.click(screen.getByRole("button", { name: "Toggle" }));
    expect(screen.getByText("theme:dark")).toBeInTheDocument();
    expect(window.localStorage.getItem(THEME_STORAGE_KEY)).toBe("dark");
    fireEvent.click(screen.getByRole("button", { name: "Toggle" }));
    expect(window.localStorage.getItem(THEME_STORAGE_KEY)).toBe("light");
  });
});
