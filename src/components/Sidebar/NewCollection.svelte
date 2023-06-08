<script lang="ts">
  import NewCollectionBrandnames from "@/components/Sidebar/NewCollectionBrandnames.svelte";
  import NewCollectionOption from "@/components/Sidebar/NewCollectionOption.svelte";
  import NewCollectionSellday from "@/components/Sidebar/NewCollectionSellday.svelte";
  import Input from "@/components/UI/Input.svelte";
  import Modal from "@/components/UI/Modal.svelte";
  import {
    commandAddCollectionElementsByOption,
    commandCreateNewCollection,
  } from "@/lib/command";
  import { collections } from "@/store/collections";

  export let isOpen: boolean;
  let name = "";
  let isNukige = false;
  let isNotNukige = false;
  let isExistPath = false;
  let isFilterBrandname = false;
  let isFilterSellday = false;

  let filterBrandnames = [""];
  let filterSellday = {
    since: undefined,
    until: undefined,
  };

  $: confirmDisabled =
    isFilterSellday && (!filterSellday.since || !filterSellday.until);

  const createNewCollection = async () => {
    const newCollection = await commandCreateNewCollection(name);
    console.log("end create new collection");
    await collections.init();
    console.log("end collections init");
    await commandAddCollectionElementsByOption(
      newCollection.id,
      isNukige,
      isNotNukige,
      isFilterBrandname ? filterBrandnames : null,
      isFilterSellday
        ? [filterSellday.since ?? "", filterSellday.until ?? ""]
        : null
    );
    name = "";
    isOpen = false;
  };
</script>

<Modal
  bind:isOpen
  title="Create new collection"
  confirmText="Create"
  fullmodal
  {confirmDisabled}
  on:confirm={createNewCollection}
>
  <div class="space-y-4">
    <Input bind:value={name} label="Name" />
    <div class="space-y-2">
      <div class="text-(text-primary body) font-medium">
        作成時に初期要素を追加する
      </div>
      <div class="text-text-tertiary pl-2 space-y-1">
        <div class="text-(text-primary body2)">
          追加されるのは有効にした項目の全てを満たすゲームです。一部の項目のみを満たすゲームは追加されません。
        </div>
        <NewCollectionOption bind:value={isNukige} label="抜きゲー" />
        <NewCollectionOption bind:value={isNotNukige} label="非抜きゲー" />
        <NewCollectionOption
          bind:value={isExistPath}
          label="ゲームの実行ファイルが存在する"
        />
        <NewCollectionBrandnames bind:filterBrandnames bind:isFilterBrandname />
        <NewCollectionSellday bind:filterSellday bind:isFilterSellday />
      </div>
    </div>
  </div>
</Modal>
