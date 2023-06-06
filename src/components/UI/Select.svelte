<script lang="ts">
  import SelectOptions from "@/components/UI/SelectOptions.svelte";
  import { Listbox, ListboxButton } from "@rgossiaux/svelte-headlessui";
  import type { Option } from "./select";

  export let options: Option<string | number>[];
  export let value: Option<string | number>["value"];
  export let iconClass: string = "";
  export let title: string | undefined = undefined;
  export let enableFilter: boolean = false;
  export let filterPlaceholder = "";
  export let bottomAddButtonText = "";

  $: selectedLabel = options.find((v) => v.value === value)?.label ?? "";
</script>

<Listbox
  {value}
  on:change={(e) => (value = e.detail)}
  let:open
  class="relative"
>
  <ListboxButton
    class="h-8 w-full flex items-center gap-2 border border-(border-button opacity-10 solid) rounded bg-bg-button px-3 transition-all hover:(border-border-button-hover bg-bg-button-hover)"
  >
    {#if iconClass}
      <div class={`${iconClass} w-4 h-4`} />
    {/if}
    <div class="text-(body text-primary) font-bold">{selectedLabel}</div>
    <div
      class="i-material-symbols-arrow-drop-down ml-auto h-4 w-4 color-text-primary transition-all"
      class:rotate-180={open}
    />
  </ListboxButton>
  {#if open}
    <SelectOptions
      {title}
      {enableFilter}
      {filterPlaceholder}
      {options}
      {bottomAddButtonText}
      on:add
    />
  {/if}
</Listbox>
