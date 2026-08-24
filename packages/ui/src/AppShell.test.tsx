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
  it("does not present sample layout as personal health data", () => {
    render(<DashboardView />);
    expect(screen.getByRole("status")).toHaveTextContent("not showing personal health data");
    expect(
      screen.getByText(/does not provide medical diagnosis/i),
    ).toBeInTheDocument();
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
    expect(screen.getByRole("navigation", { name: "Health sections" })).toBeInTheDocument();
    expect(screen.getByRole("link", { name: "Skip to content" })).toBeInTheDocument();
  });
});
