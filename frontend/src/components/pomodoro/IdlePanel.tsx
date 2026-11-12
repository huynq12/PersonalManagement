import { useId, useState } from "react";
import { useTranslation } from "react-i18next";
import { Link } from "react-router";
import { useStartSession } from "../../hooks/useStudySessions";
import { errorMessageKey } from "../../i18n/errorMessage";
import type { Subject } from "../../lib/subjects";
import Alert from "../ui/Alert";
import Button from "../ui/Button";
import Select from "../ui/Select";
import ClockFace from "./ClockFace";

interface IdlePanelProps {
  /** Subjects that can be chosen: the active ones. */
  subjects: Subject[];
  focusMinutes: number;
}

/** Nothing is running: pick a subject and start a focus session. */
export default function IdlePanel({ subjects, focusMinutes }: IdlePanelProps) {
  const { t } = useTranslation();
  const start = useStartSession();
  const [subjectId, setSubjectId] = useState("");
  const selectId = useId();

  return (
    <ClockFace label={t("pomodoro.phase.ready")} seconds={focusMinutes * 60}>
      {start.isError ? (
        <Alert variant="error">{t(errorMessageKey(start.error))}</Alert>
      ) : null}

      <div className="flex w-full max-w-xs flex-col gap-2">
        <label htmlFor={selectId} className="text-sm font-medium">
          {t("pomodoro.subject")}
        </label>
        <Select
          id={selectId}
          value={subjectId}
          onChange={(event) => setSubjectId(event.target.value)}
        >
          <option value="">{t("pomodoro.noSubject")}</option>
          {subjects.map((subject) => (
            <option key={subject.id} value={subject.id}>
              {subject.name}
            </option>
          ))}
        </Select>
        {subjects.length === 0 ? (
          <Link to="/subjects" className="text-sm underline underline-offset-4">
            {t("pomodoro.manageSubjects")}
          </Link>
        ) : null}
      </div>

      <Button
        disabled={start.isPending}
        onClick={() => start.mutate(subjectId ? { subject_id: subjectId } : {})}
      >
        {t("pomodoro.start")}
      </Button>
    </ClockFace>
  );
}
