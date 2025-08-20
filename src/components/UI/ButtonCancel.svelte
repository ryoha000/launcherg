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
  class='ml-auto h-5 w-5 {icon} flex-shrink-0 color-text-tertiary transition-all hover:color-text-primary'
  aria-label='Cancel'
></button>
