import { render, screen } from "@testing-library/react";
import { describe, expect, it } from "vitest";
import { App } from "./App";

describe("web shell", () => {
  it("asks the user to sign in before showing records", () => {
    render(<App />);
    expect(screen.getByRole("button", { name: "Sign in" })).toBeInTheDocument();
    expect(screen.getByText(/does not provide medical diagnosis/i)).toBeInTheDocument();
  });
});
