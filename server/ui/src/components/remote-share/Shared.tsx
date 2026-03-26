import type { ReactNode } from 'react'

export function SectionLabel({
  children,
}: {
  children: ReactNode
}) {
  return (
    <p className="text-[0.72rem] font-medium tracking-[0.28em] text-muted-foreground uppercase">
      {children}
    </p>
  )
}

export function HeroMetric({
  label,
  value,
}: {
  label: string
  value: string
}) {
  return (
    <div className="border-l border-white/14 pl-4 first:border-l-0 first:pl-0">
      <p className="text-[0.68rem] tracking-[0.24em] text-white/44 uppercase">{label}</p>
      <p className="mt-2 text-base font-semibold tracking-tight text-white sm:text-lg">{value}</p>
    </div>
  )
}
