<script lang='ts'>
  import { get } from 'svelte/store'
  import Button from '@/components/UI/Button.svelte'
  import { useImageQueueQuery } from '@/lib/data/queries/imageQueue'

  let showFinished = $state(false)
  let showOnlyError = $state(false)
  const unfinishedQuery = useImageQueueQuery(true)
  const finishedQuery = useImageQueueQuery(false)
  const items = $derived(showFinished ? ($finishedQuery.data ?? []) : ($unfinishedQuery.data ?? []))
  const viewItems = $derived(items.filter(it => showOnlyError ? !!it.lastError && it.lastError.length > 0 : true))

  function refresh() {
    if (showFinished)
      get(finishedQuery).refetch()
    else
      get(unfinishedQuery).refetch()
  }
  function fmtType(t: number) {
    if (t === 1)
      return 'URL'
    if (t === 2)
      return 'PATH'
    if (t === 3)
      return 'EXE'
    if (t === 4)
      return 'SHORTCUT'
    return String(t)
  }
  function fmtPreprocess(p: number) {
    if (p === 0)
      return 'None'
    if (p === 1)
      return 'Square256'
    if (p === 2)
      return 'W400'
    return String(p)
  }
</script>

<div class='h-full overflow-y-auto p-4'>
  <div class='mb-3 flex items-center gap-2'>
    <h2 class='text-(lg text-primary) font-semibold'>画像保存キュー</h2>
    <div class='ml-auto'>
      <Button variant='normal' onclick={refresh} text='更新' />
    </div>
  </div>

  <div class='mb-3 flex items-center gap-3'>
    <label class='flex items-center gap-2 text-(sm text-secondary)'>
      <input type='checkbox' bind:checked={showFinished}>
      完了済みを表示
    </label>
    <label class='flex items-center gap-2 text-(sm text-secondary)'>
      <input type='checkbox' bind:checked={showOnlyError}>
      エラーのみ
    </label>
  </div>

  {#if (showFinished ? $finishedQuery.isLoading : $unfinishedQuery.isLoading)}
    <div class='text-(text-secondary)'>読み込み中...</div>
  {:else if (showFinished ? $finishedQuery.isError : $unfinishedQuery.isError)}
    <div class='text-text-danger'>読み込みに失敗しました</div>
  {:else}
    <div class='grid grid-cols-[auto_1fr_auto_auto_auto] items-center gap-x-3 gap-y-2'>
      <div class='text-(sm text-secondary)'>ID</div>
      <div class='text-(sm text-secondary)'>SRC</div>
      <div class='text-(sm text-secondary)'>種別</div>
      <div class='text-(sm text-secondary)'>加工</div>
      <div class='text-(sm text-secondary)'>出力先</div>
      {#each viewItems as it}
        <div class='break-all text-(sm text-primary)'>{it.id}</div>
        <div class='break-all text-(sm text-primary)'>{it.src}</div>
        <div class='text-(sm text-primary)'>{fmtType(it.srcType)}</div>
        <div class='text-(sm text-primary)'>{fmtPreprocess(it.preprocess)}</div>
        <div class='break-all text-(sm text-primary)'>{it.dstPath}</div>
        {#if it.lastError}
          <div class='text-text-danger col-span-full text-(sm)'>エラー: {it.lastError}</div>
        {/if}
      {/each}
      {#if viewItems.length === 0}
        <div class='col-span-full text-(sm text-secondary)'>
          {#if showOnlyError}
            フィルタに一致するエラー項目はありません
          {:else}
            {showFinished ? '完了済みのキューはありません' : '未完了のキューはありません'}
          {/if}
        </div>
      {/if}
    </div>
  {/if}
</div>
