import { z } from 'zod'

// Validation messages are translation keys; the forms pass them through `t()`.
export type ValidationMessageKey =
  | 'validation.required'
  | 'validation.emailInvalid'
  | 'validation.passwordLength'
  | 'validation.displayNameLength'

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
