<script lang='ts'>
  import OptionButton from '@/components/UI/OptionButton.svelte'

  const { close, play, playAdmin }: {
    close: () => void
    play: () => void
    playAdmin: () => void
  } = $props()
</script>

<div>
  <div
    class='font-bold text-(text-primary body3) p-(l-4 r-2 y-2) flex items-center gap-4'
  >
    <div class='whitespace-nowrap'>Select game option</div>
    <button
      onclick={close}
      class='ml-auto w-5 h-5 i-iconoir-cancel color-text-tertiary hover:color-text-primary transition-all'
      aria-label='Close options'
    ></button>
  </div>
  <OptionButton
    text='管理者権限で起動'
    onclick={() => {
      close()
      playAdmin()
    }}
  />
  <OptionButton
    text='現在のユーザーで起動'
    onclick={() => {
      close()
      play()
    }}
  />
</div>
