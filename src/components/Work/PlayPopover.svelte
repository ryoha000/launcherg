<script lang='ts'>
  import { createEventDispatcher } from 'svelte'
  import OptionButton from '@/components/UI/OptionButton.svelte'

  const dispatcher = createEventDispatcher<{
    playAdmin: {}
    play: {}
    close: {}
  }>()
</script>

<div>
  <div
    class='font-bold text-(text-primary body3) p-(l-4 r-2 y-2) flex items-center gap-4'
  >
    <div class='whitespace-nowrap'>Select game option</div>
    <button
      onclick={() => dispatcher('close')}
      class='ml-auto w-5 h-5 i-iconoir-cancel color-text-tertiary hover:color-text-primary transition-all'
    ></button>
  </div>
  <OptionButton
    text='管理者権限で起動'
    on:click={() => {
      dispatcher('close')
      dispatcher('playAdmin')
    }}
  />
  <OptionButton
    text='現在のユーザーで起動'
    on:click={() => {
      dispatcher('close')
      dispatcher('play')
    }}
  />
</div>
