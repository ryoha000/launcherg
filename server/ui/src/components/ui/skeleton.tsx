import { cn } from '@ui/lib/utils'

function Skeleton({ className }: { className?: string }) {
  return <div className={cn('animate-pulse rounded-md bg-muted', className)} />
}

export { Skeleton }
