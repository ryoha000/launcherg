<script lang='ts'>
  import type { WorkDetailsVm } from '@/lib/command'
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
    commandDeleteWork,
    commandGetWorkPaths,
    commandOpenFolder,
    commandUpdateWorkLike,
    commandUpsertCollectionElement,

  } from '@/lib/command'
  import { registerCollectionElementDetails } from '@/lib/registerCollectionElementDetails'
  import { sidebarCollectionElements } from '@/store/sidebarCollectionElements'
  import { deleteTab, selected, tabs } from '@/store/tabs'

  interface Props {
    workDetail: WorkDetailsVm
    id: number
    seiyaUrl: string
  }

  const { workDetail, id, seiyaUrl }: Props = $props()

  let isLike = $state(!!workDetail.likeAt)

  const toggleLike = async () => {
    await commandUpdateWorkLike(workDetail.id, !isLike)
    isLike = !isLike
    sidebarCollectionElements.updateLike(id, isLike)
  }

  const lnksPromise = $derived((async () => {
    const { lnks } = await commandGetWorkPaths(workDetail.id)
    return lnks
  })())

  let isOpenImportManually = $state(false)
  const onChangeGame = async (
    exePath: string | null,
    lnkPath: string | null,
    gameCache: AllGameCacheOne,
  ) => {
    // 既存要素の EGS ID と新しい候補の EGS ID を比較して差し替え判定
    const currentEgsId = workDetail.erogamescapeId
    const isChangedGame = !currentEgsId || currentEgsId !== gameCache.id
    if (isChangedGame) {
      await commandDeleteWork(workDetail.id)
    }
    await commandUpsertCollectionElement({ exePath, lnkPath, gameCache })
    await registerCollectionElementDetails()
    await sidebarCollectionElements.refetch()
    if (isChangedGame) {
      deleteTab($tabs[$selected].id)
    }
    isOpenImportManually = false
  }

  let isOpenDelete = $state(false)
  let isOpenOtherInformation = $state(false)
  let isOpenQrCode = $state(false)
</script>

{#await lnksPromise then lnks}
  <div class='min-w-0 w-full flex flex-wrap items-center gap-4'>
    <PlayButton workDetail={workDetail} />
    <Button
      leftIcon='i-material-symbols-drive-file-rename-outline'
      text='Memo'
      onclick={() => goto(`/memos/${id}?gamename=${workDetail.title}`)}
    />
    <div class='ml-auto flex items-center gap-2'>
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
            onselectOpen={() => lnks.length > 0 && commandOpenFolder(lnks[0].lnkPath)}
            onselectOtherInfomation={() => (isOpenOtherInformation = true)}
          />
        {/snippet}
      </APopover>
    </div>
  </div>
  <ImportManually
    bind:isOpen={isOpenImportManually}
    idInput={`${id}`}
    path={(lnks[0]?.lnkPath ?? '')}
    onconfirm={onChangeGame}
    oncancel={() => (isOpenImportManually = false)}
  />
  <DeleteElement bind:isOpen={isOpenDelete} {workDetail} />
  <OtherInformation bind:isOpen={isOpenOtherInformation} {workDetail} />
  <QrCode bind:isOpen={isOpenQrCode} {id} {seiyaUrl} />
{/await}
