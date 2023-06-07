<script lang="ts">
  import Button from "@/components/UI/Button.svelte";
  import PlayButton from "@/components/Work/PlayButton.svelte";
  import { push } from "svelte-spa-router";
  import { commandGetPlayTomeMinutes, commandPlayGame } from "@/lib/command";
  import { showErrorToast } from "@/lib/toast";
  import { localStorageWritable } from "@/lib/utils";

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

  $: playTimePromise = commandGetPlayTomeMinutes(id);
</script>

<div class="flex items-center gap-4 flex-wrap w-full min-w-0">
  <PlayButton {id} on:play={(e) => play(e.detail.isAdmin)} />
  <Button
    variant="success"
    leftIcon="color-text-white i-material-symbols-drive-file-rename-outline"
    text="Memo"
    on:click={() => push(`/memos/${id}?gamename=${name}`)}
  />
  <div class="flex ml-auto items-end gap-2 h-8 min-w-0">
    <div class="text-(text-primary body2) whitespace-nowrap">Time</div>
    {#await playTimePromise then playTime}
      <div class="text-(text-primary body)">
        {`${`${Math.floor(playTime / 60)}`.padStart(2, "0")}:${`${Math.floor(
          playTime % 60
        )}`.padStart(2, "0")}`}
      </div>
    {/await}
  </div>
</div>
