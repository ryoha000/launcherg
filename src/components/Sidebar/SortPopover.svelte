<script lang="ts">
  import { SORT_LABELS, type SortOrder } from "@/components/Sidebar/sort";
  import OptionButton from "@/components/UI/OptionButton.svelte";
  import { createEventDispatcher } from "svelte";

  export let value: SortOrder;
  const dispatcher = createEventDispatcher<{ close: {} }>();

  const sortOrders: SortOrder[] = [
    "gamename-asc",
    "gamename-desc",
    "sellyear-asc",
    "sellyear-desc",
    "brandname-asc",
    "brandname-desc",
  ];
</script>

<div>
  <div
    class="font-bold text-(text-primary body3) p-(l-4 r-2 y-2) flex items-center"
  >
    <div>Select sort option</div>
    <button
      on:click={() => dispatcher("close")}
      class="ml-auto w-5 h-5 i-iconoir-cancel color-text-tertiary hover:color-text-primary transition-all"
    />
  </div>
  {#each sortOrders as sortOrder (sortOrder)}
    <OptionButton
      on:click={() => {
        value = sortOrder;
        dispatcher("close");
      }}
      selected={value === sortOrder}
      text={SORT_LABELS[sortOrder]}
      showIcon
    />
  {/each}
</div>
