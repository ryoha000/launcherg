<script lang="ts">
  import { showSidebar } from "@/store/showSidebar";
  import ButtonBase from "@/components/UI/ButtonBase.svelte";
</script>

<div class="flex flex-col h-full w-full items-center gap-4 py-4">
  <ButtonBase
    on:click={() => showSidebar.set(true)}
    appendClass="border-0px p-1 bg-transparent"
    tooltip={{
      content: "サイドバーを開く",
      placement: "bottom",
      theme: "default",
      delay: 1000,
    }}
  >
    <div
      class="i-material-symbols-left-panel-close-outline w-6 h-6 color-text-primary rotate-180"
    />
  </ButtonBase>
</div>
