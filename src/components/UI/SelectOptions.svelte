<script lang="ts">
  import FilterInput from "@/components/UI/FilterInput.svelte";
  import { ListboxOptions, ListboxOption } from "@rgossiaux/svelte-headlessui";
  import { fly } from "svelte/transition";
  import type { Option } from "./select";
  import { createEventDispatcher } from "svelte";

  export let options: Option<string | number>[];
  export let title: string | undefined = undefined;
  export let enableFilter: boolean = false;
  export let filterPlaceholder = "";
  export let bottomAddButtonText = "";

  let query = "";
  $: filteredOptions = options.filter((v) =>
    query ? v.label.toLowerCase().includes(query.toLowerCase()) : true
  );

  const dispatcher = createEventDispatcher<{ add: {} }>();
</script>

<div
  transition:fly={{ y: -40, duration: 150 }}
  class="absolute mt-2 border border-(border-primary solid) rounded bg-bg-secondary z-10"
>
  <ListboxOptions static>
    {#if title}
      <div class="flex items-center gap-8 border-(b-1px border-primary solid)">
        <div
          class="whitespace-nowrap p-(x-4 y-2) text-(body2 text-primary) font-bold"
        >
          {title}
        </div>
      </div>
    {/if}
    {#if enableFilter}
      <div class="p-2 border-(b-1px border-primary solid)">
        <FilterInput
          bind:value={query}
          placeholder={filterPlaceholder}
          autofocus
        />
      </div>
    {/if}
    <div class="flex flex-(col)">
      {#each filteredOptions as option, i (option)}
        <ListboxOption
          value={option.value}
          class={({ active }) =>
            `${active ? "bg-bg-tertiary" : ""}
                p-(x-4 y-1) ${
                  options.length - 1 !== i
                    ? "border-(b-1px solid border-primary)"
                    : ""
                } w-full flex items-center gap-2 transition-all cursor-pointer`}
          let:selected
        >
          {#if selected}
            <div
              class="i-material-symbols-check-small-rounded h-5 w-5 color-text-primary"
            />
          {:else}
            <div class="h-5 w-5" />
          {/if}
          <div class="text-(body2 text-primary) font-medium">
            {option.label}
          </div>
        </ListboxOption>
      {/each}
    </div>
    {#if bottomAddButtonText}
      <button
        class="bg-transparent hover:bg-bg-tertiary transition-all w-full p-(x-4 y-1) flex items-center gap-2"
        on:click={() => dispatcher("add")}
      >
        <div class="w-5 h-5 i-iconoir-plus color-text-primary" />
        <div class="text-(text-primary body2 left)">
          {bottomAddButtonText}
        </div>
      </button>
    {/if}
  </ListboxOptions>
</div>
