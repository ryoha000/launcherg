<script lang="ts">
  import NewCollectionBrandname from "@/components/Sidebar/NewCollectionBrandname.svelte";
  import NewCollectionOption from "@/components/Sidebar/NewCollectionOption.svelte";
  import Button from "@/components/UI/Button.svelte";
  import ButtonCancel from "@/components/UI/ButtonCancel.svelte";
  import { rand } from "@/lib/utils";
  import { writable } from "svelte/store";
  import { fly } from "svelte/transition";

  export let isFilterBrandname: boolean;
  export let filterBrandnames: string[];

  let _filterBrandnames = writable([{ id: rand(), value: "" }]);
  _filterBrandnames.subscribe(
    (v) => (filterBrandnames = v.map((v) => v.value))
  );

  const brandnames = [
    "ゆずソフト",
    "Purple Software",
    "hoy",
    "adfah",
    "hiuagehrs",
    "hgaiugh",
  ];

  const onChangeFilterBrandname = (index: number, value: string) => {
    _filterBrandnames.update((v) => {
      v[index].value = value;
      return v;
    });
  };
  const removeFilterBrandname = (index: number) => {
    _filterBrandnames.update((v) => {
      return [...v.slice(0, index), ...v.slice(index + 1)];
    });
  };
  const addFilterBrandname = async () => {
    if (
      $_filterBrandnames.length > 0 &&
      $_filterBrandnames[$_filterBrandnames.length - 1].value === ""
    ) {
      return;
    }
    _filterBrandnames.update((v) => {
      return [...v, { id: rand(), value: "" }];
    });
  };
</script>

<div class="space-y-2">
  <NewCollectionOption
    bind:value={isFilterBrandname}
    label="ブランド名が一致する"
  />
  {#if isFilterBrandname}
    <div class="space-y-4 pl-8 max-w-120">
      <div class="space-y-1" transition:fly={{ y: -40, duration: 150 }}>
        {#each $_filterBrandnames as brandname, i (brandname.id)}
          <div
            transition:fly={{ y: -40, duration: 150 }}
            class="flex gap-4 items-center"
          >
            <div class="flex-1">
              <NewCollectionBrandname
                {brandnames}
                value={brandname.value}
                on:change={(e) => onChangeFilterBrandname(i, e.detail.value)}
              />
            </div>
            <ButtonCancel on:click={() => removeFilterBrandname(i)} />
          </div>
        {/each}
      </div>
      <Button
        appendClass="m-auto mt-4"
        leftIcon="i-iconoir-plus"
        text="Add folder path"
        on:click={addFilterBrandname}
      />
    </div>
  {/if}
</div>
