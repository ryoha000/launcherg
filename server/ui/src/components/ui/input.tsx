import type { ChangeEventHandler } from 'react'

import { cn } from '@ui/lib/utils'

type InputProps = {
  autoComplete?: string
  className?: string
  id?: string
  onChange?: ChangeEventHandler<HTMLInputElement>
  placeholder?: string
  readOnly?: boolean
  type?: string
  value?: string
}

function Input({
  autoComplete,
  className,
  id,
  onChange,
  placeholder,
  readOnly = false,
  type = 'text',
  value,
}: InputProps) {
  return (
    <input
      id={id}
      type={type}
      readOnly={readOnly}
      value={value}
      onChange={onChange}
      placeholder={placeholder}
      autoComplete={autoComplete}
      className={cn(
        'h-8 w-full min-w-0 rounded-lg border border-input bg-transparent px-2.5 py-1 text-base transition-colors outline-none placeholder:text-muted-foreground focus-visible:border-ring focus-visible:ring-3 focus-visible:ring-ring/50 disabled:pointer-events-none disabled:cursor-not-allowed disabled:bg-input/50 disabled:opacity-50 md:text-sm',
        className,
      )}
    />
  )
}

export { Input }
