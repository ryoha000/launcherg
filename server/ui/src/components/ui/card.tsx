import { cn } from '@ui/lib/utils'

type CardProps = {
  children?: any
  className?: string
}

function Card({ children, className }: CardProps) {
  return (
    <div
      className={cn(
        'flex flex-col gap-4 overflow-hidden rounded-xl bg-card py-4 text-sm text-card-foreground ring-1 ring-foreground/10',
        className,
      )}
    >
      {children}
    </div>
  )
}

function CardHeader({ children, className }: CardProps) {
  return <div className={cn('grid auto-rows-min items-start gap-1 px-4', className)}>{children}</div>
}

function CardTitle({ children, className }: CardProps) {
  return <div className={cn('font-heading text-base leading-snug font-medium', className)}>{children}</div>
}

function CardDescription({ children, className }: CardProps) {
  return <div className={cn('text-sm text-muted-foreground', className)}>{children}</div>
}

function CardAction({ children, className }: CardProps) {
  return <div className={cn('self-start justify-self-end', className)}>{children}</div>
}

function CardContent({ children, className }: CardProps) {
  return <div className={cn('px-4', className)}>{children}</div>
}

function CardFooter({ children, className }: CardProps) {
  return <div className={cn('flex items-center rounded-b-xl border-t bg-muted/50 p-4', className)}>{children}</div>
}

export {
  Card,
  CardAction,
  CardContent,
  CardDescription,
  CardFooter,
  CardHeader,
  CardTitle,
}
