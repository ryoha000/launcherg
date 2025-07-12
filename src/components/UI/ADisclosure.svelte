<script lang="ts">
  import { createDisclosure } from "svelte-headlessui";
  import { fly } from "svelte/transition";

  export let label: string;
  export let defaultOpen = false;

  const disclosure = createDisclosure({
    expanded: defaultOpen,
    label,
  });
</script>

<div>
  <button
    use:disclosure.button
    class="bg-transparent rounded transition-all hover:bg-bg-button-hover w-full"
  >
    <div class="p-(x-4 y-2) flex items-center gap-4 bg-transparent">
      <div class="text-(text-primary body2)">{label}</div>
      <div
        class="i-material-symbols-arrow-drop-down ml-auto h-5 w-5 color-text-primary transition-all flex-shrink-0"
        class:rotate-180={$disclosure.expanded}
      />
    </div>
  </button>
  {#if $disclosure.expanded}
    <div transition:fly={{ y: -40, duration: 150 }}>
      <div use:disclosure.panel>
        <slot />
      </div>
    </div>
  {/if}
</div>
