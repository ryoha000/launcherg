<script lang="ts">
  import Button from "@/components/UI/Button.svelte";
  import PlayButton from "@/components/Work/PlayButton.svelte";
  import { push } from "svelte-spa-router";
  import {
    commandDeleteCollectionElement,
    commandGetCollectionElement,
    commandGetPlayTomeMinutes,
    commandOpenFolder,
    commandPlayGame,
    commandUpdateElementLike,
    commandUpsertCollectionElement,
  } from "@/lib/command";
  import { showErrorToast } from "@/lib/toast";
  import { localStorageWritable } from "@/lib/utils";
  import ButtonIcon from "@/components/UI/ButtonIcon.svelte";
  import ButtonCancel from "@/components/UI/ButtonCancel.svelte";
  import { sidebarCollectionElements } from "@/store/sidebarCollectionElements";
  import APopover from "@/components/UI/APopover.svelte";
  import SettingPopover from "@/components/Work/SettingPopover.svelte";
  import ImportManually from "@/components/Sidebar/ImportManually.svelte";
  import { deleteTab, tabs, selected } from "@/store/tabs";
  import DeleteElement from "@/components/Work/DeleteElement.svelte";
  import type { AllGameCacheOne } from "@/lib/types";
  import OtherInformation from "@/components/Work/OtherInformation.svelte";
  import { registerCollectionElementDetails } from "@/lib/registerCollectionElementDetails";

  export let name: string;
  export let id: number;

  const isAdminRecord = localStorageWritable<Record<number, boolean>>(
    "play-admin-cache",
    {}
  );

  const play = async (isAdmin: boolean | undefined) => {
    if (isAdmin !== undefined) {
      isAdminRecord.update((v) => {
        v[id] = isAdmin;
        return v;
      });
    }
    let _isAdmin: boolean = isAdmin ?? false;
    if (isAdmin === undefined) {
      const cache = $isAdminRecord[id];
      if (cache) {
        _isAdmin = cache;
      }
    }
    try {
      await commandPlayGame(id, _isAdmin);
    } catch (e) {
      showErrorToast(e as string);
    }
  };

  let isLike = false;

  const toggleLike = async () => {
    await commandUpdateElementLike(id, !isLike);
    isLike = !isLike;
    sidebarCollectionElements.updateLike(id, isLike);
  };

  $: playTimePromise = commandGetPlayTomeMinutes(id);
  $: elementPromise = (async () => {
    const element = await commandGetCollectionElement(id);
    isLike = !!element.likeAt;
    return element;
  })();

  let isOpenImportManually = false;
  const onChangeGame = async (arg: {
    exePath: string | null;
    lnkPath: string | null;
    gameCache: AllGameCacheOne;
  }) => {
    const isChangedGameId = id !== arg.gameCache.id;
    if (isChangedGameId) {
      await commandDeleteCollectionElement(id);
    }
    await commandUpsertCollectionElement(arg);
    await registerCollectionElementDetails();
    await sidebarCollectionElements.refetch();
    if (isChangedGameId) {
      deleteTab($tabs[$selected].id);
    }
    isOpenImportManually = false;
  };

  let isOpenDelete = false;
  let isOpenOtherInformation = false;
</script>

{#await elementPromise then element}
  <div class="flex items-center gap-4 flex-wrap w-full min-w-0">
    <PlayButton on:play={(e) => play(e.detail.isAdmin)} />
    <Button
      leftIcon="i-material-symbols-drive-file-rename-outline"
      text="Memo"
      on:click={() => push(`/memos/${id}?gamename=${name}`)}
    />
    <!-- <div class="flex items-end gap-2 h-8 min-w-0">
      <div class="text-(text-primary body2) whitespace-nowrap">Time</div>
      {#await playTimePromise then playTime}
        <div class="text-(text-primary body)">
          {`${`${Math.floor(playTime / 60)}`.padStart(2, "0")}:${`${Math.floor(
            playTime % 60
          )}`.padStart(2, "0")}`}
        </div>
      {/await}
    </div> -->
    <div class="flex items-center gap-2 ml-auto">
      <ButtonCancel
        icon={isLike
          ? "i-material-symbols-favorite-rounded"
          : "i-material-symbols-favorite-outline-rounded"}
        on:click={toggleLike}
      />
      <APopover let:close panelClass="right-0">
        <ButtonIcon icon="i-material-symbols-menu-rounded" slot="button" />
        <SettingPopover
          on:close={() => close(null)}
          on:selectChange={() => (isOpenImportManually = true)}
          on:selectDelete={() => (isOpenDelete = true)}
          on:selectOpen={() =>
            commandOpenFolder(element.exePath ?? element.lnkPath)}
          on:selectOtherInfomation={() => (isOpenOtherInformation = true)}
        />
      </APopover>
    </div>
  </div>
  <ImportManually
    bind:isOpen={isOpenImportManually}
    idInput={`${id}`}
    path={element.exePath ?? element.lnkPath}
    on:confirm={(e) => onChangeGame(e.detail)}
    on:cancel={() => (isOpenImportManually = false)}
  />
  <DeleteElement bind:isOpen={isOpenDelete} {element} />
  <OtherInformation bind:isOpen={isOpenOtherInformation} {element} />
{/await}
