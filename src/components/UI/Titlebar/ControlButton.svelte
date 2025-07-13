<script lang='ts'>
  interface Props {
    variant: 'normal' | 'danger'
    isFocused?: boolean
    onclick?: (e: MouseEvent) => void
    children?: import('svelte').Snippet<[]>
  }
  const { variant, isFocused, onclick, children }: Props = $props()

  const colorClass = $derived.by(() => {
    switch (variant) {
      case 'normal':
        return `${isFocused ? 'text-white' : 'text-[#797979]'} hover:bg-white/[.06] active:bg-white/[.04]`
      case 'danger':
        return `${isFocused ? 'text-white' : 'text-[#797979]'} hover:bg-[#c42b1c] hover:text-white active:bg-[#c42b1c]/90`
      default:
        throw new Error(`Unknown variant: ${variant satisfies never}`)
    }
  })
</script>

<button {onclick} class='h-full w-12 flex cursor-default items-center justify-center rounded-none bg-transparent {colorClass} transition-all' tabindex={-1}>
  {@render children?.()}
</button>
