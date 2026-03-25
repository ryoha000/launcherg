import { cva, type VariantProps } from 'class-variance-authority'

import { cn } from '@ui/lib/utils'

const alertVariants = cva(
  'grid w-full gap-0.5 rounded-lg border px-3 py-2 text-left text-sm has-[>svg]:grid-cols-[auto_1fr] has-[>svg]:gap-x-2 [&>svg]:row-span-2 [&>svg]:translate-y-0.5 [&>svg]:text-current',
  {
    variants: {
      variant: {
        default: 'bg-card text-card-foreground',
        destructive: 'bg-card text-destructive',
      },
    },
    defaultVariants: {
      variant: 'default',
    },
  },
)

type AlertProps = VariantProps<typeof alertVariants> & {
  children?: any
  className?: string
}

function Alert({ children, className, variant }: AlertProps) {
  return (
    <div role="alert" className={cn(alertVariants({ variant }), className)}>
      {children}
    </div>
  )
}

function AlertTitle({ children, className }: { children?: any, className?: string }) {
  return <div className={cn('font-heading font-medium', className)}>{children}</div>
}

function AlertDescription({ children, className }: { children?: any, className?: string }) {
  return <div className={cn('text-sm text-muted-foreground', className)}>{children}</div>
}

function AlertAction({ children, className }: { children?: any, className?: string }) {
  return <div className={cn('justify-self-end', className)}>{children}</div>
}

export { Alert, AlertAction, AlertDescription, AlertTitle }
