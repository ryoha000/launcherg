<script lang="ts">
  import { showSidebar } from "@/store/showSidebar";
  import ButtonBase from "@/components/UI/ButtonBase.svelte";
</script>

<div
  class="p-(x-2 y-4) border-(border-primary r-1px solid) flex flex-col items-center gap-4"
>
  <ButtonBase
    on:click={() => showSidebar.set(true)}
    appendClass="ml-auto border-0px p-1 bg-transparent"
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
