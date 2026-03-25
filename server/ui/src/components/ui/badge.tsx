import { cva, type VariantProps } from 'class-variance-authority'

import { cn } from '@ui/lib/utils'

const badgeVariants = cva(
  'inline-flex h-5 w-fit items-center justify-center gap-1 overflow-hidden rounded-4xl border border-transparent px-2 py-0.5 text-xs font-medium whitespace-nowrap transition-all',
  {
    variants: {
      variant: {
        default: 'bg-primary text-primary-foreground',
        secondary: 'bg-secondary text-secondary-foreground',
        destructive: 'bg-destructive/10 text-destructive',
        outline: 'border-border text-foreground',
        ghost: 'hover:bg-muted hover:text-muted-foreground',
        link: 'text-primary underline-offset-4 hover:underline',
      },
    },
    defaultVariants: {
      variant: 'default',
    },
  },
)

type BadgeProps = VariantProps<typeof badgeVariants> & {
  children?: any
  className?: string
}

function Badge({ children, className, variant = 'default' }: BadgeProps) {
  return <span className={cn(badgeVariants({ variant }), className)}>{children}</span>
}

export { Badge, badgeVariants }
