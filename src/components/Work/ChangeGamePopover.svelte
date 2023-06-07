<script lang="ts">
  import OptionButton from "@/components/UI/OptionButton.svelte";
  import { createEventDispatcher } from "svelte";

  const dispatcher = createEventDispatcher<{
    playAdmin: {};
    play: {};
    changeGame: {};
    close: {};
  }>();
</script>

<div>
  <div
    class="font-bold text-(text-primary body3) p-(l-4 r-2 y-2) flex items-center gap-4"
  >
    <div class="whitespace-nowrap">Select game option</div>
    <button
      on:click={() => dispatcher("close")}
      class="ml-auto w-5 h-5 i-iconoir-cancel color-text-tertiary hover:color-text-primary transition-all"
    />
  </div>
  <OptionButton
    text="管理者権限で起動"
    on:click={() => dispatcher("playAdmin")}
  />
  <OptionButton
    text="現在のユーザーで起動"
    on:click={() => dispatcher("play")}
  />
  <OptionButton
    text="違うゲームに変更する"
    on:click={() => dispatcher("changeGame")}
  />
</div>
