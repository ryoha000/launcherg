<script lang='ts'>
  import type { WorkDetailsVm } from '@/lib/command'
  import APopover from '@/components/UI/APopover.svelte'
  import Button from '@/components/UI/Button.svelte'
  import ButtonBase from '@/components/UI/ButtonBase.svelte'
  import { useStart } from '@/components/Work/action.svelte'
  import InstallPopover from '@/components/Work/InstallPopover.svelte'
  import PlayPopover from '@/components/Work/PlayPopover.svelte'

  interface Props {
    workDetail: WorkDetailsVm
  }

  let { workDetail }: Props = $props()

  const { start, isNotInstalled, installOptions } = useStart(workDetail)

  const installPopoverOptions = $derived(installOptions.map(option => option.store))
</script>

<div class='min-w-0 flex items-center'>
  {#if isNotInstalled}
    {#if installOptions.length > 0}
      <Button
        appendClass='rounded-r-0'
        leftIcon='i-material-symbols-download-rounded'
        text='Install'
        variant='accent-fill'
        onclick={installOptions[0].install}
      />
      <APopover>
        {#snippet button({ open })}
          <ButtonBase
            appendClass='h-8 w-8 flex items-center justify-center rounded-l-0'
            tooltip={open
              ? undefined
              : {
                content: 'インストール元の選択',
                placement: 'bottom',
                theme: 'default',
                delay: 1000,
              }}
            variant='accent-fill'
          >
            <div
              class='i-material-symbols-arrow-drop-down h-5 w-5 color-text-white'
              class:rotate-180={open}
            ></div>
          </ButtonBase>
        {/snippet}
        {#snippet children({ close })}
          <InstallPopover
            close={close}
            options={installPopoverOptions}
            install={(store) => {
              installOptions.find(option => option.store === store)?.install()
            }}
          />
        {/snippet}
      </APopover>
    {/if}
  {:else}
    <Button
      appendClass='rounded-r-0'
      leftIcon='i-material-symbols-power-rounded'
      text='Play'
      variant='success'
      onclick={() => start('default')}
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
            start('user')
          }}
          playAdmin={() => {
            start('admin')
          }}
          install={(store) => {
            installOptions.find(option => option.store === store)?.install()
          }}
          installOptions={installPopoverOptions}
        />
      {/snippet}
    </APopover>
  {/if}
</div>
