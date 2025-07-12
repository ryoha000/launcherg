<script lang='ts'>
  import type { AllGameCacheOne } from '@/lib/types'
  import { goto } from '@mateothegreat/svelte5-router'
  import ImportManually from '@/components/Sidebar/ImportManually.svelte'
  import APopover from '@/components/UI/APopover.svelte'
  import Button from '@/components/UI/Button.svelte'
  import ButtonCancel from '@/components/UI/ButtonCancel.svelte'
  import ButtonIcon from '@/components/UI/ButtonIcon.svelte'
  import DeleteElement from '@/components/Work/DeleteElement.svelte'
  import OtherInformation from '@/components/Work/OtherInformation.svelte'
  import PlayButton from '@/components/Work/PlayButton.svelte'
  import QrCode from '@/components/Work/QRCode.svelte'
  import SettingPopover from '@/components/Work/SettingPopover.svelte'
  import {
    commandDeleteCollectionElement,
    commandGetCollectionElement,
    commandOpenFolder,
    commandPlayGame,
    commandUpdateElementLike,
    commandUpsertCollectionElement,
  } from '@/lib/command'
  import { registerCollectionElementDetails } from '@/lib/registerCollectionElementDetails'
  import { showErrorToast } from '@/lib/toast'
  import { localStorageWritable } from '@/lib/utils'
  import { sidebarCollectionElements } from '@/store/sidebarCollectionElements'
  import { startProcessMap } from '@/store/startProcessMap'
  import { deleteTab, selected, tabs } from '@/store/tabs'

  interface Props {
    name: string
    id: number
    seiyaUrl: string
  }

  const { name, id, seiyaUrl }: Props = $props()

  const isAdminRecord = localStorageWritable<Record<number, boolean>>(
    'play-admin-cache',
    {},
  )

  const play = async (isAdmin: boolean | undefined) => {
    if (isAdmin !== undefined) {
      isAdminRecord.update((v) => {
        v[id] = isAdmin
        return v
      })
    }
    let _isAdmin: boolean = isAdmin ?? false
    if (isAdmin === undefined) {
      const cache = $isAdminRecord[id]
      if (cache) {
        _isAdmin = cache
      }
    }
    try {
      const processId = await commandPlayGame(id, _isAdmin)
      startProcessMap.update((v) => {
        if (processId) {
          v[id] = processId
        }
        return v
      })
    }
    catch (e) {
      showErrorToast(e as string)
    }
  }

  let isLike = $state(false)

  const toggleLike = async () => {
    await commandUpdateElementLike(id, !isLike)
    isLike = !isLike
    sidebarCollectionElements.updateLike(id, isLike)
  }

  const elementPromise = $derived((async () => {
    const element = await commandGetCollectionElement(id)
    isLike = !!element.likeAt
    return element
  })())

  let isOpenImportManually = $state(false)
  const onChangeGame = async (arg: {
    exePath: string | null
    lnkPath: string | null
    gameCache: AllGameCacheOne
  }) => {
    const isChangedGameId = id !== arg.gameCache.id
    if (isChangedGameId) {
      await commandDeleteCollectionElement(id)
    }
    await commandUpsertCollectionElement(arg)
    await registerCollectionElementDetails()
    await sidebarCollectionElements.refetch()
    if (isChangedGameId) {
      deleteTab($tabs[$selected].id)
    }
    isOpenImportManually = false
  }

  let isOpenDelete = $state(false)
  let isOpenOtherInformation = $state(false)
  let isOpenQrCode = $state(false)
</script>

{#await elementPromise then element}
  <div class='flex items-center gap-4 flex-wrap w-full min-w-0'>
    <PlayButton play={({ isAdmin }) => play(isAdmin)} />
    <Button
      leftIcon='i-material-symbols-drive-file-rename-outline'
      text='Memo'
      on:click={() => goto(`/memos/${id}?gamename=${name}`)}
    />
    <div class='flex items-center gap-2 ml-auto'>
      <ButtonCancel
        icon='i-material-symbols-qr-code'
        onclick={() => (isOpenQrCode = true)}
      />
      <ButtonCancel
        icon={isLike
          ? 'i-material-symbols-favorite-rounded'
          : 'i-material-symbols-favorite-outline-rounded'}
        onclick={toggleLike}
      />
      <APopover panelClass='right-0'>
        {#snippet button()}
          <ButtonIcon icon='i-material-symbols-menu-rounded' />
        {/snippet}
        {#snippet children({ close })}
          <SettingPopover
            onclose={() => close()}
            onselectChange={() => (isOpenImportManually = true)}
            onselectDelete={() => (isOpenDelete = true)}
            onselectOpen={() =>
              commandOpenFolder(element.exePath ?? element.lnkPath)}
            onselectOtherInfomation={() => (isOpenOtherInformation = true)}
          />
        {/snippet}
      </APopover>
    </div>
  </div>
  <ImportManually
    bind:isOpen={isOpenImportManually}
    idInput={`${id}`}
    path={element.exePath ?? element.lnkPath}
    on:confirm={e => onChangeGame(e.detail)}
    on:cancel={() => (isOpenImportManually = false)}
  />
  <DeleteElement bind:isOpen={isOpenDelete} {element} />
  <OtherInformation bind:isOpen={isOpenOtherInformation} {element} />
  <QrCode bind:isOpen={isOpenQrCode} {id} {seiyaUrl} />
{/await}
