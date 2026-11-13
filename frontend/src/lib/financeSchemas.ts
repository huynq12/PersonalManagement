import { z } from 'zod'
import type { ValidationMessageKey } from './authSchemas'
import { isIsoDate } from './dates'
import { kindSchema } from './finance'
import { parseOptionalAmount } from './settingsSchemas'

const required: ValidationMessageKey = 'validation.required'
const moneyInvalid: ValidationMessageKey = 'validation.moneyInvalid'
const dateInvalid: ValidationMessageKey = 'validation.dateInvalid'
const noteLength: ValidationMessageKey = 'validation.noteLength'
const nameLength: ValidationMessageKey = 'validation.categoryNameLength'

/** Longest category name the backend accepts, in characters. */
export const MAX_CATEGORY_NAME_LENGTH = 100
/** Longest note the backend accepts, in characters. */
export const MAX_NOTE_LENGTH = 500

/** The form that adds a category or renames and recolors one. */
export const categoryFormSchema = z.object({
  name: z
    .string()
    .refine((text) => text.trim() !== '', required)
    .refine((text) => text.trim().length <= MAX_CATEGORY_NAME_LENGTH, nameLength),
  color: z.string(),
})

export type CategoryForm = z.infer<typeof categoryFormSchema>

/**
 * The transaction form: every field is text. The amount is typed in the account currency's
 * major units, so the currency decides how many decimals are valid.
 */
export const transactionFormSchema = (currency: string) =>
  z.object({
    kind: kindSchema,
    amount: z
      .string()
      .refine((text) => text.trim() !== '', required)
      // Empty text is "clear" to `parseOptionalAmount`, which is not valid here: the rule above
      // already reported it, so only check the value when there is one.
      .refine(
        (text) => text.trim() === '' || typeof parseOptionalAmount(text, currency) === 'number',
        moneyInvalid,
      ),
    date: z.string().refine(isIsoDate, dateInvalid),
    categoryId: z.string().min(1, required),
    note: z.string().max(MAX_NOTE_LENGTH, noteLength),
  })

export type TransactionForm = z.infer<ReturnType<typeof transactionFormSchema>>
