<script lang="ts">
  import Input from "@/components/UI/Input.svelte";
  import type { Option } from "./select";
  import { createEventDispatcher } from "svelte";

  export let options: Option<string | number>[];
  export let title: string | undefined = undefined;
  export let enableFilter: boolean = false;
  export let showSelectedCheck = true;
  export let filterPlaceholder = "";
  export let bottomCreateButtonText = "";
  export let value: string | number;

  let query = "";
  $: filteredOptions = options.filter((v) =>
    query ? v.label.toLowerCase().includes(query.toLowerCase()) : true
  );

  const dispatcher = createEventDispatcher<{
    select: { value: string | number };
    create: {};
    close: {};
  }>();
</script>

<div>
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
      <Input bind:value={query} placeholder={filterPlaceholder} autofocus />
    </div>
  {/if}
  <div class="flex flex-(col)">
    {#each filteredOptions as option, i (option)}
      <button
        class={`${value === option.value ? "bg-bg-tertiary" : "bg-transparent"}
                p-(x-4 y-1) ${
                  options.length - 1 !== i
                    ? "border-(b-1px solid border-primary)"
                    : ""
                } hover:bg-bg-tertiary w-full flex items-center gap-2 transition-all cursor-pointer`}
        on:click={() => {
          value = option.value;
          dispatcher("select", { value: option.value });
          dispatcher("close");
        }}
      >
        {#if showSelectedCheck}
          <div
            class="h-5 w-5 color-text-primary"
            class:i-material-symbols-check-small-rounded={value ===
              option.value}
          />
        {/if}
        <div class="text-(body2 text-primary) font-medium">
          {option.label}
        </div>
      </button>
    {/each}
  </div>
  {#if bottomCreateButtonText}
    <button
      class="bg-transparent hover:bg-bg-tertiary transition-all w-full p-(x-4 y-1) flex items-center gap-2 border-(t-1px solid border-primary)"
      on:click={() => dispatcher("create")}
    >
      <div class="w-5 h-5 i-iconoir-plus color-text-primary" />
      <div class="text-(text-primary body2 left) whitespace-nowrap">
        {bottomCreateButtonText}
      </div>
    </button>
  {/if}
</div>
