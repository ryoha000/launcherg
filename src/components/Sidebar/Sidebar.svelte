<script lang="ts">
  import type { CollectionElementsWithLabel } from "@/lib/types";
  import { onMount } from "svelte";
  import CollectionElements from "@/components/Sidebar/CollectionElements.svelte";
  import { writable } from "svelte/store";
  import { sidebarCollectionElements } from "@/store/sidebarCollectionElements";
  import { createWritable } from "@/lib/utils";
  import { filterAndSort, type SortOrder } from "@/components/Sidebar/sort";
  import { type Option, collectionElementsToOptions } from "@/lib/trieFilter";
  import { useFilter } from "@/lib/filter";
  import Search from "@/components/Sidebar/Search.svelte";
  import Header from "@/components/Sidebar/Header.svelte";
  import { showSidebar } from "@/store/showSidebar";
  import MinimalSidebar from "@/components/Sidebar/MinimalSidebar.svelte";
  import { fly } from "svelte/transition";
  import SubHeader from "@/components/Sidebar/SubHeader.svelte";

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
  let order = writable<SortOrder>("gamename-asc");

  let displayCollectionElements: CollectionElementsWithLabel[] = [];

  filtered.subscribe(
    () => (displayCollectionElements = filterAndSort($filtered, $order))
  );
  order.subscribe(
    () => (displayCollectionElements = filterAndSort($filtered, $order))
  );

  sidebarCollectionElements.subscribe(() => {
    query.set("");
    displayCollectionElements = filterAndSort($filtered, $order);
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
          <Search bind:query={$query} bind:order={$order} />
        </div>
        <div class="mt-1 min-h-0">
          {#key displayCollectionElements}
            <CollectionElements collectionElement={displayCollectionElements} />
          {/key}
        </div>
      </div>
    </div>
  {:else}
    <div class="absolute inset-0" transition:fly={{ x: 40, duration: 150 }}>
      <MinimalSidebar />
    </div>
  {/if}
</div>
