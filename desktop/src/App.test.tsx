import { render, screen } from "@testing-library/react";
import { describe, expect, it } from "vitest";
import { App } from "./App";

describe("desktop shell", () => {
  it("renders primary Healthii navigation", () => {
    render(<App />);
    expect(screen.getByRole("navigation", { name: "Health sections" })).toBeInTheDocument();
    expect(screen.getByRole("heading", { name: /what is going on with your health/i })).toBeInTheDocument();
  });
});
