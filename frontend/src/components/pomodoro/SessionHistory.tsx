import { Trash2 } from "lucide-react";
import { useTranslation } from "react-i18next";
import { useDeleteSession } from "../../hooks/useStudySessions";
import { errorMessageKey } from "../../i18n/errorMessage";
import { formatTime } from "../../lib/dates";
import { formatDuration } from "../../lib/pomodoro";
import type { StudySession } from "../../lib/studySessions";
import type { Subject } from "../../lib/subjects";
import ColorDot from "../subjects/ColorDot";
import Alert from "../ui/Alert";
import Card from "../ui/Card";
import IconButton from "../ui/IconButton";

interface SessionHistoryProps {
  /** Today's completed sessions, newest first. */
  sessions: StudySession[];
  subjects: Subject[];
  timeZone: string;
}

/** The sessions completed today, with the total study time. Mistaken ones can be deleted. */
export default function SessionHistory({
  sessions,
  subjects,
  timeZone,
}: SessionHistoryProps) {
  const { t, i18n } = useTranslation();
  const remove = useDeleteSession();

  const totalSeconds = sessions.reduce(
    (sum, session) => sum + (session.duration_seconds ?? 0),
    0,
  );

  return (
    <Card
      title={t("pomodoro.history.title")}
      aside={
        sessions.length > 0
          ? t("pomodoro.history.summary", {
              count: sessions.length,
              total: formatDuration(totalSeconds, i18n.resolvedLanguage),
            })
          : null
      }
    >
      {remove.isError ? (
        <Alert variant="error">{t(errorMessageKey(remove.error))}</Alert>
      ) : null}

      {sessions.length === 0 ? (
        <p className="text-sm text-muted-foreground">
          {t("pomodoro.history.empty")}
        </p>
      ) : (
        <ul className="flex flex-col">
          {sessions.map((session) => {
            const subject = subjects.find(
              (item) => item.id === session.subject_id,
            );
            const time = formatTime(
              session.started_at,
              i18n.resolvedLanguage,
              timeZone,
            );
            return (
              <li
                key={session.id}
                className="flex items-center gap-3 py-1.5 text-sm"
              >
                <span className="w-16 shrink-0 text-muted-foreground tabular-nums">
                  {time}
                </span>
                <span className="flex min-w-0 flex-1 items-center gap-2">
                  {subject ? <ColorDot color={subject.color} /> : null}
                  <span className="truncate">
                    {subject ? subject.name : t("pomodoro.noSubject")}
                  </span>
                </span>
                <span className="text-muted-foreground">
                  {formatDuration(
                    session.duration_seconds ?? 0,
                    i18n.resolvedLanguage,
                  )}
                </span>
                <IconButton
                  icon={Trash2}
                  label={t("pomodoro.history.delete", { time })}
                  disabled={remove.isPending}
                  onClick={() => remove.mutate(session.id)}
                />
              </li>
            );
          })}
        </ul>
      )}
    </Card>
  );
}
