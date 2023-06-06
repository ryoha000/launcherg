<script lang="ts">
  import SelectOptions from "@/components/UI/SelectOptions.svelte";
  import type { Option } from "./select";
  import APopover from "@/components/UI/APopover.svelte";
  import { createEventDispatcher } from "svelte";

  export let options: Option<string | number>[];
  export let value: Option<string | number>["value"];
  export let iconClass: string = "";
  export let title: string | undefined = undefined;
  export let enableFilter: boolean = false;
  export let filterPlaceholder = "";
  export let bottomCreateButtonText = "";
  export let showSelectedCheck = false;

  $: selectedLabel = options.find((v) => v.value === value)?.label ?? "";

  const dispather = createEventDispatcher<{ create: {} }>();
</script>

<APopover let:open let:close>
  <div slot="button">
    <slot>
      <button
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
      </button>
    </slot>
  </div>
  <SelectOptions
    {title}
    {enableFilter}
    {filterPlaceholder}
    {options}
    {bottomCreateButtonText}
    {showSelectedCheck}
    bind:value
    on:select
    on:create={() => {
      close(null);
      dispather("create");
    }}
    on:close={() => close(null)}
  />
</APopover>
