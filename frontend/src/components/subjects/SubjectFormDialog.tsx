import { zodResolver } from "@hookform/resolvers/zod";
import { useForm } from "react-hook-form";
import type { ComponentProps } from "react";
import { useTranslation } from "react-i18next";
import { useCreateSubject, useUpdateSubject } from "../../hooks/useSubjects";
import { errorMessageKey } from "../../i18n/errorMessage";
import type { ValidationMessageKey } from "../../lib/authSchemas";
import {
  DEFAULT_SUBJECT_COLOR,
  SUBJECT_COLORS,
  subjectFormSchema,
  type SubjectForm,
} from "../../lib/subjectSchemas";
import type { Subject } from "../../lib/subjects";
import Alert from "../ui/Alert";
import Button from "../ui/Button";
import Dialog from "../ui/Dialog";
import TextField from "../ui/TextField";

interface SubjectFormDialogProps {
  /** The subject to edit; leave out to add a new one. */
  subject?: Subject;
  onClose: () => void;
}

/** Dialog with the subject form: a name and a color from the fixed palette. */
export default function SubjectFormDialog({
  subject,
  onClose,
}: SubjectFormDialogProps) {
  const { t } = useTranslation();
  const create = useCreateSubject();
  const update = useUpdateSubject();
  const save = subject ? update : create;

  const {
    register,
    handleSubmit,
    formState: { errors },
  } = useForm<SubjectForm>({
    resolver: zodResolver(subjectFormSchema),
    defaultValues: {
      name: subject?.name ?? "",
      color: subject?.color ?? DEFAULT_SUBJECT_COLOR,
    },
  });

  // The API accepts any color, so a subject may have one outside the palette; keep it selectable.
  const hasCustomColor =
    subject !== undefined &&
    !SUBJECT_COLORS.some(
      (color) => color.hex.toLowerCase() === subject.color.toLowerCase(),
    );

  const onSubmit = handleSubmit((values) => {
    if (subject) {
      update.mutate(
        { id: subject.id, changes: values },
        { onSuccess: onClose },
      );
    } else {
      create.mutate(values, { onSuccess: onClose });
    }
  });

  const nameError = errors.name?.message
    ? t(errors.name.message as ValidationMessageKey)
    : undefined;

  return (
    <Dialog
      title={t(subject ? "subjects.form.editTitle" : "subjects.form.addTitle")}
      onClose={onClose}
    >
      {save.isError ? (
        <Alert variant="error">{t(errorMessageKey(save.error))}</Alert>
      ) : null}

      <form
        className="flex flex-col gap-4"
        noValidate
        onSubmit={(event) => void onSubmit(event)}
      >
        <TextField
          label={t("subjects.form.name")}
          autoComplete="off"
          // The dialog was opened to fill in this form, so starting in its first field is expected.
          autoFocus
          error={nameError}
          {...register("name")}
        />

        <fieldset className="flex flex-col gap-2">
          <legend className="text-sm font-medium">
            {t("subjects.form.color")}
          </legend>
          <div className="flex flex-wrap gap-2">
            {SUBJECT_COLORS.map((color) => (
              <ColorOption
                key={color.hex}
                hex={color.hex}
                label={t(`subjects.colors.${color.key}`)}
                {...register("color")}
              />
            ))}
            {hasCustomColor ? (
              <ColorOption
                hex={subject.color}
                label={t("subjects.colors.custom")}
                {...register("color")}
              />
            ) : null}
          </div>
        </fieldset>

        <div className="flex justify-end gap-2">
          <Button variant="ghost" onClick={onClose}>
            {t("common.cancel")}
          </Button>
          <Button type="submit" disabled={save.isPending}>
            {t("common.save")}
          </Button>
        </div>
      </form>
    </Dialog>
  );
}

interface ColorOptionProps extends ComponentProps<"input"> {
  hex: string;
  label: string;
}

/** One swatch of the color picker: a real radio input, visually replaced by a colored circle. */
function ColorOption({ hex, label, ...input }: ColorOptionProps) {
  return (
    <label className="relative flex size-9 cursor-pointer items-center justify-center">
      <input
        type="radio"
        value={hex}
        aria-label={label}
        className="peer sr-only"
        {...input}
      />
      <span
        aria-hidden="true"
        style={{ backgroundColor: hex }}
        className="size-7 rounded-full ring-offset-2 ring-offset-background transition peer-checked:ring-2 peer-checked:ring-ring peer-focus-visible:ring-2 peer-focus-visible:ring-ring"
      />
    </label>
  );
}
