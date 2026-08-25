import { render, screen } from "@testing-library/react";
import { MemoryRouter } from "react-router-dom";
import { describe, expect, it } from "vitest";
import { AppShell } from "./AppShell";
import { Button } from "./Button";
import { DashboardView } from "./DashboardView";
import { ThemeProvider } from "./ThemeProvider";

describe("Button", () => {
  it("renders the provided label", () => {
    render(<Button>Quick add</Button>);
    expect(screen.getByRole("button", { name: "Quick add" })).toBeInTheDocument();
  });
});

describe("DashboardView", () => {
  it("states that Healthii is not a diagnosis", () => {
    render(
      <MemoryRouter>
        <DashboardView />
      </MemoryRouter>,
    );
    expect(screen.getByRole("heading", { name: /what is going on with your health/i })).toBeInTheDocument();
    expect(screen.getByText(/does not provide medical diagnosis/i)).toBeInTheDocument();
  });
});

describe("AppShell", () => {
  it("exposes primary navigation landmarks", () => {
    render(
      <ThemeProvider>
        <MemoryRouter>
          <AppShell>
            <p>Content</p>
          </AppShell>
        </MemoryRouter>
      </ThemeProvider>,
    );
    expect(screen.getAllByRole("navigation", { name: "Health sections" }).length).toBeGreaterThan(0);
    expect(screen.getByRole("link", { name: "Skip to content" })).toBeInTheDocument();
    expect(screen.getByRole("search")).toBeInTheDocument();
  });
});
