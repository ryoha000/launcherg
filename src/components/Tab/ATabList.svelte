<script lang='ts'>
  import ATab from '@/components/Tab/ATab.svelte'
  import ScrollableHorizontal from '@/components/UI/ScrollableHorizontal.svelte'
  import { selected, tabs } from '@/store/tabs'
</script>

<ScrollableHorizontal>
  <div class='grid-(~ cols-[min-content_1fr]) items-center'>
    <div class='flex items-center h-10'>
      {#each $tabs as tab, i (tab.id)}
        <ATab {tab} selected={$selected === i} />
      {/each}
    </div>
    <div
      class='w-full h-full bg-bg-disabled border-(b-1px solid border-primary)'
    ></div>
  </div>
</ScrollableHorizontal>
