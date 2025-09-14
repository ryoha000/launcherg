<script lang='ts'>
  import type { WorkDetailsVm } from '@/lib/command'
  import APopover from '@/components/UI/APopover.svelte'
  import Button from '@/components/UI/Button.svelte'
  import ButtonBase from '@/components/UI/ButtonBase.svelte'
  import { useStart } from '@/components/Work/action.svelte'
  import InstallPopover from '@/components/Work/InstallPopover.svelte'
  import PlayPopover from '@/components/Work/PlayPopover.svelte'
  import { commandOpenUrl } from '@/lib/command'
  import { useWorkLnkQuery } from '@/lib/data/queries/workLnk'
  import { useParentDmmPackKeysQuery } from '@/lib/data/queries/workParentDmmPack'

  interface Props {
    workDetail: WorkDetailsVm
  }

  let { workDetail }: Props = $props()

  const workLnkQuery = useWorkLnkQuery(workDetail.id)
  const isNotInstalled = $derived(!$workLnkQuery.data?.length)

  const parentDmmPackKeysQuery = useParentDmmPackKeysQuery(workDetail.id)

  const { start } = useStart(workDetail, workLnkQuery)

  const dmmUrlForInstall = $derived.by<string | null>(() => {
    const dmm = workDetail.dmm
    if (!dmm) {
      return null
    }
    const parent = $parentDmmPackKeysQuery.data ?? null
    const payload = {
      type: 'download',
      value: {
        game: {
          storeId: dmm.storeId,
          category: dmm.category,
          subcategory: dmm.subcategory,
        },
        parentPack: parent
          ? {
            storeId: parent.storeId,
            category: parent.category,
            subcategory: parent.subcategory,
          }
          : undefined,
      },
    }
    const url = new URL('https://dlsoft.dmm.co.jp/mylibrary/')
    url.searchParams.set('launcherg', JSON.stringify(payload))
    return url.toString()
  })
  const dlsiteUrlForInstall = $derived.by(() => {
    const dlsite = workDetail.dlsite
    if (!dlsite) {
      return null
    }
    const payload = {
      type: 'download',
      value: {
        game: { storeId: dlsite.storeId, category: dlsite.category },
      },
    }
    const url = new URL('https://play.dlsite.com/library')
    url.searchParams.set('launcherg', JSON.stringify(payload))
    return url.toString()
  })

  const installOptions = $derived.by(() => {
    const options: { store: 'DMM' | 'DLsite', installUrl: string }[] = []
    if (dmmUrlForInstall) {
      options.push({ store: 'DMM', installUrl: dmmUrlForInstall })
    }
    if (dlsiteUrlForInstall) {
      options.push({ store: 'DLsite', installUrl: dlsiteUrlForInstall })
    }
    return options
  })

  const installPopoverOptions = $derived(installOptions.map(option => option.store))

  const install = async (url: string) => {
    await commandOpenUrl(url)
  }
</script>

<div class='min-w-0 flex items-center'>
  {#if isNotInstalled}
    {#if installOptions.length > 0}
      <Button
        appendClass='rounded-r-0'
        leftIcon='i-material-symbols-download-rounded'
        text='Install'
        variant='accent-fill'
        onclick={() => install(installOptions[0].installUrl)}
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
              const url = installOptions.find(option => option.store === store)?.installUrl
              if (url) {
                install(url)
              }
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
            const url = installOptions.find(option => option.store === store)?.installUrl
            if (url) {
              install(url)
            }
          }}
          installOptions={installPopoverOptions}
        />
      {/snippet}
    </APopover>
  {/if}
</div>
