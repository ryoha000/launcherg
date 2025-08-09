<script lang='ts'>
  import ButtonBase from '@/components/UI/ButtonBase.svelte'
  import { showSidebar } from '@/store/showSidebar'
</script>

<div class='h-full w-full flex flex-col items-center gap-4 py-4'>
  <ButtonBase
    onclick={() => showSidebar.set(true)}
    appendClass='border-0px p-1 bg-transparent'
    tooltip={{
      content: 'サイドバーを開く',
      placement: 'bottom',
      theme: 'default',
      delay: 1000,
    }}
  >
    <div
      class='i-material-symbols-left-panel-close-outline h-6 w-6 rotate-180 color-text-primary'
    ></div>
  </ButtonBase>
</div>
