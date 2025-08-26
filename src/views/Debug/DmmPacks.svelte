<script lang='ts'>
  import { get } from 'svelte/store'
  import Button from '@/components/UI/Button.svelte'
  import { useAddDmmPackMutation, useDmmPackQuery, useRemoveDmmPackMutation } from '@/lib/data/queries/dmmPack'

  type Element = {
    id: number
    gamename: string
    dmm?: { id: number, collectionElementId: number, category: string, subcategory: string } | null
  }

  let elements = $state<Element[]>([])
  const dmmPackQuery = useDmmPackQuery()
  const addMutation = useAddDmmPackMutation()
  const removeMutation = useRemoveDmmPackMutation()
  let newStoreId = $state('')
  let loading = $state(false)
  let packs: number[] = $derived($dmmPackQuery.data ?? [])

  async function load() {
    loading = true
    try {
      // 既存デバッグ用の全要素取得
      const { commandGetAllElements } = await import('@/lib/command')
      const all = await commandGetAllElements()
      elements = all
      await get(dmmPackQuery).refetch()
    }
    finally {
      loading = false
    }
  }

  async function addPackManually() {
    const sid = newStoreId.trim()
    if (!sid)
      return
    // 暫定: workId を直接入力（デバッグ用途）
    const workId = Number(sid)
    if (!Number.isFinite(workId))
      return
    await get(addMutation).mutateAsync({ workId })
    newStoreId = ''
    await load()
  }

  async function removePack(id: number, workId: number) {
    await get(removeMutation).mutateAsync({ workId })
    await load()
  }

  $effect(() => {
    // 初期ロード
    void load()
  })
</script>

<div class='mx-auto h-full max-w-3xl overflow-y-auto p-6 space-y-4'>
  <h2 class='text-(lg text-primary) font-bold'>DMM パック管理</h2>
  {#if loading}
    <div class='text-(text-secondary)'>Loading...</div>
  {:else}
    <div class='overflow-hidden border border-(border-primary) rounded bg-(bg-secondary)'>
      <table class='w-full text-(sm text-primary)'>
        <thead class='text-(sm text-secondary)'>
          <tr>
            <th class='px-3 py-2 text-left'>名前</th>
            <th class='px-3 py-2 text-left'>DMM</th>
            <th class='px-3 py-2 text-left'>Pack</th>
          </tr>
        </thead>
        <tbody>
          {#each elements as el}
            {#if el.dmm}
              <tr class='border-(t border-primary)'>
                <td class='px-3 py-2'>{el.gamename}</td>
                <td class='px-3 py-2'>{el.dmm?.category}/{el.dmm?.subcategory}</td>
                <td class='px-3 py-2'>
                  <input type='checkbox' disabled title='storeId 取得未対応' />
                </td>
              </tr>
            {/if}
          {/each}
        </tbody>
      </table>
    </div>

    <div class='mt-6 space-y-2'>
      <h3 class='text-(sm text-secondary) font-semibold'>登録済み Pack</h3>
      <div class='border border-(border-primary) rounded bg-(bg-secondary)'>
        {#if $dmmPackQuery.isLoading}
          <div class='p-6 text-(center text-secondary)'>Loading...</div>
        {:else if packs.length === 0}
          <div class='p-6 text-(center text-secondary)'>なし</div>
        {:else}
          <div class='divide-(y border-primary)'>
            {#each packs as p}
              <div class='flex items-center justify-between p-3'>
                <div class='flex items-center gap-2'>
                  <div class='border border-(border-primary) rounded bg-(bg-primary) px-1 py-0.5 text-(sm text-primary) font-mono'>work_id: {p}</div>
                </div>
                <Button text='削除' onclick={() => removePack(p, p)} />
              </div>
            {/each}
          </div>
        {/if}
      </div>

      <div class='mt-2 flex items-end gap-3'>
        <div class='flex-1'>
          <div class='mb-1 text-(sm text-secondary)'>Store ID</div>
          <input class='w-full border border-(border-primary) rounded bg-(bg-primary) p-2 text-(text-primary)' placeholder='storeId を入力 (例: purple_0028pack)' bind:value={newStoreId} />
        </div>
        <Button text='追加' onclick={addPackManually} disabled={loading} />
      </div>
    </div>
  {/if}
</div>
