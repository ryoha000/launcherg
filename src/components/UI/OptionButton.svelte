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
  class="w-full flex items-center bg-transparent p-(x-4 y-2) transition-all hover:(bg-bg-button-hover) {withBorderTop
    && 'border-(t-1px solid border-primary)'}"
>
  {#if showIcon}
    <div
      class='h-5 w-5 color-text-primary'
      class:i-material-symbols-check-small-rounded={selected}
    ></div>
  {/if}
  <div class='whitespace-nowrap text-(body2 text-primary) font-bold'>
    {text}
  </div>
</button>
