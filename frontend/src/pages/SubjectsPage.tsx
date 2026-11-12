import { Plus } from "lucide-react";
import { useState } from "react";
import { useTranslation } from "react-i18next";
import ArchivedSubjects from "../components/subjects/ArchivedSubjects";
import DeleteSubjectDialog from "../components/subjects/DeleteSubjectDialog";
import SubjectFormDialog from "../components/subjects/SubjectFormDialog";
import SubjectList from "../components/subjects/SubjectList";
import Alert from "../components/ui/Alert";
import Button from "../components/ui/Button";
import { useSubjects, useUpdateSubject } from "../hooks/useSubjects";
import { errorMessageKey } from "../i18n/errorMessage";
import type { Subject } from "../lib/subjects";

/** Which dialog is open, if any. Only one can be open at a time. */
type OpenDialog =
  | { kind: "add" }
  | { kind: "edit"; subject: Subject }
  | { kind: "delete"; subject: Subject }
  | null;

/** The Subjects screen: add, edit, archive, restore, and delete the subjects of study. */
export default function SubjectsPage() {
  const { t } = useTranslation();
  const subjects = useSubjects();
  const update = useUpdateSubject();
  const [dialog, setDialog] = useState<OpenDialog>(null);

  const closeDialog = () => setDialog(null);
  const setArchived = (subject: Subject, archived: boolean) =>
    update.mutate({ id: subject.id, changes: { archived } });

  const active =
    subjects.data?.filter((subject) => subject.archived_at === null) ?? [];
  const archived =
    subjects.data?.filter((subject) => subject.archived_at !== null) ?? [];

  return (
    <main className="mx-auto flex w-full max-w-3xl flex-col gap-6 px-4 py-6 md:px-8 md:py-8">
      <header className="flex flex-wrap items-start justify-between gap-3">
        <div className="flex flex-col gap-1">
          <h1 className="text-2xl font-semibold tracking-tight md:text-3xl">
            {t("subjects.title")}
          </h1>
          <p className="text-sm text-muted-foreground">
            {t("subjects.description")}
          </p>
        </div>
        <Button onClick={() => setDialog({ kind: "add" })}>
          <Plus className="size-4" aria-hidden="true" />
          {t("subjects.add")}
        </Button>
      </header>

      {update.isError ? (
        <Alert variant="error">{t(errorMessageKey(update.error))}</Alert>
      ) : null}

      {subjects.isPending ? (
        <p className="text-sm text-muted-foreground">{t("common.loading")}</p>
      ) : subjects.isError ? (
        <div className="flex flex-col items-start gap-3">
          <Alert variant="error">{t(errorMessageKey(subjects.error))}</Alert>
          <Button variant="secondary" onClick={() => void subjects.refetch()}>
            {t("common.retry")}
          </Button>
        </div>
      ) : (
        <>
          {active.length === 0 ? (
            <p className="rounded-xl border border-dashed border-border px-6 py-12 text-center text-sm text-muted-foreground">
              {t("subjects.empty")}
            </p>
          ) : (
            <SubjectList
              subjects={active}
              onEdit={(subject) => setDialog({ kind: "edit", subject })}
              onArchive={(subject) => setArchived(subject, true)}
              onDelete={(subject) => setDialog({ kind: "delete", subject })}
            />
          )}

          {archived.length > 0 ? (
            <ArchivedSubjects
              subjects={archived}
              onRestore={(subject) => setArchived(subject, false)}
              onDelete={(subject) => setDialog({ kind: "delete", subject })}
            />
          ) : null}
        </>
      )}

      {dialog?.kind === "add" ? (
        <SubjectFormDialog onClose={closeDialog} />
      ) : null}
      {dialog?.kind === "edit" ? (
        <SubjectFormDialog subject={dialog.subject} onClose={closeDialog} />
      ) : null}
      {dialog?.kind === "delete" ? (
        <DeleteSubjectDialog subject={dialog.subject} onClose={closeDialog} />
      ) : null}
    </main>
  );
}
