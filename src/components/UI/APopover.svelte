<script lang="ts">
  import { createPopover } from "svelte-headlessui";
  import { fly } from "svelte/transition";

  export let isRelativeRoot = true;
  export let panelClass = "";

  const popover = createPopover({});
</script>

<div class={isRelativeRoot ? "relative" : ""}>
  <div use:popover.button>
    <slot name="button" open={$popover.expanded} close={popover.close} />
  </div>
  {#if $popover.expanded}
    <div
      transition:fly={{ y: -40, duration: 150 }}
      class="absolute z-10000 mt-2 border border-(border-primary solid) rounded bg-bg-secondary {panelClass}"
    >
      <div use:popover.panel>
        <slot open={$popover.expanded} close={popover.close} />
      </div>
    </div>
  {/if}
</div>
