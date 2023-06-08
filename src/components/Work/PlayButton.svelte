<script lang="ts">
  import AddGameManual from "@/components/Sidebar/AddGameManual.svelte";
  import APopover from "@/components/UI/APopover.svelte";
  import Button from "@/components/UI/Button.svelte";
  import ButtonBase from "@/components/UI/ButtonBase.svelte";
  import ChangeGamePopover from "@/components/Work/ChangeGamePopover.svelte";
  import {
    commandAddElementsToCollection,
    commandDeleteCollectionElement,
    commandGetCollectionElement,
    commandUpsertCollectionElement,
  } from "@/lib/command";
  import { sidebarCollectionElements } from "@/store/sidebarCollectionElements";
  import { deleteTab, selected, tabs } from "@/store/tabs";
  import { createEventDispatcher } from "svelte";

  export let id: number;

  $: collectionElementPromise = commandGetCollectionElement(id);

  const dispather = createEventDispatcher<{
    play: { isAdmin: boolean | undefined };
  }>();

  const onChangeGame = async (
    elementId: number,
    gamename: string,
    path: string
  ) => {
    await commandDeleteCollectionElement(id);
    await commandUpsertCollectionElement(elementId, gamename, path);
    await commandAddElementsToCollection(1, [elementId]);
    await sidebarCollectionElements.refetch();
    deleteTab($tabs[$selected].id);
  };

  let isOpenChangeId = false;
</script>

<div class="flex items-center min-w-0">
  <Button
    appendClass="w-24 h-8 flex items-center rounded-r-0 group hover:text-bg-success-hover transition-all"
    leftIcon="i-material-symbols-power-rounded group-hover:color-bg-success-hover transition-all"
    text="Play"
    on:click={() => dispather("play", { isAdmin: undefined })}
  />
  <APopover let:close>
    <ButtonBase
      appendClass="h-8 w-8 flex items-center justify-center rounded-l-0"
      tooltip={{
        content: "このゲームの設定",
        placement: "bottom",
        theme: "default",
        delay: 1000,
      }}
      slot="button"
    >
      <div class="color-ui-tertiary w-5 h-5 i-material-symbols-tune" />
    </ButtonBase>
    <ChangeGamePopover
      on:close={() => close(null)}
      on:play={() => {
        dispather("play", { isAdmin: false });
        close(null);
      }}
      on:playAdmin={() => {
        close(null);
        dispather("play", { isAdmin: true });
      }}
      on:changeGame={() => {
        close(null);
        isOpenChangeId = true;
      }}
    />
  </APopover>
</div>
{#await collectionElementPromise then element}
  <AddGameManual
    bind:isOpen={isOpenChangeId}
    withInputPath={false}
    on:add={(e) => onChangeGame(e.detail.id, e.detail.gamename, element.path)}
  />
  <div>{JSON.stringify(element)}</div>
{/await}
