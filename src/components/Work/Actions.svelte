<script lang="ts">
  import Button from "@/components/UI/Button.svelte";
  import PlayButton from "@/components/Work/PlayButton.svelte";
  import { push } from "svelte-spa-router";
  import { commandGetPlayTomeMinutes, commandPlayGame } from "@/lib/command";
  import { showErrorToast } from "@/lib/toast";

  export let name: string;
  export let id: number;

  const play = async () => {
    try {
      await commandPlayGame(id, false);
    } catch (e) {
      showErrorToast(e as string);
    }
  };

  $: playTimePromise = commandGetPlayTomeMinutes(id);
</script>

<div class="flex items-center gap-4 flex-wrap w-full min-w-0">
  <PlayButton on:play={play} />
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
