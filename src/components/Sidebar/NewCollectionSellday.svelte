<script lang="ts">
  import NewCollectionOption from "@/components/Sidebar/NewCollectionOption.svelte";
  import NewCollectionSelldayInput from "@/components/Sidebar/NewCollectionSelldayInput.svelte";
  import { fly } from "svelte/transition";

  export let isFilterSellday: boolean;
  export let filterSellday: {
    since: { year: string; month: string };
    until: { year: string; month: string };
  };

  let sinceYear = "";
  let sinceMonth = "";

  let untilYear = "";
  let untilMonth = "";

  $: {
    filterSellday = {
      since: { year: sinceYear, month: sinceMonth },
      until: { year: untilYear, month: untilMonth },
    };
  }
</script>

<div class="space-y-2">
  <NewCollectionOption
    bind:value={isFilterSellday}
    label="範囲内で発売された"
  />
  {#if isFilterSellday}
    <div class="pl-8 space-y-2" transition:fly={{ y: -40, duration: 150 }}>
      <NewCollectionSelldayInput
        label="絞り込み開始"
        bind:year={sinceYear}
        bind:month={sinceMonth}
      />
      <NewCollectionSelldayInput
        label="絞り込み終了"
        bind:year={untilYear}
        bind:month={untilMonth}
      />
    </div>
  {/if}
</div>
