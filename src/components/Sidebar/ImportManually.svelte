<script lang="ts">
  import Input from "@/components/UI/Input.svelte";
  import InputPath from "@/components/UI/InputPath.svelte";
  import Modal from "@/components/UI/Modal.svelte";
  import {
    commandGetExePathByLnk,
    commandGetGameCandidates,
  } from "@/lib/command";
  import { scrapeSql } from "@/lib/scrapeSql";
  import { showErrorToast } from "@/lib/toast";
  import { createEventDispatcher } from "svelte";

  export let isOpen: boolean;
  export let withInputPath = true;

  let idInput = "";
  let path = "";

  const dispather = createEventDispatcher<{
    confirm: { id: number; gamename: string; path: string };
  }>();

  let candidates: [number, string][] = [];
  $: {
    (async () => {
      if (!path) {
        candidates = [];
        return;
      }
      candidates = await commandGetGameCandidates(path);
    })();
  }

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

    if (path.toLowerCase().endsWith("lnk")) {
      const exePath = await commandGetExePathByLnk(path);
      dispather("confirm", { id, gamename, path: exePath });
    } else {
      dispather("confirm", { id, gamename, path });
    }
  };
  const clickCandidate = (id: number) => {
    idInput = `${id}`;
  };
</script>

<Modal
  {isOpen}
  on:close={() => (isOpen = false)}
  title="Manually import game"
  confirmText="Import"
  confirmDisabled={!idInput || (!path && withInputPath)}
  on:confirm={confirm}
>
  <div class="space-y-4">
    {#if withInputPath}
      <InputPath
        {path}
        label="EXEファイル or ショットカットファイル のパス"
        placeholder="C:\game\Monkeys!!\Monkeys!!.exe"
        on:update={(e) => (path = e.detail.value)}
      />
    {/if}
    <div class="space-y-2">
      <Input
        bind:value={idInput}
        label="ErogameScape ID or URL"
        placeholder="17909 or https://erogamescape.dyndns.org/~ap2/ero/toukei_kaiseki/game.php?game=17909"
        on:update={(e) => (idInput = e.detail.value)}
      />
      {#if candidates.length !== 0}
        <div class="space-y-1 pl-2">
          <h4 class="text-(text-primary body) font-medium mb-1">候補</h4>
          <div class="w-full">
            {#each candidates as [id, gamename] (id)}
              <button
                class={`rounded hover:bg-bg-button-hover transition-all px-4 block max-w-full ${
                  idInput === `${id}` ? "bg-bg-button" : "bg-inherit"
                }`}
                on:click={() => clickCandidate(id)}
              >
                <div
                  class="text-(text-secondary left body2) overflow-ellipsis whitespace-nowrap overflow-hidden w-full"
                >
                  {gamename}
                </div>
              </button>
            {/each}
          </div>
        </div>
      {/if}
    </div>
  </div>
</Modal>
