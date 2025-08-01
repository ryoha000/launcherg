<script lang='ts'>
  import ATab from '@/components/Tab/ATab.svelte'
  import ScrollableHorizontal from '@/components/UI/ScrollableHorizontal.svelte'
  import { selected, tabs } from '@/store/tabs'
</script>

<ScrollableHorizontal>
  <div class='grid grid-(cols-[min-content_1fr]) items-center'>
    <div class='h-10 flex items-center'>
      {#each $tabs as tab, i (tab.id)}
        <ATab {tab} selected={$selected === i} />
      {/each}
    </div>
    <div
      class='h-full w-full border-(b-1px border-primary solid) bg-bg-disabled'
    ></div>
  </div>
</ScrollableHorizontal>
