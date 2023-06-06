<script lang="ts">
  import {
    Popover,
    PopoverButton,
    PopoverPanel,
  } from "@rgossiaux/svelte-headlessui";

  export let isRelativeRoot = true;
  export let panelClass = "";
</script>

<Popover class={isRelativeRoot ? "relative" : ""}>
  <PopoverButton as="div">
    <slot name="button" />
  </PopoverButton>
  <PopoverPanel
    let:close
    class="absolute z-10000 mt-2 border border-(border-primary solid) rounded bg-bg-secondary {panelClass}"
  >
    <slot {close} />
  </PopoverPanel>
</Popover>
