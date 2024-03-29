<script lang="ts">
  import Input from "@/components/UI/Input.svelte";
  import { useFilter } from "@/lib/filter";
  import { type Option } from "@/lib/trieFilter";
  import { createWritable } from "@/lib/utils";
  import { query } from "@/store/query";
  import SimpleBar from "simplebar";
  import { createEventDispatcher } from "svelte";

  export let options: Option<string | number>[];
  export let title: string | undefined = undefined;
  export let enableFilter: boolean = false;
  export let showSelectedCheck = true;
  export let filterPlaceholder = "";
  export let bottomCreateButtonText = "";
  export let value: string | number;

  const [writableOptions, getOptions] = createWritable(options);
  $: {
    writableOptions.set(options);
  }

  const { filtered } = useFilter(query, writableOptions, getOptions);

  const dispatcher = createEventDispatcher<{
    select: { value: string | number };
    create: {};
    close: {};
  }>();

  const simplebar = (node: HTMLElement) => {
    new SimpleBar(node, { scrollbarMinSize: 64 });
  };
</script>

<div class="max-h-80 overflow-hidden flex-(~ col)">
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
      <Input bind:value={$query} placeholder={filterPlaceholder} autofocus />
    </div>
  {/if}
  <div class="flex flex-(col) overflow-y-auto min-h-full" use:simplebar>
    {#each $filtered as option, i (option)}
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
      class="bg-transparent hover:bg-bg-tertiary transition-all w-full p-(l-4 r-5 y-2) flex items-center border-(t-1px solid border-primary)"
      on:click={() => dispatcher("create")}
    >
      <div class="w-5 h-5 i-iconoir-plus color-text-primary" />
      <div class="text-(text-primary body2 left) font-bold whitespace-nowrap">
        {bottomCreateButtonText}
      </div>
    </button>
  {/if}
</div>
