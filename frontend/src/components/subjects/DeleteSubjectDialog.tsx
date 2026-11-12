import { useTranslation } from "react-i18next";
import { useDeleteSubject } from "../../hooks/useSubjects";
import { errorMessageKey } from "../../i18n/errorMessage";
import type { Subject } from "../../lib/subjects";
import Alert from "../ui/Alert";
import Button from "../ui/Button";
import Dialog from "../ui/Dialog";

interface DeleteSubjectDialogProps {
  subject: Subject;
  onClose: () => void;
}

/** Asks before deleting a subject. Past study sessions are kept, just without a subject. */
export default function DeleteSubjectDialog({
  subject,
  onClose,
}: DeleteSubjectDialogProps) {
  const { t } = useTranslation();
  const remove = useDeleteSubject();

  return (
    <Dialog title={t("subjects.deleteDialog.title")} onClose={onClose}>
      {remove.isError ? (
        <Alert variant="error">{t(errorMessageKey(remove.error))}</Alert>
      ) : null}

      <p className="text-sm text-muted-foreground">
        {t("subjects.deleteDialog.description", { name: subject.name })}
      </p>

      <div className="flex justify-end gap-2">
        {/* Cancel comes first and takes focus, so Enter does not delete by accident. */}
        <Button variant="ghost" autoFocus onClick={onClose}>
          {t("common.cancel")}
        </Button>
        <Button
          className="bg-destructive text-background hover:bg-destructive/90"
          disabled={remove.isPending}
          onClick={() => remove.mutate(subject.id, { onSuccess: onClose })}
        >
          {t("subjects.deleteDialog.confirm")}
        </Button>
      </div>
    </Dialog>
  );
}
