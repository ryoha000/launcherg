<script lang="ts">
  import Button from "@/components/UI/Button.svelte";
  import Input from "@/components/UI/Input.svelte";
  import Modal from "@/components/UI/Modal.svelte";
  import {
    commandAddElementsToCollection,
    commandUpsertCollectionElement,
  } from "@/lib/command";
  import { scrapeSql } from "@/lib/scrapeSql";
  import { showErrorToast } from "@/lib/toast";
  import type { Collection } from "@/lib/types";
  import { open } from "@tauri-apps/api/dialog";

  export let isOpen: boolean;
  export let collection: Collection | null;
  let idInput = "";
  let path = "";

  const getIdNumber = (value: string) => {
    {
      const idNumber = +value;
      if (!isNaN(idNumber)) {
        return idNumber;
      }
    }

    try {
      const url = new URL(value);
      const idString = url.searchParams.get("game");
      if (!idString) {
        return;
      }
      const idNumber = +idString;
      if (isNaN(idNumber)) {
        return;
      }
      return idNumber;
    } catch (e) {
      console.warn(e);
    }
  };
  const confirm = async () => {
    if (!collection) {
      return;
    }
    const id = getIdNumber(idInput);
    if (!id) {
      return showErrorToast("ErogameScape の id として解釈できませんでした");
    }

    const gamenames = await scrapeSql(
      `select gamename from gamelist where id = ${id};`,
      1
    );
    if (gamenames.length !== 1 || gamenames[0].length !== 1) {
      showErrorToast("指定したゲームの名前が取得できませんでした");
      return;
    }
    const gamename = gamenames[0][0];

    await commandUpsertCollectionElement(id, gamename, path);
    await commandAddElementsToCollection(collection.id, [id]);
  };
  const openDialog = async () => {
    const selected = await open({
      multiple: false,
      filters: [
        {
          name: "exe",
          extensions: ["exe", "EXE"],
        },
      ],
    });
    if (selected === null || Array.isArray(selected)) {
      return;
    }
    path = selected;
  };
</script>

{#if collection}
  <Modal
    bind:isOpen
    title="Manually import game"
    confirmText="Import"
    on:confirm={confirm}
  >
    <div class="space-y-4">
      <Input
        bind:value={idInput}
        label="ErogameScape ID or URL"
        placeholder="17909 or https://erogamescape.dyndns.org/~ap2/ero/toukei_kaiseki/game.php?game=17909"
        on:update={(e) => (idInput = e.detail.value)}
      />
      <div class="flex gap-2 items-end">
        <div class="flex-1">
          <Input
            bind:value={path}
            label="Exe file path"
            placeholder="C:\game\Monkeys!!\Monkeys!!.exe"
            on:update={(e) => (path = e.detail.value)}
          />
        </div>
        <Button
          leftIcon="i-material-symbols-folder-outline-rounded"
          on:click={openDialog}
        />
      </div>
    </div>
  </Modal>
{/if}
