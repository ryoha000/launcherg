<script lang='ts'>
  import { createDisclosure } from 'svelte-headlessui'
  import { fly } from 'svelte/transition'

  interface Props {
    label: string
    defaultOpen?: boolean
    children?: import('svelte').Snippet
  }

  const { label, defaultOpen = false, children }: Props = $props()

  const disclosure = createDisclosure({
    expanded: defaultOpen,
    label,
  })
</script>

<div>
  <button
    use:disclosure.button
    class='w-full rounded bg-transparent transition-all hover:bg-bg-button-hover'
  >
    <div class='flex items-center gap-4 bg-transparent p-(x-4 y-2)'>
      <div class='text-(body2 text-primary)'>{label}</div>
      <div
        class='i-material-symbols-arrow-drop-down ml-auto h-5 w-5 flex-shrink-0 color-text-primary transition-all'
        class:rotate-180={$disclosure.expanded}
      ></div>
    </div>
  </button>
  {#if $disclosure.expanded}
    <div transition:fly={{ y: -40, duration: 150 }}>
      <div use:disclosure.panel>
        {@render children?.()}
      </div>
    </div>
  {/if}
</div>
