<script lang='ts'>
  import { route } from '@mateothegreat/svelte5-router'
  import ButtonBase from '@/components/UI/ButtonBase.svelte'
  import { showSidebar } from '@/store/showSidebar'
  import Icon from '/icon.png'
</script>

<div class='w-full flex items-center px-2 pt-4'>
  <a href='/' use:route>
    <div class='flex items-center gap-2 w-full'>
      <img src={Icon} alt='launcherg icon' class='h-8' />
      <div class='font-logo text-(h3 text-primary)'>Launcherg</div>
    </div>
  </a>
  <ButtonBase
    onclick={() => showSidebar.set(false)}
    appendClass='ml-auto border-0px p-1 bg-transparent'
    tooltip={{
      content: 'サイドバーを閉じる',
      placement: 'bottom',
      theme: 'default',
      delay: 1000,
    }}
  >
    <div
      class='i-material-symbols-left-panel-close-outline w-6 h-6 color-text-primary'
    ></div>
  </ButtonBase>
</div>
