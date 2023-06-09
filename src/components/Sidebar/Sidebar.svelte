<script lang="ts">
  import Logo from "/logo.png";
  import Icon from "/icon.png";
  import type { CollectionElementsWithLabel } from "@/lib/types";
  import { onMount } from "svelte";
  import SearchInput from "@/components/Sidebar/SearchInput.svelte";
  import CollectionElements from "@/components/Sidebar/CollectionElements.svelte";
  import { writable } from "svelte/store";
  import { sidebarCollectionElements } from "@/store/sidebarCollectionElements";
  import { createWritable } from "@/lib/utils";
  import { link } from "svelte-spa-router";
  import { filterAndSort, type SortOrder } from "@/components/Sidebar/sort";
  import { type Option, collectionElementsToOptions } from "@/lib/trieFilter";
  import { useFilter } from "@/lib/filter";
  import { showSidebar } from "@/store/showSidebar";
  import ButtonBase from "@/components/UI/ButtonBase.svelte";
  import Button from "@/components/UI/Button.svelte";
  import Search from "@/components/Sidebar/Search.svelte";

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
  class="min-h-0 w-80 grid-(~ rows-[min-content_min-content_min-content_1fr]) border-(r-1px solid border-primary)"
>
  <div class="w-full flex items-center px-2 pt-4">
    <a href="/" use:link>
      <div class="flex items-center gap-2 w-full">
        <img src={Icon} alt="launcherg icon" class="h-8" />
        <img src={Logo} alt="launcherg logo" class="h-7" />
      </div>
    </a>
    <ButtonBase
      on:click={() => showSidebar.set(false)}
      appendClass="ml-auto border-0px p-1 bg-transparent"
      tooltip={{
        content: "サイドバーを閉じる",
        placement: "bottom",
        theme: "default",
        delay: 1000,
      }}
    >
      <div
        class="i-material-symbols-left-panel-close-outline w-6 h-6 color-text-primary"
      />
    </ButtonBase>
  </div>
  <div class="mt-4 w-full px-2 flex items-center">
    <div class="text-(text-primary body) font-bold pl-2">登録したゲーム</div>
    <Button
      text="Add"
      leftIcon="i-material-symbols-computer-outline-rounded"
      variant="success"
      appendClass="ml-auto"
    />
  </div>
  <div class="w-full mt-2 px-2">
    <Search bind:query={$query} bind:order={$order} />
  </div>
  <div class="mt-1 min-h-0">
    {#key displayCollectionElements}
      <CollectionElements collectionElement={displayCollectionElements} />
    {/key}
  </div>
</div>
