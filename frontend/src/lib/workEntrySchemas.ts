import { z } from 'zod'
import type { ValidationMessageKey } from './authSchemas'
import { isIsoDate, type IsoDate } from './dates'
import { parseOptionalHours } from './settingsSchemas'

const required: ValidationMessageKey = 'validation.required'
const hoursInvalid: ValidationMessageKey = 'validation.hoursInvalid'
const dateInvalid: ValidationMessageKey = 'validation.dateInvalid'
const dateInFuture: ValidationMessageKey = 'validation.dateInFuture'
const noteLength: ValidationMessageKey = 'validation.noteLength'

/** Longest note the backend accepts, in characters. */
export const MAX_NOTE_LENGTH = 500

/**
 * The work entry form: every field is text. `today` is the user's current date, because the
 * backend refuses entries for future days.
 */
export const workEntryFormSchema = (today: IsoDate) =>
  z.object({
    date: z
      .string()
      .refine(isIsoDate, dateInvalid)
      // ISO dates compare correctly as plain strings. Skipped for an invalid date so the user
      // sees only one message.
      .refine((date) => !isIsoDate(date) || date <= today, dateInFuture),
    hours: z
      .string()
      .refine((text) => text.trim() !== '', required)
      // `parseOptionalHours` reads empty text as "clear", which is not valid here: the first
      // rule already reported it, so only check the value when there is one.
      .refine((text) => text.trim() === '' || parseOptionalHours(text) !== undefined, hoursInvalid),
    note: z.string().max(MAX_NOTE_LENGTH, noteLength),
  })

export type WorkEntryForm = z.infer<ReturnType<typeof workEntryFormSchema>>
