<script lang="ts">
  import Logo from "/logo.png";
  import Icon from "/icon.png";
  import { collections } from "@/store/collections";
  import type { Collection, CollectionElement } from "@/lib/types";
  import { onMount } from "svelte";
  import SearchInput from "@/components/Sidebar/SearchInput.svelte";
  import CollectionSelect from "@/components/Sidebar/CollectionSelect.svelte";
  import CollectionElements from "@/components/Sidebar/CollectionElements.svelte";
  import { writable } from "svelte/store";
  import { sidebarCollectionElements } from "@/store/sidebarCollectionElements";
  import { createWritable } from "@/lib/utils";
  import { link } from "svelte-spa-router";
  import type { SortOrder } from "@/components/Sidebar/sort";

  onMount(() => collections.init());

  const selectedColection = writable<Collection | null>(null);
  selectedColection.subscribe(async (v) => {
    if (v) {
      sidebarCollectionElements.init(v.id);
    }
  });

  let displayCollectionElements: CollectionElement[] = [];
  const filterAndSort = () => {
    const filteredTemp = sidebarCollectionElements
      .value()
      .filter((v) => v.gamename.includes(getQuery()));

    const isGamename = getOrder().includes("gamename");
    const isAsc = getOrder().includes("asc");
    const multiplyer = isAsc ? 1 : -1;

    displayCollectionElements = [...filteredTemp].sort((a, b) => {
      if (isGamename) {
        return a.gamename.localeCompare(b.gamename, "ja") * multiplyer;
      }
      return 1;
    });
  };

  let [query, getQuery] = createWritable("");
  let [order, getOrder] = createWritable<SortOrder>("gamename-asc");

  collections.subscribe(() => filterAndSort());
  query.subscribe(() => filterAndSort());
  order.subscribe(() => filterAndSort());

  sidebarCollectionElements.subscribe(() => {
    query.set("");
    order.set("gamename-asc");
    filterAndSort();
  });
</script>

<div
  class="min-h-0 grid-(~ rows-[min-content_min-content_min-content_1fr]) border-(r-1px solid border-primary)"
>
  <a href="/" use:link>
    <div class="flex items-center gap-2 p-(x-2 t-4)">
      <img src={Icon} alt="launcherg icon" class="h-8" />
      <img src={Logo} alt="launcherg logo" class="h-7" />
    </div>
  </a>
  <div class="mt-4 w-full px-2">
    <CollectionSelect
      collections={$collections}
      bind:value={$selectedColection}
      bind:order={$order}
    />
  </div>
  <div class="w-full mt-2 px-2">
    <SearchInput bind:value={$query} placeholder="Filter by title" />
  </div>
  <div class="mt-1 min-h-0">
    {#if $selectedColection}
      {#key displayCollectionElements}
        <CollectionElements
          collection={$selectedColection}
          collectionElement={displayCollectionElements}
        />
      {/key}
    {/if}
  </div>
</div>
