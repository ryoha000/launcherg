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
  import { commandGetCollectionElements } from "@/lib/command";

  onMount(() => collections.init());

  let query = "";

  const selectedColection = writable<Collection | null>(null);
  let selectedCollectionElements: CollectionElement[] = [];
  selectedColection.subscribe(async (v) => {
    if (!v) {
      return;
    }
    selectedCollectionElements = await commandGetCollectionElements(v.id);
    console.log(selectedCollectionElements);
  });

  $: displayCollectionElements = selectedCollectionElements.filter((v) =>
    query ? v.gamename.toLowerCase().includes(query.toLowerCase()) : true
  );
</script>

<div class="p-(x-2 y-4)">
  <Link to="/">
    <div class="flex items-center gap-2 px-2">
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
    <SearchInput bind:value={query} placeholder="Filter by title" />
  </div>
  <div class="mt-1">
    {#key displayCollectionElements}
      <CollectionElements collectionElement={displayCollectionElements} />
    {/key}
  </div>
</div>
