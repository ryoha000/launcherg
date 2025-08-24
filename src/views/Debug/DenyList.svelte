<script lang='ts'>
  import type { DenyListItemVm } from '@/lib/command'
  import { get } from 'svelte/store'
  import Button from '@/components/UI/Button.svelte'
  import { useAddDenyListMutation, useDenyListQuery, useRemoveDenyListMutation } from '@/lib/data/queries/denyList'

  const STORE_TYPES = [
    { value: 1, label: 'DMM' },
    { value: 2, label: 'DLsite' },
  ]

  const denyListQuery = useDenyListQuery()
  const addMutation = useAddDenyListMutation()
  const removeMutation = useRemoveDenyListMutation()
  let items = $state<DenyListItemVm[]>([])
  $effect(() => {
    const q = get(denyListQuery)
    items = q.data ?? []
  })
  let storeType = $state(1)
  let storeId = $state('')
  let name = $state('')
  let loading = $derived.by(() => {
    const q = get(denyListQuery)
    const a = get(addMutation)
    const r = get(removeMutation)
    return !!(q.isLoading || a.isPending || r.isPending)
  })

  const add = async () => {
    const id = storeId.trim()
    const nm = name.trim()
    if (!id || !nm)
      return
    await get(addMutation).mutateAsync({ storeType, storeId: id, name: nm })
    storeId = ''
    name = ''
  }

  const remove = async (it: DenyListItemVm) => {
    await get(removeMutation).mutateAsync({ storeType: it.storeType, storeId: it.storeId })
  }
</script>

<div class='mx-auto h-full max-w-3xl overflow-y-auto p-6'>
  <div class='mb-4 flex items-end gap-3'>
    <div>
      <div class='mb-1 text-(sm text-secondary)'>ストア</div>
      <select bind:value={storeType} class='border border-(border-primary) rounded bg-(bg-primary) p-2 text-(text-primary)'>
        {#each STORE_TYPES as st}
          <option value={st.value}>{st.label}</option>
        {/each}
      </select>
    </div>
    <div class='flex-1'>
      <div class='mb-1 text-(sm text-secondary)'>Store ID</div>
      <input bind:value={storeId} class='w-full border border-(border-primary) rounded bg-(bg-primary) p-2 text-(text-primary)' placeholder='例: dmm: product id / dlsite: RJxxxxxx など' />
    </div>
    <div class='flex-1'>
      <div class='mb-1 text-(sm text-secondary)'>Name</div>
      <input bind:value={name} class='w-full border border-(border-primary) rounded bg-(bg-primary) p-2 text-(text-primary)' placeholder='例: ゲームタイトル' />
    </div>
    <Button text='追加' onclick={add} disabled={loading} />
  </div>

  <div class='border border-(border-primary) rounded bg-(bg-secondary)'>
    {#if items.length === 0}
      <div class='p-6 text-(center text-secondary)'>登録がありません</div>
    {:else}
      <div class='divide-(y border-primary)'>
        {#each items as it}
          <div class='flex items-center justify-between p-3'>
            <div class='text-(sm text-primary) font-mono'>[{it.storeType === 1 ? 'DMM' : 'DLsite'}] {it.storeId} — {it.name}</div>
            <Button text='削除' onclick={() => remove(it)} />
          </div>
        {/each}
      </div>
    {/if}
  </div>
</div>
