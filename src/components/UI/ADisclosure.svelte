<script lang="ts">
  import {
    Disclosure,
    DisclosureButton,
    DisclosurePanel,
  } from "@rgossiaux/svelte-headlessui";
  import { fly } from "svelte/transition";

  export let label: string;
</script>

<Disclosure let:open>
  <DisclosureButton
    class="bg-transparent rounded transition-all hover:bg-bg-button-hover w-full"
  >
    <div class="p-(x-4 y-2) flex items-center gap-4 bg-transparent">
      <div class="text-(text-primary body2)">{label}</div>
      <div
        class="i-material-symbols-arrow-drop-down ml-auto h-5 w-5 color-text-primary transition-all flex-shrink-0"
        class:rotate-180={open}
      />
    </div>
  </DisclosureButton>
  {#if open}
    <div transition:fly={{ y: -40, duration: 150 }}>
      <DisclosurePanel static>
        <slot />
      </DisclosurePanel>
    </div>
  {/if}
</Disclosure>
