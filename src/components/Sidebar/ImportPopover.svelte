<script lang="ts">
  import OptionButton from "@/components/UI/OptionButton.svelte";
  import { createEventDispatcher } from "svelte";

  const dispatcher = createEventDispatcher<{
    close: {};
    startAuto: {};
    startManual: {};
  }>();
</script>

<div>
  <div
    class="font-bold text-(text-primary body3) p-(l-4 r-2 y-2) flex items-center gap-4"
  >
    <div class="whitespace-nowrap">Select adding game option</div>
    <button
      on:click={() => dispatcher("close")}
      class="ml-auto w-5 h-5 i-iconoir-cancel color-text-tertiary hover:color-text-primary transition-all"
    />
  </div>
  <OptionButton
    text="自動でフォルダから追加"
    on:click={() => {
      dispatcher("close");
      dispatcher("startAuto");
    }}
  />
  <OptionButton
    text="手動で追加"
    on:click={() => {
      dispatcher("close");
      dispatcher("startManual");
    }}
  />
</div>
