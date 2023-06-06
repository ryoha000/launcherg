<script lang="ts">
  import OptionButton from "@/components/UI/OptionButton.svelte";
  import { createEventDispatcher } from "svelte";

  export let showDelete: boolean;

  const dispatcher = createEventDispatcher<{
    close: {};
    changeName: {};
    delete: {};
  }>();
</script>

<div>
  <div
    class="font-bold text-(text-primary body3) p-(l-4 r-2 y-2) flex items-center gap-4"
  >
    <div class="whitespace-nowrap">Select change option</div>
    <button
      on:click={() => dispatcher("close")}
      class="ml-auto w-5 h-5 i-iconoir-cancel color-text-tertiary hover:color-text-primary transition-all"
    />
  </div>
  <OptionButton text="名前の変更" on:click={() => dispatcher("changeName")} />
  {#if showDelete}
    <OptionButton text="削除" on:click={() => dispatcher("delete")} />
  {/if}
</div>
