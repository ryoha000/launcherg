<script lang='ts'>
  import APopover from '@/components/UI/APopover.svelte'
  import Button from '@/components/UI/Button.svelte'
  import ButtonBase from '@/components/UI/ButtonBase.svelte'
  import PlayPopover from '@/components/Work/PlayPopover.svelte'

  let { play }: { play: (arg: { isAdmin: boolean | undefined }) => void } = $props()
</script>

<div class='flex items-center min-w-0'>
  <Button
    appendClass='rounded-r-0'
    leftIcon='i-material-symbols-power-rounded'
    text='Play'
    variant='success'
    onclick={() => play({ isAdmin: undefined })}
  />
  <APopover>
    {#snippet button({ open })}
      <ButtonBase
        appendClass='h-8 w-8 flex items-center justify-center rounded-l-0'
        tooltip={open ? undefined : {
          content: 'このゲームの設定',
          placement: 'bottom',
          theme: 'default',
          delay: 1000,
        }}
        variant='success'
      >
        <div
          class='color-text-white w-5 h-5 i-material-symbols-arrow-drop-down'
          class:rotate-180={open}
        ></div>
      </ButtonBase>
    {/snippet}
    {#snippet children({ close })}
      <PlayPopover
        close={close}
        play={() => {
          play({ isAdmin: false })
        }}
        playAdmin={() => {
          play({ isAdmin: true })
        }}
      />
    {/snippet}
  </APopover>
</div>
