<script lang='ts'>
  interface Props {
    icon?: string
    onclick?: (e: Event) => void
  }

  const { icon = 'i-iconoir-cancel', onclick }: Props = $props()
</script>

<button
  {onclick}
  type='button'
  tabindex={-1}
  class='ml-auto w-5 h-5 {icon} color-text-tertiary hover:color-text-primary transition-all flex-shrink-0'
  aria-label='Cancel'
></button>
