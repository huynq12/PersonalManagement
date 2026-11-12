import { useState } from "react";
import { useTranslation } from "react-i18next";
import BreakPanel from "../components/pomodoro/BreakPanel";
import IdlePanel from "../components/pomodoro/IdlePanel";
import RunningPanel from "../components/pomodoro/RunningPanel";
import SessionHistory from "../components/pomodoro/SessionHistory";
import Alert from "../components/ui/Alert";
import Button from "../components/ui/Button";
import { useNow } from "../hooks/useNow";
import { useSession } from "../hooks/useSession";
import { useSettings } from "../hooks/useSettings";
import { useCurrentSession, useSessionsOn } from "../hooks/useStudySessions";
import { useSubjects } from "../hooks/useSubjects";
import { errorMessageKey } from "../i18n/errorMessage";
import { todayIn } from "../lib/dates";
import { currentBreak } from "../lib/pomodoro";

/**
 * The Pomodoro screen: a focus session counts down, then a break follows.
 *
 * What is running comes from the server, so a reload restores the timer. Breaks are not stored;
 * they are worked out from today's completed sessions (see `currentBreak`).
 */
export default function PomodoroPage() {
  const { t } = useTranslation();
  const user = useSession();
  const now = useNow();

  // "Today" is the date in the user's time zone. Read it once per visit so the list does not
  // jump to another day while the page is open.
  const timeZone =
    user.data?.timezone ?? Intl.DateTimeFormat().resolvedOptions().timeZone;
  const [today] = useState(() => todayIn(timeZone));

  const settings = useSettings();
  const subjects = useSubjects();
  const current = useCurrentSession();
  const sessions = useSessionsOn(today);
  const [skippedAfterSessionId, setSkippedAfterSessionId] = useState<
    string | null
  >(null);

  const completed = (sessions.data ?? []).filter(
    (session) => session.status === "completed",
  );
  const activeBreak = settings.data
    ? currentBreak(completed, settings.data, now, skippedAfterSessionId)
    : null;

  const queries = [settings, subjects, current, sessions];
  const failed = queries.find((query) => query.isError);
  const isLoading = queries.some((query) => query.isPending);

  return (
    <main className="mx-auto flex w-full max-w-3xl flex-col gap-6 px-4 py-6 md:px-8 md:py-8">
      <header className="flex flex-col gap-1">
        <h1 className="text-2xl font-semibold tracking-tight md:text-3xl">
          {t("pomodoro.title")}
        </h1>
        <p className="text-sm text-muted-foreground">
          {t("pomodoro.description")}
        </p>
      </header>

      {failed ? (
        <div className="flex flex-col items-start gap-3">
          <Alert variant="error">{t(errorMessageKey(failed.error))}</Alert>
          <Button
            variant="secondary"
            onClick={() =>
              queries.forEach((query) => query.isError && void query.refetch())
            }
          >
            {t("common.retry")}
          </Button>
        </div>
      ) : isLoading || !settings.data || !subjects.data || !sessions.data ? (
        <p className="text-sm text-muted-foreground">{t("common.loading")}</p>
      ) : (
        <>
          {current.data ? (
            <RunningPanel
              session={current.data}
              subject={subjects.data.find(
                (item) => item.id === current.data?.subject_id,
              )}
              now={now}
            />
          ) : activeBreak ? (
            <BreakPanel
              activeBreak={activeBreak}
              now={now}
              onSkip={() =>
                setSkippedAfterSessionId(activeBreak.afterSessionId)
              }
            />
          ) : (
            <IdlePanel
              subjects={subjects.data.filter(
                (item) => item.archived_at === null,
              )}
              focusMinutes={settings.data.focus_minutes}
            />
          )}

          <SessionHistory
            sessions={completed}
            subjects={subjects.data}
            timeZone={timeZone}
          />
        </>
      )}
    </main>
  );
}
