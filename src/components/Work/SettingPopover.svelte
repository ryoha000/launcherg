<script lang="ts">
  import OptionButton from "@/components/UI/OptionButton.svelte";
  import { createEventDispatcher } from "svelte";

  const dispatcher = createEventDispatcher<{
    selectChange: {};
    selectDelete: {};
    selectOpen: {};
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
    text="違うゲームに変更する"
    on:click={() => {
      dispatcher("close");
      dispatcher("selectChange");
    }}
  />
  <OptionButton
    text="登録したゲームから削除する"
    on:click={() => {
      dispatcher("close");
      dispatcher("selectDelete");
    }}
  />
  <OptionButton
    text="ゲームのフォルダーを開く"
    on:click={() => {
      dispatcher("close");
      dispatcher("selectOpen");
    }}
  />
</div>
