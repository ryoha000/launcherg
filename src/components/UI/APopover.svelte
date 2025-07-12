<script lang='ts'>
  import { createPopover } from 'svelte-headlessui'
  import { fly } from 'svelte/transition'

  interface Props {
    isRelativeRoot?: boolean
    panelClass?: string
    button?: import('svelte').Snippet<[{ open: boolean, close: (v?: null) => void }]>
    children?: import('svelte').Snippet<[{ open: boolean, close: (v?: null) => void }]>
  }

  const {
    isRelativeRoot = true,
    panelClass = '',
    button,
    children,
  }: Props = $props()

  const popover = createPopover({})
</script>

<div class={isRelativeRoot ? 'relative' : ''}>
  <div use:popover.button>
    {@render button?.({ open: $popover.expanded, close: popover.close })}
  </div>
  {#if $popover.expanded}
    <div
      transition:fly={{ y: -40, duration: 150 }}
      class='absolute z-10000 mt-2 border border-(border-primary solid) rounded bg-bg-secondary {panelClass}'
    >
      <div use:popover.panel>
        {@render children?.({ open: $popover.expanded, close: popover.close })}
      </div>
    </div>
  {/if}
</div>
