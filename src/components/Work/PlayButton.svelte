<script lang='ts'>
  import type { GameInstallStatus } from '@/lib/types'
  import APopover from '@/components/UI/APopover.svelte'
  import Button from '@/components/UI/Button.svelte'
  import ButtonBase from '@/components/UI/ButtonBase.svelte'
  import PlayPopover from '@/components/Work/PlayPopover.svelte'

  interface Props {
    gameStatus: GameInstallStatus
    play?: (arg: { isAdmin: boolean | undefined }) => void
    install?: () => void
  }

  let { gameStatus, play, install }: Props = $props()

  const isInstalled = $derived(gameStatus === 'installed')
  const canInstall = $derived(gameStatus === 'owned-not-installed')
</script>

<div class='min-w-0 flex items-center'>
  {#if isInstalled && play}
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
          tooltip={open
            ? undefined
            : {
              content: 'このゲームの設定',
              placement: 'bottom',
              theme: 'default',
              delay: 1000,
            }}
          variant='success'
        >
          <div
            class='i-material-symbols-arrow-drop-down h-5 w-5 color-text-white'
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
  {:else if canInstall && install}
    <Button
      leftIcon='i-material-symbols-download-rounded'
      text='Install'
      variant='accent'
      onclick={install}
    />
  {/if}
</div>
