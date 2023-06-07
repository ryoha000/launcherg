<script lang="ts">
  import NewCollectionBrandnames from "@/components/Sidebar/NewCollectionBrandnames.svelte";
  import NewCollectionOption from "@/components/Sidebar/NewCollectionOption.svelte";
  import NewCollectionSellday from "@/components/Sidebar/NewCollectionSellday.svelte";
  import Input from "@/components/UI/Input.svelte";
  import Modal from "@/components/UI/Modal.svelte";
  import { commandCreateNewCollection } from "@/lib/command";
  import { collections } from "@/store/collections";

  export let isOpen: boolean;
  let name = "";
  let isNukige = false;
  let isNotNukige = false;
  let isExistPath = false;
  let isFilterBrandname = false;
  let isFilterSellday = false;

  const brandnames = [
    "ゆずソフト",
    "Purple Software",
    "hoy",
    "adfah",
    "hiuagehrs",
    "hgaiugh",
  ];
  let filterBrandnames = [""];
  let filterSellday = {
    since: { year: "", month: "" },
    until: { year: "", month: "" },
  };

  const createNewCollection = async () => {
    await commandCreateNewCollection(name);
    await collections.init();
    name = "";
    isOpen = false;
  };
</script>

<Modal
  bind:isOpen
  title="Create new collection"
  confirmText="Create"
  fullmodal
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
          有効にした項目のいずれかを満たすゲームは他の項目を満たさない場合でも追加されます
        </div>
        <NewCollectionOption bind:value={isNukige} label="抜きゲー" />
        <NewCollectionOption bind:value={isNotNukige} label="非抜きゲー" />
        <NewCollectionOption
          bind:value={isExistPath}
          label="ゲームの実行ファイルが存在する"
        />
        <NewCollectionBrandnames bind:filterBrandnames bind:isFilterBrandname />
        <NewCollectionSellday bind:filterSellday {isFilterSellday} />
      </div>
    </div>
  </div>
</Modal>
