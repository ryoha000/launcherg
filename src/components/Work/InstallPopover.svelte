<script lang='ts'>
  import OptionButton from '@/components/UI/OptionButton.svelte'

  const { close, install, options }: {
    close: () => void
    install: (store: 'DMM' | 'DLsite') => void
    options: ('DMM' | 'DLsite')[]
  } = $props()
</script>

<div>
  <div
    class='flex items-center gap-4 p-(y-2 l-4 r-2) text-(body3 text-primary) font-bold'
  >
    <div class='whitespace-nowrap'>Select store for install</div>
    <button
      onclick={close}
      class='i-iconoir-cancel ml-auto h-5 w-5 color-text-tertiary transition-all hover:color-text-primary'
      aria-label='Close options'
    ></button>
  </div>
  {#each options as option}
    <OptionButton
      text={option}
      onclick={() => {
        close()
        install(option)
      }}
    />
  {/each}
</div>
