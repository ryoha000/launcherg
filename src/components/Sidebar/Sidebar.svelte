<script lang="ts">
  import type { CollectionElementsWithLabel } from "@/lib/types";
  import { onMount } from "svelte";
  import CollectionElements from "@/components/Sidebar/CollectionElements.svelte";
  import { sidebarCollectionElements } from "@/store/sidebarCollectionElements";
  import { createWritable, localStorageWritable } from "@/lib/utils";
  import { type SortOrder } from "@/components/Sidebar/sort";
  import { type Option, collectionElementsToOptions } from "@/lib/trieFilter";
  import { useFilter } from "@/lib/filter";
  import Search from "@/components/Sidebar/Search.svelte";
  import Header from "@/components/Sidebar/Header.svelte";
  import { showSidebar } from "@/store/showSidebar";
  import MinimalSidebar from "@/components/Sidebar/MinimalSidebar.svelte";
  import { fly } from "svelte/transition";
  import SubHeader from "@/components/Sidebar/SubHeader.svelte";
  import { searchAttributes } from "@/components/Sidebar/searchAttributes";
  import { search } from "@/components/Sidebar/search";

  onMount(async () => {
    await sidebarCollectionElements.refetch();
  });

  const [elementOptions, getElementOptions] = createWritable<Option<number>[]>(
    []
  );
  sidebarCollectionElements.subscribe((v) =>
    elementOptions.set(collectionElementsToOptions(v))
  );

  const { query, filtered } = useFilter(elementOptions, getElementOptions);
  let order = localStorageWritable<SortOrder>("sort-order", "gamename-asc");
  const { attributes, toggleEnable } = searchAttributes();

  const shown = sidebarCollectionElements.shown;
  filtered.subscribe(() => shown.set(search($filtered, $attributes, $order)));
  attributes.subscribe(() => shown.set(search($filtered, $attributes, $order)));
  order.subscribe(() => shown.set(search($filtered, $attributes, $order)));

  sidebarCollectionElements.subscribe(() => {
    shown.set(search($filtered, $attributes, $order));
  });
</script>

<div
  class="min-h-0 relative border-(r-1px solid border-primary) transition-all"
  class:w-80={$showSidebar}
  class:w-12={!$showSidebar}
>
  {#if $showSidebar}
    <div class="absolute inset-0" transition:fly={{ x: -40, duration: 150 }}>
      <div
        class="min-h-0 relative w-full h-full grid-(~ rows-[min-content_min-content_min-content_1fr])"
      >
        <Header />
        <SubHeader />
        <div class="w-full mt-2 px-2">
          <Search
            bind:query={$query}
            bind:order={$order}
            attributes={$attributes}
            on:toggleAttributeEnabled={(e) => toggleEnable(e.detail.key)}
          />
        </div>
        <div class="mt-1 min-h-0">
          <CollectionElements collectionElement={$shown} />
        </div>
      </div>
    </div>
  {:else}
    <div class="absolute inset-0" transition:fly={{ x: 40, duration: 150 }}>
      <MinimalSidebar />
    </div>
  {/if}
</div>
