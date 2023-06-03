<script lang="ts">
  import { Link } from "svelte-routing";
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

  onMount(() => collections.init());

  const selectedColection = writable<Collection | null>(null);
  selectedColection.subscribe(async (v) => {
    if (v) {
      sidebarCollectionElements.init(v.id);
    }
  });

  let displayCollectionElements: CollectionElement[] = [];
  const filterAndSort = () => {
    displayCollectionElements = sidebarCollectionElements
      .value()
      .filter((v) => v.gamename.includes(getQuery()))
      .sort((a, b) => {
        if (getOrder() === "gamename") {
          return a.gamename.localeCompare(b.gamename, "ja");
        }
        return 1;
      });
  };

  let [query, getQuery] = createWritable("");
  let [order, getOrder] = createWritable<"gamename">("gamename");

  query.subscribe(() => filterAndSort());
  order.subscribe(() => filterAndSort());

  sidebarCollectionElements.subscribe(() => {
    query.set("");
    order.set("gamename");
    filterAndSort();
  });
</script>

<div
  class="min-h-0 px-2 grid-(~ rows-[min-content_min-content_min-content_1fr])"
>
  <Link to="/">
    <div class="flex items-center gap-2 p-(x-2 t-4)">
      <img src={Icon} alt="launcherg icon" class="h-8" />
      <img src={Logo} alt="launcherg logo" class="h-7" />
    </div>
  </Link>
  <div class="mt-4 w-full">
    <CollectionSelect
      collections={$collections}
      bind:value={$selectedColection}
    />
  </div>
  <div class="w-full mt-2">
    <SearchInput bind:value={$query} placeholder="Filter by title" />
  </div>
  <div class="mt-1 min-h-0">
    {#key $sidebarCollectionElements}
      <CollectionElements collectionElement={displayCollectionElements} />
    {/key}
  </div>
</div>
