<script lang="ts">
  import ZappingGameItem from "@/components/Home/ZappingGameItem.svelte";
  import MasonryItem from "@/components/UI/MasonryItem.svelte";
  import MasonryWrapper from "@/components/UI/MasonryWrapper.svelte";
  import { sidebarCollectionElements } from "@/store/sidebarCollectionElements";

  const shown = sidebarCollectionElements.shown;
  $: flattenShown = $shown.flatMap((v) => v.elements);
</script>

<div class="space-y-2">
  <h3 class="text-(text-primary h3) font-medium">登録したゲーム</h3>
  <MasonryWrapper let:rowGap let:rowHeight>
    <svelte:fragment>
      {#each flattenShown as ele}
        <MasonryItem {rowGap} {rowHeight}>
          <ZappingGameItem collectionElement={ele} />
        </MasonryItem>
      {/each}
    </svelte:fragment>
  </MasonryWrapper>
</div>
