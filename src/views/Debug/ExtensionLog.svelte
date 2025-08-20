<script lang='ts'>
  import Button from '@/components/UI/Button.svelte'
  import { onMount } from 'svelte'
  import { commandGetNativeHostLogs, type HostLogItem } from '@/lib/command'

  const LEVELS = [
    { value: 0, label: 'すべて' },
    { value: 1, label: 'Info' },
    { value: 2, label: 'Warn' },
    { value: 3, label: 'Error' },
  ]
  const TYPES = [
    { value: -1, label: 'すべて' },
    { value: 0, label: 'Unknown' },
    { value: 1, label: 'ReceiveDmmSyncGamesRequest' },
    { value: 2, label: 'ReceiveDlsiteSyncGamesRequest' },
    { value: 10, label: 'ImageQueueWorkerStarted' },
    { value: 11, label: 'ImageQueueWorkerFinished' },
    { value: 20, label: 'ImageQueueItemStarted' },
    { value: 21, label: 'ImageQueueItemSucceeded' },
    { value: 22, label: 'ImageQueueItemFailed' },
  ]

  let items = $state<HostLogItem[]>([])
  let total = $state(0)
  let limit = $state(50)
  let offset = $state(0)
  let levelFilter = $state(0) // 0=All
  let typeFilter = $state(-1) // -1=All
  let loading = $state(false)

  const levelBadgeClass = (lv: number) => {
    switch (lv) {
      case 1: return 'text-blue-600'
      case 2: return 'text-yellow-600'
      case 3: return 'text-red-600'
      default: return 'text-gray-600'
    }
  }
  const levelLabel = (lv: number) => LEVELS.find(l => l.value === lv)?.label ?? String(lv)
  const typeLabel = (t: number) => TYPES.find(tp => tp.value === t)?.label ?? String(t)
  const fmt = (s: string) => new Date(s).toLocaleString('ja-JP')

  const reload = async () => {
    loading = true
    try {
      offset = 0
      const res = await commandGetNativeHostLogs({
        limit,
        offset,
        level: levelFilter === 0 ? undefined : levelFilter,
        typ: typeFilter === -1 ? undefined : typeFilter,
      })
      items = res.items
      total = res.total
    } finally {
      loading = false
    }
  }

  const loadMore = async () => {
    if (items.length >= total) return
    loading = true
    try {
      const nextOffset = offset + limit
      const res = await commandGetNativeHostLogs({
        limit,
        offset: nextOffset,
        level: levelFilter === 0 ? undefined : levelFilter,
        typ: typeFilter === -1 ? undefined : typeFilter,
      })
      items = [...items, ...res.items]
      offset = nextOffset
      total = res.total
    } finally {
      loading = false
    }
  }

  onMount(reload)
</script>

<div class='mx-auto h-full max-w-5xl overflow-y-auto p-6'>
  <div class='mb-4 flex items-end gap-3'>
    <div>
      <div class='text-(sm text-secondary) mb-1'>レベル</div>
      <select bind:value={levelFilter} class='border border-(border-primary) rounded bg-(bg-primary) p-2 text-(text-primary)'>
        {#each LEVELS as lv}
          <option value={lv.value}>{lv.label}</option>
        {/each}
      </select>
    </div>
    <div>
      <div class='text-(sm text-secondary) mb-1'>タイプ</div>
      <select bind:value={typeFilter} class='border border-(border-primary) rounded bg-(bg-primary) p-2 text-(text-primary) min-w-72'>
        {#each TYPES as tp}
          <option value={tp.value}>{tp.label}</option>
        {/each}
      </select>
    </div>
    <Button text='更新' onclick={reload} disabled={loading} />
    <div class='text-(sm text-secondary) ml-auto'>合計 {total} 件</div>
  </div>

  <div class='rounded border border-(border-primary) bg-(bg-secondary)'>
    {#if items.length === 0}
      <div class='p-6 text-center text-(text-secondary)'>ログがありません</div>
    {:else}
      <div class='max-h-[70vh] overflow-y-auto divide-y divide-(border-primary)'>
        {#each items as it}
          <div class='p-3 hover:bg-(bg-tertiary) transition-colors'>
            <div class='mb-1 flex items-center gap-3'>
              <span class='text-(xs text-tertiary) font-mono'>{fmt(it.created_at)}</span>
              <span class={'text-(xs font-medium) ' + levelBadgeClass(it.level)}>{levelLabel(it.level)}</span>
              <span class='text-(xs text-secondary) rounded bg-(bg-primary) px-1'>{typeLabel(it.typ)}</span>
              <span class='text-(xs text-tertiary)'>#{it.id}</span>
            </div>
            <div class='whitespace-pre-wrap break-words text-(sm text-primary) font-mono'>{it.message}</div>
          </div>
        {/each}
      </div>
      {#if items.length < total}
        <div class='p-3 border-t border-(border-primary) text-center'>
          <Button text={loading ? '読み込み中...' : 'もっと読む'} onclick={loadMore} disabled={loading} />
        </div>
      {/if}
    {/if}
  </div>
</div>


