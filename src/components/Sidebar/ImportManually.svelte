<script lang="ts">
  import Button from "@/components/UI/Button.svelte";
  import Input from "@/components/UI/Input.svelte";
  import Modal from "@/components/UI/Modal.svelte";
  import { scrapeSql } from "@/lib/scrapeSql";
  import { showErrorToast } from "@/lib/toast";
  import { open } from "@tauri-apps/api/dialog";
  import { createEventDispatcher } from "svelte";

  export let isOpen: boolean;
  export let withInputPath = true;
  let idInput = "";
  let path = "";

  const dispather = createEventDispatcher<{
    add: { id: number; gamename: string; path: string };
  }>();

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

    dispather("add", { id, gamename, path });
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

<Modal
  {isOpen}
  on:close={() => (isOpen = false)}
  title="Manually import game"
  confirmText="Import"
  confirmDisabled={!idInput || !path}
  on:confirm={confirm}
>
  <div class="space-y-4">
    <Input
      bind:value={idInput}
      label="ErogameScape ID or URL"
      placeholder="17909 or https://erogamescape.dyndns.org/~ap2/ero/toukei_kaiseki/game.php?game=17909"
      on:update={(e) => (idInput = e.detail.value)}
    />
    {#if withInputPath}
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
    {/if}
  </div>
</Modal>
