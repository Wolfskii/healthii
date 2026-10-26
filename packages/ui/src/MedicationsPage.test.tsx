import { fireEvent, render, screen, waitFor, within } from "@testing-library/react";
import { MemoryRouter } from "react-router-dom";
import { afterEach, describe, expect, it, vi } from "vitest";
import type { Medication } from "@healthii/types";
import { MedicationsPage, todayIn } from "./pages";
import { SessionProvider } from "./session";

function medication(overrides: Partial<Medication>): Medication {
  return {
    id: "m1",
    user_id: "u1",
    name: "Levothyroxine",
    kind: "medication",
    dosage: "50 mcg",
    schedule: null,
    started_on: "2026-01-05",
    ended_on: null,
    notes: null,
    active: true,
    created_at: "2026-01-05T08:00:00Z",
    updated_at: "2026-01-05T08:00:00Z",
    ...overrides,
  };
}

function jsonResponse(body: unknown, status = 200) {
  return new Response(JSON.stringify(body), {
    status,
    headers: { "Content-Type": "application/json" },
  });
}

afterEach(() => {
  vi.unstubAllGlobals();
});

describe("todayIn", () => {
  it("formats the date in the account timezone", () => {
    const instant = new Date("2026-10-25T23:30:00Z");
    expect(todayIn("UTC", instant)).toBe("2026-10-25");
    expect(todayIn("Europe/Warsaw", instant)).toBe("2026-10-26");
  });

  it("falls back to the device date for an unknown timezone", () => {
    expect(todayIn("Mars/Olympus_Mons", new Date())).toMatch(/^\d{4}-\d{2}-\d{2}$/);
  });
});

describe("MedicationsPage", () => {
  it("splits current from stopped and stops a medication through PUT", async () => {
    let rows = [
      medication({}),
      medication({ id: "m2", name: "Amoxicillin", dosage: null, started_on: null, ended_on: "2026-02-01", active: false }),
    ];
    const fetchMock = vi.fn(async (input: RequestInfo | URL, init?: RequestInit) => {
      const url = String(input);
      if (url.endsWith("/api/v1/medications/m1") && init?.method === "PUT") {
        const body = JSON.parse(String(init.body));
        rows = rows.map((row) => (row.id === "m1" ? { ...row, ...body, active: false } : row));
        return jsonResponse(rows[0]);
      }
      return jsonResponse(rows);
    });
    vi.stubGlobal("fetch", fetchMock);

    render(
      <SessionProvider baseUrl="http://api.test">
        <MemoryRouter>
          <MedicationsPage />
        </MemoryRouter>
      </SessionProvider>,
    );

    const current = await screen.findByText("Levothyroxine · 50 mcg");
    expect(screen.getByRole("heading", { name: "Stopped or not started" })).toBeInTheDocument();
    expect(screen.getByText(/Stopped 2026-02-01/)).toBeInTheDocument();

    const row = current.closest(".hii-list-row") as HTMLElement;
    fireEvent.click(within(row).getByRole("button", { name: "Stop today" }));

    await waitFor(() => expect(screen.getByText("Nothing current.")).toBeInTheDocument());
    const put = fetchMock.mock.calls.find(([, init]) => init?.method === "PUT");
    expect(put).toBeDefined();
    const sent = JSON.parse(String(put?.[1]?.body));
    expect(sent).toMatchObject({ name: "Levothyroxine", started_on: "2026-01-05" });
    expect(sent.ended_on).toMatch(/^\d{4}-\d{2}-\d{2}$/);
  });

  it("asks for confirmation before deleting a record", async () => {
    const fetchMock = vi.fn(async (_input: RequestInfo | URL, init?: RequestInit) =>
      init?.method === "DELETE" ? new Response(null, { status: 204 }) : jsonResponse([medication({})]),
    );
    vi.stubGlobal("fetch", fetchMock);
    render(
      <SessionProvider baseUrl="http://api.test">
        <MemoryRouter>
          <MedicationsPage />
        </MemoryRouter>
      </SessionProvider>,
    );
    fireEvent.click(await screen.findByRole("button", { name: "Delete Levothyroxine · 50 mcg" }));
    expect(fetchMock.mock.calls.some(([, init]) => init?.method === "DELETE")).toBe(false);

    fireEvent.click(screen.getByRole("button", { name: "Keep" }));
    expect(screen.queryByRole("button", { name: /Confirm delete/ })).not.toBeInTheDocument();

    fireEvent.click(screen.getByRole("button", { name: "Delete Levothyroxine · 50 mcg" }));
    fireEvent.click(screen.getByRole("button", { name: "Confirm delete Levothyroxine · 50 mcg" }));
    await waitFor(() =>
      expect(fetchMock.mock.calls.some(([, init]) => init?.method === "DELETE")).toBe(true),
    );
  });

  it("shows the server's validation message instead of failing silently", async () => {
    vi.stubGlobal(
      "fetch",
      vi.fn(async (_input: RequestInfo | URL, init?: RequestInit) =>
        init?.method === "POST"
          ? jsonResponse({ error: { code: "VALIDATION_ERROR", message: "A name is required" } }, 400)
          : jsonResponse([]),
      ),
    );
    render(
      <SessionProvider baseUrl="http://api.test">
        <MemoryRouter>
          <MedicationsPage />
        </MemoryRouter>
      </SessionProvider>,
    );
    fireEvent.change(screen.getByLabelText("Name"), { target: { value: " " } });
    fireEvent.submit(screen.getByRole("button", { name: "Save" }).closest("form") as HTMLFormElement);
    expect(await screen.findByRole("alert")).toHaveTextContent("A name is required");
  });
});
