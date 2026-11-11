import type { ComponentProps } from 'react'
import { BUTTON_CLASSES, type ButtonVariant } from './buttonClasses'

interface ButtonProps extends ComponentProps<'button'> {
  variant?: ButtonVariant
}

export default function Button({
  variant = 'primary',
  className = '',
  type = 'button',
  ...props
}: ButtonProps) {
  return <button type={type} className={`${BUTTON_CLASSES[variant]} ${className}`} {...props} />
}
