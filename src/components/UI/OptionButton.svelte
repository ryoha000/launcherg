<script lang='ts'>
  interface Props {
    selected?: boolean
    text: string
    withBorderTop?: boolean
    showIcon?: boolean
    onclick?: (e: Event) => void
  }

  const {
    selected = false,
    text,
    withBorderTop = true,
    showIcon = false,
    onclick,
  }: Props = $props()
</script>

<button
  {onclick}
  class="p-(x-4 y-2) flex items-center bg-transparent transition-all hover:(bg-bg-button-hover) w-full {withBorderTop
    && 'border-(t-1px solid border-primary)'}"
>
  {#if showIcon}
    <div
      class='h-5 w-5 color-text-primary'
      class:i-material-symbols-check-small-rounded={selected}
    ></div>
  {/if}
  <div class='text-(text-primary body2) font-bold whitespace-nowrap'>
    {text}
  </div>
</button>
