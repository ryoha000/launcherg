import type { MouseEventHandler } from 'react'

import { cva, type VariantProps } from 'class-variance-authority'

import { cn } from '@ui/lib/utils'

const buttonVariants = cva(
  'inline-flex items-center justify-center rounded-lg border border-transparent text-sm font-medium whitespace-nowrap transition-all outline-none focus-visible:border-ring focus-visible:ring-3 focus-visible:ring-ring/50 disabled:pointer-events-none disabled:opacity-50 [&_svg]:pointer-events-none [&_svg]:shrink-0',
  {
    variants: {
      variant: {
        default: 'bg-primary text-primary-foreground hover:bg-primary/90',
        outline: 'border-border bg-background hover:bg-muted hover:text-foreground',
        secondary: 'bg-secondary text-secondary-foreground hover:bg-secondary/80',
        ghost: 'hover:bg-muted hover:text-foreground',
        destructive: 'bg-destructive/10 text-destructive hover:bg-destructive/20',
        link: 'text-primary underline-offset-4 hover:underline',
      },
      size: {
        default: 'h-8 gap-1.5 px-3',
        sm: 'h-7 gap-1 px-2.5 text-[0.8rem]',
        lg: 'h-11 gap-1.5 px-4',
      },
    },
    defaultVariants: {
      variant: 'default',
      size: 'default',
    },
  },
)

type ButtonProps = VariantProps<typeof buttonVariants> & {
  children?: any
  className?: string
  disabled?: boolean
  onClick?: MouseEventHandler<HTMLButtonElement>
  type?: 'button' | 'reset' | 'submit'
}

function Button({
  children,
  className,
  disabled,
  onClick,
  size = 'default',
  type = 'button',
  variant = 'default',
}: ButtonProps) {
  return (
    <button
      type={type}
      disabled={disabled}
      onClick={onClick}
      className={cn(buttonVariants({ variant, size }), className)}
    >
      {children}
    </button>
  )
}

export { Button, buttonVariants }
