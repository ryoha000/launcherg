<script lang='ts'>
  import type { Props as TippyOption } from 'tippy.js'
  import { convertFileSrc } from '@tauri-apps/api/core'
  import { onMount } from 'svelte'
  import { get } from 'svelte/store'
  import tippy from 'tippy.js'
  import ConfirmDeleteOnCheckModal from '@/components/Setting/Download/ConfirmDeleteOnCheckModal.svelte'
  import APopover from '@/components/UI/APopover.svelte'
  import Button from '@/components/UI/Button.svelte'
  import Checkbox from '@/components/UI/Checkbox.svelte'
  import { commandDeleteCollectionElement } from '@/lib/command'
  import { useWorkDetailsAllQuery } from '@/lib/data/queries/workDetails'
  import { useAddWorkOmitMutation, useRemoveWorkOmitMutation } from '@/lib/data/queries/workOmit'
  import { showErrorToast, showInfoToast } from '@/lib/toast'
  import { settings } from '@/store/settings'
  import { sidebarCollectionElements } from '@/store/sidebarCollectionElements'

  const workDetailsQuery = useWorkDetailsAllQuery()

  // ストア種別のマルチセレクト（1: DMM, 2: DLsite）
  let storeFilter: number[] = $state([1, 2])
  const storeFilterLabel = $derived.by(() => {
    if (storeFilter.length === 2)
      return 'DMM, DLsite'
    if (storeFilter.length === 1)
      return storeFilter[0] === 1 ? 'DMM' : 'DLsite'
    return '未選択'
  })
  let keyword = $state('')

  // チェック時の単体削除確認（今後表示しない設定付き）
  const getAutoDeletePref = () => get(settings).storeMapped.autoDeleteOnCheck
  const setAutoDeletePref = (v: boolean) => {
    settings.update(s => ({
      ...s,
      storeMapped: { ...s.storeMapped, autoDeleteOnCheck: v },
    }))
  }
  let isOpenConfirmDelete = $state(false)
  let confirmDeleteTarget = $state(null as { id: number, title: string } | null)
  let dontAskAgain = $state(false)
  const performDeleteElement = async (id: number, _title?: string) => {
    try {
      await commandDeleteCollectionElement(id)
      await sidebarCollectionElements.refetch()
      await get(workDetailsQuery).refetch()
      showInfoToast('削除しました')
    }
    catch (e) {
      console.error(e)
      showErrorToast('削除に失敗しました')
    }
    finally {
      isOpenConfirmDelete = false
      confirmDeleteTarget = null
      dontAskAgain = false
    }
  }
  const maybeDeleteOnFlagSet = async (collectionElementId?: number, title?: string) => {
    if (!collectionElementId)
      return
    if (getAutoDeletePref()) {
      await performDeleteElement(collectionElementId, title)
      return
    }
    confirmDeleteTarget = { id: collectionElementId, title: title ?? '' }
    isOpenConfirmDelete = true
  }

  function resetConfirmDeleteState() {
    isOpenConfirmDelete = false
    confirmDeleteTarget = null
    dontAskAgain = false
  }
  async function onConfirmDeleteModal() {
    if (dontAskAgain)
      setAutoDeletePref(true)
    if (confirmDeleteTarget)
      await performDeleteElement(confirmDeleteTarget.id, confirmDeleteTarget.title)
  }

  async function onConfirmDeleteFromChild(checked: boolean) {
    if (checked)
      setAutoDeletePref(true)
    await onConfirmDeleteModal()
  }

  const addDenyMutation = useAddWorkOmitMutation()
  const removeDenyMutation = useRemoveWorkOmitMutation()
  const disabledDenyList = $derived($addDenyMutation.isPending || $removeDenyMutation.isPending)

  // 表示件数系（WorkDetails を直接参照）
  const items = $derived.by(() => ($workDetailsQuery.data ?? []))
  const totalCount = $derived.by(() => (items.length))
  const dmmCount = $derived.by(() => (items.filter(w => !!w.dmm).length))
  const dlsiteCount = $derived.by(() => (items.filter(w => !!w.dlsite).length))
  const denyListTotal = $derived.by(() => (items.reduce((acc, w) => acc + (w.isDmmOmitted ? 1 : 0) + (w.isDlsiteOmitted ? 1 : 0), 0)))

  const filteredItems = $derived.by(() => {
    const q = keyword.trim().toLowerCase()
    return items.filter((w) => {
      if (storeFilter.length > 0) {
        const matchDmm = storeFilter.includes(1) && !!w.dmm
        const matchDl = storeFilter.includes(2) && !!w.dlsite
        if (!matchDmm && !matchDl)
          return false
      }
      if (!q)
        return true
      const inTitle = w.title.toLowerCase().includes(q)
      return inTitle
    })
  })

  const tooltipAction = (node: HTMLElement, tooltip?: Partial<TippyOption>) => {
    if (!tooltip)
      return
    const tp = tippy(node, tooltip)
    return {
      update(newTooltip?: Partial<TippyOption>) {
        if (!newTooltip)
          return
        tp.setProps(newTooltip)
      },
      destroy() {
        tp.destroy()
      },
    }
  }

  const updateDenied = async (arg: {
    collectionElementId?: number
    storeType: number
    storeId: string
    title: string
    nextValue: boolean
    prevValue: boolean
    workId: number
  }) => {
    const { collectionElementId, storeType, storeId, title, nextValue, prevValue, workId } = arg
    if (nextValue === prevValue)
      return
    if (!collectionElementId && !nextValue) {
      // omit のみ運用: 未登録のゲームでは『連携除外』は解除できません
      showErrorToast('未登録のゲームでは『連携除外』は解除できません。')
      return
    }
    try {
      if (nextValue) {
        // 統合後: workId ベースで登録
        await get(addDenyMutation).mutateAsync({ workId })
      }
      else {
        await get(removeDenyMutation).mutateAsync({ workId })
      }
      const storeLabel = storeType === 1 ? 'DMM' : 'DLsite'
      showInfoToast(nextValue
        ? `「連携除外」設定: ${title}（${storeLabel} / ${storeId}）`
        : `「連携除外」解除: ${title}（${storeLabel} / ${storeId}）`,
      )
      if (nextValue) {
        await maybeDeleteOnFlagSet(collectionElementId, title)
      }
    }
    catch (e) {
      console.error(e)
      showErrorToast(`「連携除外」の${nextValue ? '設定' : '解除'}に失敗しました: ${title}`)
    }
  }

  onMount(async () => {
    await get(workDetailsQuery).refetch()
  })
</script>

<div class='grid grid-(rows-[auto_auto_auto_auto_1fr]) h-full w-full p-4'>
  <div class='mb-2 text-(h3 text-primary)'>ダウンロード購入作品の管理</div>
  <div class='mb-3 text-(sm text-secondary) -mt-1'>
    取り込み内容を随時見直し、不要な項目を適切に整理できます。<br />
    設定した除外は今後の連携にも反映され、再取り込みを防止します。
  </div>
  <div class='mb-1 flex items-center gap-3'>
    <APopover panelClass='right-0 min-w-56'>
      {#snippet button()}
        <Button
          text={storeFilterLabel}
          rightIcon='i-material-symbols-arrow-drop-down-rounded'
        />
      {/snippet}
      {#snippet children()}
        <div class='p-2'>
          <label class='mb-1 flex cursor-pointer select-none items-center gap-2'>
            <Checkbox
              value={storeFilter.includes(1)}
              on:update={e => (storeFilter = e.detail.value ? Array.from(new Set([...storeFilter, 1])) : storeFilter.filter(v => v !== 1))}
            />
            <span class='border-(1px border-primary solid) rounded-full px-3 py-(0.5) text-(sm)'>DMM</span>
          </label>
          <label class='flex cursor-pointer select-none items-center gap-2'>
            <Checkbox
              value={storeFilter.includes(2)}
              on:update={e => (storeFilter = e.detail.value ? Array.from(new Set([...storeFilter, 2])) : storeFilter.filter(v => v !== 2))}
            />
            <span class='border-(1px border-primary solid) rounded-full px-3 py-(0.5) text-(sm)'>DLsite</span>
          </label>
        </div>
      {/snippet}
    </APopover>
    <input
      class='ml-2 max-w-xs w-full border border-(border-primary) rounded bg-(bg-primary) p-2 text-(text-primary)'
      placeholder='タイトル検索'
      bind:value={keyword}
    />
    <div class='ml-auto text-(sm text-secondary)'>
      全 {totalCount} 件
      <span class='ml-3'>DMM {dmmCount} 件</span>
      <span class='ml-2'>DLsite {dlsiteCount} 件</span>
      <span class='ml-2'>除外 {denyListTotal} 件</span>
    </div>
  </div>
  <div class='mb-2 flex items-center justify-end gap-2'>
  </div>
  <div class='overflow-hidden border-(1px border-primary solid) rounded'>
    <div class='max-h-full overflow-auto'>
      <table class='w-full border-separate border-spacing-0 table-fixed whitespace-nowrap text-(left text-primary)'>
        <thead class='sticky top-0 z-20 bg-bg-primary'>
          <tr>
            <th class='w-24 border-(b border-primary) px-2 py-2'></th>
            <th class='w-18 border-(b border-primary) px-2 py-2'></th>
            <th class='w-36 border-(b border-primary) px-2 py-2'>タイトル</th>
            <th class='w-28 border-(b border-primary) px-2 py-2'>EGS</th>
            <th class='w-32 border-(b border-primary) px-2 py-2'>
              <div class='flex items-center gap-1'>
                連携除外
                <span
                  use:tooltipAction={{ content: '拡張機能から送られたこのIDをゲームとして取り込みません（同期時も無視されます）。いつでも解除できます。', placement: 'top', theme: 'default' }}
                  class='i-material-symbols-help-outline-rounded h-4 w-4 color-text-tertiary'
                ></span>
              </div>
            </th>

          </tr>
        </thead>
        <tbody>
          {#each filteredItems as item}
            <tr class='border-(b border-primary solid)'>
              <td class='px-2 py-1'>
                <div class='flex gap-1'>
                  {#if item.dmm}
                    <span class='inline-flex items-center border-(1px border-primary solid) rounded-full px-2 py-(0.5) text-(xs text-secondary)'>DMM</span>
                  {/if}
                  {#if item.dlsite}
                    <span class='inline-flex items-center border-(1px border-primary solid) rounded-full px-2 py-(0.5) text-(xs text-secondary)'>DLsite</span>
                  {/if}
                </div>
              </td>
              <td class='px-2 py-1'>
                {#if item.thumbnail}
                  <div class='h-12 w-20 overflow-hidden rounded bg-bg-secondary'>
                    <img src={convertFileSrc(item.thumbnail)} alt='thumbnail' class='h-full w-full object-cover' />
                  </div>
                {:else}
                  <div class='h-full w-full'>
                    <div class='h-full w-full'></div>
                  </div>
                {/if}
              </td>
              <td class='w-36 overflow-hidden text-ellipsis whitespace-nowrap px-2 py-1'>{item.title}</td>
              <td class='px-2 py-1'>
                {#if item.erogamescapeId}
                  <a
                    class='text-(sm text-link) underline'
                    href={`https://erogamescape.dyndns.org/~ap2/ero/toukei_kaiseki/game.php?game=${item.erogamescapeId}`}
                    target='_blank'
                    rel='noreferrer noopener'
                  >{item.erogamescapeId}</a>
                {:else}
                  <span class='text-(sm text-secondary)'>未連携</span>
                {/if}
              </td>
              <td class='px-2 py-1'>
                <div class='flex gap-4'>
                  {#if item.dmm}
                    <label class='flex items-center gap-2'>
                      <Checkbox value={item.isDmmOmitted} on:update={e => updateDenied({ collectionElementId: item.collectionElementId ?? undefined, storeType: 1, storeId: item.dmm!.storeId, title: item.title, nextValue: e.detail.value, prevValue: item.isDmmOmitted, workId: item.id })} disabled={disabledDenyList} />
                      <span>DMM: {item.isDmmOmitted ? '除外' : '未設定'}</span>
                    </label>
                  {/if}
                  {#if item.dlsite}
                    <label class='flex items-center gap-2'>
                      <Checkbox value={item.isDlsiteOmitted} on:update={e => updateDenied({ collectionElementId: item.collectionElementId ?? undefined, storeType: 2, storeId: item.dlsite!.storeId, title: item.title, nextValue: e.detail.value, prevValue: item.isDlsiteOmitted, workId: item.id })} disabled={disabledDenyList} />
                      <span>DLsite: {item.isDlsiteOmitted ? '除外' : '未設定'}</span>
                    </label>
                  {/if}
                </div>
              </td>

            </tr>
          {/each}
        </tbody>
      </table>
    </div>
  </div>
</div>
<ConfirmDeleteOnCheckModal
  isOpen={isOpenConfirmDelete}
  targetTitle={confirmDeleteTarget ? confirmDeleteTarget.title : null}
  on:confirm={e => onConfirmDeleteFromChild(e.detail.dontAskAgain)}
  on:cancel={resetConfirmDeleteState}
  on:close={resetConfirmDeleteState}
/>
