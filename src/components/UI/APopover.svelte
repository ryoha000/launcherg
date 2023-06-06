<script lang="ts">
  import {
    Popover,
    PopoverButton,
    PopoverPanel,
  } from "@rgossiaux/svelte-headlessui";
  import { fly } from "svelte/transition";

  export let isRelativeRoot = true;
  export let panelClass = "";
</script>

<Popover class={isRelativeRoot ? "relative" : ""} let:open>
  <PopoverButton as="div">
    <slot name="button" {open} {close} />
  </PopoverButton>
  {#if open}
    <div
      transition:fly={{ y: -40, duration: 150 }}
      class="absolute z-10000 mt-2 border border-(border-primary solid) rounded bg-bg-secondary {panelClass}"
    >
      <PopoverPanel static let:close>
        <slot {open} {close} />
      </PopoverPanel>
    </div>
  {/if}
</Popover>
