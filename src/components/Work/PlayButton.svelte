<script lang="ts">
  import ImportManually from "@/components/Sidebar/ImportManually.svelte";
  import APopover from "@/components/UI/APopover.svelte";
  import Button from "@/components/UI/Button.svelte";
  import ButtonBase from "@/components/UI/ButtonBase.svelte";
  import ChangeGamePopover from "@/components/Work/ChangeGamePopover.svelte";
  import {
    commandDeleteCollectionElement,
    commandUpsertCollectionElement,
  } from "@/lib/command";
  import { sidebarCollectionElements } from "@/store/sidebarCollectionElements";
  import { deleteTab, selected, tabs } from "@/store/tabs";
  import { createEventDispatcher } from "svelte";

  export let id: number;
  export let path: string;

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
    await sidebarCollectionElements.refetch();
    deleteTab($tabs[$selected].id);
  };

  let isOpenChangeId = false;
</script>

<div class="flex items-center min-w-0">
  <Button
    appendClass="rounded-r-0"
    leftIcon="i-material-symbols-power-rounded"
    text="Play"
    variant="success"
    on:click={() => dispather("play", { isAdmin: undefined })}
  />
  <APopover let:open let:close>
    <ButtonBase
      appendClass="h-8 w-8 flex items-center justify-center rounded-l-0"
      tooltip={{
        content: "このゲームの設定",
        placement: "bottom",
        theme: "default",
        delay: 1000,
      }}
      variant="success"
      slot="button"
    >
      <div
        class="color-text-white w-5 h-5 i-material-symbols-arrow-drop-down"
        class:rotate-180={open}
      />
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
  <ImportManually
    bind:isOpen={isOpenChangeId}
    withInputPath={false}
    on:confirm={(e) => onChangeGame(e.detail.id, e.detail.gamename, path)}
  />
</div>
