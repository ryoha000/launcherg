<script lang='ts'>
  import OptionButton from '@/components/UI/OptionButton.svelte'

  const { close, play, playAdmin, install, installOptions }: {
    close: () => void
    play: () => void
    playAdmin: () => void
    install: (store: 'DMM' | 'DLsite') => void
    installOptions: ('DMM' | 'DLsite')[]
  } = $props()
</script>

<div>
  <div
    class='flex items-center gap-4 p-(y-2 l-4 r-2) text-(body3 text-primary) font-bold'
  >
    <div class='whitespace-nowrap'>Select game option</div>
    <button
      onclick={close}
      class='i-iconoir-cancel ml-auto h-5 w-5 color-text-tertiary transition-all hover:color-text-primary'
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
  {#each installOptions as option}
    <OptionButton
      text={`${option}からインストールする`}
      onclick={() => {
        close()
        install(option)
      }}
    />
  {/each}
</div>
