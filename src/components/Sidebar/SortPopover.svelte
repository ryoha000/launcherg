<script lang="ts">
  import SortOptionButton from "@/components/Sidebar/SortOptionButton.svelte";
  import type { SortOrder } from "@/components/Sidebar/sort";
  import { createEventDispatcher } from "svelte";

  export let value: SortOrder;
  const dispatcher = createEventDispatcher<{ close: {} }>();
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
  <SortOptionButton
    on:click={() => (value = "gamename-asc")}
    selected={value === "gamename-asc"}
    text="タイトルで並び替え(昇順)"
  />
  <SortOptionButton
    on:click={() => (value = "gamename-desc")}
    selected={value === "gamename-desc"}
    text="タイトルで並び替え(降順)"
  />
</div>
