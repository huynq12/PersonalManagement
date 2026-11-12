import { cleanup, screen, waitFor, within } from "@testing-library/react";
import userEvent from "@testing-library/user-event";
import { afterEach, beforeEach, describe, expect, it, vi } from "vitest";
import i18n from "../i18n/i18n";
import viLocale from "../i18n/locales/vi.json";
import { setAccessToken } from "../lib/api";
import {
  completedSession,
  FakePomodoro,
  runningSession,
  type FakeSession,
} from "../test/fakePomodoro";
import { FakeSubjects, subject } from "../test/fakeSubjects";
import {
  renderApp,
  signedIn,
  stubApi,
  type RecordedRequest,
} from "../test/testUtils";

// 10:00 on 2026-11-12 in the test user's time zone (UTC+7).
const NOW = new Date("2026-11-12T03:00:00Z");

let backend: FakePomodoro;
let requests: RecordedRequest[];

function setUpBackend(sessions: FakeSession[] = []) {
  backend = new FakePomodoro(
    new FakeSubjects([
      subject("subject-math", "Math", { color: "#3B82F6" }),
      subject("subject-art", "Art", { color: "#EC4899" }),
      subject("subject-old", "Latin", { archived_at: "2026-11-01T00:00:00Z" }),
    ]),
    sessions,
  );
}

function renderPomodoro() {
  requests = stubApi(signedIn, backend.handle);
  renderApp("/pomodoro");
}

/** Requests sent to the study session endpoints with the given method. */
function sessionRequests(method: string) {
  return requests.filter(
    (request) =>
      request.method === method && request.path.startsWith("/study-sessions"),
  );
}

async function clock() {
  return (await screen.findByRole("timer")).textContent;
}

beforeEach(async () => {
  await i18n.changeLanguage("en");
  // Only the date is faked, so timers and promises keep running normally.
  vi.useFakeTimers({ toFake: ["Date"] });
  vi.setSystemTime(NOW);
  setUpBackend();
});

afterEach(() => {
  cleanup();
  vi.useRealTimers();
  vi.unstubAllGlobals();
  setAccessToken(null);
});

describe("when nothing is running", () => {
  it("offers the active subjects and the focus length from the settings", async () => {
    backend.settings.focus_minutes = 30;
    renderPomodoro();

    expect(
      await screen.findByRole("heading", { name: "Pomodoro", level: 1 }),
    ).toBeTruthy();
    const picker = await screen.findByLabelText("Subject");
    const options = within(picker)
      .getAllByRole("option")
      .map((option) => option.textContent);
    expect(options).toEqual(["No subject", "Art", "Math"]);
    expect(screen.getByRole("timer").textContent).toBe("30:00");
    expect(screen.getByRole("button", { name: "Start focus" })).toBeTruthy();
  });

  it("points to the subjects screen when there are none", async () => {
    backend.subjects.subjects = [];
    renderPomodoro();

    const link = await screen.findByRole("link", { name: "Manage subjects" });

    expect(link.getAttribute("href")).toBe("/subjects");
  });

  it("shows an error with a retry button when the data cannot be loaded", async () => {
    backend.failWhen = (request) => request.path === "/settings";
    renderPomodoro();

    expect((await screen.findByRole("alert")).textContent).toBe(
      "The server ran into a problem. Please try again later.",
    );
    backend.failWhen = null;
    await userEvent.click(screen.getByRole("button", { name: "Try again" }));

    expect(
      await screen.findByRole("button", { name: "Start focus" }),
    ).toBeTruthy();
  });
});

describe("starting a session", () => {
  it("starts the countdown for the chosen subject", async () => {
    renderPomodoro();

    await userEvent.selectOptions(
      await screen.findByLabelText("Subject"),
      "Math",
    );
    await userEvent.click(screen.getByRole("button", { name: "Start focus" }));

    expect(
      await screen.findByRole("button", { name: "Cancel session" }),
    ).toBeTruthy();
    expect(await clock()).toBe("25:00");
    expect(screen.getByText("Focus")).toBeTruthy();
    expect(screen.getByText("Math")).toBeTruthy();
    expect(sessionRequests("POST")[0].body).toEqual({
      subject_id: "subject-math",
    });
  });

  it("starts without a subject when none is chosen", async () => {
    renderPomodoro();

    await userEvent.click(
      await screen.findByRole("button", { name: "Start focus" }),
    );

    expect(
      await screen.findByRole("button", { name: "Cancel session" }),
    ).toBeTruthy();
    expect(sessionRequests("POST")[0].body).toEqual({});
  });

  it("shows an error and stays idle when the session cannot be started", async () => {
    renderPomodoro();
    const start = await screen.findByRole("button", { name: "Start focus" });
    backend.failWhen = (request) => request.method === "POST";

    await userEvent.click(start);

    expect((await screen.findByRole("alert")).textContent).toBe(
      "The server ran into a problem. Please try again later.",
    );
    expect(screen.getByRole("button", { name: "Start focus" })).toBeTruthy();
  });
});

describe("a running session", () => {
  it("is restored after a reload with the time that is left", async () => {
    setUpBackend([
      runningSession("session-1", 10, { subject_id: "subject-art" }),
    ]);

    renderPomodoro();

    expect(await clock()).toBe("15:00");
    expect(screen.getByText("Art")).toBeTruthy();
    expect(screen.queryByRole("button", { name: "Start focus" })).toBeNull();
  });

  it("counts down as time passes", async () => {
    setUpBackend([runningSession("session-1", 10)]);
    renderPomodoro();
    expect(await clock()).toBe("15:00");

    vi.setSystemTime(Date.now() + 5000);

    await waitFor(
      () => expect(screen.getByRole("timer").textContent).toBe("14:55"),
      {
        timeout: 3000,
      },
    );
  });

  it("can be cancelled, and then does not count", async () => {
    setUpBackend([runningSession("session-1", 3)]);
    renderPomodoro();

    await userEvent.click(
      await screen.findByRole("button", { name: "Cancel session" }),
    );

    expect(
      await screen.findByRole("button", { name: "Start focus" }),
    ).toBeTruthy();
    expect(sessionRequests("POST")[0].path).toBe(
      "/study-sessions/session-1/cancel",
    );
    expect(screen.getByText("No completed sessions yet today.")).toBeTruthy();
  });

  it("is completed automatically when the time is up, and a short break starts", async () => {
    setUpBackend([
      runningSession("session-1", 25, { subject_id: "subject-math" }),
    ]);

    renderPomodoro();

    expect(await screen.findByText("Short break")).toBeTruthy();
    expect(await clock()).toBe("05:00");
    expect(sessionRequests("POST").map((request) => request.path)).toEqual([
      "/study-sessions/session-1/complete",
    ]);
    const history = within(
      screen.getByRole("region", { name: "Today's sessions" }),
    );
    expect(history.getByText("Math")).toBeTruthy();
    expect(history.getByText("25 min")).toBeTruthy();
  });

  it("is completed once even if it ran out while the tab was closed", async () => {
    setUpBackend([runningSession("session-1", 90)]);

    renderPomodoro();

    expect(await screen.findByText("Short break")).toBeTruthy();
    expect(sessionRequests("POST")).toHaveLength(1);
    // The server caps the time at the planned length.
    expect(backend.sessions[0].duration_seconds).toBe(25 * 60);
  });
});

describe("breaks", () => {
  it("is long after every fourth completed session of the day", async () => {
    setUpBackend([
      completedSession("s-1", 200),
      completedSession("s-2", 150),
      completedSession("s-3", 100),
      runningSession("session-4", 25),
    ]);

    renderPomodoro();

    expect(await screen.findByText("Long break")).toBeTruthy();
    expect(await clock()).toBe("15:00");
  });

  it("is restored after a reload with the time that is left", async () => {
    setUpBackend([completedSession("session-1", 2)]);

    renderPomodoro();

    expect(await screen.findByText("Short break")).toBeTruthy();
    expect(await clock()).toBe("03:00");
  });

  it("is over once its time has passed", async () => {
    setUpBackend([completedSession("session-1", 10)]);

    renderPomodoro();

    expect(
      await screen.findByRole("button", { name: "Start focus" }),
    ).toBeTruthy();
    expect(screen.queryByText("Short break")).toBeNull();
  });

  it("can be skipped", async () => {
    setUpBackend([completedSession("session-1", 1)]);
    renderPomodoro();

    await userEvent.click(
      await screen.findByRole("button", { name: "Skip break" }),
    );

    expect(
      await screen.findByRole("button", { name: "Start focus" }),
    ).toBeTruthy();
    expect(screen.queryByText("Short break")).toBeNull();
  });
});

describe("today’s sessions", () => {
  it("lists completed sessions with their time, subject, and length, and a total", async () => {
    setUpBackend([
      completedSession("s-1", 200, { subject_id: "subject-math" }),
      completedSession("s-2", 150, { duration_seconds: 20 * 60 }),
      completedSession("s-3", 100, {
        status: "cancelled",
        duration_seconds: null,
      }),
    ]);

    renderPomodoro();

    const history = within(
      await screen.findByRole("region", { name: "Today's sessions" }),
    );
    const rows = history.getAllByRole("listitem");
    expect(rows).toHaveLength(2);
    // Newest first.
    expect(within(rows[0]).getByText("No subject")).toBeTruthy();
    expect(within(rows[0]).getByText("20 min")).toBeTruthy();
    expect(within(rows[1]).getByText("Math")).toBeTruthy();
    expect(within(rows[1]).getByText("25 min")).toBeTruthy();
    expect(history.getByText("2 sessions · 45 min")).toBeTruthy();
  });

  it("deletes a session after the user asks", async () => {
    setUpBackend([
      completedSession("s-1", 200, { subject_id: "subject-math" }),
    ]);
    renderPomodoro();

    const history = within(
      await screen.findByRole("region", { name: "Today's sessions" }),
    );
    await userEvent.click(
      await history.findByRole("button", { name: /^Delete session/ }),
    );

    expect(
      await history.findByText("No completed sessions yet today."),
    ).toBeTruthy();
    expect(sessionRequests("DELETE")[0].path).toBe("/study-sessions/s-1");
  });
});

describe("language", () => {
  it("renders in Vietnamese", async () => {
    await i18n.changeLanguage("vi");

    renderPomodoro();

    expect(
      await screen.findByRole("heading", {
        name: viLocale.pomodoro.title,
        level: 1,
      }),
    ).toBeTruthy();
    expect(
      await screen.findByRole("button", { name: viLocale.pomodoro.start }),
    ).toBeTruthy();
  });
});
