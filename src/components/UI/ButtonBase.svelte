<script lang='ts'>
  import type { Props as TippyOption } from 'tippy.js'
  import type { Variant } from '@/components/UI/button'
  import tippy from 'tippy.js'

  interface Props {
    appendClass?: string
    type?: 'button' | 'submit' | undefined
    tooltip?: Partial<TippyOption> | undefined
    disabled?: boolean
    variant?: Variant
    children?: import('svelte').Snippet
    onclick?: (e: Event) => void
  }

  let {
    appendClass = '',
    type = undefined,
    tooltip = undefined,
    disabled = false,
    variant = 'normal',
    children,
    onclick,
  }: Props = $props()

  const tooltipAction = (node: HTMLElement) => {
    if (!tooltip) {
      return
    }

    const tp = tippy(node, tooltip)

    return {
      update() {
        if (!tooltip) {
          return
        }
        tp.setProps(tooltip)
      },
      destroy() {
        tp.destroy()
      },
    }
  }

  let buttonVariantClass = $derived.by(() => {
    switch (variant) {
      case 'normal':
        return 'bg-bg-button border-(~ border-button opacity-10 solid) text-text-primary hover:(border-border-button-hover bg-bg-button-hover)'
      case 'accent':
        return 'bg-bg-button border-(~ border-button opacity-10 solid) text-accent-accent hover:(border-accent-accent bg-accent-accent text-text-secondary)'
      case 'error':
        return 'bg-bg-button border-(~ border-button opacity-10 solid) text-accent-error hover:(border-accent-error bg-accent-error text-text-secondary)'
      case 'success':
        return disabled
          ? 'bg-bg-success-disabled border-(~ solid border-success-disabled) text-text-success-disabled'
          : 'bg-accent-success border-(~ solid accent-success) text-text-white hover:bg-bg-success-hover'
      default:
        throw new Error(`Unknown variant: ${variant satisfies never}`)
    }
  })
</script>

<button
  use:tooltipAction
  {type}
  {disabled}
  class={`rounded transition-all ${buttonVariantClass} ${appendClass}`}
  {onclick}
>
  {@render children?.()}
</button>
