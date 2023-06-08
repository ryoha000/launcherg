<script lang="ts">
  import Logo from "/logo.png";
  import Icon from "/icon.png";
  import { collections } from "@/store/collections";
  import type {
    Collection,
    CollectionElement,
    CollectionElementsWithLabel,
  } from "@/lib/types";
  import { onMount } from "svelte";
  import SearchInput from "@/components/Sidebar/SearchInput.svelte";
  import CollectionSelect from "@/components/Sidebar/CollectionSelect.svelte";
  import CollectionElements from "@/components/Sidebar/CollectionElements.svelte";
  import { writable } from "svelte/store";
  import { sidebarCollectionElements } from "@/store/sidebarCollectionElements";
  import { createWritable } from "@/lib/utils";
  import { link } from "svelte-spa-router";
  import { filterAndSort, type SortOrder } from "@/components/Sidebar/sort";
  import {
    useTrieFilter,
    type Option,
    collectionElementsToOptions,
  } from "@/lib/trieFilter";

  onMount(() => collections.init());

  const selectedColection = writable<Collection | null>(null);
  selectedColection.subscribe(async (v) => {
    if (v) {
      sidebarCollectionElements.init(v.id);
    }
  });

  const [elementOptions, getElementOptions] = createWritable<Option<number>[]>(
    []
  );
  sidebarCollectionElements.subscribe((v) =>
    elementOptions.set(collectionElementsToOptions(v))
  );

  const { query, filtered } = useTrieFilter(elementOptions, getElementOptions);
  let order = writable<SortOrder>("gamename-asc");

  let displayCollectionElements: CollectionElementsWithLabel[] = [];

  collections.subscribe(
    () => (displayCollectionElements = filterAndSort($filtered, $order))
  );
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
  class="min-h-0 max-w-full grid-(~ rows-[min-content_min-content_min-content_1fr]) border-(r-1px solid border-primary)"
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
