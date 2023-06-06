<script lang="ts">
  import Button from "@/components/UI/Button.svelte";
  import PlayButton from "@/components/Work/PlayButton.svelte";
  import { push } from "svelte-spa-router";

  export let name: string;
  export let id: number;
</script>

<div class="flex items-center gap-4 flex-wrap w-full min-w-0">
  <PlayButton />
  <Button
    variant="success"
    leftIcon="color-text-white i-material-symbols-drive-file-rename-outline"
    text="Memo"
    on:click={() => push(`/memos/${id}?gamename=${name}`)}
  />
  <div class="flex ml-auto items-end gap-2 h-8 min-w-0">
    <div class="text-(text-primary body2) whitespace-nowrap">Time</div>
    <div class="text-(text-primary body)">03:22</div>
  </div>
</div>
