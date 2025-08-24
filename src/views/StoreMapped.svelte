<script lang='ts'>
  import type { Props as TippyOption } from 'tippy.js'
  import type { StoreMappedElementVm } from '@/lib/command'
  import { convertFileSrc } from '@tauri-apps/api/core'
  import { onMount } from 'svelte'
  import { get } from 'svelte/store'
  import tippy from 'tippy.js'
  import APopover from '@/components/UI/APopover.svelte'
  import Button from '@/components/UI/Button.svelte'
  import Checkbox from '@/components/UI/Checkbox.svelte'
  import Modal from '@/components/UI/Modal.svelte'
  import { commandGetStoreMappedElements } from '@/lib/command'
  import { useAddDenyListMutation, useDenyListQuery, useRemoveDenyListMutation } from '@/lib/data/queries/denyList'
  import { useAddDmmPackMutation, useDmmPackQuery, useRemoveDmmPackMutation } from '@/lib/data/queries/dmmPack'
  import { showErrorToast, showInfoToast } from '@/lib/toast'
  import { createWritable } from '@/lib/utils'
  import { sidebarCollectionElements } from '@/store/sidebarCollectionElements'
  // import { sidebarCollectionElements } from '@/store/sidebarCollectionElements'

  const [{ subscribe, set }, value] = createWritable<StoreMappedElementVm[]>([])
  export const items = { subscribe, value }

  let selected: StoreMappedElementVm | null = null
  // ボタンのdisabled等に使っていないため削除
  // let loading = $state(false)
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
  // 行操作はトグルのみ（削除ボタンは列から撤去）

  // 一括削除モーダル
  let isOpenDeleteSelected = $state(false)
  const deletableItems = $derived.by<StoreMappedElementVm[]>(() =>
    $items.filter(v => v.alreadyDenied || (v.storeType === 1 && v.isDmmPack)),
  )
  const openDeleteModal = () => {
    if (deletableItems.length === 0) {
      showErrorToast('削除対象がありません')
      return
    }
    isOpenDeleteSelected = true
  }
  const confirmDelete = async () => {
    try {
      // まとめて削除: collectionElementId 単位
      const ids: number[] = Array.from(new Set(deletableItems.map(v => v.collectionElementId)))
      // フロント即時反映
      set(value().filter(v => !ids.includes(v.collectionElementId)))
      // バックエンド削除
      for (const id of ids) {
        try {
          await (await import('@/lib/command')).commandDeleteCollectionElement(id)
        }
        catch (e) {
          console.error(e)
        }
      }
      await sidebarCollectionElements.refetch()
      showInfoToast('削除しました')
    }
    catch (e) {
      console.error(e)
      showErrorToast('削除に失敗しました')
    }
    finally {
      isOpenDeleteSelected = false
    }
  }

  const denyListQuery = useDenyListQuery()
  const denyListTotal = $derived<number | undefined>($denyListQuery.data?.length)

  const addDenyMutation = useAddDenyListMutation()
  const removeDenyMutation = useRemoveDenyListMutation()

  const disabledDenyList = $derived($denyListQuery.isLoading || $addDenyMutation.isPending || $removeDenyMutation.isPending)

  const dmmPackQuery = useDmmPackQuery()
  const dmmPackTotal = $derived<number | undefined>($dmmPackQuery.data?.length)

  const addDmmPackMutation = useAddDmmPackMutation()
  const removeDmmPackMutation = useRemoveDmmPackMutation()

  const disabledDmmPack = $derived($dmmPackQuery.isLoading || $addDmmPackMutation.isPending || $removeDmmPackMutation.isPending)

  const refetch = async () => {
    const newElements = await commandGetStoreMappedElements()
    set(newElements)
    // 選択状態の維持
    if (selected) {
      const exists = newElements.find(v => v.collectionElementId === selected!.collectionElementId)
      selected = exists || null
    }
  }

  const checkIsDenied = (storeType: number, storeId: string) => {
    return !!$denyListQuery.data?.some(v => v.storeType === storeType && v.storeId === storeId)
  }
  const checkIsDmmPack = (storeId: string) => {
    return !!$dmmPackQuery.data?.some(v => v.storeId === storeId)
  }

  onMount(async () => {
    await refetch()
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
  }) => {
    const { collectionElementId, storeType, storeId, title, nextValue } = arg
    const prev = checkIsDenied(storeType, storeId)
    if (nextValue === prev)
      return
    if (!collectionElementId && !nextValue && !checkIsDmmPack(storeId)) {
      showErrorToast('未登録のゲームでは『連携除外』または『パック作品』のいずれかを選択してください。両方を未選択にはできません。')
      return
    }
    try {
      if (nextValue) {
        await get(addDenyMutation).mutateAsync({ storeType, storeId, name: title })
      }
      else {
        await get(removeDenyMutation).mutateAsync({ storeType, storeId })
      }
      const storeLabel = storeType === 1 ? 'DMM' : 'DLsite'
      showInfoToast(nextValue
        ? `「連携除外」設定: ${title}（${storeLabel} / ${storeId}）`
        : `「連携除外」解除: ${title}（${storeLabel} / ${storeId}）`,
      )
    }
    catch (e) {
      console.error(e)
      showErrorToast(`「連携除外」の${nextValue ? '設定' : '解除'}に失敗しました: ${title}`)
    }
  }

  const updateDmmPack = async (arg: {
    collectionElementId?: number
    storeId: string
    title: string
    nextValue: boolean
  }) => {
    const { collectionElementId, storeId, title, nextValue } = arg
    const prev = checkIsDmmPack(storeId)
    if (nextValue === prev)
      return
    if (!collectionElementId && !nextValue && !checkIsDenied(1, storeId)) {
      showErrorToast('未登録のゲームでは『連携除外』または『パック作品』のいずれかを選択してください。両方を未選択にはできません。')
      return
    }
    try {
      if (nextValue) {
        await get(addDmmPackMutation).mutateAsync({ storeId, name: title })
      }
      else {
        await get(removeDmmPackMutation).mutateAsync({ storeId })
      }
      showInfoToast(nextValue
        ? `「パック作品」設定: ${title}（DMM / ${storeId}）`
        : `「パック作品」解除: ${title}（DMM / ${storeId}）`,
      )
    }
    catch (e) {
      console.error(e)
      showErrorToast(`「パック作品」の${nextValue ? '設定' : '解除'}に失敗しました: ${title}`)
    }
  }

  // 表示件数系の派生値
  const filteredItems = $derived.by(() => {
    interface ResultContent {
      storeType: number
      storeId: string
      title: string
      collectionElementId?: number
      thumbnail?: string
      alreadyDenied: boolean
      isDmmPack: boolean
    }
    const resultMap = new Map<string, ResultContent>()
    const keyFn = (storeType: number, storeId: string) => `${storeType}:${storeId}`
    const update = (content: ResultContent) => {
      const key = keyFn(content.storeType, content.storeId)
      const existing = resultMap.get(key)
      if (existing) {
        resultMap.set(key, {
          ...existing,
          ...content,
        })
      }
      else {
        resultMap.set(key, content)
      }
    }
    const q = keyword.trim().toLowerCase()

    $denyListQuery.data?.filter((v) => {
      if (!q)
        return true
      return v.name.toLowerCase().includes(q)
    }).forEach((v) => {
      update({
        storeType: v.storeType,
        storeId: v.storeId,
        title: v.name,
        thumbnail: undefined,
        alreadyDenied: true,
        isDmmPack: false,
      })
    })
    $dmmPackQuery.data?.filter((v) => {
      if (!q)
        return true
      return v.name.toLowerCase().includes(q)
    }).forEach((v) => {
      update({
        storeType: 1,
        storeId: v.storeId,
        title: v.name,
        thumbnail: undefined,
        alreadyDenied: false,
        isDmmPack: true,
      })
    })
    $items.filter((v) => {
      if (storeFilter.length > 0 && !storeFilter.includes(v.storeType))
        return false
      if (!q)
        return true
      return v.title.toLowerCase().includes(q)
    }).forEach((v) => {
      update({
        storeType: v.storeType,
        storeId: v.storeId,
        title: v.title,
        collectionElementId: v.collectionElementId,
        thumbnail: v.thumbnail,
        alreadyDenied: v.alreadyDenied,
        isDmmPack: v.isDmmPack,
      })
    })

    return Array.from(resultMap.entries()).map(([key, content]) => ({
      ...content,
      key,
    }))
  })

  const totalCount = $derived.by(() => $items.length)
  const dmmCount = $derived.by(() => $items.filter(v => v.storeType === 1).length)
  const dlsiteCount = $derived.by(() => $items.filter(v => v.storeType === 2).length)
</script>

<div class='grid grid-(rows-[auto_auto_auto_auto_1fr]) h-full w-full p-4'>
  <div class='mb-2 text-(h3 text-primary)'>ダウンロード購入作品の管理</div>
  <div class='mb-3 text-(sm text-secondary) -mt-1'>
    取り込み内容を随時見直し、不要な項目やパック親項目を適切に整理できます。<br />
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
      placeholder='キーワード検索（タイトル/ID）'
      bind:value={keyword}
    />
    <div class='ml-auto text-(sm text-secondary)'>
      全 {totalCount} 件
      <span class='ml-3'>DMM {dmmCount} 件</span>
      <span class='ml-2'>DLsite {dlsiteCount} 件</span>
      <span class='ml-2'>除外 {denyListTotal} 件</span>
      <span class='ml-2'>パック {dmmPackTotal} 件</span>
    </div>
  </div>
  <div class='mb-2 flex items-center justify-end gap-2'>
    <Button text='再取得' onclick={refetch} />
    <Button
      text='除外・パック指定を一括削除'
      tooltip={{ content: '連携除外/パック作品にチェックが入っている要素を全て削除します', placement: 'bottom', theme: 'default' }}
      variant='error'
      onclick={openDeleteModal}
    />
  </div>
  <div class='overflow-hidden border-(1px border-primary solid) rounded'>
    <div class='max-h-full overflow-auto'>
      <table class='w-full border-separate border-spacing-0 table-fixed whitespace-nowrap text-(left text-primary)'>
        <thead class='sticky top-0 z-20 bg-bg-primary'>
          <tr>
            <th class='w-16 border-(b border-primary) px-2 py-2'>ソース</th>
            <th class='w-18 border-(b border-primary) px-2 py-2'></th>
            <th class='w-36 border-(b border-primary) px-2 py-2'>タイトル</th>
            <th class='w-32 border-(b border-primary) px-2 py-2'>
              <div class='flex items-center gap-1'>
                連携除外
                <span
                  use:tooltipAction={{ content: '拡張機能から送られたこのIDをゲームとして取り込みません（同期時も無視されます）。いつでも解除できます。', placement: 'top', theme: 'default' }}
                  class='i-material-symbols-help-outline-rounded h-4 w-4 color-text-tertiary'
                ></span>
              </div>
            </th>
            <th class='w-36 border-(b border-primary) px-2 py-2'>
              <div class='flex items-center gap-1'>
                パック作品
                <span
                  use:tooltipAction={{ content: 'DMMのセット商品（複数作品を含む）として扱います。含まれる個別作品を取得するための特別処理を有効にします。DMM以外には適用されません。', placement: 'top', theme: 'default' }}
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
                <span class='inline-flex items-center border-(1px border-primary solid) rounded-full px-2 py-(0.5) text-(xs text-secondary)'>
                  {item.storeType === 1 ? 'DMM' : 'DLsite'}
                </span>
              </td>
              <td class='px-2 py-1'>
                <div class='h-12 w-20 overflow-hidden rounded bg-bg-secondary'>
                  {#if item.thumbnail}
                    <img src={convertFileSrc(item.thumbnail)} alt='' class='h-full w-full object-cover' loading='lazy' decoding='async' />
                  {:else}
                    <div class='h-full w-full'></div>
                  {/if}
                </div>
              </td>
              <td class='w-36 overflow-hidden text-ellipsis whitespace-nowrap px-2 py-1'>{item.title}</td>
              <td class='px-2 py-1'>
                <label class='flex items-center gap-2'>
                  <Checkbox value={checkIsDenied(item.storeType, item.storeId)} on:update={e => updateDenied({ ...item, nextValue: e.detail.value })} disabled={disabledDenyList} />
                  <span>{checkIsDenied(item.storeType, item.storeId) ? '除外中' : '未設定'}</span>
                </label>
              </td>
              <td class='px-2 py-1'>
                {#if item.storeType === 1}
                  <label class='flex items-center gap-2'>
                    <Checkbox value={checkIsDmmPack(item.storeId)} on:update={e => updateDmmPack({ ...item, nextValue: e.detail.value })} disabled={disabledDmmPack} />
                    <span>{checkIsDmmPack(item.storeId) ? 'パック中' : '未設定'}</span>
                  </label>
                {:else}
                  <span class='opacity-50'>対象外</span>
                {/if}
              </td>
            </tr>
          {/each}
        </tbody>
      </table>
    </div>
  </div>
</div>
<Modal
  isOpen={isOpenDeleteSelected}
  title='確認'
  confirmText='削除する'
  cancelText='キャンセル'
  onconfirm={confirmDelete}
  oncancel={() => (isOpenDeleteSelected = false)}
  onclose={() => (isOpenDeleteSelected = false)}
>
  {#snippet children()}
    <div class='space-y-2'>
      <div>以下の要素を削除します。よろしいですか？</div>
      <ul class='list-disc pl-6'>
        {#each deletableItems.slice(0, 10) as it}
          <li>{it.title}（{it.brand}）</li>
        {/each}
        {#if deletableItems.length > 10}
          <li>...ほか {deletableItems.length - 10} 件</li>
        {/if}
      </ul>
    </div>
  {/snippet}
  {#snippet footer()}
    <div class='flex items-center border-(t-1px border-primary solid) p-4'>
      <div class='ml-auto flex items-center gap-2'>
        <Button text='キャンセル' onclick={() => (isOpenDeleteSelected = false)} />
        <Button variant='error' text='削除する' onclick={confirmDelete} />
      </div>
    </div>
  {/snippet}
</Modal>
