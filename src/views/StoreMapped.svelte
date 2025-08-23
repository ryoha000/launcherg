<script lang='ts'>
  import type { Props as TippyOption } from 'tippy.js'
  import type { StoreMappedElementVm } from '@/lib/command'
  import { convertFileSrc, invoke } from '@tauri-apps/api/core'
  import { onMount } from 'svelte'
  import tippy from 'tippy.js'
  import APopover from '@/components/UI/APopover.svelte'
  import Button from '@/components/UI/Button.svelte'
  import Checkbox from '@/components/UI/Checkbox.svelte'
  import Modal from '@/components/UI/Modal.svelte'
  import { commandDenyListAdd, commandDenyListAll, commandDenyListRemove, commandGetStoreMappedElements } from '@/lib/command'
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

  const refetch = async () => {
    set(await commandGetStoreMappedElements())
    if (selected) {
      const exists = value().find(v => v.collectionElementId === selected!.collectionElementId)
      selected = exists || null
    }
  }

  // DenyList / DMM Pack の総件数
  let denyListTotal = $state(0)
  let dmmPackTotal = $state(0)

  const refetchTotals = async () => {
    try {
      const deny = await commandDenyListAll()
      denyListTotal = deny.length
    }
    catch {}
    try {
      const packs = await invoke<Array<{ id: number, storeId: string }>>('dmm_pack_all')
      dmmPackTotal = packs.length
    }
    catch {}
  }

  onMount(async () => {
    await Promise.all([refetch(), refetchTotals()])
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

  const updateDenied = async (
    collectionElementId: number,
    storeType: number,
    storeId: string,
    title: string,
    nextValue: boolean,
  ) => {
    const prev = value().find(v => v.collectionElementId === collectionElementId)?.alreadyDenied ?? false
    if (nextValue === prev)
      return
    // 楽観的更新
    set(value().map(v => v.collectionElementId === collectionElementId ? { ...v, alreadyDenied: nextValue } : v))
    try {
      if (nextValue) {
        await commandDenyListAdd(storeType, storeId, title)
      }
      else {
        await commandDenyListRemove(storeType, storeId)
      }
      showInfoToast('更新しました')
      await refetchTotals()
    }
    catch (e) {
      console.error(e)
      // ロールバック
      set(value().map(v => v.collectionElementId === collectionElementId ? { ...v, alreadyDenied: prev } : v))
      showErrorToast('更新に失敗しました')
    }
  }

  // なし

  // なし

  const updateDmmPack = async (
    collectionElementId: number,
    storeId: string,
    title: string,
    nextValue: boolean,
  ) => {
    const prev = value().find(v => v.collectionElementId === collectionElementId)?.isDmmPack ?? false
    if (nextValue === prev)
      return
    // 楽観的更新
    set(value().map(v => v.collectionElementId === collectionElementId ? { ...v, isDmmPack: nextValue } : v))
    try {
      const { commandDmmPackAdd, commandDmmPackRemove } = await import('@/lib/command')
      if (nextValue) {
        await commandDmmPackAdd(storeId, title)
      }
      else {
        await commandDmmPackRemove(storeId)
      }
      showInfoToast('更新しました')
      await refetchTotals()
    }
    catch (e) {
      console.error(e)
      // ロールバック
      set(value().map(v => v.collectionElementId === collectionElementId ? { ...v, isDmmPack: prev } : v))
      showErrorToast('更新に失敗しました')
    }
  }

  // 表示件数系の派生値
  const filteredItems = $derived.by<StoreMappedElementVm[]>(() => {
    const q = keyword.trim().toLowerCase()
    return $items.filter((it) => {
      if (storeFilter.length > 0 && !storeFilter.includes(it.storeType))
        return false
      if (!q)
        return true
      return (
        it.title.toLowerCase().includes(q)
        || it.brand.toLowerCase().includes(q)
        || it.storeId.toLowerCase().includes(q)
      )
    })
  })

  const totalCount = $derived.by(() => $items.length)
  const dmmCount = $derived.by(() => filteredItems.filter(v => v.storeType === 1).length)
  const dlsiteCount = $derived.by(() => filteredItems.filter(v => v.storeType === 2).length)
</script>

<div class='grid grid-(rows-[auto_auto_auto_1fr]) h-full w-full p-4'>
  <div class='mb-2 text-(h3 text-primary)'>ダウンロード購入作品の管理</div>
  <div class='mb-3 text-(sm text-secondary) -mt-1'>
    取り込み内容を随時見直し、不要な項目やパック親項目を適切に整理できます。<br />
    設定した除外は今後の連携にも反映され、再取り込みを防止します。
  </div>
  <div class='mb-2 flex items-center gap-3'>
    <div class='text-(sm text-secondary)'>絞り込み:</div>
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
      placeholder='キーワード検索（タイトル/ブランド/ID）'
      bind:value={keyword}
    />
    <Button text='再取得' onclick={refetch} />
    <Button
      text='除外・パック指定を一括削除'
      tooltip={{ content: '連携除外/パック作品にチェックが入っている要素を全て削除します', placement: 'bottom', theme: 'default' }}
      variant='error'
      onclick={openDeleteModal}
    />
    <div class='ml-auto text-(sm text-secondary)'>
      全 {totalCount} 件
      <span class='ml-3'>DMM {dmmCount} 件</span>
      <span class='ml-2'>DLsite {dlsiteCount} 件</span>
      <span class='ml-2'>除外 {denyListTotal} 件</span>
      <span class='ml-2'>パック {dmmPackTotal} 件</span>
    </div>
  </div>
  <div class='overflow-hidden border-(1px border-primary solid) rounded'>
    <div class='max-h-full overflow-auto'>
      <table class='w-full border-separate border-spacing-0 table-fixed whitespace-nowrap text-(left text-primary)'>
        <thead class='sticky top-0 z-20 bg-bg-primary'>
          <tr>
            <th class='w-16 border-(b border-primary) px-2 py-2'>ソース</th>
            <th class='w-18 border-(b border-primary) px-2 py-2'></th>
            <th class='w-36 border-(b border-primary) px-2 py-2'>タイトル</th>
            <th class='w-36 border-(b border-primary) px-2 py-2'>ブランド</th>
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
              <td class='w-36 overflow-hidden text-ellipsis whitespace-nowrap px-2 py-1'>{item.brand}</td>
              <td class='px-2 py-1'>
                <label class='flex items-center gap-2'>
                  <Checkbox value={item.alreadyDenied} on:update={e => updateDenied(item.collectionElementId, item.storeType, item.storeId, item.title, e.detail.value)} />
                  <span>{item.alreadyDenied ? '除外中' : '未設定'}</span>
                </label>
              </td>
              <td class='px-2 py-1'>
                {#if item.storeType === 1}
                  <label class='flex items-center gap-2'>
                    <Checkbox value={item.isDmmPack} on:update={e => updateDmmPack(item.collectionElementId, item.storeId, item.title, e.detail.value)} />
                    <span>{item.isDmmPack ? 'パック中' : '未設定'}</span>
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
