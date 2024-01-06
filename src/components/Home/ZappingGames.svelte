<script lang="ts">
  import ZappingGameItem from "@/components/Home/ZappingGameItem.svelte";
  import ZappingGamesContainer from "@/components/Home/ZappingGamesContainer.svelte";
  import MasonryItem from "@/components/UI/MasonryItem.svelte";
  import MasonryWrapper from "@/components/UI/MasonryWrapper.svelte";
  import { useMasonry } from "@/components/UI/masonry";
  import { sidebarCollectionElements } from "@/store/sidebarCollectionElements";

  const shown = sidebarCollectionElements.shown;
  $: flattenShown = $shown.flatMap((v) => v.elements);
</script>

<ZappingGamesContainer elements={flattenShown} />
