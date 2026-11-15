import { z } from 'zod'

// Validation messages are translation keys; the forms pass them through `t()`.
export type ValidationMessageKey =
  | 'validation.required'
  | 'validation.emailInvalid'
  | 'validation.passwordLength'
  | 'validation.displayNameLength'
  | 'validation.habitNameLength'
  | 'validation.categoryNameLength'
  | 'validation.taskTitleLength'
  | 'validation.weekdaysRequired'
  | 'validation.minutesRange'
  | 'validation.longBreakEveryRange'
  | 'validation.moneyInvalid'
  | 'validation.hoursInvalid'
  | 'validation.dateInvalid'
  | 'validation.dateInFuture'
  | 'validation.noteLength'
  | 'validation.subjectNameLength'

const required: ValidationMessageKey = 'validation.required'
const emailInvalid: ValidationMessageKey = 'validation.emailInvalid'
const passwordLength: ValidationMessageKey = 'validation.passwordLength'
const displayNameLength: ValidationMessageKey = 'validation.displayNameLength'

const email = z.string().trim().pipe(z.email(emailInvalid))

export const loginFormSchema = z.object({
  email,
  // No length rule here: the rules may have been different when the password was chosen.
  password: z.string().min(1, required),
})

export type LoginForm = z.infer<typeof loginFormSchema>

// Mirrors the backend rules: 8 to 128 characters, no composition rules.
export const registerFormSchema = z.object({
  email,
  displayName: z.string().trim().max(100, displayNameLength),
  password: z.string().min(8, passwordLength).max(128, passwordLength),
})

export type RegisterForm = z.infer<typeof registerFormSchema>

export const profileFormSchema = z.object({
  displayName: z.string().trim().min(1, required).max(100, displayNameLength),
})

export type ProfileForm = z.infer<typeof profileFormSchema>
